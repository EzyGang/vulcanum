use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::services::model_providers::errors::ModelProvidersError;

const NONCE_LEN: usize = 12;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Clone)]
pub struct CredentialCipher {
    cipher: Aes256Gcm,
}

impl CredentialCipher {
    #[must_use]
    pub fn new(secret: &str) -> Self {
        let key = Sha256::digest(secret.as_bytes());
        Self {
            cipher: Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key)),
        }
    }

    pub fn encrypt_json<T>(&self, value: &T) -> Result<serde_json::Value, ModelProvidersError>
    where
        T: Serialize,
    {
        let plaintext = serde_json::to_vec(value).map_err(|_| ModelProvidersError::Crypto)?;
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let ciphertext = self
            .cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|_| ModelProvidersError::Crypto)?;
        serde_json::to_value(EncryptedPayload {
            nonce: STANDARD.encode(nonce_bytes),
            ciphertext: STANDARD.encode(ciphertext),
        })
        .map_err(|_| ModelProvidersError::Crypto)
    }

    pub fn decrypt_json<T>(&self, value: &serde_json::Value) -> Result<T, ModelProvidersError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let payload = serde_json::from_value::<EncryptedPayload>(value.clone())
            .map_err(|_| ModelProvidersError::Crypto)?;
        let nonce_bytes = STANDARD
            .decode(payload.nonce)
            .map_err(|_| ModelProvidersError::Crypto)?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(ModelProvidersError::Crypto);
        }
        let ciphertext = STANDARD
            .decode(payload.ciphertext)
            .map_err(|_| ModelProvidersError::Crypto)?;
        let plaintext = self
            .cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| ModelProvidersError::Crypto)?;
        serde_json::from_slice(&plaintext).map_err(|_| ModelProvidersError::Crypto)
    }
}
