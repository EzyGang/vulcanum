use std::io::{Read, Write};
use std::net::TcpListener;
use std::sync::Arc;
use std::thread;

use chrono::Utc;
use tokio::sync::{watch, RwLock};
use uuid::Uuid;

use vulcanum_shared::client::ApiClient;
use vulcanum_shared::worker_state::WorkerState;

use crate::daemon::job::execution::event_reporter::poll_cancel_request;

#[tokio::test]
async fn cancel_poll_sets_receiver_when_server_reports_cancel() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test server");
    let base_url = format!("http://{}", listener.local_addr().expect("local addr"));
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept request");
        let mut request = [0_u8; 4096];
        let read = stream.read(&mut request).expect("read request");
        let request = String::from_utf8_lossy(&request[..read]);
        assert!(request.contains("POST /api/v1/jobs/"));
        assert!(request
            .to_ascii_lowercase()
            .contains("authorization: bearer token"));

        let body = r#"{"accepted":0,"should_cancel":true}"#;
        let response = format!(
            "HTTP/1.1 200 OK\r\ncontent-type: application/json\r\ncontent-length: {}\r\nconnection: close\r\n\r\n{}",
            body.len(),
            body,
        );
        stream
            .write_all(response.as_bytes())
            .expect("write response");
    });
    let client = Arc::new(ApiClient::new(base_url));
    let worker_state = Arc::new(RwLock::new(WorkerState {
        worker_id: Uuid::new_v4(),
        instance_url: String::new(),
        access_token: "token".to_owned(),
        refresh_token: "refresh".to_owned(),
        expires_at: Utc::now() + chrono::Duration::minutes(30),
        max_concurrent_jobs: 1,
    }));
    let (cancel_tx, cancel_rx) = watch::channel(false);

    poll_cancel_request(&client, &worker_state, Uuid::new_v4(), &cancel_tx).await;

    assert!(*cancel_rx.borrow());
    server.join().expect("server thread should finish");
}
