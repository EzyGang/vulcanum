use std::sync::LazyLock;

use rcgen::generate_simple_self_signed;
use rustls::pki_types::PrivateKeyDer;
use tokio::io::{AsyncRead, AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;
use tokio::sync::{oneshot, Mutex};
use tokio_rustls::TlsAcceptor;
use uuid::Uuid;

pub(crate) static SSL_CERT_FILE_LOCK: LazyLock<Mutex<()>> = LazyLock::new(|| Mutex::new(()));

pub(crate) struct KaneoTestServer {
    pub(crate) instance_url: String,
    pub(crate) certificate_path: std::path::PathBuf,
    pub(crate) request: oneshot::Receiver<serde_json::Value>,
}

pub(crate) async fn start_kaneo_server(update_status: u16) -> KaneoTestServer {
    let _ = rustls::crypto::aws_lc_rs::default_provider().install_default();
    let certificate = generate_simple_self_signed(vec!["localhost".to_owned()])
        .expect("generate test certificate");
    let certificate_path = std::env::temp_dir().join(format!("kaneo-test-{}.pem", Uuid::new_v4()));
    std::fs::write(&certificate_path, certificate.cert.pem()).expect("write test certificate");

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(
            vec![certificate.cert.der().clone()],
            PrivateKeyDer::Pkcs8(certificate.key_pair.serialize_der().into()),
        )
        .expect("configure test TLS server");
    let listener = TcpListener::bind("127.0.0.1:0")
        .await
        .expect("bind test server");
    let port = listener
        .local_addr()
        .expect("read test server address")
        .port();
    let (sender, receiver) = oneshot::channel();

    tokio::spawn(async move {
        let acceptor = TlsAcceptor::from(std::sync::Arc::new(config));
        for _ in 0..2 {
            let (stream, _) = listener.accept().await.expect("accept test request");
            let mut stream = acceptor.accept(stream).await.expect("accept test TLS");
            let request = read_request(&mut stream).await;
            let (status, body) = match request.0.as_str() {
                "PUT" if update_status == 400 => (
                    "400 Bad Request",
                    "invalid description: secret body".to_owned(),
                ),
                _ => ("200 OK", kaneo_task_response()),
            };
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .await
                .expect("write test response");

            if request.0 == "PUT" {
                sender.send(request.1).expect("capture PUT request");
                break;
            }
        }
    });

    KaneoTestServer {
        instance_url: format!("localhost:{port}"),
        certificate_path,
        request: receiver,
    }
}

async fn read_request<S>(stream: &mut S) -> (String, serde_json::Value)
where
    S: AsyncRead + Unpin,
{
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 1024];
    let header_end = loop {
        let count = stream.read(&mut chunk).await.expect("read test request");
        buffer.extend_from_slice(&chunk[..count]);
        if let Some(index) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let (method, content_length) = {
        let header = std::str::from_utf8(&buffer[..header_end]).expect("request headers are UTF-8");
        let content_length = header
            .lines()
            .find_map(|line| {
                line.split_once(": ").and_then(|(name, value)| {
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.parse::<usize>().ok())
                        .flatten()
                })
            })
            .unwrap_or(0);
        let method = header
            .split_whitespace()
            .next()
            .expect("request method")
            .to_owned();
        (method, content_length)
    };
    while buffer.len() < header_end + content_length {
        let count = stream.read(&mut chunk).await.expect("read request body");
        buffer.extend_from_slice(&chunk[..count]);
    }

    let body = serde_json::from_slice(&buffer[header_end..header_end + content_length])
        .expect("request body is JSON");
    (method, body)
}

fn kaneo_task_response() -> String {
    serde_json::json!({
        "id": "task-1",
        "projectId": "project-1",
        "number": 1,
        "position": 7.5,
        "title": "Current title",
        "description": "Current body",
        "status": "in-progress",
        "priority": "urgent",
        "dueDate": "2026-01-10T00:00:00Z",
        "startDate": "2026-01-03T00:00:00Z",
        "userId": "user-1",
        "createdAt": "2026-01-01T00:00:00Z",
        "updatedAt": null,
        "assigneeName": "Agent",
        "assigneeId": "user-1",
        "labels": []
    })
    .to_string()
}
