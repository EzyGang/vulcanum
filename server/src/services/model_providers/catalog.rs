use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};

use serde::Deserialize;
use tokio::sync::RwLock;

use crate::services::model_providers::errors::ModelProvidersError;
use crate::services::model_providers::model::{CatalogModel, CatalogProvider, CatalogResponse};

const MODELS_DEV_URL: &str = "https://models.dev/api.json";
const CATALOG_TTL: Duration = Duration::from_secs(60 * 60);

#[derive(Clone)]
pub struct ModelCatalogClient {
    client: reqwest::Client,
    cache: Arc<RwLock<Option<CachedCatalog>>>,
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
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
            cache: Arc::new(RwLock::new(None)),
        }
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

        let raw = self
            .client
            .get(MODELS_DEV_URL)
            .send()
            .await
            .map_err(|e| ModelProvidersError::Catalog(e.to_string()))?
            .error_for_status()
            .map_err(|e| ModelProvidersError::Catalog(e.to_string()))?
            .json::<HashMap<String, RawProvider>>()
            .await
            .map_err(|e| ModelProvidersError::Catalog(e.to_string()))?;

        let catalog = parse_catalog(raw);
        let mut guard = self.cache.write().await;
        *guard = Some(CachedCatalog {
            fetched_at: Instant::now(),
            catalog: catalog.clone(),
        });
        Ok(catalog)
    }

    pub async fn validate_provider(&self, provider_key: &str) -> Result<(), ModelProvidersError> {
        let catalog = self.catalog().await?;
        match catalog.providers.iter().any(|p| p.id == provider_key) {
            true => Ok(()),
            false => Err(ModelProvidersError::UnknownProvider(
                provider_key.to_owned(),
            )),
        }
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
}

impl Default for ModelCatalogClient {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_catalog(raw: HashMap<String, RawProvider>) -> CatalogResponse {
    let mut providers: Vec<CatalogProvider> = raw
        .into_values()
        .map(|provider| {
            let mut models: Vec<CatalogModel> = provider
                .models
                .into_values()
                .map(|model| CatalogModel {
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
                })
                .collect();
            models.sort_by(|a, b| a.name.cmp(&b.name));
            CatalogProvider {
                id: provider.id,
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
