use aes_gcm::aead::{Aead, AeadCore, KeyInit, OsRng};
use aes_gcm::{Aes256Gcm, Nonce};
use base64::engine::general_purpose::{STANDARD, URL_SAFE_NO_PAD};
use base64::Engine;

use crate::models::model_providers::errors::ModelProvidersError;

const KEY_LEN: usize = 32;
const NONCE_LEN: usize = 12;

#[derive(Clone)]
pub struct SecretCipher {
    cipher: Aes256Gcm,
}

#[derive(Clone, Debug, serde::Deserialize)]
pub struct EncryptedSecret {
    pub nonce: String,
    pub ciphertext: String,
}

impl SecretCipher {
    pub fn new(encoded_key: &str) -> Result<Self, ModelProvidersError> {
        let key = STANDARD.decode(encoded_key).map_err(|_| {
            ModelProvidersError::SecretEncryption("invalid base64 secret key".to_owned())
        })?;
        if key.len() != KEY_LEN {
            return Err(ModelProvidersError::SecretEncryption(format!(
                "MODEL_PROVIDER_SECRET_KEY must decode to {KEY_LEN} bytes"
            )));
        }

        Ok(Self {
            cipher: Aes256Gcm::new_from_slice(&key).map_err(|_| {
                ModelProvidersError::SecretEncryption("invalid secret key".to_owned())
            })?,
        })
    }

    pub fn encrypt(&self, plaintext: &str) -> Result<EncryptedSecret, ModelProvidersError> {
        let nonce = Aes256Gcm::generate_nonce(&mut OsRng);
        let ciphertext = self
            .cipher
            .encrypt(&nonce, plaintext.as_bytes())
            .map_err(|_| ModelProvidersError::SecretEncryption("encryption failed".to_owned()))?;

        Ok(EncryptedSecret {
            nonce: URL_SAFE_NO_PAD.encode(nonce),
            ciphertext: URL_SAFE_NO_PAD.encode(ciphertext),
        })
    }

    pub fn decrypt(&self, secret: &EncryptedSecret) -> Result<String, ModelProvidersError> {
        let nonce_bytes = URL_SAFE_NO_PAD
            .decode(&secret.nonce)
            .map_err(|_| ModelProvidersError::SecretDecryption)?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(ModelProvidersError::SecretDecryption);
        }

        let ciphertext = URL_SAFE_NO_PAD
            .decode(&secret.ciphertext)
            .map_err(|_| ModelProvidersError::SecretDecryption)?;
        let plaintext = self
            .cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| ModelProvidersError::SecretDecryption)?;

        String::from_utf8(plaintext).map_err(|_| ModelProvidersError::SecretDecryption)
    }
}
