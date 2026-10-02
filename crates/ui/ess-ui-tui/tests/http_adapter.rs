//! The terminal bound to a served surface (beyond10x/ess#311, S3): `HttpAdapter` reads and commands
//! the paths `ess_ui_check::binding` computes, every command answer is classified by
//! `ess_ui::binding::classify`, and a refusal shows where the user acted — on the open form
//! overlay, on the confirm overlay, or beside the action row — keeping the draft.
//!
//! Every acceptance case runs against the Rust gatepass server from
//! `examples/gatepass-realization`, started on `127.0.0.1:0`, and the screen cases drive the app
//! headless through ratatui's `TestBackend`. The polling case counts reads with a stub adapter.

use std::collections::BTreeMap;
use std::io::BufRead as _;
use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::OnceLock;

use ess_ui::binding::{classify, Answer, Binding};
use ess_ui_tui::{App, DataAdapter, HttpAdapter, Options, ReadRequest, Region};
use serde_yaml::Value;

const WIDTH: u16 = 120;
const HEIGHT: u16 = 40;

const RECEPTIONIST: &str = "Actor gatepass.visit.Receptionist";

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../..")
}

fn read(path: &Path) -> String {
    std::fs::read_to_string(path).unwrap_or_else(|error| panic!("{}: {error}", path.display()))
}

/// A front desk over `examples/gatepass`: the expected list with two row actions, the
/// registration form as an overlay whose draft is a whole valid visit except its length, and a
/// confirm overlay that signs a visitor out.
const DESK: &str = "format: ess-ui/1
app: desk
model: gatepass
placement_profile: fat
shells: {app: {regions: {main: {kind: page_outlet}}}}
navigation: {home: desk, sections: [{name: all, pages: [desk]}]}
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
          - {name: depart, label: Depart, does: visit.SignOutVisitor, bind: {visit_id: row.visit_id}}
          - {name: leave, label: Leave, opens: leave}
    overlays:
      register:
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
      leave:
        kind: dialog
        title: Sign out
        component: confirm
        does: visit.SignOutVisitor
        params: {visit_id: row.visit_id}
        body: Sign the visitor out?
";

/// `examples/gatepass` as `(label, text)` sources.
fn gatepass() -> Vec<(String, String)> {
    let root = root().join("examples/gatepass");
    ["system.yaml", "components.yaml", "domains/visit.yaml"]
        .iter()
        .map(|file| ((*file).to_owned(), read(&root.join(file))))
        .collect()
}

fn document() -> ess_ui::Document {
    ess_ui::load_str(DESK).unwrap_or_else(|error| panic!("the desk loads: {error}"))
}

fn binding() -> Binding {
    ess_ui_check::binding(&document(), &gatepass())
        .unwrap_or_else(|error| panic!("the desk binds: {error}"))
}

// ── the server ────────────────────────────────────────────────────────────────────────────────

/// Builds `examples/gatepass-realization`'s server into its own target directory, once: a nested
/// `cargo build` into the running test's target directory would wait on its lock.
fn gatepass_server() -> &'static Path {
    static BUILT: OnceLock<PathBuf> = OnceLock::new();
    BUILT.get_or_init(|| {
        let target = Path::new(env!("CARGO_TARGET_TMPDIR")).join("gatepass-target");
        let built = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args([
                "build",
                "--offline",
                "--quiet",
                "--bin",
                "gatepass-server",
                "--manifest-path",
            ])
            .arg(root().join("examples/gatepass-realization/Cargo.toml"))
            .arg("--target-dir")
            .arg(&target)
            .env_remove("CARGO_TARGET_DIR")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        assert!(
            built.status.success(),
            "the Rust gatepass server builds: {}",
            String::from_utf8_lossy(&built.stderr)
        );
        target.join("debug/gatepass-server")
    })
}

/// Kills the server when the case ends, pass or fail.
struct Served(Child);

impl Drop for Served {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}

/// Starts a fresh server on an ephemeral port of `127.0.0.1`; returns it and its base URL.
fn serve() -> (Served, String) {
    let mut child = Command::new(gatepass_server())
        .env("PORT", "0")
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .spawn()
        .expect("the server starts");
    let stdout = child.stdout.take().expect("piped");
    let served = Served(child);
    let mut lines = std::io::BufReader::new(stdout).lines();
    let first = lines.next().expect("a startup record").expect("text");
    let record: serde_json::Value = serde_json::from_str(&first).expect("the record is JSON");
    let port = record["runtime"]["port"]
        .as_u64()
        .unwrap_or_else(|| panic!("the record names the port: {first}"));
    std::thread::spawn(move || lines.for_each(drop));
    (served, format!("http://127.0.0.1:{port}"))
}

