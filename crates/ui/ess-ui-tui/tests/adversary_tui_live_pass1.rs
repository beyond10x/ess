//! Adversary pass 1 on the terminal bound to a served surface (story:ui-tui-live-binding): where a
//! refusal shows and what a command's answer does to the screen, the base URL the adapter
//! accepts, and the HTTP/1.1 framing it reads, driven against scripted adapters and raw TCP stubs.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use ess_ui::binding::{Answer, Binding};
use ess_ui_tui::{App, DataAdapter, HttpAdapter, Options, ReadRequest, ReadResult};
use serde_yaml::Value;

const WIDTH: u16 = 200;
const HEIGHT: u16 = 50;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

fn gatepass() -> Vec<(String, String)> {
    let root = root().join("examples/gatepass");
    ["system.yaml", "components.yaml", "domains/visit.yaml"]
        .iter()
        .map(|file| ((*file).to_owned(), read(&root.join(file))))
        .collect()
}

/// The register overlay of the desk: a whole valid visit except its length.
const REGISTER: &str = "      register:
        kind: dialog
        title: Register a visit
        component: form
        does: visit.RegisterVisit
        fields:
          - visitor
          - building
          - host
          - {field: expected_minutes, as: number}
          - expected_stay
          - deposit
          - escorts
          - notes
          - on_watchlist
        draft:
          type: RegisterDraft
          class: draft
          default:
            visitor: Ada
            building: North
            host: {kind: employee, value: E1}
            expected_minutes: 0
            expected_stay: PT30M
            deposit: {amount: '0', currency: EUR}
            escorts: []
            notes: {}
            on_watchlist: false
";

/// A desk over `examples/gatepass` whose expected list offers `row_actions` (lines of YAML list
/// items) and whose page holds `overlays` (lines of YAML mapping entries) beside `register`.
fn desk(row_actions: &str, overlays: &str) -> ess_ui::Document {
    let text = format!(
        "format: ess-ui/1
app: desk
model: gatepass
placement_profile: fat
shells: {{app: {{regions: {{main: {{kind: page_outlet}}}}}}}}
navigation: {{home: desk, sections: [{{name: all, pages: [desk]}}]}}
pages:
  desk:
    kind: detail_page
    title: Desk
    sections:
      - name: expected
        component: collection
        reads: visit.ExpectedVisits
        columns: [visitor, building]
        row_actions:
{row_actions}    overlays:
{REGISTER}{overlays}"
    );
    ess_ui::load_str(&text).unwrap_or_else(|error| panic!("the desk loads: {error}\n{text}"))
}

fn binding_of(document: &ess_ui::Document) -> Binding {
    ess_ui_check::binding(document, &gatepass())
        .unwrap_or_else(|error| panic!("the desk binds: {error}"))
}

fn display_of(binding: &Binding, command: &str, error: &str) -> String {
    binding.components["pass-service"].commands[command].errors[error]
        .display
        .clone()
}

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adv-live1")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

// ── a scripted adapter ────────────────────────────────────────────────────────────────────────

/// What a [`Scripted`] adapter was asked.
#[derive(Default)]
struct Log {
    reads: usize,
    commands: Vec<(String, BTreeMap<String, Value>)>,
}

impl Log {
    fn sent(&self, command: &str) -> usize {
        self.commands
            .iter()
            .filter(|(sent, _)| sent == command)
            .count()
    }
}

/// Answers every read with one expected visit, and each command with the answer scripted for it
/// (accepted when none is).
struct Scripted {
    log: Rc<RefCell<Log>>,
    answers: Rc<RefCell<BTreeMap<String, Answer>>>,
}

