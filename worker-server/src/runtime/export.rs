use serde::Deserialize;

use vulcanum_shared::runtime::errors::HarnessError;
use vulcanum_shared::runtime::types::SessionExport;

#[derive(Debug, Deserialize)]
struct ExportData {
    messages: Vec<ExportMessage>,
}

#[derive(Debug, Deserialize)]
struct ExportMessage {
    role: String,
    tokens: Option<ExportTokens>,
    model: Option<ExportModel>,
    parts: Vec<ExportPart>,
}

#[derive(Debug, Deserialize)]
struct ExportTokens {
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
    #[serde(rename = "providerID")]
    provider_id: Option<String>,
    #[serde(rename = "modelID")]
    model_id: Option<String>,
}

#[derive(Debug, Deserialize)]
struct ExportPart {
    #[serde(rename = "type")]
    part_type: String,
    text: Option<String>,
}

pub fn parse_export(raw: &str) -> Result<SessionExport, HarnessError> {
    let data: ExportData = serde_json::from_str(raw)
        .map_err(|e| HarnessError::OutputParse(format!("export json parse failed: {e}")))?;

    let mut input_tokens: u64 = 0;
    let mut output_tokens: u64 = 0;
    let mut cache_read_tokens: u64 = 0;
    let mut cache_write_tokens: u64 = 0;
    let mut model_used: Option<String> = None;
    let mut pr_url: Option<String> = None;
    let mut raw_output_parts: Vec<String> = Vec::new();

    for msg in &data.messages {
        if let Some(tokens) = &msg.tokens {
            input_tokens += tokens.input.unwrap_or(0);
            output_tokens += tokens.output.unwrap_or(0);
            let cache = tokens.cache.as_ref();
            cache_read_tokens += cache.and_then(|c| c.read).unwrap_or(0);
            cache_write_tokens += cache.and_then(|c| c.write).unwrap_or(0);
        }

        if model_used.is_none() {
            if let Some(model) = &msg.model {
                match (&model.provider_id, &model.model_id) {
                    (Some(p), Some(m)) => model_used = Some(format!("{p}/{m}")),
                    (None, Some(m)) => model_used = Some(m.to_owned()),
                    _ => (),
                }
            }
        }

        if msg.role == "assistant" {
            for part in &msg.parts {
                if part.part_type == "text" {
                    if let Some(text) = &part.text {
                        raw_output_parts.push(text.to_owned());
                        if pr_url.is_none() {
                            pr_url = extract_pr_url(text);
                        }
                    }
                }
            }
        }
    }

    let tokens_used = input_tokens + output_tokens + cache_read_tokens + cache_write_tokens;

    Ok(SessionExport {
        exit_code: 0,
        tokens_used,
        input_tokens,
        output_tokens,
        cache_read_tokens,
        cache_write_tokens,
        pr_url,
        duration_ms: 0,
        model_used,
        raw_output: Some(raw_output_parts.join("\n")),
    })
}

fn extract_pr_url(text: &str) -> Option<String> {
    let patterns = [
        "https://github.com/",
        "https://gitlab.com/",
        "https://bitbucket.org/",
    ];

    for line in text.lines() {
        for pattern in patterns {
            if let Some(pos) = line.find(pattern) {
                let url_part = &line[pos..];
                if let Some(end) = url_part.find(|c: char| c.is_whitespace() || c == ')') {
                    return Some(url_part[..end].to_owned());
                }
                return Some(url_part.to_owned());
            }
        }
    }

    None
}
