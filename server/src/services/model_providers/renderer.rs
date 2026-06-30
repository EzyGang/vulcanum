use std::collections::HashMap;

use serde_json::json;
use vulcanum_shared::api_types::{AgentBackend, AgentConfigPayload};

use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::ModelProviderConfig;
use crate::services::model_providers::auth::credentials::{
    parse_auth, ParsedAuth, OPENAI_PROVIDER_KEY,
};
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::auth::opencode_auth::openai_auth_content;

const OPENAI_CODEX_OAUTH_TOKEN_ENV: &str = "OPENAI_CODEX_OAUTH_TOKEN";

#[derive(Debug)]
pub struct RenderedAgentConfig {
    pub agent_config: AgentConfigPayload,
    pub env: HashMap<String, String>,
}

#[derive(Debug, Default)]
pub struct RenderedModelConfig {
    pub opencode_config: String,
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
                    config_json: rendered.opencode_config,
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
    let mut provider_json = serde_json::Map::new();
    let mut opencode_auth_content: Option<String> = None;

    for provider in connected {
        let mut options = serde_json::Map::new();
        match parse_auth(&provider.credentials, cipher)? {
            ParsedAuth::ApiKey(credentials) => {
                for (key, secret) in credentials {
                    if secret.is_empty() {
                        continue;
                    }
                    let env_key = credential_env_key(&key);
                    env.insert(env_key.clone(), secret);
                    options.insert("apiKey".to_owned(), json!(format!("{{env:{env_key}}}")));
                }
                provider_json.insert(provider.provider_key.clone(), json!({ "options": options }));
            }
            ParsedAuth::DeviceOAuth(credential) if provider.provider_key == OPENAI_PROVIDER_KEY => {
                opencode_auth_content = Some(openai_auth_content(&credential)?);
                provider_json.insert(provider.provider_key.clone(), json!({ "options": options }));
            }
            ParsedAuth::DeviceOAuth(_) => (),
            ParsedAuth::None => (),
        }
    }

    let primary = model_ref(selection.primary_provider_key, selection.primary_model_id);
    let small = model_ref(selection.small_provider_key, selection.small_model_id);

    let mut root = serde_json::Map::new();
    root.insert("permission".to_owned(), permission_config());
    if !provider_json.is_empty() {
        root.insert(
            "provider".to_owned(),
            serde_json::Value::Object(provider_json),
        );
    }
    if let Some(value) = primary {
        root.insert("model".to_owned(), json!(value));
    }
    if let Some(value) = small {
        root.insert("small_model".to_owned(), json!(value));
    }

    Ok(RenderedModelConfig {
        opencode_config: serde_json::Value::Object(root).to_string(),
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

    for provider in connected {
        match parse_auth(&provider.credentials, cipher)? {
            ParsedAuth::ApiKey(credentials) => {
                for (key, secret) in credentials {
                    if secret.is_empty() {
                        continue;
                    }
                    env.insert(credential_env_key(&key), secret);
                }
            }
            ParsedAuth::DeviceOAuth(credential) if provider.provider_key == OPENAI_PROVIDER_KEY => {
                if !credential.access.is_empty() {
                    env.insert(OPENAI_CODEX_OAUTH_TOKEN_ENV.to_owned(), credential.access);
                }
            }
            ParsedAuth::DeviceOAuth(_) | ParsedAuth::None => (),
        }
    }

    if let Some(provider) = selection.primary_provider_key {
        if !provider.is_empty() {
            env.insert("PI_PROVIDER".to_owned(), provider.to_owned());
        }
    }
    if let Some(model) = selection.primary_model_id {
        if !model.is_empty() {
            env.insert("PI_MODEL".to_owned(), model.to_owned());
        }
    }
    if let Some(model) = selection.small_model_id {
        if !model.is_empty() {
            env.insert("PI_SMALL_MODEL".to_owned(), model.to_owned());
        }
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

fn permission_config() -> serde_json::Value {
    json!({
        "*": "allow",
        "question": "deny",
    })
}

fn credential_env_key(key: &str) -> String {
    decode_legacy_snake_case_env_key(key).unwrap_or_else(|| key.to_owned())
}

fn decode_legacy_snake_case_env_key(key: &str) -> Option<String> {
    if !key.starts_with('_') || !key.contains("__") {
        return None;
    }

    let trimmed = key.trim_matches('_');
    if trimmed.is_empty() {
        return None;
    }

    let mut parts: Vec<String> = Vec::new();
    for segment in trimmed.split("__") {
        if segment.is_empty()
            || !segment
                .chars()
                .all(|ch| ch == '_' || ch.is_ascii_lowercase())
        {
            return None;
        }

        let part = segment.replace('_', "").to_ascii_uppercase();
        if part.is_empty() {
            return None;
        }
        parts.push(part);
    }

    match parts.is_empty() {
        true => None,
        false => Some(parts.join("_")),
    }
}
