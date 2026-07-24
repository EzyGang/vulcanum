use std::collections::HashMap;
use std::io::Write;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::Arc;

use flate2::write::GzEncoder;
use flate2::Compression;
use sha2::{Digest, Sha256};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

use crate::update::service::ServiceRestarter;

#[derive(Clone, Default)]
pub(super) struct FakeRestarter {
    calls: Arc<AtomicUsize>,
}

impl FakeRestarter {
    pub(super) fn calls(&self) -> usize {
        self.calls.load(Ordering::SeqCst)
    }
}

impl ServiceRestarter for FakeRestarter {
    fn restart(&self) -> anyhow::Result<()> {
        self.calls.fetch_add(1, Ordering::SeqCst);
        Ok(())
    }
}

pub(super) struct TestServer {
    pub(super) base_url: String,
}

impl TestServer {
    pub(super) async fn start<F>(expected_requests: usize, build_routes: F) -> Self
    where
        F: FnOnce(&str) -> HashMap<String, Vec<u8>>,
    {
        let listener = TcpListener::bind("127.0.0.1:0")
            .await
            .expect("test server should bind");
        let address = listener
            .local_addr()
            .expect("test server should have an address");
        let base_url = format!("http://{address}");
        let routes = Arc::new(build_routes(&base_url));

        tokio::spawn(async move {
            for _ in 0..expected_requests {
                let (mut stream, _) = listener
                    .accept()
                    .await
                    .expect("test server should accept request");
                let mut request = Vec::new();
                let mut buffer = [0_u8; 1024];
                loop {
                    let read = stream
                        .read(&mut buffer)
                        .await
                        .expect("test server should read request");
                    if read == 0 {
                        break;
                    }
                    request.extend_from_slice(&buffer[..read]);
                    if request.windows(4).any(|window| window == b"\r\n\r\n") {
                        break;
                    }
                }
                let path = request_path(&request);
                let (status, body) = match routes.get(path) {
                    Some(body) => ("200 OK", body.as_slice()),
                    None => ("404 Not Found", &[][..]),
                };
                let headers = format!(
                    "HTTP/1.1 {status}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
                    body.len()
                );
                stream
                    .write_all(headers.as_bytes())
                    .await
                    .expect("test server should write headers");
                stream
                    .write_all(body)
                    .await
                    .expect("test server should write body");
            }
        });

        Self { base_url }
    }
}

pub(super) fn release_routes(
    base_url: &str,
    tag: &str,
    target: &str,
    archive: Vec<u8>,
    checksum: Vec<u8>,
) -> HashMap<String, Vec<u8>> {
    let archive_name = format!("vulcanum-{target}.tar.gz");
    let checksum_name = format!("{archive_name}.sha256");
    let release = serde_json::json!({
        "tag_name": tag,
        "assets": [
            {
                "name": archive_name,
                "browser_download_url": format!("{base_url}/archive")
            },
            {
                "name": checksum_name,
                "browser_download_url": format!("{base_url}/checksum")
            }
        ]
    });
    HashMap::from([
        ("/latest".to_owned(), release.to_string().into_bytes()),
        ("/archive".to_owned(), archive),
        ("/checksum".to_owned(), checksum),
    ])
}

pub(super) fn release_archive(cli: &[u8], worker: &[u8]) -> Vec<u8> {
    let encoder = GzEncoder::new(Vec::new(), Compression::default());
    let mut archive = tar::Builder::new(encoder);
    append_file(&mut archive, "vulcanum", cli);
    append_file(&mut archive, "vulcanum-server", worker);
    let encoder = archive
        .into_inner()
        .expect("test archive should finish writing");
    encoder.finish().expect("test gzip stream should finish")
}

pub(super) fn checksum(archive: &[u8]) -> Vec<u8> {
    format!("{:x}  archive.tar.gz\n", Sha256::digest(archive)).into_bytes()
}

fn append_file<W>(archive: &mut tar::Builder<W>, name: &str, body: &[u8])
where
    W: Write,
{
    let mut header = tar::Header::new_gnu();
    header.set_size(body.len() as u64);
    header.set_mode(0o755);
    header.set_cksum();
    archive
        .append_data(&mut header, name, body)
        .expect("test file should be added to archive");
}

fn request_path(request: &[u8]) -> &str {
    std::str::from_utf8(request)
        .expect("request should be UTF-8")
        .lines()
        .next()
        .and_then(|line| line.split_whitespace().nth(1))
        .expect("request should contain a path")
}
