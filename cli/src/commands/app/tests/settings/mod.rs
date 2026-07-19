use uuid::Uuid;
use vulcanum_shared::constants::DEFAULT_TEAM_ID;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::settings::{clear_team_with, list_with, set_team_with};
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const OLD_TEAM_ID: Uuid = Uuid::from_u128(41);
const REFRESHED: &str = r#"{"access_token":"new-access-secret","refresh_token":"new-refresh-secret","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;

#[tokio::test]
async fn settings_list_renders_complete_redacted_snapshot() {
    let server = FakeServer::start(vec![
        refresh_response(),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok(
            "GET",
            "/api/v1/providers",
            r#"[{"id":"00000000-0000-0000-0000-000000000031","name":"Linear\nProd","provider_type":"linear","instance_url":"https://user:password@linear.example/path?api_key=tracker-secret#fragment","api_key":"tracker-api-key"}]"#,
        ),
        Response::ok(
            "GET",
            "/api/v1/model-providers",
            r#"[{"id":"00000000-0000-0000-0000-000000000032","display_name":"OpenAI","provider_key":"openai","auth_type":"device_oauth","credential_fields":["organization"],"oauth":{"account_id":"account-secret","email":"dev@example.com"},"credential_values":{"api_key":"model-secret"}}]"#,
        ),
        Response::ok("GET", "/api/v1/github/installation", "null"),
    ]);
    let mut fixture = Fixture::new(session(&server.url, Some(TEAM_ID)));

    fixture
        .run_list(None)
        .await
        .expect("settings list should succeed");
    let requests = server.finish();
    let output = fixture.output();

    assert_eq!(requests.len(), 5);
    let scoped_request = requests
        .iter()
        .find(|request| request.target == "/api/v1/providers")
        .expect("scoped request should be recorded");
    assert_eq!(
        scoped_request
            .headers
            .get("authorization")
            .map(String::as_str),
        Some("Bearer new-access-secret")
    );
    assert_eq!(
        scoped_request.headers.get("x-team-id").map(String::as_str),
        Some(TEAM_ID.to_string().as_str())
    );
    assert_ordered(
        &output,
        &[
            "Settings —",
            "Model Selection",
            "Task Trackers",
            "Model Providers",
            "GitHub App",
        ],
    );
    assert!(output.contains("Primary provider: OpenAI (openai)"));
    assert!(output.contains("Primary model: gpt-5"));
    assert!(output.contains("Small provider: -"));
    assert!(output.contains("Small model: -"));
    assert!(output.contains(r"Linear\nProd"));
    assert!(output.contains("https://linear.example/path"));
    assert!(output.contains("organization"));
    assert!(output.contains("dev@example.com"));
    assert!(output.contains("Status: disconnected"));
    for secret in [
        "old-access",
        "old-refresh",
        "new-access-secret",
        "new-refresh-secret",
        "user",
        "password",
        "tracker-secret",
        "tracker-api-key",
        "model-secret",
        "account-secret",
        "fragment",
    ] {
        assert!(!output.contains(secret), "output leaked {secret}");
    }
}

#[tokio::test]
async fn failing_settings_endpoint_writes_no_partial_snapshot() {
    let server = FakeServer::start(vec![
        refresh_response(),
        Response::ok("GET", &team_target(), team_response()),
        Response::ok("GET", "/api/v1/providers", "[]"),
        Response::status(
            "GET",
            "/api/v1/model-providers",
            "500 Internal Server Error",
            r#"{"error":"credential-secret"}"#,
        ),
        Response::ok("GET", "/api/v1/github/installation", "null"),
    ]);
    let mut fixture = Fixture::new(session(&server.url, Some(TEAM_ID)));

    let error = fixture
        .run_list(None)
        .await
        .expect_err("settings endpoint should fail");
    server.finish();

    assert_eq!(error.to_string(), "List model providers failed: HTTP 500");
    assert_eq!(fixture.output(), "");
    assert!(!format!("{error:?}").contains("credential-secret"));
}

#[tokio::test]
async fn team_set_validates_and_persists_selected_team() {
    let server = FakeServer::start(vec![
        refresh_response(),
        Response::ok("GET", &team_target(), team_response()),
    ]);
    let mut fixture = Fixture::new(session(&server.url, Some(OLD_TEAM_ID)));

    fixture
        .run_set(TEAM_ID)
        .await
        .expect("team set should succeed");
    server.finish();

    assert_eq!(fixture.saved.len(), 2);
    assert_eq!(fixture.saved[0].team_id, Some(OLD_TEAM_ID));
    assert_eq!(fixture.saved[1].team_id, Some(TEAM_ID));
    assert_eq!(fixture.output(), format!("Pinned team Core ({TEAM_ID}).\n"));
}

