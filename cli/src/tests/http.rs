use std::collections::BTreeMap;
use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

#[derive(Clone)]
pub(crate) struct Response {
    method: String,
    target: String,
    status: String,
    body: String,
}

impl Response {
    pub(crate) fn ok(method: &str, target: &str, body: impl Into<String>) -> Self {
        Self::status(method, target, "200 OK", body)
    }

    pub(crate) fn status(
        method: &str,
        target: &str,
        status: &str,
        body: impl Into<String>,
    ) -> Self {
        Self {
            method: method.to_owned(),
            target: target.to_owned(),
            status: status.to_owned(),
            body: body.into(),
        }
    }
}

pub(crate) struct Request {
    pub(crate) method: String,
    pub(crate) target: String,
    pub(crate) headers: BTreeMap<String, String>,
    pub(crate) body: String,
}

pub(crate) struct FakeServer {
    pub(crate) url: String,
    handle: thread::JoinHandle<Vec<Request>>,
}

impl FakeServer {
    pub(crate) fn start(mut responses: Vec<Response>) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").expect("test server should bind");
        let url = format!("http://{}", listener.local_addr().expect("test address"));
        let handle = thread::spawn(move || {
            let mut requests = Vec::with_capacity(responses.len());
            while !responses.is_empty() {
                let (mut stream, _) = listener.accept().expect("request should connect");
                let request = read_request(&mut stream);
                let index = responses
                    .iter()
                    .position(|response| {
                        response.method == request.method && response.target == request.target
                    })
                    .unwrap_or_else(|| panic!("unexpected {} {}", request.method, request.target));
                let response = responses.remove(index);
                write!(
                    stream,
                    "HTTP/1.1 {}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{}",
                    response.status,
                    response.body.len(),
                    response.body
                )
                .expect("response should write");
                requests.push(request);
            }
            requests
        });
        Self { url, handle }
    }

    pub(crate) fn finish(self) -> Vec<Request> {
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
    let headers_text = String::from_utf8_lossy(&bytes[..header_end]).into_owned();
    let content_length = headers_text
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
    let mut lines = headers_text.lines();
    let mut request_line = lines
        .next()
        .expect("request line should exist")
        .split_whitespace();
    let method = request_line.next().expect("request method").to_owned();
    let target = request_line.next().expect("request target").to_owned();
    let headers = lines
        .filter_map(|line| line.split_once(':'))
        .map(|(name, value)| (name.to_ascii_lowercase(), value.trim().to_owned()))
        .collect();
    let body = String::from_utf8(bytes[header_end..header_end + content_length].to_vec())
        .expect("body utf8");
    Request {
        method,
        target,
        headers,
        body,
    }
}
