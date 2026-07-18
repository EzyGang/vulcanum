use chrono::Utc;
use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use crate::commands::app::args::DirectModelProviderAuth;
use crate::commands::app::settings::model_providers::{self, UpdateOptions};
use crate::commands::app::settings::models::{self, ModelSlot};
use crate::commands::app::settings::runtime::SettingsRuntime;
use crate::commands::app::settings::task_trackers;
use crate::commands::app::AppRuntime;
use crate::tests::http::{FakeServer, Response};

const TEAM_ID: Uuid = Uuid::from_u128(42);
const TRACKER_ID: Uuid = Uuid::from_u128(49);
const PROVIDER_ID: Uuid = Uuid::from_u128(50);
const REFRESHED: &str = r#"{"access_token":"new-access","refresh_token":"new-refresh","refresh_expires_at":"2031-02-03T04:05:06Z"}"#;
const TEAM: &str = r#"{"id":"00000000-0000-0000-0000-00000000002a","name":"Core","primary_model_provider_key":null,"primary_model_id":null,"small_model_provider_key":null,"small_model_id":null}"#;
const PROVIDER: &str = r#"{"id":"00000000-0000-0000-0000-000000000032","display_name":"OpenAI","provider_key":"openai","auth_type":"api_key","credential_fields":["OPENAI_API_KEY"],"oauth":null}"#;

#[tokio::test]
async fn model_set_validates_connection_and_catalog_before_atomic_patch() {
    let server = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok("GET", "/api/v1/model-providers", format!("[{PROVIDER}]")),
        Response::ok(
            "GET",
            "/api/v1/model-providers/catalog",
            r#"{"providers":[{"id":"openai","name":"OpenAI","env":["OPENAI_API_KEY"],"models":[{"id":"gpt-5","name":"GPT-5"}]}]}"#,
        ),
        Response::ok("PATCH", &format!("/api/v1/teams/{TEAM_ID}"), TEAM),
    ]);
    let mut harness = Harness::new(&server.url);
    harness
        .run_model_set(ModelSlot::Primary, "openai", "gpt-5")
        .await
        .expect("model set should succeed");
    let requests = server.finish();
    let patch = requests.last().expect("patch should be sent");

    assert_eq!(
        patch.body,
        r#"{"primary_model_provider_key":"openai","primary_model_id":"gpt-5"}"#
    );
    assert_eq!(patch.headers.get("x-team-id"), Some(&TEAM_ID.to_string()));
    assert!(harness.output().contains("openai/gpt-5"));
    assert_eq!(harness.saved.len(), 1);
}

#[tokio::test]
async fn model_clear_sends_null_pair_and_omits_other_slot() {
    let server = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok("PATCH", &format!("/api/v1/teams/{TEAM_ID}"), TEAM),
    ]);
    let mut harness = Harness::new(&server.url);
    harness
        .run_model_clear(ModelSlot::Small)
        .await
        .expect("model clear should succeed");
    let patch = server.finish().pop().expect("patch should be sent");

    assert_eq!(
        patch.body,
        r#"{"small_model_provider_key":null,"small_model_id":null}"#
    );
}

#[tokio::test]
async fn tracker_add_reads_secret_after_team_resolution_and_never_outputs_it() {
    let response = format!(
        r#"{{"id":"{TRACKER_ID}","name":"Kaneo","provider_type":"kaneo","instance_url":"https://tasks.example"}}"#
    );
    let server = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok("POST", "/api/v1/providers", response),
    ]);
    let mut harness = Harness::new(&server.url);
    let mut settings = settings_runtime(r#"{"api_key":"tracker-secret"}"#);
    harness
        .run_tracker_add(&mut settings)
        .await
        .expect("tracker add should succeed");
    let request = server.finish().pop().expect("create should be sent");
    let output = harness.output();

    assert_eq!(
        request.body,
        r#"{"name":"Kaneo","instance_url":"https://tasks.example","api_key":"tracker-secret"}"#
    );
    assert!(!output.contains("tracker-secret"));
}

#[tokio::test]
async fn provider_update_to_none_sends_no_credentials_and_uses_exact_id() {
    let updated = format!(
        r#"{{"id":"{PROVIDER_ID}","display_name":"OpenAI","provider_key":"openai","auth_type":"none","credential_fields":[],"oauth":null}}"#
    );
    let existing = format!(
        r#"{{"id":"{PROVIDER_ID}","display_name":"OpenAI","provider_key":"openai","auth_type":"api_key","credential_fields":["OPENAI_API_KEY"],"oauth":null}}"#
    );
    let server = FakeServer::start(vec![
        refresh(),
        team(),
        Response::ok("GET", "/api/v1/model-providers", format!("[{existing}]")),
        Response::ok(
            "PATCH",
            &format!("/api/v1/model-providers/{PROVIDER_ID}"),
            updated,
        ),
    ]);
    let mut harness = Harness::new(&server.url);
    let mut settings = settings_runtime("");
    harness
        .run_provider_update(&mut settings)
        .await
        .expect("provider update should succeed");
    let request = server.finish().pop().expect("update should be sent");

    assert_eq!(request.body, r#"{"auth_type":"none"}"#);
    assert_eq!(
        request.target,
        format!("/api/v1/model-providers/{PROVIDER_ID}")
    );
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

    async fn run_model_set(
        &mut self,
        slot: ModelSlot,
        provider_key: &str,
        model_id: &str,
    ) -> anyhow::Result<()> {
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
        models::set_with(slot, provider_key, model_id, Some(TEAM_ID), &mut app).await
    }

    async fn run_model_clear(&mut self, slot: ModelSlot) -> anyhow::Result<()> {
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
        models::clear_with(slot, Some(TEAM_ID), &mut app).await
    }

    async fn run_tracker_add(&mut self, settings: &mut SettingsRuntime) -> anyhow::Result<()> {
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
        task_trackers::add_with(
            "Kaneo",
            "https://tasks.example",
            true,
            Some(TEAM_ID),
            &mut app,
            settings,
        )
        .await
    }

    async fn run_provider_update(&mut self, settings: &mut SettingsRuntime) -> anyhow::Result<()> {
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
        model_providers::update_with(
            UpdateOptions {
                id: PROVIDER_ID,
                name: None,
                auth: Some(DirectModelProviderAuth::None),
                credentials_stdin: false,
                prompt_credentials: false,
                team: Some(TEAM_ID),
            },
            &mut app,
            settings,
        )
        .await
    }

    fn output(&self) -> String {
        String::from_utf8(self.stdout.clone()).expect("output should be UTF-8")
    }
}

fn settings_runtime(input: &str) -> SettingsRuntime {
    let input = input.to_owned();
    SettingsRuntime {
        stdin_is_terminal: false,
        stderr: Box::new(Vec::<u8>::new()),
        read_stdin: Box::new(move || Ok(input.clone())),
        prompt_hidden: Box::new(|_| Ok(String::new())),
        open_browser: Box::new(|_| Ok(())),
        sleep: Box::new(|_| Box::pin(async {})),
        now: Box::new(Utc::now),
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
