use prelay_client::identity::windows::WindowsIdentity;
use std::{
    io::{Read, Write},
    net::TcpListener,
    sync::{
        atomic::{AtomicUsize, Ordering},
        Arc,
    },
    thread,
    time::{Duration, Instant},
};

pub(super) fn identity() -> WindowsIdentity {
    WindowsIdentity {
        machine_id: "machine-a".into(),
        account_sid: "S-1-5-21-100".into(),
        display_name: "Ada".into(),
    }
}

#[test]
fn request_header_matching_is_case_insensitive() {
    let request = "GET /api/providers HTTP/1.1\r\nauthorization: Bearer device-secret\r\n\r\n";

    assert_header_value(request, "Authorization", "Bearer device-secret");
}

pub(super) fn assert_header_value(request: &str, name: &str, expected_value: &str) {
    let value = request
        .split("\r\n")
        .skip(1)
        .take_while(|line| !line.is_empty())
        .find_map(|line| {
            let (header_name, value) = line.split_once(':')?;
            header_name
                .eq_ignore_ascii_case(name)
                .then_some(value.trim())
        });

    assert_eq!(value, Some(expected_value), "expected {name} header");
}

pub(super) fn one_response_server(
    status: &str,
    body: &'static str,
    assert_request: impl FnOnce(&str) + Send + 'static,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
    let address = listener.local_addr().expect("read test relay address");
    let status = status.to_string();
    let server = thread::spawn(move || {
        let (mut stream, _) = listener.accept().expect("accept registration request");
        let request = read_http_request(&mut stream);
        assert_request(&request);
        let response = format!(
            "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream
            .write_all(response.as_bytes())
            .expect("write registration response");
    });
    (format!("http://{address}"), server)
}

pub(super) fn registration_server(
    requests: Arc<AtomicUsize>,
) -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
    listener
        .set_nonblocking(true)
        .expect("configure test relay listener");
    let address = listener.local_addr().expect("read test relay address");
    let server = thread::spawn(move || {
        let deadline = Instant::now() + Duration::from_millis(300);
        let mut captured_requests = Vec::new();
        while Instant::now() < deadline {
            match listener.accept() {
                Ok((mut stream, _)) => {
                    stream
                        .set_nonblocking(false)
                        .expect("configure accepted relay connection");
                    let request = read_http_request(&mut stream);
                    assert!(request.starts_with("POST /api/identities HTTP/1.1"));
                    captured_requests.push(request);
                    let request_number = requests.fetch_add(1, Ordering::SeqCst);
                    let (status, body) = if request_number == 0 {
                        (
                            "201 Created",
                            r#"{"identity_id":"identity-a","created":true}"#,
                        )
                    } else {
                        (
                            "400 Bad Request",
                            r#"{"error":{"code":"identity_already_registered","message":"already registered"}}"#,
                        )
                    };
                    let response = format!(
                        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                        body.len()
                    );
                    stream
                        .write_all(response.as_bytes())
                        .expect("write registration response");
                }
                Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                    thread::sleep(Duration::from_millis(5));
                }
                Err(error) => panic!("accept registration request: {error}"),
            }
        }
        captured_requests
    });
    (format!("http://{address}"), server)
}

pub(super) fn two_registration_response_server() -> (String, thread::JoinHandle<usize>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
    let address = listener.local_addr().expect("read test relay address");
    let server = thread::spawn(move || {
        for _ in 0..2 {
            let (mut stream, _) = listener.accept().expect("accept registration request");
            let request = read_http_request(&mut stream);
            assert!(request.starts_with("POST /api/identities HTTP/1.1"));
            let body = r#"{"identity_id":"identity-a","created":false}"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write registration response");
        }
        2
    });
    (format!("http://{address}"), server)
}

pub(super) fn two_response_server(
    responses: [(&'static str, &'static str); 2],
    assert_requests: impl FnOnce(&[String]) + Send + 'static,
) -> (String, thread::JoinHandle<()>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
    let address = listener.local_addr().expect("read test relay address");
    let server = thread::spawn(move || {
        let mut requests = Vec::new();
        for (status, body) in responses {
            let (mut stream, _) = listener.accept().expect("accept relay request");
            requests.push(read_http_request(&mut stream));
            let response = format!(
                "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream
                .write_all(response.as_bytes())
                .expect("write relay response");
        }
        assert_requests(&requests);
    });
    (format!("http://{address}"), server)
}

pub(super) fn retry_registration_server() -> (String, thread::JoinHandle<Vec<String>>) {
    let listener = TcpListener::bind("127.0.0.1:0").expect("bind test relay");
    let address = listener.local_addr().expect("read test relay address");
    let server = thread::spawn(move || {
        let mut requests = Vec::new();
        for attempt in 0..2 {
            let (mut stream, _) = listener.accept().expect("accept registration request");
            requests.push(read_http_request(&mut stream));
            if attempt == 1 {
                let body = r#"{"identity_id":"identity-a","created":true}"#;
                let response = format!(
                    "HTTP/1.1 201 Created\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                    body.len()
                );
                stream
                    .write_all(response.as_bytes())
                    .expect("write registration response");
            }
        }
        requests
    });
    (format!("http://{address}"), server)
}

fn read_http_request(stream: &mut std::net::TcpStream) -> String {
    let mut request = Vec::new();
    let mut chunk = [0u8; 1024];
    let header_end = loop {
        let read = stream.read(&mut chunk).expect("read registration request");
        assert_ne!(read, 0, "registration request ended before its headers");
        request.extend_from_slice(&chunk[..read]);
        if let Some(index) = request.windows(4).position(|window| window == b"\r\n\r\n") {
            break index + 4;
        }
    };
    let headers = std::str::from_utf8(&request[..header_end]).expect("request headers are UTF-8");
    let content_length = headers
        .lines()
        .find_map(|line| line.strip_prefix("content-length: "))
        .or_else(|| {
            headers
                .lines()
                .find_map(|line| line.strip_prefix("Content-Length: "))
        })
        .map(|value| {
            value
                .parse::<usize>()
                .expect("registration request content length is numeric")
        })
        .unwrap_or_default();
    while request.len() < header_end + content_length {
        let read = stream
            .read(&mut chunk)
            .expect("read registration request body");
        assert_ne!(read, 0, "registration request ended before its body");
        request.extend_from_slice(&chunk[..read]);
    }
    String::from_utf8(request).expect("registration request is UTF-8")
}
