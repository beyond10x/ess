//! Small actual Firefox `BiDi` harness; profiles, sockets and every receipt remain caller-owned.
use serde_json::{json, Value};
use std::{
    ffi::OsStr,
    fs::{self, File},
    io::{Read, Write},
    net::{TcpListener, TcpStream},
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    sync::{
        atomic::{AtomicBool, AtomicU64, AtomicUsize, Ordering},
        Arc, Mutex, MutexGuard, PoisonError,
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
    evidence: PathBuf,
}
/// The startup deadline every fixture in this process is judged against.
pub const STARTUP_DEADLINE: Duration = Duration::from_secs(30);

/// What one `BiDi` call may take once the browser is up, and why it is not the startup budget.
///
/// `upgrade` clamps the socket to what is LEFT of the startup deadline so that a single connect
/// iteration cannot overrun that deadline by its own length — it did, by 20s, which is why the
/// clamp exists. But that socket becomes `Browser::stream` and is the browser's transport for its
/// whole life, so the clamp used to decide how long every later `session.new`, `open`, `evaluate`
/// and `receive` had. A browser that became ready LATE then served its first call with whatever
/// few milliseconds were left.
///
/// Measured by the wave-24 unit-1 pass-2 adversary with a control that differed in one thing:
/// a stand-in ready at 2.700s of a 3.000s deadline died in `receive` with a bare `WouldBlock`, no
/// stage, no measured startup and no `firefox.stderr`; the same browser ready at 0.050s passed the
/// same call in 0.85s. The harm scales with slowness, which is the one condition a loaded CI runner
/// guarantees.
///
/// 20s is what this transport had before the clamp was introduced, restored here as a property of
/// the SESSION rather than of the start.
pub const SESSION_TIMEOUT: Duration = Duration::from_secs(20);

/// RFC 6455 section 1.3's worked example `Sec-WebSocket-Key`, base64 of the ASCII text
/// `the sample nonce`. A WebSocket key is a handshake nonce and not a credential — the protocol
/// requires the client to send one and the server to hash it back, and the RFC prints this exact
/// pair so implementations can check their hashing. Named rather than inlined because a secret
/// scanner reads a base64 literal as a generic API key, and a name plus this sentence is the
/// answer to that, where an allow-list entry would only silence it.
const RFC6455_EXAMPLE_KEY: &str = "dGhlIHNhbXBsZSBub25jZQ=="; // gitleaks:allow

/// The `Sec-WebSocket-Accept` the RFC prints for [`RFC6455_EXAMPLE_KEY`]: the key concatenated with
/// the protocol's fixed GUID, SHA-1'd, base64'd. A server that returns this proved it read the key.
const RFC6455_EXAMPLE_ACCEPT: &str = "s3pPLMBiTxaQ9kYGzzhZRbK+xOo="; // gitleaks:allow

/// Startup is the phase that competes for CPU and the only phase the deadline
/// judges: fixtures in one test binary otherwise launch Firefox at the same
/// instant, and a runner slow enough to lose that race fails every one of them
/// at once. Hold this from spawn to `BiDi` readiness; everything after it runs in
/// parallel as before.
static STARTUP: Mutex<()> = Mutex::new(());
/// How many startups sit between a spawned child and `BiDi` readiness right now,
/// and the most there have ever been. These count the startup itself rather than
/// the lock that serializes it: a fixture that released `STARTUP` early would
/// leave real startups overlapping while every acquisition still looked orderly,
/// and only a counter with the startup's own extent can say so.
static IN_STARTUP: AtomicUsize = AtomicUsize::new(0);
static PEAK_IN_STARTUP: AtomicUsize = AtomicUsize::new(0);

/// The largest number of Firefox startups this process ever had in flight.
/// Read by the fixture's own cases; the other browser suites only start browsers.
#[allow(dead_code)]
pub fn peak_concurrent_startups() -> usize {
    PEAK_IN_STARTUP.load(Ordering::SeqCst)
}

struct Starting {
    // A fixture that panicked inside startup poisons nothing a later fixture cares about.
    _guard: MutexGuard<'static, ()>,
}
impl Starting {
    fn begin() -> Self {
        Self {
            _guard: STARTUP.lock().unwrap_or_else(PoisonError::into_inner),
        }
    }
}

/// One startup, counted for exactly as long as it is actually running.
struct InStartup;
impl InStartup {
    fn begin() -> Self {
        let live = IN_STARTUP.fetch_add(1, Ordering::SeqCst) + 1;
        PEAK_IN_STARTUP.fetch_max(live, Ordering::SeqCst);
        Self
    }
}
impl Drop for InStartup {
    fn drop(&mut self) {
        IN_STARTUP.fetch_sub(1, Ordering::SeqCst);
    }
}

/// Where a start was lost. Every give-up site on the startup path names one of
/// these and `ALL` names every one of them: the fixture's own case derives the
/// list from this enum's own source text, so a variant that is added and not
/// listed fails a case rather than shipping as a stage nothing reports.
#[derive(Clone, Copy, Debug)]
pub enum Stage<'a> {
    /// The process was never created, so no startup was ever timed.
    Spawn(&'a str),
    /// The child was gone before it reached `BiDi` readiness.
    Exited,
    /// The deadline expired before Firefox announced a `BiDi` endpoint.
    Announce,
    /// The deadline expired before the announced endpoint completed an upgrade.
    Connect,
    /// The upgrade was lost before the deadline: a read, write or socket failure.
    Upgrade(&'a str),
}
impl Stage<'_> {
    /// Every stage the fixture can give up a start at.
    #[allow(dead_code)]
    pub const ALL: &'static [Stage<'static>] = &[
        Stage::Spawn("<os error>"),
        Stage::Exited,
        Stage::Announce,
        Stage::Connect,
        Stage::Upgrade("<io error>"),
    ];
    /// The name a refusal reports this stage under.
    pub fn label(self) -> String {
        match self {
            Stage::Spawn(error) => format!("spawn ({error})"),
            Stage::Exited => "exited".to_owned(),
            Stage::Announce => "announce".to_owned(),
            Stage::Connect => "connect".to_owned(),
            Stage::Upgrade(error) => format!("upgrade ({error})"),
        }
    }
    /// Whether a startup was timed at all before this stage gave up.
    fn timed_a_startup(self) -> bool {
        !matches!(self, Stage::Spawn(_))
    }
}

