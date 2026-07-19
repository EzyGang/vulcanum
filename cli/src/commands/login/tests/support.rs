pub(super) use crate::tests::http::{FakeServer, Response};
use uuid::Uuid;
use vulcanum_shared::state::app::AppSession;

use super::super::{run_with, LoginRuntime};

pub(super) const STATUS: &str =
    r#"{"access_token_ttl_minutes":15,"code_ttl_minutes":10,"refresh_token_ttl_days":30}"#;
pub(super) const TOKENS: &str = r#"{"access_token":"access-secret","refresh_token":"refresh-secret","refresh_expires_at":"2030-01-02T03:04:05Z"}"#;

pub(super) struct Fixture {
    pub terminal: bool,
    pub stdin: String,
    pub prompt_instance: Option<String>,
    pub prompt_password: String,
    pub prompt_code: String,
    pub loaded: Option<AppSession>,
    pub load_error: bool,
    pub load_calls: usize,
    pub save_error: bool,
    pub open_error: bool,
    stdout: Vec<u8>,
    stderr: Vec<u8>,
    pub saved: Vec<AppSession>,
    pub opened: Vec<String>,
}

impl Fixture {
    pub fn new(terminal: bool) -> Self {
        Self {
            terminal,
            stdin: String::new(),
            prompt_instance: None,
            prompt_password: String::new(),
            prompt_code: String::new(),
            loaded: None,
            load_error: false,
            load_calls: 0,
            save_error: false,
            open_error: false,
            stdout: Vec::new(),
            stderr: Vec::new(),
            saved: Vec::new(),
            opened: Vec::new(),
        }
    }

    pub async fn run(
        &mut self,
        instance: Option<String>,
        password_stdin: bool,
        auth_code: Option<String>,
        no_browser: bool,
    ) -> anyhow::Result<()> {
        let mut read_line = || Ok(self.stdin.clone());
        let mut prompt_instance = |_| {
            self.prompt_instance
                .clone()
                .ok_or_else(|| anyhow::anyhow!("missing instance prompt"))
        };
        let mut prompt_password = || Ok(self.prompt_password.clone());
        let mut prompt_code = || Ok(self.prompt_code.clone());
        let mut open_browser = |url: &str| {
            self.opened.push(url.to_owned());
            match self.open_error {
                true => anyhow::bail!("browser unavailable"),
                false => Ok(()),
            }
        };
        let mut load_session = || {
            self.load_calls += 1;
            match self.load_error {
                true => anyhow::bail!("failed to parse state from C:\\broken\\app.json"),
                false => Ok(self.loaded.clone()),
            }
        };
        let mut save_session = |session: &AppSession| {
            if self.save_error {
                anyhow::bail!("save failed");
            }
            self.saved.push(session.clone());
            Ok(())
        };
        let mut runtime = LoginRuntime {
            stdin_is_terminal: self.terminal,
            stdout: &mut self.stdout,
            stderr: &mut self.stderr,
            read_line: &mut read_line,
            prompt_instance: &mut prompt_instance,
            prompt_password: &mut prompt_password,
            prompt_code: &mut prompt_code,
            open_browser: &mut open_browser,
            load_session: &mut load_session,
            save_session: &mut save_session,
        };
        run_with(
            instance,
            password_stdin,
            auth_code,
            no_browser,
            &mut runtime,
        )
        .await
    }

    pub fn assert_saved_session(&self, instance_url: &str, team_id: Option<Uuid>) {
        let mut expected = session(instance_url);
        expected.team_id = team_id;
        assert_eq!(self.saved, vec![expected]);
    }

    pub fn assert_secrets_not_emitted(&self, secrets: &[&str]) {
        let output = format!("{}{}", self.stdout_text(), self.stderr_text());
        for secret in secrets {
            assert!(!output.contains(secret), "output leaked {secret}");
        }
    }

    pub fn stdout_text(&self) -> String {
        String::from_utf8(self.stdout.clone()).expect("stdout should be utf8")
    }

    pub fn stderr_text(&self) -> String {
        String::from_utf8(self.stderr.clone()).expect("stderr should be utf8")
    }
}

pub(super) fn session(instance_url: &str) -> AppSession {
    serde_json::from_value(serde_json::json!({
        "instance_url": instance_url,
        "access_token": "access-secret",
        "refresh_token": "refresh-secret",
        "refresh_expires_at": "2030-01-02T03:04:05Z",
    }))
    .expect("session should deserialize")
}
