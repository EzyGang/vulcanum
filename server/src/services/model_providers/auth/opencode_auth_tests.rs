use crate::services::model_providers::auth::credentials::OAuthCredential;
use crate::services::model_providers::auth::opencode_auth::openai_auth_content;

#[test]
fn materializes_opencode_auth_json() {
    let content = openai_auth_content(&OAuthCredential {
        provider: "openai_chatgpt".to_owned(),
        account_id: Some("acct".to_owned()),
        email: None,
        expires: 1782942233000,
        refresh: "refresh".to_owned(),
        access: "access".to_owned(),
    })
    .expect("auth content");
    let value: serde_json::Value = serde_json::from_str(&content).expect("json");

    assert_eq!(value["openai"]["type"], "oauth");
    assert_eq!(value["openai"]["refresh"], "refresh");
    assert_eq!(value["openai"]["access"], "access");
    assert_eq!(value["openai"]["expires"], 1782942233000_i64);
    assert_eq!(value["openai"]["accountId"], "acct");
}
