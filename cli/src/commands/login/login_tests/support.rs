use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use vulcanum_shared::app_state::AppSession;

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

    pub fn assert_saved_session(&self, instance_url: &str) {
        assert_eq!(self.saved, vec![session(instance_url)]);
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

#[derive(Clone)]
pub(super) struct Response {
    status: &'static str,
    body: String,
}

impl Response {
    pub fn ok(body: &str) -> Self {
        Self {
            status: "200 OK",
            body: body.to_owned(),
        }
    }

    pub fn unauthorized(message: &str) -> Self {
        Self {
            status: "401 Unauthorized",
            body: format!(r#"{{"error":"{message}"}}"#),
        }
    }
}

pub(super) struct Request {
    pub target: String,
    pub body: String,
}

pub(super) struct FakeServer {
    pub url: String,
    handle: thread::JoinHandle<Vec<Request>>,
}

impl FakeServer {
    pub fn start(responses: Vec<Response>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let url = format!("http://{}", listener.local_addr().expect("test address"));
        let handle = thread::spawn(move || {
            responses
                .into_iter()
                .map(|response| {
                    let (mut stream, _) = listener.accept().expect("request should connect");
                    let request = read_request(&mut stream);
                    write!(
                        stream,
                        "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                        response.status,
                        response.body.len(),
                        response.body
                    )
                    .expect("response should write");
                    request
                })
                .collect()
        });
        Self { url, handle }
    }

    pub fn finish(self) -> Vec<Request> {
        self.handle.join().expect("server should finish")
    }
}

fn read_request(stream: &mut TcpStream) -> Request {
    let mut bytes = Vec::new();
    let mut buffer = [0_u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut buffer).expect("request should read");
        bytes.extend_from_slice(&buffer[..read]);
        let Some(index) = bytes.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        break index + 4;
    };
    let headers = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
    let content_length = headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length: ")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(0);
    while bytes.len() < header_end + content_length {
        let read = stream.read(&mut buffer).expect("body should read");
        bytes.extend_from_slice(&buffer[..read]);
    }
    let target = headers
        .split_whitespace()
        .nth(1)
        .expect("request target")
        .to_owned();
    let body = String::from_utf8(bytes[header_end..header_end + content_length].to_vec())
        .expect("body utf8");
    Request { target, body }
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
