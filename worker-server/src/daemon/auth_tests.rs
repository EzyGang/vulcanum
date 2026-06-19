use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, OnceLock};

use chrono::{Duration, Utc};
use tokio::sync::RwLock;
use uuid::Uuid;

use vulcanum_shared::api_error::ApiError;
use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::auth::with_retry_on_401;

static ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

#[tokio::test]
async fn retries_once_with_refreshed_access_token_after_401() {
    let _guard = ENV_LOCK.get_or_init(|| Mutex::new(())).lock();
    let _env = TestHome::new();
    let expires_at = Utc::now() + Duration::hours(1);
    let server = RefreshServer::start(expires_at.to_rfc3339());
    let client = Arc::new(ApiClient::new(server.base_url()));
    let worker_state = Arc::new(RwLock::new(worker_state("old-token", "refresh-token")));
    let attempts = Arc::new(AtomicUsize::new(0));
    let seen_tokens = Arc::new(Mutex::new(Vec::new()));

    let result = with_retry_on_401(&client, &worker_state, |token| {
        let attempts = attempts.clone();
        let seen_tokens = seen_tokens.clone();
        async move {
            seen_tokens.lock().unwrap().push(token.clone());
            match attempts.fetch_add(1, Ordering::SeqCst) {
                0 => Err(ApiError {
                    status: 401,
                    body: "expired".to_owned(),
                }
                .into()),
                _ => Ok(token),
            }
        }
    })
    .await
    .unwrap();

    assert_eq!(result, "new-token");
    assert_eq!(attempts.load(Ordering::SeqCst), 2);
    assert_eq!(server.join(), 1);

    let state = worker_state.read().await;
    assert_eq!(state.access_token, "new-token");
    assert_eq!(state.refresh_token, "new-refresh-token");
    assert_eq!(state.expires_at, expires_at);
    assert_eq!(
        *seen_tokens.lock().unwrap(),
        vec!["old-token".to_owned(), "new-token".to_owned()]
    );
}

#[tokio::test]
async fn does_not_refresh_or_retry_non_401_errors() {
    let client = Arc::new(ApiClient::new("http://127.0.0.1:9"));
    let worker_state = Arc::new(RwLock::new(worker_state("old-token", "refresh-token")));
    let attempts = Arc::new(AtomicUsize::new(0));

    let result = with_retry_on_401(&client, &worker_state, |token| {
        let attempts = attempts.clone();
        async move {
            assert_eq!(token, "old-token");
            attempts.fetch_add(1, Ordering::SeqCst);
            Err::<(), anyhow::Error>(
                ApiError {
                    status: 403,
                    body: "forbidden".to_owned(),
                }
                .into(),
            )
        }
    })
    .await;

    assert!(result.is_err());
    assert_eq!(attempts.load(Ordering::SeqCst), 1);
    assert_eq!(worker_state.read().await.access_token, "old-token");
}

fn worker_state(access_token: &str, refresh_token: &str) -> WorkerState {
    WorkerState {
        worker_id: Uuid::new_v4(),
        instance_url: "http://127.0.0.1".to_owned(),
        access_token: access_token.to_owned(),
        refresh_token: refresh_token.to_owned(),
        expires_at: Utc::now(),
        max_concurrent_jobs: 1,
    }
}

struct RefreshServer {
    base_url: String,
    handle: std::thread::JoinHandle<usize>,
}

impl RefreshServer {
    fn start(expires_at: String) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let handle = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().unwrap();
            let mut request = [0; 1024];
            let read = stream.read(&mut request).unwrap();
            let request = String::from_utf8_lossy(&request[..read]);
            assert!(request.starts_with("POST /api/v1/workers/refresh "));
            assert!(request.contains(r#""refresh_token":"refresh-token""#));

            let body = format!(
                r#"{{"access_token":"new-token","refresh_token":"new-refresh-token","expires_at":"{expires_at}"}}"#
            );
            let response = format!(
                "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
                body.len(),
                body
            );
            stream.write_all(response.as_bytes()).unwrap();
            1
        });

        Self { base_url, handle }
    }

    fn base_url(&self) -> &str {
        &self.base_url
    }

    fn join(self) -> usize {
        self.handle.join().unwrap()
    }
}

struct TestHome {
    home: Option<String>,
    userprofile: Option<String>,
    path: std::path::PathBuf,
}

impl TestHome {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!("vulcanum-auth-test-{}", Uuid::new_v4()));
        std::fs::create_dir_all(&path).unwrap();
        let home = std::env::var("HOME").ok();
        let userprofile = std::env::var("USERPROFILE").ok();
        std::env::set_var("HOME", &path);
        std::env::set_var("USERPROFILE", &path);

        Self {
            home,
            userprofile,
            path,
        }
    }
}

impl Drop for TestHome {
    fn drop(&mut self) {
        match &self.home {
            Some(value) => std::env::set_var("HOME", value),
            None => std::env::remove_var("HOME"),
        }
        match &self.userprofile {
            Some(value) => std::env::set_var("USERPROFILE", value),
            None => std::env::remove_var("USERPROFILE"),
        }
        let _ = std::fs::remove_dir_all(&self.path);
    }
}
