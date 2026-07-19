use std::cell::RefCell;
use std::io::{self, Write};
use std::rc::Rc;
use std::time::Duration;

use chrono::{TimeZone, Utc};
use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::settings::device_oauth::connect_openai_with;
use crate::commands::app::settings::github::{connect_with, disconnect_with};
use crate::commands::app::settings::runtime::SettingsRuntime;
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const ATTEMPT_ID: Uuid = Uuid::from_u128(60);
const PROVIDER_ID: Uuid = Uuid::from_u128(61);
const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;
const TEAM: &str = r#"{"id":"00000000-0000-0000-0000-00000000002a","name":"Core","primary_model_provider_key":null,"primary_model_id":null,"small_model_provider_key":null,"small_model_id":null}"#;

#[tokio::test]
async fn openai_device_flow_waits_for_pending_then_reports_connected() {
    let start = format!(
        r#"{{"attempt_id":"{ATTEMPT_ID}","verification_uri":"https://example.test/device","user_code":"ABCD","interval_seconds":5,"expires_at":"2030-01-01T01:00:00Z"}}"#
    );
    let connected = format!(
        r#"{{"status":"connected","provider":{{"id":"{PROVIDER_ID}","display_name":"ChatGPT Plus","provider_key":"openai","auth_type":"device_oauth","credential_fields":[],"oauth":{{"account_id":"acct","email":null}}}}}}"#
    );
    let server = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok("POST", "/api/v1/model-providers/device-flows", start),
        Response::ok(
            "POST",
            &format!("/api/v1/model-providers/device-flows/{ATTEMPT_ID}/poll"),
            r#"{"status":"pending","next_poll_at":"2030-01-01T00:00:10Z"}"#,
        ),
        Response::ok(
            "POST",
            &format!("/api/v1/model-providers/device-flows/{ATTEMPT_ID}/poll"),
            connected,
        ),
    ]);
    let mut harness = Harness::new(&server.url);
    let sleeps = Rc::new(RefCell::new(Vec::new()));
    let recorded = Rc::clone(&sleeps);
    let mut settings = runtime();
    settings.sleep = Box::new(move |duration| {
        recorded.borrow_mut().push(duration);
        Box::pin(async {})
    });
    harness
        .run_openai(&mut settings)
        .await
        .expect("device flow should connect");
    let requests = server.finish();

    assert_eq!(
        &*sleeps.borrow(),
        &[Duration::from_secs(5), Duration::from_secs(10)]
    );
    assert_eq!(
        requests[2].body,
        r#"{"provider_key":"openai","device_provider":"openai_chatgpt","display_name":"CLI OpenAI"}"#
    );
    assert!(harness.output().contains("ABCD"));
    assert!(harness.output().contains("Connected OpenAI"));
}

#[tokio::test]
async fn github_connect_prints_manual_url_and_browser_failure_is_non_fatal() {
    let server = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok(
            "GET",
            "/api/v1/github/auth-url",
            r#"{"url":"https://github.test/install?state=short"}"#,
        ),
    ]);
    let mut harness = Harness::new(&server.url);
    let stderr = Rc::new(RefCell::new(Vec::new()));
    let mut settings = runtime();
    settings.stderr = Box::new(SharedWriter(Rc::clone(&stderr)));
    settings.open_browser = Box::new(|_| anyhow::bail!("browser failed with private details"));
    harness
        .run_github_connect(&mut settings)
        .await
        .expect("browser failure should not abort");
    server.finish();

    assert!(harness.output().contains("state=short"));
    assert!(harness.output().contains("connection initiated"));
    let warning = String::from_utf8(stderr.borrow().clone()).expect("warning should be UTF-8");
    assert!(warning.contains("could not open the browser"));
    assert!(!warning.contains("private details"));
}

