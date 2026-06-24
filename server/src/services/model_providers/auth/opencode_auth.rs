use serde::Serialize;
use std::collections::HashMap;

use crate::services::model_providers::auth::credentials::{OAuthCredential, OPENAI_PROVIDER_KEY};
use crate::services::model_providers::errors::ModelProvidersError;

#[derive(Serialize)]
struct OpenCodeOAuthEntry<'a> {
    #[serde(rename = "type")]
    auth_type: &'static str,
    refresh: &'a str,
    access: &'a str,
    expires: i64,
    #[serde(rename = "accountId")]
    #[serde(skip_serializing_if = "Option::is_none")]
    account_id: Option<&'a str>,
}

pub fn openai_auth_content(credential: &OAuthCredential) -> Result<String, ModelProvidersError> {
    let mut entries: HashMap<&str, OpenCodeOAuthEntry<'_>> = HashMap::new();
    entries.insert(
        OPENAI_PROVIDER_KEY,
        OpenCodeOAuthEntry {
            auth_type: "oauth",
            refresh: &credential.refresh,
            access: &credential.access,
            expires: credential.expires,
            account_id: credential.account_id.as_deref(),
        },
    );

    serde_json::to_string(&entries)
        .map_err(|e| ModelProvidersError::InvalidAuthConfig(e.to_string()))
}
