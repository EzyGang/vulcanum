mod omp;
mod opencode;

use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::api::wire::{AgentBackend, AgentConfigPayload};

use crate::models::model_providers::model::ModelProviderConfig;
use crate::services::model_providers::auth::credentials::{
    encrypted_oauth_credentials, OAuthCredential,
};
use crate::services::model_providers::auth::encryption::SecretCipher;
use crate::services::model_providers::renderer::{
    render_agent_config, render_opencode_config, ModelSelection,
};
fn test_cipher() -> SecretCipher {
    SecretCipher::new("AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA=").expect("test cipher")
}