fn adapter(binding: &Binding, base: &str, authorization: &str) -> HttpAdapter {
    let bases = BTreeMap::from([("pass-service".to_owned(), base.to_owned())]);
    HttpAdapter::new(binding.clone(), &bases, Some(authorization.to_owned()))
        .unwrap_or_else(|error| panic!("the adapter is built: {error}"))
}

fn input(yaml: &str) -> BTreeMap<String, Value> {
    serde_yaml::from_str(yaml).expect("the input parses")
}

/// A whole valid registration, `expected_minutes` long.
fn registration(minutes: i64) -> BTreeMap<String, Value> {
    input(&format!(
        "{{visitor: Ada, building: North, host: {{kind: employee, value: E1}}, \
         expected_minutes: {minutes}, expected_stay: PT30M, \
         deposit: {{amount: '0', currency: EUR}}, escorts: [], notes: {{}}, on_watchlist: false}}"
    ))
}

fn expected(adapter: &HttpAdapter) -> Vec<Value> {
    adapter
        .read(&ReadRequest {
            view: "visit.ExpectedVisits".to_owned(),
            fixture: None,
            params: BTreeMap::new(),
        })
        .unwrap_or_else(|error| panic!("ExpectedVisits reads: {error}"))
        .rows
}

fn display_of(binding: &Binding, command: &str, error: &str) -> String {
    binding.components["pass-service"].commands[command].errors[error]
        .display
        .clone()
}

// ── the adapter ───────────────────────────────────────────────────────────────────────────────

#[test]
fn reads_and_commands_reach_the_gatepass_rust_server() {
    let binding = binding();
    let (_server, base) = serve();
    let mut desk = adapter(&binding, &base, RECEPTIONIST);

    // The view reads from its served path, empty at first.
    assert_eq!(expected(&desk), Vec::<Value>::new());

    // RegisterVisit as a Receptionist is accepted, and the visit reads back.
    let (status, body) = desk
        .post("visit.RegisterVisit", &registration(30))
        .unwrap_or_else(|error| panic!("the command is answered: {error}"));
    assert!((200..300).contains(&status), "{status} {body}");
    assert_eq!(classify(status, &body), Answer::Accepted, "{body}");
    assert_eq!(
        desk.run("visit.RegisterVisit", &registration(45)),
        Answer::Accepted
    );
    let rows = expected(&desk);
    assert_eq!(rows.len(), 2, "{rows:?}");
    assert_eq!(rows[0]["visitor"], Value::String("Ada".into()));
    let visit_id = rows[0]["visit_id"].clone();

    // A zero-length visit is refused 422 with its declared error and payload.
    let (status, body) = desk
        .post("visit.RegisterVisit", &registration(0))
        .expect("answered");
    assert_eq!(status, 422, "{body}");
    let refused = Answer::Refused {
        error: "gatepass.visit.InvalidVisitLength".to_owned(),
        payload: Some(serde_json::json!({"submitted": 0})),
    };
    assert_eq!(classify(status, &body), refused);
    assert_eq!(desk.run("visit.RegisterVisit", &registration(0)), refused);
    assert_eq!(expected(&desk).len(), 2, "nothing was recorded");

    // As a SecurityAuditor, whom no grant admits: not granted, naming the actor.
    let mut auditor = adapter(&binding, &base, "Actor gatepass.visit.SecurityAuditor");
    let (status, body) = auditor
        .post("visit.RegisterVisit", &registration(30))
        .expect("answered");
    assert_eq!(status, 403, "{body}");
    assert_eq!(
        auditor.run("visit.RegisterVisit", &registration(30)),
        Answer::NotGranted {
            actor: Some("gatepass.visit.SecurityAuditor".to_owned())
        }
    );

    // Signing out a visit that is still Expected is refused 409 with the state it is in.
    let mut sign_out = BTreeMap::new();
    sign_out.insert("visit_id".to_owned(), visit_id);
    let (status, body) = desk
        .post("visit.SignOutVisitor", &sign_out)
        .expect("answered");
    assert_eq!(status, 409, "{body}");
    assert_eq!(
        desk.run("visit.SignOutVisitor", &sign_out),
        Answer::Refused {
            error: "gatepass.visit.VisitStateConflict".to_owned(),
            payload: Some(serde_json::json!({"state": "Expected"})),
        }
    );
}

// ── where a refusal shows ─────────────────────────────────────────────────────────────────────

fn state_dir(test: &str) -> PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR"))
        .join("ess-ui-tui-http")
        .join(test);
    if dir.exists() {
        std::fs::remove_dir_all(&dir).expect("a stale state dir is removed");
    }
    dir
}