impl DataAdapter for Scripted {
    fn read(&self, _: &ReadRequest) -> Result<ReadResult, String> {
        self.log.borrow_mut().reads += 1;
        let row: Value = serde_yaml::from_str(
            "{id: v1, visit_id: v1, visitor: Ada, building: North, state: Expected}",
        )
        .expect("the row parses");
        Ok(ReadResult {
            rows: vec![row],
            ..ReadResult::default()
        })
    }
    fn run(&mut self, command: &str, input: &BTreeMap<String, Value>) -> Answer {
        self.log
            .borrow_mut()
            .commands
            .push((command.to_owned(), input.clone()));
        self.answers
            .borrow()
            .get(command)
            .cloned()
            .unwrap_or(Answer::Accepted)
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

type Answers = Rc<RefCell<BTreeMap<String, Answer>>>;

fn scripted(document: ess_ui::Document, test: &str) -> (App, Rc<RefCell<Log>>, Answers, Binding) {
    let binding = binding_of(&document);
    let log = Rc::new(RefCell::new(Log::default()));
    let answers: Answers = Rc::new(RefCell::new(BTreeMap::new()));
    let app = App::bound(
        document,
        Box::new(Scripted {
            log: Rc::clone(&log),
            answers: Rc::clone(&answers),
        }),
        binding.clone(),
        Options::new(state_dir(test)),
    )
    .unwrap_or_else(|error| panic!("the desk runs bound: {error}"));
    (app, log, answers, binding)
}

fn conflict() -> Answer {
    Answer::Refused {
        error: "gatepass.visit.VisitStateConflict".to_owned(),
        payload: Some(serde_json::json!({"state": "Expected"})),
    }
}

// ── where a refusal shows ─────────────────────────────────────────────────────────────────────

/// A refusal shown on an inline confirm belongs to that attempt. Cancelling and choosing the
/// action again opens a fresh confirm, which must not already say the command was refused.
#[test]
fn a_reopened_inline_confirm_shows_no_refusal_from_the_last_attempt() {
    let (mut app, log, answers, binding) = scripted(
        desk(
            "          - {name: depart, label: Depart, does: visit.SignOutVisitor, \
             bind: {visit_id: row.visit_id}, confirm: {title: Sign the visitor out}}\n",
            "",
        ),
        "inline-confirm",
    );
    answers
        .borrow_mut()
        .insert("visit.SignOutVisitor".to_owned(), conflict());
    let display = display_of(
        &binding,
        "gatepass.visit.SignOutVisitor",
        "gatepass.visit.VisitStateConflict",
    );
    app.focus_section("expected");
    app.render_text(WIDTH, HEIGHT);
    app.keys("d");
    app.keys("y");
    let refused = app.render_text(WIDTH, HEIGHT);
    assert!(
        refused.contains(&display),
        "the refusal shows on the confirm:\n{refused}"
    );
    assert_eq!(log.borrow().sent("visit.SignOutVisitor"), 1);

    // Cancelled, then chosen again: a new confirm, nothing sent yet.
    app.keys("n");
    let closed = app.render_text(WIDTH, HEIGHT);
    assert!(!closed.contains(&display), "{closed}");
    app.keys("d");
    let reopened = app.render_text(WIDTH, HEIGHT);
    assert!(
        reopened.contains("Sign the visitor out"),
        "the confirm is open again:\n{reopened}"
    );
    assert_eq!(log.borrow().sent("visit.SignOutVisitor"), 1, "nothing sent");
    assert!(
        !reopened.contains(&display),
        "a freshly opened confirm already shows the last attempt's refusal:\n{reopened}"
    );
}

/// An action confirmed through a confirm overlay that sends a command of its own: the confirm's
/// command is accepted, the action's is refused, the confirm reopens. Confirming again retries
/// what was refused; the command that was already accepted must not be sent a second time.
#[test]
fn retrying_a_refused_confirmed_action_does_not_resend_the_accepted_confirm_command() {
    let (mut app, log, answers, _) = scripted(
        desk(
            "          - {name: admit, label: Admit, does: visit.AdmitVisitor, \
             bind: {visit_id: row.visit_id}, confirm: leave}\n",
            "      leave:
        kind: dialog
        title: Sign out first
        component: confirm
        does: visit.SignOutVisitor
        params: {visit_id: row.visit_id}
        body: Sign the visitor out first?
",
        ),
        "confirm-resend",
    );
    answers
        .borrow_mut()
        .insert("visit.AdmitVisitor".to_owned(), conflict());
    app.focus_section("expected");
    app.render_text(WIDTH, HEIGHT);
    app.keys("a");
    app.keys("y");
    let screen = app.render_text(WIDTH, HEIGHT);
    assert!(
        screen.contains("Sign the visitor out first?"),
        "the confirm is back after the refusal:\n{screen}"
    );
    assert_eq!(log.borrow().sent("visit.SignOutVisitor"), 1);
    assert_eq!(log.borrow().sent("visit.AdmitVisitor"), 1);

    app.keys("y");
    assert_eq!(
        log.borrow().sent("visit.AdmitVisitor"),
        2,
        "the refused action is retried"
    );
    assert_eq!(
        log.borrow().sent("visit.SignOutVisitor"),
        1,
        "the confirm's command, already accepted, was sent again: {:?}",
        log.borrow().commands
    );
}

// ── what an answer does to the screen ─────────────────────────────────────────────────────────

/// A `501` with `committed: true` says the command's effect stands. What the screen read is out
/// of date exactly as after an accepted command, so it is read again.
#[test]
fn a_committed_unfinished_answer_reads_the_screen_again() {
    let (mut app, log, answers, _) = scripted(
        desk(
            "          - {name: depart, label: Depart, does: visit.SignOutVisitor, \
             bind: {visit_id: row.visit_id}}\n",
            "",
        ),
        "unfinished-committed",
    );
    app.focus_section("expected");
    app.render_text(WIDTH, HEIGHT);

    // Control: an accepted command reads the list again.
    let before = log.borrow().reads;
    app.keys("d");
    assert!(
        log.borrow().reads > before,
        "an accepted command reads again"
    );

    answers.borrow_mut().insert(
        "visit.SignOutVisitor".to_owned(),
        Answer::Unfinished { committed: true },
    );
    let before = log.borrow().reads;
    app.keys("d");
    let screen = app.render_text(WIDTH, HEIGHT);
    assert!(screen.contains("do not send it again"), "{screen}");
    assert!(
        log.borrow().reads > before,
        "the effect was committed, and the list was not read again ({before} reads before and after)"
    );
}

// ── the base URL ──────────────────────────────────────────────────────────────────────────────

/// A base URL whose authority cannot be connected to (a port out of range, not a number, an
/// unclosed IPv6 bracket) is refused when the adapter is built, before the terminal is touched,
/// rather than answering every read and command as "No answer from the server".
#[test]
fn a_base_url_with_a_malformed_authority_is_refused_when_the_adapter_is_built() {
    let binding = binding_of(&desk("          - {name: leave, opens: register}\n", ""));
    let accepted: Vec<&str> = [
        "http://127.0.0.1:99999",
        "http://127.0.0.1:port",
        "http://[::1",
        "http://desk:",
    ]
    .into_iter()
    .filter(|url| {
        let bases = BTreeMap::from([("pass-service".to_owned(), (*url).to_owned())]);
        HttpAdapter::new(binding.clone(), &bases, None).is_ok()
    })
    .collect();
    assert_eq!(accepted, Vec::<&str>::new(), "accepted as base URLs");
}

// ── HTTP/1.1 framing, against a raw TCP stub ──────────────────────────────────────────────────

/// A one-thread HTTP stub on `127.0.0.1:0`: each connection's request is read whole (headers
/// and `Content-Length` body) and recorded, then answered with the bytes `answer` gives; when
/// it also says `hold`, the connection is kept open until the client closes it (20 s at most).
struct Stub {
    port: u16,
    requests: Arc<Mutex<Vec<String>>>,
}

fn stub<F>(answer: F) -> Stub
where
    F: Fn(&str) -> (Vec<u8>, bool) + Send + 'static,
{
    let listener = TcpListener::bind("127.0.0.1:0").expect("a port");
    let port = listener.local_addr().expect("bound").port();
    let requests = Arc::new(Mutex::new(Vec::new()));
    let recorded = Arc::clone(&requests);
    std::thread::spawn(move || {
        for stream in listener.incoming() {
            let Ok(mut stream) = stream else { continue };
            let request = read_request(&mut stream);
            recorded.lock().expect("unpoisoned").push(request.clone());
            let (bytes, hold) = answer(&request);
            let _ = stream.write_all(&bytes);
            let _ = stream.flush();
            if hold {
                let _ = stream.set_read_timeout(Some(Duration::from_secs(20)));
                let mut sink = [0_u8; 64];
                while matches!(stream.read(&mut sink), Ok(read) if read > 0) {}
            }
        }
    });
    Stub { port, requests }
}

fn read_request(stream: &mut TcpStream) -> String {
    let _ = stream.set_read_timeout(Some(Duration::from_secs(5)));
    let mut buffer = Vec::new();
    let mut chunk = [0_u8; 4096];
    loop {
        if let Some(end) = buffer.windows(4).position(|window| window == b"\r\n\r\n") {
            let head = String::from_utf8_lossy(&buffer[..end]).to_string();
            let length = head
                .lines()
                .find_map(|line| {
                    let (name, value) = line.split_once(':')?;
                    name.eq_ignore_ascii_case("content-length")
                        .then(|| value.trim().parse::<usize>().ok())?
                })
                .unwrap_or(0);
            if buffer.len() >= end + 4 + length {
                return String::from_utf8_lossy(&buffer).to_string();
            }
        }
        match stream.read(&mut chunk) {
            Ok(0) | Err(_) => return String::from_utf8_lossy(&buffer).to_string(),
            Ok(read) => buffer.extend_from_slice(&chunk[..read]),
        }
    }
}

fn answer(status: &str, body: &str, close: bool) -> Vec<u8> {
    format!(
        "HTTP/1.1 {status}\r\nContent-Type: application/json\r\nContent-Length: {}\r\n{}\r\n{body}",
        body.len(),
        if close { "Connection: close\r\n" } else { "" }
    )
    .into_bytes()
}

const ACCEPTED: &str = "{\"outcome\":\"registered\",\"published\":[]}";

fn registration() -> BTreeMap<String, Value> {
    serde_yaml::from_str(
        "{visitor: Ada, building: North, host: {kind: employee, value: E1}, \
         expected_minutes: 30, expected_stay: PT30M, \
         deposit: {amount: '0', currency: EUR}, escorts: [], notes: {}, on_watchlist: false}",
    )
    .expect("the input parses")
}

fn adapter_at(base: &str) -> HttpAdapter {
    let binding = binding_of(&desk("          - {name: leave, opens: register}\n", ""));
    let bases = BTreeMap::from([("pass-service".to_owned(), base.to_owned())]);
    HttpAdapter::new(
        binding,
        &bases,
        Some("Actor gatepass.visit.Receptionist".to_owned()),
    )
    .unwrap_or_else(|error| panic!("the adapter is built: {error}"))
}

/// HTTP/1.1 lets a server send an interim `1xx` answer before the final one, and a client must
/// read past it (RFC 9110 §15.2). The command's answer is the final `202`.
#[test]
fn an_interim_100_continue_is_read_past_to_the_final_answer() {
    let server = stub(|_| {
        let mut bytes = b"HTTP/1.1 100 Continue\r\n\r\n".to_vec();
        bytes.extend(answer("202 Accepted", ACCEPTED, true));
        (bytes, false)
    });
    let mut adapter = adapter_at(&format!("http://127.0.0.1:{}", server.port));
    assert_eq!(
        adapter.run("visit.RegisterVisit", &registration()),
        Answer::Accepted
    );
}

/// An answer whole by its `Content-Length` is complete when its last byte arrives. A server (or
/// a proxy in front of it) that keeps the connection open must not hold the terminal for the
/// whole 15 s timeout and then turn an accepted command into "No answer from the server".
#[test]
fn an_answer_complete_by_its_content_length_is_not_held_until_the_connection_closes() {
    let server = stub(|_| (answer("202 Accepted", ACCEPTED, false), true));
    let mut adapter = adapter_at(&format!("http://127.0.0.1:{}", server.port));
    let started = Instant::now();
    let answered = adapter.run("visit.RegisterVisit", &registration());
    let took = started.elapsed();
    assert_eq!(
        server.requests.lock().expect("unpoisoned").len(),
        1,
        "the stub saw the command"
    );
    assert_eq!(
        (answered, took < Duration::from_secs(3)),
        (Answer::Accepted, true),
        "answered after {took:?}"
    );
}