#[tokio::test]
async fn github_disconnect_is_idempotent_or_deletes_exact_installation() {
    let absent = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok("GET", "/api/v1/github/installation", "null"),
    ]);
    let mut harness = Harness::new(&absent.url);
    harness
        .run_github_disconnect()
        .await
        .expect("absent installation should succeed");
    absent.finish();
    assert!(harness.output().contains("already disconnected"));

    let present = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok(
            "GET",
            "/api/v1/github/installation",
            r#"{"id":9,"account_login":"octocat"}"#,
        ),
        Response::status(
            "DELETE",
            "/api/v1/github/installation/9",
            "204 No Content",
            "",
        ),
    ]);
    let mut harness = Harness::new(&present.url);
    harness
        .run_github_disconnect()
        .await
        .expect("installation should disconnect");
    let requests = present.finish();

    assert_eq!(
        requests.last().map(|request| request.target.as_str()),
        Some("/api/v1/github/installation/9")
    );
    assert!(harness.output().contains("octocat"));
}

struct Harness {
    loaded: AppSession,
    stdout: Vec<u8>,
    saved: Vec<AppSession>,
}

impl Harness {
    fn new(instance_url: &str) -> Self {
        Self {
            loaded: session(instance_url),
            stdout: Vec::new(),
            saved: Vec::new(),
        }
    }

    async fn run_openai(&mut self, settings: &mut SettingsRuntime) -> anyhow::Result<()> {
        let mut load = || Ok(Some(self.loaded.clone()));
        let mut save = |session: &AppSession| {
            self.saved.push(session.clone());
            Ok(())
        };
        let mut app = AppRuntime {
            stdout: &mut self.stdout,
            load_session: &mut load,
            save_session: &mut save,
        };
        connect_openai_with(Some("CLI OpenAI"), true, Some(TEAM_ID), &mut app, settings).await
    }

    async fn run_github_connect(&mut self, settings: &mut SettingsRuntime) -> anyhow::Result<()> {
        let mut load = || Ok(Some(self.loaded.clone()));
        let mut save = |session: &AppSession| {
            self.saved.push(session.clone());
            Ok(())
        };
        let mut app = AppRuntime {
            stdout: &mut self.stdout,
            load_session: &mut load,
            save_session: &mut save,
        };
        connect_with(false, Some(TEAM_ID), &mut app, settings).await
    }

    async fn run_github_disconnect(&mut self) -> anyhow::Result<()> {
        let mut load = || Ok(Some(self.loaded.clone()));
        let mut save = |session: &AppSession| {
            self.saved.push(session.clone());
            Ok(())
        };
        let mut app = AppRuntime {
            stdout: &mut self.stdout,
            load_session: &mut load,
            save_session: &mut save,
        };
        disconnect_with(Some(TEAM_ID), &mut app).await
    }

    fn output(&self) -> String {
        String::from_utf8(self.stdout.clone()).expect("output should be UTF-8")
    }
}

struct SharedWriter(Rc<RefCell<Vec<u8>>>);

impl Write for SharedWriter {
    fn write(&mut self, buffer: &[u8]) -> io::Result<usize> {
        self.0.borrow_mut().extend_from_slice(buffer);
        Ok(buffer.len())
    }

    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}

fn runtime() -> SettingsRuntime {
    SettingsRuntime {
        stdin_is_terminal: true,
        stderr: Box::new(Vec::<u8>::new()),
        read_stdin: Box::new(|| Ok(String::new())),
        prompt_hidden: Box::new(|_| Ok(String::new())),
        open_browser: Box::new(|_| Ok(())),
        sleep: Box::new(|_| Box::pin(async {})),
        now: Box::new(|| {
            Utc.with_ymd_and_hms(2030, 1, 1, 0, 0, 0)
                .single()
                .expect("valid time")
        }),
    }
}

fn refresh() -> Response {
    Response::ok("POST", "/api/v1/auth/refresh", REFRESHED)
}

fn team() -> Response {
    Response::ok("GET", &format!("/api/v1/teams/{TEAM_ID}"), TEAM)
}

fn session(instance_url: &str) -> AppSession {
    let mut session: AppSession = serde_json::from_value(serde_json::json!({
        "instance_url": instance_url,
        "access_token": "old-access",
        "refresh_token": "old-refresh",
        "refresh_expires_at": "2030-01-02T03:04:05Z"
    }))
    .expect("session should deserialize");
    session.team_id = Some(TEAM_ID);
    session
}