/// The desk running against a fresh server, with one visit registered beforehand when `seeded`.
fn desk(test: &str, seeded: bool) -> (Served, App, HttpAdapter, Binding) {
    let binding = binding();
    let (server, base) = serve();
    let mut probe = adapter(&binding, &base, RECEPTIONIST);
    if seeded {
        assert_eq!(
            probe.run("visit.RegisterVisit", &registration(30)),
            Answer::Accepted
        );
    }
    let app = App::bound(
        document(),
        Box::new(adapter(&binding, &base, RECEPTIONIST)),
        binding.clone(),
        Options::new(state_dir(test)),
    )
    .unwrap_or_else(|error| panic!("the desk runs bound: {error}"));
    (server, app, probe, binding)
}

/// The region recorded at `path` on the last frame.
fn region(app: &App, path: &str) -> Region {
    app.regions()
        .into_iter()
        .find(|region| region.path == path)
        .unwrap_or_else(|| panic!("{path} was drawn: {:?}", app.regions()))
}

/// The screen line holding `needle`, and its index.
fn line_of<'s>(screen: &'s str, needle: &str) -> (usize, &'s str) {
    screen
        .lines()
        .enumerate()
        .find(|(_, line)| line.contains(needle))
        .unwrap_or_else(|| panic!("`{needle}` is on the screen:\n{screen}"))
}

fn inside(region: &Region, row: usize) -> bool {
    let row = u16::try_from(row).expect("a screen row");
    row >= region.area.y && row < region.area.y + region.area.height
}

#[test]
fn a_refusal_shows_on_the_open_form() {
    let (_server, mut app, probe, binding) = desk("form", false);
    let display = display_of(
        &binding,
        "gatepass.visit.RegisterVisit",
        "gatepass.visit.InvalidVisitLength",
    );
    app.open_overlay("register");
    app.keys("<c-s>");
    let screen = app.render_text(WIDTH, HEIGHT);
    // The overlay stays open, the refusal inside it, with the error's field.
    let overlay = region(&app, "pages/desk/overlays/register");
    let (at, _) = line_of(&screen, &display);
    assert!(
        inside(&overlay, at),
        "the refusal is on the form:\n{screen}"
    );
    let (payload, _) = line_of(&screen, "submitted");
    assert!(inside(&overlay, payload), "{screen}");
    // The draft is kept, and nothing was recorded.
    assert!(screen.contains("Ada"), "the draft is kept:\n{screen}");
    assert_eq!(expected(&probe).len(), 0);

    // Corrected in place and sent again: accepted, the overlay closes and the list reads again.
    app.keys("jjj<enter><bs>30<esc><c-s>");
    let screen = app.render_text(WIDTH, HEIGHT);
    assert!(!screen.contains(&display), "{screen}");
    assert!(
        app.regions()
            .iter()
            .all(|region| region.path != "pages/desk/overlays/register"),
        "the accepted form closes:\n{screen}"
    );
    assert_eq!(expected(&probe).len(), 1);
    assert_eq!(app.rows("expected").len(), 1, "{screen}");
}

#[test]
fn a_refusal_shows_on_the_confirm() {
    let (_server, mut app, probe, binding) = desk("confirm", true);
    let display = display_of(
        &binding,
        "gatepass.visit.SignOutVisitor",
        "gatepass.visit.VisitStateConflict",
    );
    app.focus_section("expected");
    let before = app.render_text(WIDTH, HEIGHT);
    assert!(before.contains("Ada"), "{before}");
    app.keys("l");
    app.keys("y");
    let screen = app.render_text(WIDTH, HEIGHT);
    // The confirm stays open, the refusal inside it, naming the state the visit is in.
    let overlay = region(&app, "pages/desk/overlays/leave");
    let (at, _) = line_of(&screen, &display);
    assert!(
        inside(&overlay, at),
        "the refusal is on the confirm:\n{screen}"
    );
    let (state, _) = line_of(&screen, "Expected");
    assert!(inside(&overlay, state), "{screen}");
    assert_eq!(expected(&probe).len(), 1, "the visit did not move");
}

#[test]
fn a_refusal_shows_beside_the_action() {
    let (_server, mut app, probe, binding) = desk("action", true);
    let display = display_of(
        &binding,
        "gatepass.visit.SignOutVisitor",
        "gatepass.visit.VisitStateConflict",
    );
    app.focus_section("expected");
    let before = app.render_text(WIDTH, HEIGHT);
    let (hint, _) = line_of(&before, "d Depart");
    app.keys("d");
    let screen = app.render_text(WIDTH, HEIGHT);
    // The refusal is drawn in the section, on the line under the action row that sent it.
    let section = region(&app, "pages/desk/sections/expected");
    let (at, _) = line_of(&screen, &display);
    assert!(
        inside(&section, at),
        "the refusal is in the section:\n{screen}"
    );
    let (row, _) = line_of(&screen, "d Depart");
    assert_eq!(row, hint, "{screen}");
    assert!(
        at > row && at <= row + 2,
        "beside the action row:\n{screen}"
    );
    assert!(screen.contains("Expected"), "{screen}");
    assert_eq!(expected(&probe).len(), 1, "the visit did not move");
}

