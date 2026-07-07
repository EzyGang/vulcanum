use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::RwLock;

use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::{CatalogModel, CatalogProvider, CatalogResponse};
use crate::services::model_providers::auth::credentials::OPENAI_PROVIDER_KEY;

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CATALOG_TTL: Duration = Duration::from_secs(60 * 60);
const CATALOG_HTTP_TIMEOUT: Duration = Duration::from_secs(15);
const CODEX_ALLOWED_OPENAI_MODELS: &[&str] =
    &["gpt-5.5", "gpt-5.3-codex-spark", "gpt-5.4", "gpt-5.4-mini"];
const CODEX_DISALLOWED_OPENAI_MODELS: &[&str] = &["gpt-5.5-pro"];

#[derive(Clone)]
pub struct ModelCatalogClient {
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedCatalog>>>,
}

#[must_use]
pub fn is_codex_compatible_openai_model(model_id: &str) -> bool {
    if CODEX_DISALLOWED_OPENAI_MODELS.contains(&model_id) {
        return false;
    }
    if CODEX_ALLOWED_OPENAI_MODELS.contains(&model_id) {
        return true;
    }

    let Some(version) = model_id.strip_prefix("gpt-") else {
        return false;
    };
    let Some((major, minor)) = parse_major_minor(version) else {
        return false;
    };

    major > 5 || major == 5 && minor > 4
}

fn parse_major_minor(value: &str) -> Option<(u32, u32)> {
    let mut parts = value.split('.');
    let major = parts.next()?.parse::<u32>().ok()?;
    let minor = parts.next()?.parse::<u32>().ok()?;
    match parts.next() {
        Some(_) => None,
        None => Some((major, minor)),
    }
}

struct CachedCatalog {
    fetched_at: Instant,
    catalog: CatalogResponse,
}

#[derive(Debug, Deserialize)]
struct RawProvider {
    id: String,
    name: String,
    #[serde(default)]
    doc: String,
    #[serde(default)]
    env: Vec<String>,
    models: HashMap<String, RawModel>,
}

#[derive(Debug, Deserialize)]
struct RawModel {
    id: String,
    name: String,
    #[serde(default)]
    status: Option<String>,
    #[serde(default)]
    limit: Option<RawLimit>,
    #[serde(default)]
    cost: Option<RawCost>,
    #[serde(default)]
    attachment: bool,
    #[serde(default)]
    reasoning: bool,
    #[serde(default)]
    tool_call: bool,
    #[serde(default)]
    structured_output: bool,
}

#[derive(Debug, Deserialize)]
struct RawLimit {
    #[serde(default)]
    context: Option<i64>,
    #[serde(default)]
    output: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct RawCost {
    #[serde(default)]
    input: Option<f64>,
    #[serde(default)]
    output: Option<f64>,
}

impl ModelCatalogClient {
    pub fn new() -> Result<Self, ModelProvidersError> {
        let client = reqwest::Client::builder()
            .timeout(CATALOG_HTTP_TIMEOUT)
            .build()
            .map_err(|e| ModelProvidersError::Catalog(format!("building catalog client: {e}")))?;
        Ok(Self {
            client,
            cache: Arc::new(RwLock::new(None)),
        })
    }

    pub async fn catalog(&self) -> Result<CatalogResponse, ModelProvidersError> {
        {
            let guard = self.cache.read().await;
            if let Some(cached) = guard.as_ref() {
                if cached.fetched_at.elapsed() < CATALOG_TTL {
                    return Ok(cached.catalog.clone());
                }
            }
        }

        let response = self.client.get(MODELS_DEV_URL).send().await.map_err(|e| {
            ModelProvidersError::Catalog(format!("fetching models.dev catalog: {e}"))
        })?;
        if !response.status().is_success() {
            return Err(ModelProvidersError::Catalog(format!(
                "fetching models.dev catalog returned HTTP {}",
                response.status()
            )));
        }

        let raw = response
            .json::<HashMap<String, RawProvider>>()
            .await
            .map_err(|e| {
                ModelProvidersError::Catalog(format!("parsing models.dev catalog: {e}"))
            })?;

        let catalog = parse_catalog(raw);
        let mut guard = self.cache.write().await;
        *guard = Some(CachedCatalog {
            fetched_at: Instant::now(),
            catalog: catalog.clone(),
        });
        Ok(catalog)
    }

