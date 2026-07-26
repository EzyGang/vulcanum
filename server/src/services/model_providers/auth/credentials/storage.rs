use super::*;

pub(super) fn parse_stored_auth(
    stored: StoredAuth,
    cipher: &SecretCipher,
) -> Result<ParsedAuth, ModelProvidersError> {
    if stored.schema_version != 1 {
        return Err(ModelProvidersError::InvalidAuthConfig(
            "unsupported model provider auth schema".to_owned(),
        ));
    }

    match stored.auth_type {
        ModelProviderAuthType::ApiKey => {
            let api_key = stored.api_key.ok_or_else(|| {
                ModelProvidersError::InvalidAuthConfig("missing api key auth".to_owned())
            })?;
            let mut credentials: HashMap<String, String> = HashMap::new();
            for field in &api_key.fields {
                let normalized_field = normalize_credential_env_key(field)?;
                let secret = api_key
                    .secrets
                    .get(field)
                    .or_else(|| api_key.secrets.get(normalized_field.as_str()))
                    .ok_or_else(|| {
                        ModelProvidersError::InvalidAuthConfig(format!(
                            "missing encrypted credential for {normalized_field}"
                        ))
                    })?;
                if credentials
                    .insert(normalized_field.clone(), cipher.decrypt(secret)?)
                    .is_some()
                {
                    return Err(ModelProvidersError::InvalidAuthConfig(format!(
                        "duplicate encrypted credential for {normalized_field}"
                    )));
                }
            }
            for secret_field in api_key.secrets.keys() {
                let normalized = normalize_credential_env_key(secret_field)?;
                if !api_key.fields.iter().any(|field| {
                    normalize_credential_env_key(field)
                        .map(|candidate| candidate == normalized)
                        .unwrap_or(false)
                }) {
                    return Err(ModelProvidersError::InvalidAuthConfig(format!(
                        "undeclared encrypted credential for {normalized}"
                    )));
                }
            }
            Ok(ParsedAuth::ApiKey(credentials))
        }
        ModelProviderAuthType::DeviceOauth => {
            let oauth = stored.device_oauth.ok_or_else(|| {
                ModelProvidersError::InvalidAuthConfig("missing oauth auth".to_owned())
            })?;
            Ok(ParsedAuth::DeviceOAuth(OAuthCredential {
                provider: oauth.provider,
                account_id: oauth.account_id,
                email: oauth.email,
                expires: oauth.expires,
                refresh: cipher.decrypt(&oauth.refresh)?,
                access: cipher.decrypt(&oauth.access)?,
            }))
        }
        ModelProviderAuthType::None => Ok(ParsedAuth::None),
    }
}

pub(super) fn parse_legacy_api_key_auth(
    credentials: &serde_json::Value,
) -> Result<ParsedAuth, ModelProvidersError> {
    let api_keys = parse_api_key_credentials(credentials)?;

    match api_keys.is_empty() {
        true => Ok(ParsedAuth::None),
        false => Ok(ParsedAuth::ApiKey(api_keys)),
    }
}

pub(super) fn parse_api_key_credentials(
    credentials: &serde_json::Value,
) -> Result<HashMap<String, String>, ModelProvidersError> {
    let object = credentials.as_object().ok_or_else(|| {
        ModelProvidersError::InvalidAuthConfig("credentials must be an object".to_owned())
    })?;
    let mut api_keys: HashMap<String, String> = HashMap::new();
    for (key, value) in object {
        let normalized_key = normalize_credential_env_key(key)?;
        match value.as_str() {
            Some(secret) if !secret.is_empty() => {
                if api_keys
                    .insert(normalized_key.clone(), secret.to_owned())
                    .is_some()
                {
                    return Err(ModelProvidersError::InvalidAuthConfig(format!(
                        "duplicate credential field {normalized_key}"
                    )));
                }
            }
            Some(_) => (),
            None => {
                return Err(ModelProvidersError::InvalidAuthConfig(
                    "credential values must be strings".to_owned(),
                ));
            }
        }
    }
    Ok(api_keys)
}

pub(super) fn validate_credential_env_key(key: &str) -> Result<(), ModelProvidersError> {
    if key.is_empty()
        || !key
            .chars()
            .all(|ch| ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_')
        || key.starts_with('_')
        || key.chars().next().is_some_and(|ch| ch.is_ascii_digit())
    {
        return Err(ModelProvidersError::InvalidAuthConfig(format!(
            "invalid credential env field {key}"
        )));
    }
    if DANGEROUS_ENV_KEYS.contains(&key)
        || DANGEROUS_ENV_PREFIXES
            .iter()
            .any(|prefix| key.starts_with(prefix))
    {
        return Err(ModelProvidersError::InvalidAuthConfig(format!(
            "credential env field {key} is not allowed"
        )));
    }
    Ok(())
}

pub(super) fn encrypted_secrets_value(
    secrets: HashMap<String, EncryptedSecret>,
) -> serde_json::Map<String, serde_json::Value> {
    secrets
        .into_iter()
        .map(|(key, secret)| (key, encrypted_secret_value(&secret)))
        .collect()
}

pub(super) fn encrypted_secret_value(secret: &EncryptedSecret) -> serde_json::Value {
    serde_json::json!({
        "nonce": secret.nonce.as_str(),
        "ciphertext": secret.ciphertext.as_str(),
    })
}

pub(super) fn decode_legacy_snake_case_env_key(key: &str) -> Option<String> {
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