/// How a startup this runner never delivered is reported. A start that missed
/// the deadline is a statement about the machine, not about `BiDi`: say so, and
/// carry the measurement and Firefox's own stderr into the runner output so the
/// reader never has to go looking for a log the next job deletes. A stage that
/// timed no startup says so, rather than printing a sub-millisecond number
/// beside a deadline that number was never compared against.
pub fn startup_refusal(
    stage: Stage<'_>,
    elapsed: Duration,
    deadline: Duration,
    evidence: &Path,
) -> String {
    let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap_or_default();
    let measurement = if stage.timed_a_startup() {
        format!(
            "\x20 measured startup: {:.3}s (spawn to give-up)\n\
\x20 deadline:         {:.3}s ({})\n",
            elapsed.as_secs_f64(),
            deadline.as_secs_f64(),
            if elapsed >= deadline {
                "expired"
            } else {
                "not reached; this start ended for the reason above"
            },
        )
    } else {
        "\x20 measured startup: not timed; the browser was never spawned\n".to_owned()
    };
    format!(
        "fixture environment refusal: this runner did not start Firefox for BiDi. \
The browser fixture is refusing its environment, not a BiDi protocol defect.\n\
\x20 stage:            {}\n\
{measurement}\
\x20 evidence:         {}\n\
\x20 firefox.stderr ({} bytes):\n{log}",
        stage.label(),
        evidence.display(),
        log.len(),
    )
}