    #[cfg(test)]
    pub(crate) async fn from_catalog(catalog: CatalogResponse) -> Self {
        Self {
            client: reqwest::Client::builder()
                .timeout(CATALOG_HTTP_TIMEOUT)
                .build()
                .expect("build catalog test client"),
            cache: Arc::new(RwLock::new(Some(CachedCatalog {
                fetched_at: Instant::now(),
                catalog,
            }))),
        }
    }

    pub async fn validate_provider(&self, provider_key: &str) -> Result<(), ModelProvidersError> {
        let catalog = self.catalog().await?;
        self.catalog_provider(&catalog, provider_key).map(|_| ())
    }

    pub async fn validate_model(
        &self,
        provider_key: &str,
        model_id: &str,
    ) -> Result<(), ModelProvidersError> {
        let catalog = self.catalog().await?;
        let provider = catalog
            .providers
            .iter()
            .find(|p| p.id == provider_key)
            .ok_or_else(|| ModelProvidersError::UnknownProvider(provider_key.to_owned()))?;
        match provider.models.iter().any(|m| m.id == model_id) {
            true => Ok(()),
            false => Err(ModelProvidersError::UnknownModel {
                provider_key: provider_key.to_owned(),
                model_id: model_id.to_owned(),
            }),
        }
    }

    pub async fn validate_credential_fields(
        &self,
        provider_key: &str,
        fields: &[String],
    ) -> Result<(), ModelProvidersError> {
        let catalog = self.catalog().await?;
        let provider = self.catalog_provider(&catalog, provider_key)?;
        let allowed = provider.env.iter().collect::<HashSet<&String>>();
        for field in fields {
            if !allowed.contains(field) {
                return Err(ModelProvidersError::InvalidAuthConfig(format!(
                    "credential field {field} is not allowed for provider {provider_key}"
                )));
            }
        }
        Ok(())
    }

    fn catalog_provider<'a>(
        &self,
        catalog: &'a CatalogResponse,
        provider_key: &str,
    ) -> Result<&'a CatalogProvider, ModelProvidersError> {
        catalog
            .providers
            .iter()
            .find(|p| p.id == provider_key)
            .ok_or_else(|| ModelProvidersError::UnknownProvider(provider_key.to_owned()))
    }
}

fn parse_catalog(raw: HashMap<String, RawProvider>) -> CatalogResponse {
    let mut providers: Vec<CatalogProvider> = raw
        .into_values()
        .map(|provider| {
            let provider_id = provider.id;
            let mut models: Vec<CatalogModel> = provider
                .models
                .into_values()
                .map(|model| {
                    let opencode_chatgpt_compatible = provider_id == OPENAI_PROVIDER_KEY
                        && is_codex_compatible_openai_model(&model.id);
                    CatalogModel {
                        id: model.id,
                        name: model.name,
                        status: model.status,
                        context_limit: model.limit.as_ref().and_then(|l| l.context),
                        output_limit: model.limit.as_ref().and_then(|l| l.output),
                        input_cost: model.cost.as_ref().and_then(|c| c.input),
                        output_cost: model.cost.as_ref().and_then(|c| c.output),
                        attachment: model.attachment,
                        reasoning: model.reasoning,
                        tool_call: model.tool_call,
                        structured_output: model.structured_output,
                        opencode_chatgpt_compatible,
                    }
                })
                .collect();
            models.sort_by(|a, b| a.name.cmp(&b.name));
            CatalogProvider {
                id: provider_id,
                name: provider.name,
                doc: provider.doc,
                env: provider.env,
                models,
            }
        })
        .collect();
    providers.sort_by(|a, b| a.name.cmp(&b.name));
    CatalogResponse { providers }
}
