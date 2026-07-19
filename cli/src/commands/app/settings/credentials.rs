use serde_json::{Map, Value};
use vulcanum_shared::api::app::model_providers::CatalogProvider;

use crate::commands::app::settings::runtime::SettingsRuntime;

pub(super) fn task_tracker_credentials(
    use_stdin: bool,
    runtime: &mut SettingsRuntime,
) -> anyhow::Result<String> {
    if use_stdin {
        let input = (runtime.read_stdin)()?;
        return parse_task_tracker_credentials(&input);
    }
    require_terminal(runtime)?;
    let value = (runtime.prompt_hidden)("Task tracker API key")?;
    if value.is_empty() {
        anyhow::bail!("Task tracker API key must not be empty");
    }
    Ok(value)
}

pub(super) fn model_provider_credentials(
    use_stdin: bool,
    provider: Option<&CatalogProvider>,
    runtime: &mut SettingsRuntime,
) -> anyhow::Result<Value> {
    if use_stdin {
        let input = (runtime.read_stdin)()?;
        return parse_model_provider_credentials(&input);
    }
    require_terminal(runtime)?;
    let provider =
        provider.ok_or_else(|| anyhow::anyhow!("Model provider is not in the catalog"))?;
    let mut fields = provider.env.clone();
    fields.sort();
    if fields.is_empty() {
        anyhow::bail!("Model provider catalog exposes no API-key credential fields");
    }

    let mut credentials = Map::new();
    for field in fields {
        let value = (runtime.prompt_hidden)(&field)?;
        if !value.is_empty() {
            credentials.insert(field, Value::String(value));
        }
    }
    if credentials.is_empty() {
        anyhow::bail!("At least one model provider credential is required");
    }
    Ok(Value::Object(credentials))
}

fn require_terminal(runtime: &SettingsRuntime) -> anyhow::Result<()> {
    if runtime.stdin_is_terminal {
        return Ok(());
    }
    anyhow::bail!("stdin is not a terminal; pass --credentials-stdin")
}

fn parse_task_tracker_credentials(input: &str) -> anyhow::Result<String> {
    let value: Value = serde_json::from_str(input)
        .map_err(|error| anyhow::anyhow!("Invalid credentials JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Task tracker credentials must be a JSON object"))?;
    if object.len() != 1 || !object.contains_key("api_key") {
        anyhow::bail!("Task tracker credentials must contain exactly the api_key field");
    }
    let api_key = object["api_key"]
        .as_str()
        .ok_or_else(|| anyhow::anyhow!("Task tracker api_key must be a string"))?;
    if api_key.is_empty() {
        anyhow::bail!("Task tracker api_key must not be empty");
    }
    Ok(api_key.to_owned())
}

fn parse_model_provider_credentials(input: &str) -> anyhow::Result<Value> {
    let value: Value = serde_json::from_str(input)
        .map_err(|error| anyhow::anyhow!("Invalid credentials JSON: {error}"))?;
    let object = value
        .as_object()
        .ok_or_else(|| anyhow::anyhow!("Model provider credentials must be a JSON object"))?;
    if object.is_empty() {
        anyhow::bail!("Model provider credentials must not be empty");
    }
    for (name, value) in object {
        if name.is_empty() {
            anyhow::bail!("Model provider credential names must not be empty");
        }
        match value.as_str() {
            Some(value) if !value.is_empty() => (),
            Some(_) => anyhow::bail!("Model provider credential values must not be empty"),
            None => anyhow::bail!("Model provider credential values must be strings"),
        }
    }
    Ok(value)
}
