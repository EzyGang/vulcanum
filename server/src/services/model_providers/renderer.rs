use std::collections::{HashMap, HashSet};

use vulcanum_shared::api::wire::{AgentBackend, AgentConfigPayload, OpenCodeProviderConfig};

use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::ModelProviderConfig;
use crate::services::model_providers::auth::credentials::{
    normalize_credential_env_key, parse_auth, ParsedAuth, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::opencode_auth::openai_auth_content;

const OPENAI_CODEX_OAUTH_TOKEN_ENV: &str = "OPENAI_CODEX_OAUTH_TOKEN";
const OMP_OPENAI_CODEX_PROVIDER_KEY: &str = "openai-codex";
const VULCANUM_OMP_PROVIDER_ENV: &str = "VULCANUM_OMP_PROVIDER";
const VULCANUM_OMP_MODEL_ENV: &str = "VULCANUM_OMP_MODEL";
const VULCANUM_OMP_SMOL_ENV: &str = "VULCANUM_OMP_SMOL";

#[derive(Debug)]
pub struct RenderedAgentConfig {
    pub agent_config: AgentConfigPayload,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct RenderedModelConfig {
    pub providers: HashMap<String, OpenCodeProviderConfig>,
    pub model: Option<String>,
    pub small_model: Option<String>,
    pub env: HashMap<String, String>,
    pub opencode_auth_content: Option<String>,
}

#[derive(Debug)]
pub struct ModelSelection<'a> {
    pub primary_provider_key: Option<&'a str>,
    pub primary_model_id: Option<&'a str>,
    pub small_provider_key: Option<&'a str>,
    pub small_model_id: Option<&'a str>,
}

pub fn render_agent_config(
    backend: AgentBackend,
    connected: &[ModelProviderConfig],
    cipher: &SecretCipher,
    selection: ModelSelection<'_>,
) -> Result<RenderedAgentConfig, ModelProvidersError> {
    match backend {
        AgentBackend::OpenCode => {
            let rendered = render_opencode_config(connected, cipher, selection)?;
            Ok(RenderedAgentConfig {
                agent_config: AgentConfigPayload::OpenCode {
                    providers: rendered.providers,
                    model: rendered.model,
                    small_model: rendered.small_model,
                    auth_content: rendered.opencode_auth_content,
                },
                env: rendered.env,
            })
        }
        AgentBackend::OmpRpc => render_omp_config(connected, cipher, selection),
    }
}

pub fn render_opencode_config(
    connected: &[ModelProviderConfig],
    cipher: &SecretCipher,
    selection: ModelSelection<'_>,
) -> Result<RenderedModelConfig, ModelProvidersError> {
    let mut env: HashMap<String, String> = HashMap::new();
    let mut providers: HashMap<String, OpenCodeProviderConfig> = HashMap::new();
    let mut opencode_auth_content: Option<String> = None;

    for provider in connected {
        let mut options = HashMap::new();
        match parse_auth(&provider.credentials, cipher)? {
            ParsedAuth::ApiKey(credentials) => {
                for (key, secret) in credentials {
                    if secret.is_empty() {
                        continue;
                    }
                    let env_key = normalize_credential_env_key(&key)?;
                    env.insert(env_key.clone(), secret);
                    options.insert("apiKey".to_owned(), format!("{{env:{env_key}}}"));
                }
                providers.insert(
                    provider.provider_key.clone(),
                    OpenCodeProviderConfig { options },
                );
            }
            ParsedAuth::DeviceOAuth(credential) if provider.provider_key == OPENAI_PROVIDER_KEY => {
                opencode_auth_content = Some(openai_auth_content(&credential)?);
                providers.insert(
                    provider.provider_key.clone(),
                    OpenCodeProviderConfig { options },
                );
            }
            ParsedAuth::DeviceOAuth(_) => (),
            ParsedAuth::None => (),
        }
    }

    let model = model_ref(selection.primary_provider_key, selection.primary_model_id);
    let small_model = model_ref(selection.small_provider_key, selection.small_model_id);

    Ok(RenderedModelConfig {
        providers,
        model,
        small_model,
        env,
        opencode_auth_content,
    })
}

fn render_omp_config(
    connected: &[ModelProviderConfig],
    cipher: &SecretCipher,
    selection: ModelSelection<'_>,
) -> Result<RenderedAgentConfig, ModelProvidersError> {
    let mut env: HashMap<String, String> = HashMap::new();
    let mut omp_oauth_provider_keys: HashSet<String> = HashSet::new();
    for provider in connected {
        match parse_auth(&provider.credentials, cipher)? {
            ParsedAuth::ApiKey(credentials) => {
                for (key, secret) in credentials {
                    if secret.is_empty() {
                        continue;
                    }
                    env.insert(normalize_credential_env_key(&key)?, secret);
                }
            }
            ParsedAuth::DeviceOAuth(credential) if provider.provider_key == OPENAI_PROVIDER_KEY => {
                omp_oauth_provider_keys.insert(provider.provider_key.clone());
                if !credential.access.is_empty() {
                    env.insert(OPENAI_CODEX_OAUTH_TOKEN_ENV.to_owned(), credential.access);
                }
            }
            ParsedAuth::DeviceOAuth(_) | ParsedAuth::None => (),
        }
    }

    if let Some(provider) = selection.primary_provider_key {
        if !provider.is_empty() {
            env.insert(
                VULCANUM_OMP_PROVIDER_ENV.to_owned(),
                omp_provider_key(provider, &omp_oauth_provider_keys).to_owned(),
            );
        }
    }
    if let Some(model) = selection.primary_model_id {
        if !model.is_empty() {
            env.insert(VULCANUM_OMP_MODEL_ENV.to_owned(), model.to_owned());
        }
    }
    if let Some(smol) = omp_smol_ref(
        selection.small_provider_key,
        selection.small_model_id,
        selection.primary_provider_key,
        &omp_oauth_provider_keys,
    ) {
        env.insert(VULCANUM_OMP_SMOL_ENV.to_owned(), smol);
    }

    Ok(RenderedAgentConfig {
        agent_config: AgentConfigPayload::OmpRpc { config_yml: None },
        env,
    })
}

fn model_ref(provider_key: Option<&str>, model_id: Option<&str>) -> Option<String> {
    match (provider_key, model_id) {
        (Some(provider), Some(model)) if !provider.is_empty() && !model.is_empty() => {
            Some(format!("{provider}/{model}"))
        }
        _ => None,
    }
}

fn omp_smol_ref(
    provider_key: Option<&str>,
    model_id: Option<&str>,
    primary_provider_key: Option<&str>,
    oauth_provider_keys: &HashSet<String>,
) -> Option<String> {
    let model = model_id?;
    if model.is_empty() {
        return None;
    }

    match provider_key {
        Some(provider) if !provider.is_empty() && primary_provider_key != Some(provider) => {
            Some(format!(
                "{}/{}",
                omp_provider_key(provider, oauth_provider_keys),
                model
            ))
        }
        _ => Some(model.to_owned()),
    }
}

fn omp_provider_key<'a>(provider_key: &'a str, oauth_provider_keys: &HashSet<String>) -> &'a str {
    match provider_key {
        OPENAI_PROVIDER_KEY if oauth_provider_keys.contains(provider_key) => {
            OMP_OPENAI_CODEX_PROVIDER_KEY
        }
        _ => provider_key,
    }
}
