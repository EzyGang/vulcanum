use base64::engine::general_purpose::STANDARD;
use base64::Engine;

use crate::services::model_providers::auth::encryption::SecretCipher;

#[test]
fn encrypt_decrypt_round_trip() {
    let key = STANDARD.encode([7_u8; 32]);
    let cipher = SecretCipher::new(&key).expect("valid key");

    let encrypted = cipher.encrypt("secret").expect("encrypt");
    let decrypted = cipher.decrypt(&encrypted).expect("decrypt");

    assert_eq!(decrypted, "secret");
}

#[test]
fn rejects_invalid_key_size() {
    let key = STANDARD.encode([7_u8; 31]);

    assert!(SecretCipher::new(&key).is_err());
}
