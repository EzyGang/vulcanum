use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

use uuid::Uuid;

use crate::client::tests::support::serve_once;
use crate::client::{probe_url_with_scheme_fallback, ApiClient};

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

#[tokio::test]
async fn probe_url_with_scheme_fallback_trims_whitespace_before_falling_back_to_http() {
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let addr = listener.local_addr().expect("listener addr");
    let request_handle = thread::spawn(move || {
        let (mut https_stream, _) = listener.accept().expect("https probe should connect");
        let mut first_byte = [0_u8; 1];
        let _ = https_stream.read(&mut first_byte);
        drop(https_stream);

        let (mut http_stream, _) = listener.accept().expect("http fallback should connect");
        let request = read_http_request(&mut http_stream);
        let body =
            r#"{"access_token_ttl_minutes":15,"code_ttl_minutes":10,"refresh_token_ttl_days":30}"#;
        write!(
            http_stream,
            "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
            body.len(),
            body
        )
        .expect("response should write");

        request
    });

    let input = format!(" \n127.0.0.1:{}/ \t", addr.port());
    let result = probe_url_with_scheme_fallback(&input).await;
    let (resolved_url, status) = match result {
        Ok(value) => value,
        Err(err) => {
            unblock_probe_server(addr);
            let _ = request_handle.join();
            panic!("trimmed URL should fall back to local HTTP server: {err}");
        }
    };
    let request = request_handle.join().expect("request thread should finish");

    assert_eq!(resolved_url, format!("http://127.0.0.1:{}", addr.port()));
    assert!(
        request.starts_with("GET /api/v1/status "),
        "status probe should request the status endpoint:\n{request}"
    );
    assert_eq!(status.access_token_ttl_minutes, 15);
    assert_eq!(status.code_ttl_minutes, 10);
    assert_eq!(status.refresh_token_ttl_days, 30);
}

#[tokio::test]
async fn auth_mode_uses_existing_mode_endpoint() {
    let (base_url, request_handle) = serve_once("200 OK", r#"{"is_single_user":true}"#);

    let response = ApiClient::new(base_url)
        .auth_mode()
        .await
        .expect("auth mode should parse");
    let request = request_handle.join().expect("server should finish");

    assert!(response.is_single_user);
    assert!(request.starts_with("GET /api/v1/auth/mode "));
}

#[tokio::test]
async fn instance_login_sends_password_and_parses_token_timestamp() {
    let (base_url, request_handle) = serve_once("200 OK", auth_tokens());

    let response = ApiClient::new(base_url)
        .instance_login("instance-secret")
        .await
        .expect("instance login should parse");
    let request = request_handle.join().expect("server should finish");

    assert!(request.starts_with("POST /api/v1/auth/instance-login "));
    assert!(request.ends_with(r#"{"password":"instance-secret"}"#));
    assert_eq!(response.access_token, "access-token");
    assert_eq!(response.refresh_token, "refresh-token");
    assert_eq!(
        response.refresh_expires_at.to_rfc3339(),
        "2030-01-02T03:04:05+00:00"
    );
}

#[tokio::test]
async fn auth_exchange_sends_callback_code() {
    let (base_url, request_handle) = serve_once("200 OK", auth_tokens());

    ApiClient::new(base_url)
        .exchange_auth_code("one-time-code")
        .await
        .expect("auth code exchange should parse");
    let request = request_handle.join().expect("server should finish");

    assert!(request.starts_with("POST /api/v1/auth/exchange "));
    assert!(request.ends_with(r#"{"code":"one-time-code"}"#));
}

#[tokio::test]
async fn app_auth_errors_preserve_server_message() {
    let (base_url, request_handle) =
        serve_once("401 Unauthorized", r#"{"error":"invalid password"}"#);

    let error = ApiClient::new(base_url)
        .instance_login("wrong")
        .await
        .expect_err("401 should fail");
    request_handle.join().expect("server should finish");

    assert!(error.to_string().contains("invalid password"));
}

fn auth_tokens() -> &'static str {
    r#"{"access_token":"access-token","refresh_token":"refresh-token","refresh_expires_at":"2030-01-02T03:04:05Z"}"#
}

fn read_http_request(stream: &mut TcpStream) -> String {
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

    String::from_utf8(request).expect("request should be utf8")
}

fn unblock_probe_server(addr: std::net::SocketAddr) {
    for _ in 0..2 {
        if let Ok(mut stream) = TcpStream::connect(addr) {
            let _ = stream.write_all(b"x");
        }
    }
}
