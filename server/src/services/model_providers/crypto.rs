use std::fmt;

use aes_gcm::aead::{Aead, KeyInit};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use base64::engine::general_purpose::STANDARD;
use base64::Engine;
use pbkdf2::pbkdf2_hmac;
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::Sha256;

use crate::services::model_providers::errors::ModelProvidersError;

const NONCE_LEN: usize = 12;
const SALT_LEN: usize = 16;
const KDF_ITERATIONS: u32 = 100_000;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncryptedPayload {
    pub salt: String,
    pub nonce: String,
    pub ciphertext: String,
}

#[derive(Debug, Deserialize)]
struct EncryptedPayloadRef<'a> {
    salt: &'a str,
    nonce: &'a str,
    ciphertext: &'a str,
}

#[derive(Clone)]
pub struct CredentialCipher {
    secret: String,
}

impl fmt::Debug for CredentialCipher {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("CredentialCipher").finish_non_exhaustive()
    }
}

impl CredentialCipher {
    #[must_use]
    pub fn new(secret: &str) -> Self {
        Self {
            secret: secret.to_owned(),
        }
    }

    pub fn encrypt_json<T>(&self, value: &T) -> Result<serde_json::Value, ModelProvidersError>
    where
        T: Serialize,
    {
        let plaintext = serde_json::to_vec(value).map_err(|_| ModelProvidersError::Crypto)?;
        let mut salt_bytes = [0_u8; SALT_LEN];
        rand::thread_rng().fill_bytes(&mut salt_bytes);
        let mut nonce_bytes = [0_u8; NONCE_LEN];
        rand::thread_rng().fill_bytes(&mut nonce_bytes);
        let nonce = Nonce::from_slice(&nonce_bytes);
        let cipher = self.cipher_for_salt(&salt_bytes);
        let ciphertext = cipher
            .encrypt(nonce, plaintext.as_ref())
            .map_err(|_| ModelProvidersError::Crypto)?;
        serde_json::to_value(EncryptedPayload {
            salt: STANDARD.encode(salt_bytes),
            nonce: STANDARD.encode(nonce_bytes),
            ciphertext: STANDARD.encode(ciphertext),
        })
        .map_err(|_| ModelProvidersError::Crypto)
    }

    pub fn decrypt_json<T>(&self, value: &serde_json::Value) -> Result<T, ModelProvidersError>
    where
        T: for<'de> Deserialize<'de>,
    {
        let payload =
            EncryptedPayloadRef::deserialize(value).map_err(|_| ModelProvidersError::Crypto)?;
        let salt_bytes = STANDARD
            .decode(payload.salt)
            .map_err(|_| ModelProvidersError::Crypto)?;
        if salt_bytes.len() != SALT_LEN {
            return Err(ModelProvidersError::Crypto);
        }
        let nonce_bytes = STANDARD
            .decode(payload.nonce)
            .map_err(|_| ModelProvidersError::Crypto)?;
        if nonce_bytes.len() != NONCE_LEN {
            return Err(ModelProvidersError::Crypto);
        }
        let ciphertext = STANDARD
            .decode(payload.ciphertext)
            .map_err(|_| ModelProvidersError::Crypto)?;
        let cipher = self.cipher_for_salt(&salt_bytes);
        let plaintext = cipher
            .decrypt(Nonce::from_slice(&nonce_bytes), ciphertext.as_ref())
            .map_err(|_| ModelProvidersError::Crypto)?;
        serde_json::from_slice(&plaintext).map_err(|_| ModelProvidersError::Crypto)
    }

    #[must_use]
    fn cipher_for_salt(&self, salt: &[u8]) -> Aes256Gcm {
        let mut key = [0_u8; 32];
        pbkdf2_hmac::<Sha256>(self.secret.as_bytes(), salt, KDF_ITERATIONS, &mut key);
        Aes256Gcm::new(Key::<Aes256Gcm>::from_slice(&key))
    }
}
