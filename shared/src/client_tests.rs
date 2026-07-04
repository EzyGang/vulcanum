use std::io::{Read, Write};
use std::net::TcpListener;
use std::thread;

use uuid::Uuid;

use crate::client::ApiClient;

#[tokio::test]
async fn refresh_github_token_sends_explicit_empty_body() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let base_url = format!("http://{}", listener.local_addr().expect("listener addr"));
    let request_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("request should connect");
        let mut request = Vec::new();
        let mut buffer = [0_u8; 1024];
        loop {
            let read = stream.read(&mut buffer).expect("request should read");
            if read == 0 {
                break;
            }
            request.extend_from_slice(&buffer[..read]);
            if request.windows(4).any(|window| window == b"\r\n\r\n") {
                break;
            }
        }

        let body = r#"{"github_token":null,"github_token_expires_at":null}"#;
        write!(
            stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response should write");

        String::from_utf8(request).expect("request should be utf8")
    });

    ApiClient::new(base_url)
        .refresh_github_token(Uuid::new_v4(), "worker-token")
        .await
        .expect("refresh should parse response");

    let request = request_handle.join().expect("request thread should finish");
    let normalized = request.to_ascii_lowercase();

    assert!(
        normalized.contains("content-length: 0"),
        "request should include explicit empty body length:\n{request}"
    );
}