impl Browser {
    pub fn new(evidence: &Path) -> Self {
        Self::launch(evidence, STARTUP_DEADLINE).unwrap_or_else(|refusal| panic!("{refusal}"))
    }
    /// Start Firefox and reach `BiDi` readiness, or refuse with the reason.
    pub fn launch(evidence: &Path, deadline: Duration) -> Result<Self, String> {
        let firefox = std::env::var_os("ESS_FIREFOX").unwrap_or_else(|| "firefox".into());
        Self::launch_program(evidence, &firefox, deadline)
    }
    // startup-path: begin
    // Every line between here and `startup-path: end` is on the path from a
    // caller asking for a browser to a browser that answers BiDi, and a start
    // lost on it is reported through `startup_refusal` and nothing else. A line
    // here that can end the process another way has to say which of the two
    // exceptions it is:
    //   `startup-path: harness` — this runner's own filesystem or process table,
    //      which is a broken machine and not a start that was lost; and
    //   `startup-path: defect`  — a deliberate BiDi defect signal, kept because a
    //      browser that answered is a browser that started.
    // `no_unaccounted_panic_site_can_end_a_start` in coverage_browser.rs reads
    // this region and holds that class, so a further give-up site cannot arrive
    // unnamed the way the WebSocket upgrade read did.
    /// The same startup against a named program, so the fixture's own refusal
    /// path can be exercised without a Firefox that misbehaves on demand.
    pub fn launch_program(
        evidence: &Path,
        program: &OsStr,
        deadline: Duration,
    ) -> Result<Self, String> {
        static NEXT_PROFILE: AtomicU64 = AtomicU64::new(0);
        let profile_root =
            std::env::var_os("ESS_BROWSER_TMPDIR").map_or_else(std::env::temp_dir, PathBuf::from);
        let profile = profile_root.join(format!(
            "ess-bidi-{}-{}",
            std::process::id(),
            NEXT_PROFILE.fetch_add(1, Ordering::Relaxed)
        ));
        // startup-path: harness
        fs::create_dir_all(&profile).unwrap();
        let mut command = Command::new(program);
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
                // startup-path: harness
                File::create(evidence.join("firefox.stdout")).unwrap(),
            ))
            .stderr(Stdio::from(
                // startup-path: harness
                File::create(evidence.join("firefox.stderr")).unwrap(),
            ));
        // startup-path: harness
        fs::write(evidence.join("firefox.command"), format!("{command:?}\n")).unwrap();
        fs::write(
            evidence.join("browser-profile.txt"),
            format!("{}\n", profile.display()),
        )
        // startup-path: harness
        .unwrap();
        let starting = Starting::begin();
        let started = Instant::now();
        let mut child = match command.spawn() {
            Ok(child) => OwnedChild(child),
            Err(error) => {
                return Err(startup_refusal(
                    Stage::Spawn(&error.to_string()),
                    started.elapsed(),
                    deadline,
                    evidence,
                ))
            }
        };
        // startup-path: harness
        fs::write(evidence.join("firefox.pid"), format!("{}\n", child.0.id())).unwrap();
        let (stream, response) = {
            // Counted for the whole startup rather than for the lock. A fixture
            // that released `starting` any earlier would leave these overlapping,
            // and the peak this counter records is what says so.
            let _in_startup = InStartup::begin();
            Self::reach_bidi(&mut child, evidence, started, deadline)
        }?;
        // startup-path: harness
        fs::write(evidence.join("websocket-handshake.txt"), &response).unwrap();
        // startup-path: defect
        assert!(response.starts_with("HTTP/1.1 101"), "{response}");
        // startup-path: defect
        assert!(response.contains(RFC6455_EXAMPLE_ACCEPT), "{response}");
        drop(starting);
        let mut browser = Self {
            _child: child,
            stream,
            next: 0,
            // startup-path: harness
            receipt: File::create(evidence.join("bidi.jsonl")).unwrap(),
            evidence: evidence.to_path_buf(),
        };
        browser.call("session.new", &json!({"capabilities":{"alwaysMatch":{}}}));
        Ok(browser)
    }

    /// Reach a `BiDi`-ready socket on the endpoint Firefox announced, or report
    /// the start this runner did not deliver.
    fn reach_bidi(
        child: &mut OwnedChild,
        evidence: &Path,
        started: Instant,
        deadline: Duration,
    ) -> Result<(TcpStream, String), String> {
        let port = Self::assigned_bidi_port(child, evidence, started, deadline)?;
        // The last thing the announced endpoint said, if it said anything at all.
        let mut answered: Option<String> = None;
        loop {
            // startup-path: harness
            if child.0.try_wait().unwrap().is_some() {
                return Err(startup_refusal(
                    Stage::Exited,
                    started.elapsed(),
                    deadline,
                    evidence,
                ));
            }
            let elapsed = started.elapsed();
            if elapsed >= deadline {
                return Err(Self::connect_give_up(
                    elapsed,
                    deadline,
                    evidence,
                    port,
                    answered.as_deref(),
                ));
            }
            let Ok(mut stream) = TcpStream::connect(("127.0.0.1", port)) else {
                thread::sleep(Duration::from_millis(50));
                continue;
            };
            // The socket inherits what is left of the deadline. A fixed timeout
            // here is not a bound on startup at all: it lets a single iteration
            // overrun the deadline by its own length, and it did, by 20s.
            let remaining = deadline
                .saturating_sub(elapsed)
                .max(Duration::from_millis(1));
            let response = match Self::upgrade(&mut stream, port, remaining) {
                Ok(response) => response,
                Err(error) => {
                    // startup-path: harness
                    if child.0.try_wait().unwrap().is_some() {
                        return Err(startup_refusal(
                            Stage::Exited,
                            started.elapsed(),
                            deadline,
                            evidence,
                        ));
                    }
                    let elapsed = started.elapsed();
                    if elapsed >= deadline {
                        return Err(Self::connect_give_up(
                            elapsed,
                            deadline,
                            evidence,
                            port,
                            answered.as_deref(),
                        ));
                    }
                    return Err(startup_refusal(
                        Stage::Upgrade(&error.to_string()),
                        elapsed,
                        deadline,
                        evidence,
                    ));
                }
            };
            // Firefox can listen before registering /session. TCP readiness alone is not
            // BiDi readiness. Keep the startup response as evidence; retain the same deadline.
            if response.starts_with("HTTP/1.1 404 ") {
                // startup-path: harness
                fs::write(evidence.join("websocket-startup-response.txt"), &response).unwrap();
                answered = Some(response);
                thread::sleep(Duration::from_millis(50));
                continue;
            }
            // The startup clamp does not outlive the startup. This socket is about to become the
            // browser's transport; the budget it carried was a bound on STARTING, and keeping it
            // made every later call inherit whatever was left over. See `SESSION_TIMEOUT`.
            if let Err(error) = stream
                .set_read_timeout(Some(SESSION_TIMEOUT))
                .and_then(|()| stream.set_write_timeout(Some(SESSION_TIMEOUT)))
            {
                // startup-path: harness
                let reason = format!("restoring the session timeouts: {error}");
                return Err(startup_refusal(
                    Stage::Upgrade(&reason),
                    started.elapsed(),
                    deadline,
                    evidence,
                ));
            }
            return Ok((stream, response));
        }
    }

    /// One upgrade attempt, bounded by what is left of the startup deadline.
    fn upgrade(stream: &mut TcpStream, port: u16, remaining: Duration) -> std::io::Result<String> {
        stream.set_read_timeout(Some(remaining))?;
        stream.set_write_timeout(Some(remaining))?;
        let handshake = format!("GET /session HTTP/1.1\r\nHost: 127.0.0.1:{port}\r\nUpgrade: websocket\r\nConnection: Upgrade\r\nSec-WebSocket-Key: {RFC6455_EXAMPLE_KEY}\r\nSec-WebSocket-Version: 13\r\n\r\n");
        stream.write_all(handshake.as_bytes())?;
        let mut response = Vec::new();
        while !response.ends_with(b"\r\n\r\n") {
            let mut byte = [0];
            stream.read_exact(&mut byte)?;
            response.push(byte[0]);
        }
        // A browser answering bytes that are not text has still answered: leave
        // that to the handshake assertions rather than a panic with no stage.
        Ok(String::from_utf8_lossy(&response).into_owned())
    }

    /// How the connect phase gives up once the deadline has expired. A browser
    /// that answered the upgrade with HTTP is running and listening on the port
    /// it announced, so this runner did start it: the refusal's claim would be
    /// false, and this says what was seen instead of picking a side it cannot
    /// see. A port that never answered anything is a start this runner lost, and
    /// that is the refusal.
    fn connect_give_up(
        elapsed: Duration,
        deadline: Duration,
        evidence: &Path,
        port: u16,
        answered: Option<&str>,
    ) -> String {
        let Some(response) = answered else {
            return startup_refusal(Stage::Connect, elapsed, deadline, evidence);
        };
        // startup-path: defect
        panic!(
            "Firefox did not expose BiDi: it announced 127.0.0.1:{port}, stayed alive, and \
             answered the upgrade with a response that is not 101 for {:.3}s. This runner did \
             start a browser, so it is not a fixture environment refusal; whether /session is \
             defective or this runner never gave the browser the CPU to register it is decided \
             by the response and the log below.\n\
             \x20 evidence:         {}\n\x20 last startup response:\n{response}",
            elapsed.as_secs_f64(),
            evidence.display(),
        );
    }

    fn assigned_bidi_port(
        child: &mut OwnedChild,
        evidence: &Path,
        started: Instant,
        deadline: Duration,
    ) -> Result<u16, String> {
        // Firefox documents the assigned BiDi endpoint on stderr. Read only a
        // complete line from this child's private log, under the startup deadline.
        loop {
            // startup-path: harness
            if child.0.try_wait().unwrap().is_some() {
                return Err(startup_refusal(
                    Stage::Exited,
                    started.elapsed(),
                    deadline,
                    evidence,
                ));
            }
            let elapsed = started.elapsed();
            if elapsed >= deadline {
                return Err(startup_refusal(
                    Stage::Announce,
                    elapsed,
                    deadline,
                    evidence,
                ));
            }
            let log = fs::read_to_string(evidence.join("firefox.stderr")).unwrap_or_default();
            if let Some(port) = log.split_inclusive('\n').find_map(|line| {
                line.strip_suffix('\n')?
                    .trim_end()
                    .strip_prefix("WebDriver BiDi listening on ws://127.0.0.1:")?
                    .parse::<u16>()
                    .ok()
                    .filter(|port| *port != 0)
            }) {
                return Ok(port);
            }
            thread::sleep(Duration::from_millis(50));
        }
    }
    // startup-path: end

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
    /// The evidence directory this browser's start, stderr and `BiDi` receipt were written to.
    /// Read by a suite that reuses one browser across fixtures; the others start one per fixture.
    #[allow(dead_code)]
    pub fn evidence(&self) -> &Path {
        &self.evidence
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