// ── polling ───────────────────────────────────────────────────────────────────────────────────

/// Counts the reads it answers, each with no rows; accepts every command.
struct Counting(std::rc::Rc<std::cell::Cell<usize>>);

impl DataAdapter for Counting {
    fn read(&self, _: &ReadRequest) -> Result<ess_ui_tui::ReadResult, String> {
        self.0.set(self.0.get() + 1);
        Ok(ess_ui_tui::ReadResult::default())
    }
    fn run(&mut self, _: &str, _: &BTreeMap<String, Value>) -> Answer {
        Answer::Accepted
    }
    fn load_state(&self, _: &str) -> Option<Value> {
        None
    }
    fn store_state(&mut self, _: &str, _: Value) {}
}

/// The desk with its expected list `live:`, its read refreshed every `refresh` (none when
/// empty) and `degrades` as the section's.
fn live_desk(refresh: &str, degrades: &str) -> ess_ui::Document {
    let reads = if refresh.is_empty() {
        "reads: visit.ExpectedVisits".to_owned()
    } else {
        format!("reads: {{view: visit.ExpectedVisits, refresh: {refresh}}}")
    };
    let text = DESK
        .replace(
            "        reads: visit.ExpectedVisits\n",
            &format!(
                "        {reads}\n        live: {{channel: visits, effect: insert_top}}\n{degrades}"
            ),
        )
        .replace(
            "pages:\n",
            "channels:\n  visits:\n    carries: {events: [gatepass.visit.VisitRegistered]}\n    \
             direction: server_to_client\n    delivery: every_event\n    resume: from_last_seen\n    scope: account\n    \
             lifecycle: [connecting, live, reconnecting, stale, closed]\n    \
             reconnect: {backoff: {from: 1s, to: 30s}}\npages:\n",
        );
    ess_ui::load_str(&text).unwrap_or_else(|error| panic!("the live desk loads: {error}"))
}

fn bound_with(
    document: ess_ui::Document,
    test: &str,
) -> (
    Result<App, ess_ui_tui::TuiError>,
    std::rc::Rc<std::cell::Cell<usize>>,
) {
    let reads = std::rc::Rc::new(std::cell::Cell::new(0));
    let app = App::bound(
        document,
        Box::new(Counting(std::rc::Rc::clone(&reads))),
        binding(),
        Options::new(state_dir(test)),
    );
    (app, reads)
}

#[test]
fn a_bound_live_section_polls_from_one_second_to_a_day() {
    use std::time::Duration;

    // `refresh: 2s`: read once on open, again only once two seconds have passed since it answered.
    let (app, reads) = bound_with(live_desk("2s", ""), "poll");
    let mut app = app.unwrap_or_else(|error| panic!("{error}"));
    app.advance(Duration::ZERO);
    let opened = reads.get();
    assert!(opened >= 1, "the list is read on open");
    app.advance(Duration::from_millis(1500));
    assert_eq!(reads.get(), opened, "no poll before its interval");
    app.advance(Duration::from_millis(500));
    assert_eq!(reads.get(), opened + 1, "one poll at its interval");
    app.advance(Duration::from_secs(2));
    assert_eq!(reads.get(), opened + 2);

    // Without `refresh:` the section polls every 5 s.
    let (app, reads) = bound_with(live_desk("", ""), "poll-default");
    let mut app = app.unwrap_or_else(|error| panic!("{error}"));
    let opened = reads.get();
    app.advance(Duration::from_millis(4999));
    assert_eq!(reads.get(), opened);
    app.advance(Duration::from_millis(1));
    assert_eq!(reads.get(), opened + 1);

    // Outside 1 s to 24 h, or not a duration, refused at the read's `refresh`.
    for refresh in ["500ms", "25h", "soon"] {
        let (app, _) = bound_with(live_desk(refresh, ""), "poll-refused");
        let error = app
            .err()
            .unwrap_or_else(|| panic!("`{refresh}` is refused"));
        assert!(
            error
                .to_string()
                .starts_with("pages/desk/sections/expected/reads/refresh"),
            "{refresh}: {error}"
        );
    }
    // A section that refuses to poll is refused at its `live`.
    let (app, _) = bound_with(
        live_desk("2s", "        degrades: {no_live: refuse}\n"),
        "poll-no-live",
    );
    let error = app.err().expect("no_live: refuse is refused");
    assert!(
        error
            .to_string()
            .starts_with("pages/desk/sections/expected/live"),
        "{error}"
    );
}
