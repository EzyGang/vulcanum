use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::models::model_providers::errors::ModelProvidersError;
use crate::models::model_providers::model::ModelProviderConfig;
use crate::services::model_providers::auth::credentials::{parse_auth, to_response, ParsedAuth};
use crate::services::model_providers::auth::encryption::SecretCipher;

#[test]
fn legacy_plaintext_credentials_parse_as_api_key_auth() {
    let cipher = test_cipher();
    let auth =
        parse_auth(&serde_json::json!({"OPENAI_API_KEY": "secret"}), &cipher).expect("parse auth");

    match auth {
        ParsedAuth::ApiKey(credentials) => {
            assert_eq!(
                credentials.get("OPENAI_API_KEY"),
                Some(&"secret".to_owned())
            );
        }
        _ => panic!("expected api key auth"),
    }
}

#[test]
fn safe_dto_does_not_include_secret_fields() {
    let cipher = test_cipher();
    let provider = ModelProviderConfig {
        id: uuid::Uuid::new_v4(),
        team_id: uuid::Uuid::new_v4(),
        provider_key: "openai".to_owned(),
        display_name: "OpenAI".to_owned(),
        credentials: serde_json::json!({"OPENAI_API_KEY": "secret"}),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let response = to_response(provider, &cipher).expect("response");
    let value = serde_json::to_value(response).expect("serialize");

    assert!(value.get("credentials").is_none());
    assert_eq!(value["credential_fields"][0], "OPENAI_API_KEY");
}

#[test]
fn stored_api_key_auth_rejects_declared_fields_missing_encrypted_secret() {
    let cipher = test_cipher();
    let credentials = serde_json::json!({
        "schema_version": 1,
        "auth_type": "api_key",
        "api_key": {
            "fields": ["OPENAI_API_KEY"],
            "secrets": {},
        },
        "device_oauth": null,
    });

    assert_invalid_auth_config(
        parse_auth(&credentials, &cipher),
        "missing encrypted credential for OPENAI_API_KEY",
    );

    let provider = ModelProviderConfig {
        id: uuid::Uuid::new_v4(),
        team_id: uuid::Uuid::new_v4(),
        provider_key: "openai".to_owned(),
        display_name: "OpenAI".to_owned(),
        credentials,
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };
    assert_invalid_auth_config(
        to_response(provider, &cipher),
        "missing encrypted credential for OPENAI_API_KEY",
    );
}

fn test_cipher() -> SecretCipher {
    SecretCipher::new(&STANDARD.encode([3_u8; 32])).expect("valid cipher")
}

fn assert_invalid_auth_config<T>(result: Result<T, ModelProvidersError>, expected_message: &str) {
    match result {
        Err(ModelProvidersError::InvalidAuthConfig(message)) => {
            assert_eq!(message, expected_message);
        }
        _ => panic!("expected invalid auth config"),
    }
}