#[tokio::test]
async fn failed_team_set_retains_prior_pin_after_token_rotation() {
    let server = FakeServer::start(vec![
        refresh_response(),
        Response::status(
            "GET",
            &team_target(),
            "404 Not Found",
            r#"{"error":"membership-secret"}"#,
        ),
    ]);
    let mut fixture = Fixture::new(session(&server.url, Some(OLD_TEAM_ID)));

    let error = fixture
        .run_set(TEAM_ID)
        .await
        .expect_err("inaccessible team should fail");
    server.finish();

    assert_eq!(fixture.saved.len(), 1);
    assert_eq!(fixture.saved[0].team_id, Some(OLD_TEAM_ID));
    assert!(error.to_string().contains(&TEAM_ID.to_string()));
    assert!(!format!("{error:?}").contains("membership-secret"));
}

#[tokio::test]
async fn team_clear_only_changes_pin_for_each_auth_mode() {
    for (single_user, expected_pin, expected_output) in [
        (false, None, "Cleared the pinned team.\n".to_owned()),
        (
            true,
            Some(DEFAULT_TEAM_ID),
            format!("Reset the pinned team to {DEFAULT_TEAM_ID}.\n"),
        ),
    ] {
        let server = FakeServer::start(vec![Response::ok(
            "GET",
            "/api/v1/auth/mode",
            format!(r#"{{"is_single_user":{single_user}}}"#),
        )]);
        let original = session(&server.url, Some(OLD_TEAM_ID));
        let mut fixture = Fixture::new(original.clone());

        fixture
            .run_clear()
            .await
            .expect("team clear should succeed");
        server.finish();

        let saved = fixture.saved.last().expect("session should save");
        assert_eq!(saved.team_id, expected_pin);
        assert_eq!(saved.instance_url, original.instance_url);
        assert_eq!(saved.access_token, original.access_token);
        assert_eq!(saved.refresh_token, original.refresh_token);
        assert_eq!(fixture.output(), expected_output);
    }
}

struct Fixture {
    loaded: AppSession,
    stdout: Vec<u8>,
    saved: Vec<AppSession>,
}

impl Fixture {
    fn new(loaded: AppSession) -> Self {
        Self {
            loaded,
            stdout: Vec::new(),
            saved: Vec::new(),
        }
    }

    async fn run_list(&mut self, team: Option<Uuid>) -> anyhow::Result<()> {
        let mut load = || Ok(Some(self.loaded.clone()));
        let mut save = |session: &AppSession| {
            self.saved.push(session.clone());
            Ok(())
        };
        let mut runtime = AppRuntime {
            stdout: &mut self.stdout,
            load_session: &mut load,
            save_session: &mut save,
        };
        list_with(team, &mut runtime).await
    }

    async fn run_set(&mut self, team: Uuid) -> anyhow::Result<()> {
        let mut load = || Ok(Some(self.loaded.clone()));
        let mut save = |session: &AppSession| {
            self.saved.push(session.clone());
            Ok(())
        };
        let mut runtime = AppRuntime {
            stdout: &mut self.stdout,
            load_session: &mut load,
            save_session: &mut save,
        };
        set_team_with(team, &mut runtime).await
    }

    async fn run_clear(&mut self) -> anyhow::Result<()> {
        let mut load = || Ok(Some(self.loaded.clone()));
        let mut save = |session: &AppSession| {
            self.saved.push(session.clone());
            Ok(())
        };
        let mut runtime = AppRuntime {
            stdout: &mut self.stdout,
            load_session: &mut load,
            save_session: &mut save,
        };
        clear_team_with(&mut runtime).await
    }

    fn output(&self) -> String {
        String::from_utf8(self.stdout.clone()).expect("output should be utf8")
    }
}

fn refresh_response() -> Response {
    Response::ok("POST", "/api/v1/auth/refresh", REFRESHED)
}

fn team_target() -> String {
    format!("/api/v1/teams/{TEAM_ID}")
}

fn team_response() -> &'static str {
    r#"{"id":"00000000-0000-0000-0000-00000000002a","name":"Core","primary_model_provider_key":"openai","primary_model_id":"gpt-5","small_model_provider_key":null,"small_model_id":null}"#
}

fn session(instance_url: &str, team_id: Option<Uuid>) -> AppSession {
    let mut session: AppSession = serde_json::from_value(serde_json::json!({
        "instance_url": instance_url,
        "access_token": "old-access",
        "refresh_token": "old-refresh",
        "refresh_expires_at": "2030-01-02T03:04:05Z",
    }))
    .expect("session should deserialize");
    session.team_id = team_id;
    session
}

fn assert_ordered(haystack: &str, needles: &[&str]) {
    let mut start = 0;
    for needle in needles {
        let index = haystack[start..]
            .find(needle)
            .unwrap_or_else(|| panic!("missing {needle} in {haystack}"));
        start += index + needle.len();
    }
}
