use base64::engine::general_purpose::URL_SAFE_NO_PAD;
use base64::Engine;

use crate::services::model_providers::auth::openai_chatgpt::extract_account_id;

#[test]
fn extracts_account_id_from_jwt_claims() {
    let token = jwt_with_claims(serde_json::json!({"chatgpt_account_id": "acct_1"}));

    assert_eq!(extract_account_id(&token), Some("acct_1".to_owned()));
}

#[test]
fn extracts_account_id_from_organizations_fallback() {
    let token = jwt_with_claims(serde_json::json!({"organizations": [{"id": "org_1"}]}));

    assert_eq!(extract_account_id(&token), Some("org_1".to_owned()));
}

fn jwt_with_claims(claims: serde_json::Value) -> String {
    format!(
        "{}.{}.sig",
        URL_SAFE_NO_PAD.encode(br#"{"alg":"none"}"#),
        URL_SAFE_NO_PAD.encode(claims.to_string())
    )
}
