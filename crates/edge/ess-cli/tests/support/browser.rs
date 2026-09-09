//! Small actual Firefox `BiDi` harness; profiles, sockets and every receipt remain caller-owned.
use serde_json::{json, Value};
use std::{
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, Ordering},
        Arc,
    },
    thread::{self, JoinHandle},
    time::{Duration, Instant},
};

pub struct Server {
    pub url: String,
    stopped: Arc<AtomicBool>,
    thread: Option<JoinHandle<()>>,
}
impl Server {
    pub fn new(root: &Path) -> Self {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let url = format!("http://{}", listener.local_addr().unwrap());
        listener.set_nonblocking(true).unwrap();
        let stopped = Arc::new(AtomicBool::new(false));
        let flag = stopped.clone();
        let root = root.to_path_buf();
        let thread = thread::spawn(move || {
            while !flag.load(Ordering::Relaxed) {
                match listener.accept() {
                    Ok((stream, _)) => serve(stream, &root),
                    Err(error) if error.kind() == std::io::ErrorKind::WouldBlock => {
                        thread::sleep(Duration::from_millis(5));
                    }
                    Err(error) => panic!("browser fixture listener: {error}"),
                }
            }
        });
        Self {
            url,
            stopped,
            thread: Some(thread),
        }
    }
}
impl Drop for Server {
    fn drop(&mut self) {
        self.stopped.store(true, Ordering::Relaxed);
        self.thread.take().unwrap().join().unwrap();
    }
}
fn serve(mut stream: TcpStream, root: &Path) {
    stream
        .set_read_timeout(Some(Duration::from_secs(3)))
        .unwrap();
    let mut request = Vec::new();
    while !request.ends_with(b"\r\n\r\n") && request.len() < 8192 {
        let mut byte = [0];
        if stream.read_exact(&mut byte).is_err() {
            return;
        }
        request.push(byte[0]);
    }
    let request = String::from_utf8(request).unwrap();
    let path = request
        .split_whitespace()
        .nth(1)
        .unwrap_or("/")
        .split('?')
        .next()
        .unwrap();
    if path.split('/').any(|segment| segment == "..") {
        return;
    }
    let path = root.join(path.trim_start_matches('/'));
    let body = fs::read(&path);
    let (status, bytes) =
        body.map_or_else(|_| ("404 Not Found", Vec::new()), |bytes| ("200 OK", bytes));
    let mime = match path.extension().and_then(|s| s.to_str()) {
        Some("js") => "text/javascript",
        Some("json") => "application/json",
        _ => "text/html",
    };
    let header = format!("HTTP/1.1 {status}\r\nContent-Type: {mime}; charset=utf-8\r\nContent-Length: {}\r\nCache-Control: no-store\r\nConnection: close\r\n\r\n", bytes.len());
    let _ = stream
        .write_all(header.as_bytes())
        .and_then(|()| stream.write_all(&bytes));
}

struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
pub struct Browser {
    _child: OwnedChild,
    stream: TcpStream,
    next: u64,
    receipt: File,
}
fn assigned_bidi_port(child: &mut OwnedChild, evidence: &Path, deadline: Instant) -> u16 {
    // Firefox documents the assigned BiDi endpoint on stderr. Read only a
    // complete line from this child's private log, under the startup deadline.
    loop {
        assert!(
            child.0.try_wait().unwrap().is_none(),
            "Firefox exited; read firefox.stderr"
        );
        assert!(
            Instant::now() < deadline,
            "Firefox did not announce BiDi; read firefox.stderr"
        );
        let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap();
        if let Some(port) = log.split_inclusive('\n').find_map(|line| {
            line.strip_suffix('\n')?
                .trim_end()
                .strip_prefix("WebDriver BiDi listening on ws://127.0.0.1:")?
                .parse::<u16>()
                .ok()
                .filter(|port| *port != 0)
        }) {
            return port;
        }
        thread::sleep(Duration::from_millis(50));
    }
}
impl Browser {
    pub fn new(evidence: &Path) -> Self {
        static NEXT_PROFILE: AtomicU64 = AtomicU64::new(0);
        let profile_root =
            std::env::var_os("ESS_BROWSER_TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
        let profile = profile_root.join(format!(
            "ess-bidi-{}-{}",
            std::process::id(),
            NEXT_PROFILE.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir_all(&profile).unwrap();
        let firefox = std::env::var_os("ESS_FIREFOX").unwrap_or_else(|| "firefox".into());
        let mut command = Command::new(firefox);
        command
            .args(["--headless", "--no-remote", "--remote-debugging-port"])
            // Firefox binds its own ephemeral port. Releasing a temporary Rust
            // listener before launch lets another parallel fixture reuse it.
            .arg("0")
            .arg("--profile")
            .arg(&profile)
            .arg("about:blank")
            .env("TMPDIR", &profile_root)
            .env("MOZ_HEADLESS", "1")
            .stdout(Stdio::from(
                File::create(evidence.join("firefox.stdout")).unwrap(),
            ))
            .stderr(Stdio::from(
                File::create(evidence.join("firefox.stderr")).unwrap(),
            ));
        fs::write(evidence.join("firefox.command"), format!("{command:?}\n")).unwrap();
        fs::write(
            evidence.join("browser-profile.txt"),
            format!("{}\n", profile.display()),
        )
        .unwrap();
        let mut child = OwnedChild(command.spawn().expect("required actual Firefox starts"));
        fs::write(evidence.join("firefox.pid"), format!("{}\n", child.0.id())).unwrap();
        let deadline = Instant::now() + Duration::from_secs(30);
        let port = assigned_bidi_port(&mut child, evidence, deadline);
        let (stream, response) = loop {
            assert!(
                child.0.try_wait().unwrap().is_none(),
                "Firefox exited; read firefox.stderr"
            );
            assert!(
                Instant::now() < deadline,
                "Firefox did not expose BiDi; read firefox.stderr"
            );
            let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
                thread::sleep(Duration::from_millis(50));
                continue;
            };
            stream
                .set_read_timeout(Some(Duration::from_secs(20)))
                .unwrap();
            stream
                .set_write_timeout(Some(Duration::from_secs(20)))
                .unwrap();
            let handshake = format!("GET /session HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: dGhlIHNhbXBsZSBub25jZQ==\r\nSec-WebSocket-Version: 13\r\n\r\n");
            stream.write_all(handshake.as_bytes()).unwrap();
            let mut response = Vec::new();
            while !response.ends_with(b"\r\n\r\n") {
                let mut byte = [0];
                stream.read_exact(&mut byte).unwrap();
                response.push(byte[0]);
            }
            let response = String::from_utf8(response).unwrap();
            // Firefox can listen before registering /session. TCP readiness alone is not
            // BiDi readiness. Keep the startup response as evidence; retain the same deadline.
            if response.starts_with("HTTP/1.1 404 ") {
                fs::write(evidence.join("websocket-startup-response.txt"), &response).unwrap();
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            break (stream, response);
        };
        fs::write(evidence.join("websocket-handshake.txt"), &response).unwrap();
        assert!(response.starts_with("HTTP/1.1 101"), "{response}");
        assert!(
            response.contains("s3pPLMBiTxaQ9kYGzzhZRbK+xOo="),
            "{response}"
        );
        let mut browser = Self {
            _child: child,
            stream,
            next: 0,
            receipt: File::create(evidence.join("bidi.jsonl")).unwrap(),
        };
        browser.call("session.new", &json!({"capabilities":{"alwaysMatch":{}}}));
        browser
    }
    fn call(&mut self, method: &str, params: &Value) -> Value {
        self.next += 1;
        let request = json!({"id":self.next,"method":method,"params":params});
        writeln!(self.receipt, "{}", json!({"sent":request})).unwrap();
        self.send(1, request.to_string().as_bytes());
        loop {
            let (opcode, payload) = self.receive();
            if opcode == 9 {
                self.send(10, &payload);
                continue;
            }
            assert_eq!(opcode, 1, "unexpected WebSocket frame");
            let value: Value = serde_json::from_slice(&payload).unwrap();
            writeln!(self.receipt, "{}", json!({"received":value})).unwrap();
            if value["id"].as_u64() == Some(self.next) {
                assert_ne!(value["type"], "error", "{value}");
                return value["result"].clone();
            }
        }
    }
    fn send(&mut self, opcode: u8, payload: &[u8]) {
        let mut bytes = vec![0x80 | opcode];
        if payload.len() < 126 {
            bytes.push(0x80 | u8::try_from(payload.len()).unwrap());
        } else if u16::try_from(payload.len()).is_ok() {
            bytes.push(0x80 | 0x7e);
            bytes.extend(u16::try_from(payload.len()).unwrap().to_be_bytes());
        } else {
            bytes.push(0x80 | 0x7f);
            bytes.extend(u64::try_from(payload.len()).unwrap().to_be_bytes());
        }
        let mask = [0x12, 0x34, 0x56, 0x78];
        bytes.extend(mask);
        bytes.extend(
            payload
                .iter()
                .enumerate()
                .map(|(i, byte)| byte ^ mask[i % 4]),
        );
        self.stream.write_all(&bytes).unwrap();
    }
    fn receive(&mut self) -> (u8, Vec<u8>) {
        let mut header = [0; 2];
        self.stream.read_exact(&mut header).unwrap();
        assert_ne!(header[0] & 0x80, 0, "fragmented test response");
        assert_eq!(header[1] & 0x80, 0, "server frames are unmasked");
        let length = match header[1] & 0x7f {
            126 => {
                let mut bytes = [0; 2];
                self.stream.read_exact(&mut bytes).unwrap();
                u64::from(u16::from_be_bytes(bytes))
            }
            127 => {
                let mut bytes = [0; 8];
                self.stream.read_exact(&mut bytes).unwrap();
                u64::from_be_bytes(bytes)
            }
            length => u64::from(length),
        };
        assert!(length < 4_194_304, "unexpectedly large test response");
        let mut payload = vec![0; usize::try_from(length).unwrap()];
        self.stream.read_exact(&mut payload).unwrap();
        (header[0] & 0x0f, payload)
    }
    pub fn open(&mut self, url: &str) -> String {
        let result = self.call("browsingContext.create", &json!({"type":"tab"}));
        let context = result["context"].as_str().unwrap().to_owned();
        self.call(
            "browsingContext.navigate",
            &json!({"context":context,"url":url,"wait":"complete"}),
        );
        context
    }
    pub fn evaluate(&mut self, context: &str, expression: &str) -> Value {
        let result = self.call(
            "script.evaluate",
            &json!({"expression":expression,"target":{"context":context},"awaitPromise":true}),
        );
        assert_eq!(result["type"], "success", "{result}");
        serde_json::from_str(
            result["result"]["value"]
                .as_str()
                .expect("script returns a JSON string"),
        )
        .unwrap()
    }
}
