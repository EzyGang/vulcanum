use crate::client::tests::app::{assert_header, TEAM_ID};
use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

#[tokio::test]
async fn github_auth_url_is_typed_and_team_scoped() {
    let (base_url, handle) = serve_once(
        "200 OK",
        r#"{"url":"https://github.com/apps/vulcanum/installations/new?state=short"}"#,
    );
    let response = ApiClient::new(base_url)
        .get_github_auth_url(TEAM_ID, "app-access")
        .await
        .expect("auth URL should parse");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with("GET /api/v1/github/auth-url "));
    assert_header(&wire, "x-team-id", &TEAM_ID.to_string());
    assert!(response.url.contains("state=short"));
}

#[tokio::test]
async fn github_delete_accepts_no_content() {
    let (base_url, handle) = serve_once("204 No Content", "");
    ApiClient::new(base_url)
        .delete_github_app_installation(TEAM_ID, 9, "app-access")
        .await
        .expect("GitHub delete should accept 204");
    let wire = handle.join().expect("server should finish");

    assert!(wire.starts_with("DELETE /api/v1/github/installation/9 "));
    assert_header(&wire, "authorization", "Bearer app-access");
}
