use std::collections::HashMap;

use serde_json::json;

use crate::services::model_providers::model::ModelProviderConfig;

#[derive(Debug, Default)]
pub struct RenderedModelConfig {
    pub opencode_config: String,
    pub env: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ModelSelection<'a> {
    pub primary_provider_key: Option<&'a str>,
    pub primary_model_id: Option<&'a str>,
    pub small_provider_key: Option<&'a str>,
    pub small_model_id: Option<&'a str>,
}

#[must_use]
pub fn render_opencode_config(
    connected: &[ModelProviderConfig],
    selection: ModelSelection<'_>,
) -> RenderedModelConfig {
    let mut env: HashMap<String, String> = HashMap::new();
    let mut provider_json = serde_json::Map::new();

    for provider in connected {
        let mut options = serde_json::Map::new();
        if let Some(credentials) = provider.credentials.as_object() {
            for (key, value) in credentials {
                match value.as_str() {
                    Some(secret) if !secret.is_empty() => {
                        let env_key = credential_env_key(key);
                        env.insert(env_key.clone(), secret.to_owned());
                        options.insert("apiKey".to_owned(), json!(format!("{{env:{env_key}}}")));
                    }
                    Some(_) => (),
                    None => tracing::warn!(
                        provider_key = %provider.provider_key,
                        credential_key = %key,
                        "Skipping non-string model provider credential"
                    ),
                }
            }
        }
        provider_json.insert(provider.provider_key.clone(), json!({ "options": options }));
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

    RenderedModelConfig {
        opencode_config: serde_json::Value::Object(root).to_string(),
        env,
    }
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

        let part = segment
            .chars()
            .filter(|ch| *ch != '_')
            .collect::<String>()
            .to_ascii_uppercase();
        if part.is_empty() {
            return None;
        }
        parts.push(part);
    }

    match parts.len() {
        0 | 1 => None,
        _ => Some(parts.join("_")),
    }
}
