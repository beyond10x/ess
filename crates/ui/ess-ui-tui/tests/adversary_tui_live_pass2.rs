//! Adversary pass 2 on the terminal bound to a served surface (story:ui-tui-live-binding), after
//! correction 1: a confirm's memory of what was already done, the HTTP/1.1 framing the new reader
//! accepts, and the hosts a base URL may name, driven against scripted adapters and raw TCP stubs.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::io::{Read as _, Write as _};
use std::net::{TcpListener, TcpStream};
use std::panic::AssertUnwindSafe;
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

/// The register overlay of the desk: a whole valid visit.
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
            expected_minutes: 30
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

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-adv-live2")
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
    commands: Vec<String>,
}

impl Log {
    fn sent(&self, command: &str) -> usize {
        self.commands.iter().filter(|sent| *sent == command).count()
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
        let row: Value = serde_yaml::from_str(
            "{id: v1, visit_id: v1, visitor: Ada, building: North, state: Expected}",
        )
        .expect("the row parses");
        Ok(ReadResult {
            rows: vec![row],
            ..ReadResult::default()
        })
    }
    fn run(&mut self, command: &str, _: &BTreeMap<String, Value>) -> Answer {
        self.log.borrow_mut().commands.push(command.to_owned());
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

fn scripted(document: ess_ui::Document, test: &str) -> (App, Rc<RefCell<Log>>, Answers) {
    let binding = binding_of(&document);
    let log = Rc::new(RefCell::new(Log::default()));
    let answers: Answers = Rc::new(RefCell::new(BTreeMap::new()));
    let app = App::bound(
        document,
        Box::new(Scripted {
            log: Rc::clone(&log),
            answers: Rc::clone(&answers),
        }),
        binding,
        Options::new(state_dir(test)),
    )
    .unwrap_or_else(|error| panic!("the desk runs bound: {error}"));
    (app, log, answers)
}

// ── a confirm's memory of what was already done ───────────────────────────────────────────────

/// `ess_ui::binding::Answer::Unfinished` says a client must not retry a command whose answer is
/// `committed: true`: its effect stands, and the screen tells the user "do not send it again".
/// A confirm whose own command answered so stays open, and confirming it again must not send that
/// command a second time; correction 1 remembers only an accepted confirm command.
#[test]
fn a_confirm_command_answered_committed_unfinished_is_not_sent_again_on_retry() {
    let (mut app, log, answers) = scripted(
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
        "confirm-committed",
    );
    answers.borrow_mut().insert(
        "visit.SignOutVisitor".to_owned(),
        Answer::Unfinished { committed: true },
    );
    app.focus_section("expected");
    app.render_text(WIDTH, HEIGHT);
    app.keys("a");
    app.keys("y");
    let screen = app.render_text(WIDTH, HEIGHT);
    assert!(
        screen.contains("do not send it again") && screen.contains("Sign the visitor out first?"),
        "the confirm stays open and says the effect stands:\n{screen}"
    );
    assert_eq!(log.borrow().sent("visit.SignOutVisitor"), 1);

    app.keys("y");
    assert_eq!(
        log.borrow().sent("visit.SignOutVisitor"),
        1,
        "the committed confirm command was sent again: {:?}",
        log.borrow().commands
    );
}

/// The same contract on a form: a submit answered `committed: true` keeps the form open with its
/// draft, and the next ctrl-s sends the committed command again.
#[test]
fn a_form_submit_answered_committed_unfinished_is_not_sent_again() {
    let (mut app, log, answers) = scripted(
        desk("          - {name: leave, opens: register}\n", ""),
        "form-committed",
    );
    answers.borrow_mut().insert(
        "visit.RegisterVisit".to_owned(),
        Answer::Unfinished { committed: true },
    );
    app.render_text(WIDTH, HEIGHT);
    app.open_overlay("register");
    app.keys("<c-s>");
    let screen = app.render_text(WIDTH, HEIGHT);
    assert!(screen.contains("do not send it again"), "{screen}");
    assert_eq!(log.borrow().sent("visit.RegisterVisit"), 1);

    app.keys("<c-s>");
    assert_eq!(
        log.borrow().sent("visit.RegisterVisit"),
        1,
        "the committed registration was sent a second time: {:?}",
        log.borrow().commands
    );
}

// ── HTTP/1.1 framing, against a raw TCP stub ──────────────────────────────────────────────────

/// A one-thread HTTP stub on `127.0.0.1:0`: each connection's request is read whole and
/// recorded, then answered with the bytes `answer` gives, and the connection closed.
struct Stub {
    port: u16,
    requests: Arc<Mutex<Vec<String>>>,
}

fn stub<F>(answer: F) -> Stub
where
    F: Fn(&str) -> Vec<u8> + Send + 'static,
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
            let _ = stream.write_all(&answer(&request));
            let _ = stream.flush();
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
    HttpAdapter::new(binding, &bases, None)
        .unwrap_or_else(|error| panic!("the adapter is built: {error}"))
}

/// Sends the registration to a stub answering `bytes`, and returns the answer and how long it
/// took, or the panic message if sending panicked.
fn command_against(bytes: Vec<u8>) -> Result<(Answer, Duration), String> {
    let server = stub(move |_| bytes.clone());
    let mut adapter = adapter_at(&format!("http://127.0.0.1:{}", server.port));
    let started = Instant::now();
    let answered = std::panic::catch_unwind(AssertUnwindSafe(|| {
        adapter.run("visit.RegisterVisit", &registration())
    }))
    .map_err(|panic| {
        panic
            .downcast_ref::<String>()
            .cloned()
            .or_else(|| panic.downcast_ref::<&str>().map(|text| (*text).to_owned()))
            .unwrap_or_default()
    })?;
    assert_eq!(server.requests.lock().expect("unpoisoned").len(), 1);
    Ok((answered, started.elapsed()))
}

/// A chunk size is hexadecimal of any length, and a hostile or broken server may declare one no
/// buffer can hold. The reader gives such an answer up as no answer; it must not panic the
/// terminal (`size + 2` overflows `usize` for `FFFFFFFFFFFFFFFF`).
#[test]
fn a_chunk_size_beyond_usize_is_no_answer_not_a_panic() {
    let bytes =
        b"HTTP/1.1 202 Accepted\r\nTransfer-Encoding: chunked\r\n\r\nFFFFFFFFFFFFFFFF\r\n{\"o"
            .to_vec();
    let answered = command_against(bytes).map(|(answer, _)| answer);
    assert_eq!(answered, Ok(Answer::Transport));
}

/// Two `Content-Length` fields with different values leave the framing undecidable, and a
/// recipient must treat that as an unrecoverable error (RFC 9112 §6.3, rule 5), not pick one.
#[test]
fn two_differing_content_lengths_are_no_answer() {
    let bytes = format!(
        "HTTP/1.1 202 Accepted\r\nContent-Length: 2\r\nContent-Length: {}\r\n\r\n{ACCEPTED}",
        ACCEPTED.len()
    )
    .into_bytes();
    let answered = command_against(bytes).map(|(answer, _)| answer);
    assert_eq!(answered, Ok(Answer::Transport));
}

// ── the base URL ──────────────────────────────────────────────────────────────────────────────

/// A host that cannot be connected to as written (an IPv6 literal that is no address, a
/// percent-encoded name sent verbatim as `Host` and to the resolver) is refused when the adapter
/// is built, as correction 1 does for a malformed port, rather than answering every read and
/// command as "No answer from the server".
#[test]
fn a_base_url_whose_host_names_no_host_is_refused_when_the_adapter_is_built() {
    let binding = binding_of(&desk("          - {name: leave, opens: register}\n", ""));
    let accepted: Vec<&str> = ["http://[zz]:8080", "http://%31%32%37.0.0.1:8080"]
        .into_iter()
        .filter(|url| {
            let bases = BTreeMap::from([("pass-service".to_owned(), (*url).to_owned())]);
            HttpAdapter::new(binding.clone(), &bases, None).is_ok()
        })
        .collect();
    assert_eq!(accepted, Vec::<&str>::new(), "accepted as base URLs");
}
