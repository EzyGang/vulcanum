mod contracts;
mod mutations;

use uuid::Uuid;

use crate::client::tests::support::serve_once;
use crate::client::ApiClient;

const TEAM_ID: Uuid = Uuid::from_u128(1);
const TOKEN_RESPONSE: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2030-01-02T03:04:05Z"}"#;
const TEAM_RESPONSE: &str = r#"{"id":"00000000-0000-0000-0000-000000000001","name":"Core","primary_model_provider_key":"openai","primary_model_id":"gpt-5","small_model_provider_key":null,"small_model_id":null,"ignored":true}"#;

#[tokio::test]
async fn app_refresh_posts_refresh_token() {
    let (base_url, handle) = serve_once("200 OK", TOKEN_RESPONSE);

    let tokens = ApiClient::new(base_url)
        .refresh_app_session("old-refresh")
        .await
        .expect("refresh should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with("POST /api/v1/auth/refresh "));
    assert!(request.ends_with(r#"{"refresh_token":"old-refresh"}"#));
    assert_eq!(tokens.access_token, "new-access");
}

#[tokio::test]
async fn team_reads_send_bearer_auth_and_parse_extra_fields() {
    let body = format!("[{TEAM_RESPONSE}]");
    let (base_url, handle) = serve_once("200 OK", body);
    let teams = ApiClient::new(base_url)
        .list_teams("app-access")
        .await
        .expect("teams should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with("GET /api/v1/teams "));
    assert_header(&request, "authorization", "Bearer app-access");
    assert_eq!(teams[0].name, "Core");

    let (base_url, handle) = serve_once("200 OK", TEAM_RESPONSE);
    let team = ApiClient::new(base_url)
        .get_team(TEAM_ID, "app-access")
        .await
        .expect("team should parse");
    let request = handle.join().expect("server should finish");

    assert!(request.starts_with(&format!("GET /api/v1/teams/{TEAM_ID} ")));
    assert_header(&request, "authorization", "Bearer app-access");
    assert_eq!(team.primary_model_id.as_deref(), Some("gpt-5"));
}

#[tokio::test]
async fn team_scoped_app_reads_send_team_and_bearer_headers() {
    let worker = r#"[{"id":"00000000-0000-0000-0000-000000000002","name":"build-a","last_seen":null,"status":"busy","active_jobs":1,"max_concurrent_jobs":3,"extra":"ignored"}]"#;
    let (base_url, handle) = serve_once("200 OK", worker);
    let workers = ApiClient::new(base_url)
        .list_workers(TEAM_ID, "app-access")
        .await
        .expect("workers should parse");
    assert_scoped_request(
        handle.join().expect("server should finish"),
        "/api/v1/workers",
    );
    assert_eq!(workers[0].active_jobs, 1);

    let tracker = r#"[{"id":"00000000-0000-0000-0000-000000000003","name":"Linear","provider_type":"linear","instance_url":"https://linear.app","api_key":"ignored"}]"#;
    let (base_url, handle) = serve_once("200 OK", tracker);
    let trackers = ApiClient::new(base_url)
        .list_task_trackers(TEAM_ID, "app-access")
        .await
        .expect("trackers should parse");
    assert_scoped_request(
        handle.join().expect("server should finish"),
        "/api/v1/providers",
    );
    assert_eq!(trackers[0].provider_type, "linear");

    let provider = r#"[{"id":"00000000-0000-0000-0000-000000000004","display_name":"OpenAI","provider_key":"openai","auth_type":"device_oauth","credential_fields":["organization"],"oauth":{"account_id":"acct","email":"dev@example.com"},"credential_values":{"secret":"ignored"}}]"#;
    let (base_url, handle) = serve_once("200 OK", provider);
    let providers = ApiClient::new(base_url)
        .list_model_providers(TEAM_ID, "app-access")
        .await
        .expect("providers should parse");
    assert_scoped_request(
        handle.join().expect("server should finish"),
        "/api/v1/model-providers",
    );
    assert_eq!(
        providers[0]
            .oauth
            .as_ref()
            .and_then(|oauth| oauth.email.as_deref()),
        Some("dev@example.com")
    );

    let installation = r#"{"account_login":"octocat","id":9}"#;
    let (base_url, handle) = serve_once("200 OK", installation);
    let github = ApiClient::new(base_url)
        .get_github_app_installation(TEAM_ID, "app-access")
        .await
        .expect("installation should parse");
    assert_scoped_request(
        handle.join().expect("server should finish"),
        "/api/v1/github/installation",
    );
    assert_eq!(
        github.expect("installation should exist").account_login,
        "octocat"
    );

    let (base_url, handle) = serve_once("200 OK", "null");
    let github = ApiClient::new(base_url)
        .get_github_app_installation(TEAM_ID, "app-access")
        .await
        .expect("null installation should parse");
    assert_scoped_request(
        handle.join().expect("server should finish"),
        "/api/v1/github/installation",
    );
    assert_eq!(github, None);
}

fn assert_scoped_request(request: String, target: &str) {
    assert!(request.starts_with(&format!("GET {target} ")));
    assert_header(&request, "authorization", "Bearer app-access");
    assert_header(&request, "x-team-id", TEAM_ID.to_string().as_str());
}

fn assert_header(request: &str, name: &str, value: &str) {
    let expected = format!("{name}: {value}").to_ascii_lowercase();
    assert!(
        request
            .to_ascii_lowercase()
            .lines()
            .any(|line| line == expected),
        "missing {name} header in request: {request}"
    );
}
