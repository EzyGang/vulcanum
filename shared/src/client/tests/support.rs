use std::io::{Read, Write};
use std::net::{TcpListener, TcpStream};
use std::thread;

pub(crate) fn serve_once(
    status: &'static str,
    body: impl Into<String>,
) -> (String, thread::JoinHandle<String>) {
    let body = body.into();
    let listener = TcpListener::bind("127.0.0.1:0").expect("test listener should bind");
    let base_url = format!("http://{}", listener.local_addr().expect("listener addr"));
    let request_handle = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("request should connect");
        let request = read_complete_http_request(&mut stream);
        write!(
            stream,
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len(),
        )
        .expect("response should write");
        request
    });
    (base_url, request_handle)
}

fn read_complete_http_request(stream: &mut TcpStream) -> String {
    let mut request = Vec::new();
    let mut buffer = [0_u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut buffer).expect("request should read");
        request.extend_from_slice(&buffer[..read]);
        let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") else {
            continue;
        };
        break index + 4;
    };
    let headers = String::from_utf8_lossy(&request[..header_end]);
    let content_length = headers
        .lines()
        .find_map(|line| {
            line.to_ascii_lowercase()
                .strip_prefix("content-length: ")
                .and_then(|value| value.parse::<usize>().ok())
        })
        .unwrap_or(0);
    while request.len() < header_end + content_length {
        let read = stream.read(&mut buffer).expect("request body should read");
        request.extend_from_slice(&buffer[..read]);
    }
    String::from_utf8(request).expect("request should be utf8")
}
