use serde::Deserialize;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::SessionExport;

#[derive(Debug, Deserialize)]
struct ExportData {
    info: ExportInfo,
}

#[derive(Debug, Deserialize)]
struct ExportInfo {
    tokens: ExportInfoTokens,
    model: Option<ExportModel>,
}

#[derive(Debug, Deserialize)]
struct ExportInfoTokens {
    input: Option<u64>,
    output: Option<u64>,
    #[serde(default)]
    cache: Option<ExportCache>,
}

#[derive(Debug, Deserialize)]
struct ExportCache {
    read: Option<u64>,
    write: Option<u64>,
}

#[derive(Debug, Deserialize)]
struct ExportModel {
    id: Option<String>,
    #[serde(rename = "providerID")]
    provider_id: Option<String>,
}

pub fn parse_export(raw: &str) -> Result<SessionExport, HarnessError> {
    let data: ExportData = serde_json::from_str(raw)
        .map_err(|e| HarnessError::OutputParse(format!("export json parse failed: {e}")))?;

    let tokens = &data.info.tokens;
    let input_tokens = tokens.input.unwrap_or(0);
    let output_tokens = tokens.output.unwrap_or(0);
    let cache_read_tokens = tokens.cache.as_ref().and_then(|c| c.read).unwrap_or(0);
    let cache_write_tokens = tokens.cache.as_ref().and_then(|c| c.write).unwrap_or(0);
    let tokens_used = input_tokens + output_tokens + cache_read_tokens + cache_write_tokens;

    let model_used = data
        .info
        .model
        .as_ref()
        .and_then(|m| match (&m.provider_id, &m.id) {
            (Some(p), Some(m_id)) => Some(format!("{p}/{m_id}")),
            (None, Some(m_id)) => Some(m_id.to_owned()),
            _ => None,
        });

    Ok(SessionExport {
        exit_code: 0,
        tokens_used,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
        duration_ms: 0,
        model_used,
        raw_output: None,
    })
}
