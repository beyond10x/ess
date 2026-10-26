//! Adversary cases, second pass, for the correction round of the generated event decoders and
//! `*_with_events` entry points.
//!
//! The model nests wire-named fields every way a decoder reaches them (a struct whose one field's
//! wire name is the other field's model name, a case-only wire name, a wire name equal to the
//! model name, a list, a map, a union variant and an optional of a wire-named struct), declares
//! a field-less command and a field-less event, and keeps the `at_least_once` / `on_failure:
//! retry` binding of the first pass so the log can hold something when a call starts. The
//! harness is synthesized, built with warnings denied and run; each test reads its lines.

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::process::{Command, Output};

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::http::{self, Served};
use ess_synth::{synthesize_laid_out, OutputLayout, Synthesis, Target};

const MODEL: &str = r#"format: ess/19
system: adv
version: v1
domain: adv.core
types:
  - name: adv.core.Address
    kind: struct
    fields:
      - {name: post_code, type: String, naming: {wire: postalCode}}
      - {name: lines, type: List<String>}
  - name: adv.core.Swap
    kind: struct
    fields:
      - {name: x, type: String, naming: {wire: y}}
      - {name: y, type: String, naming: {wire: z}}
  - name: adv.core.Cased
    kind: struct
    fields:
      - {name: amount, type: String, naming: {wire: Amount}}
      - {name: same, type: String, naming: {wire: same}}
  - name: adv.core.Shape
    kind: union
    tag: kind
    variants:
      home: adv.core.Address
      label: String
events:
  - name: adv.core.Shaped
    fields:
      - {name: swap, type: adv.core.Swap}
      - {name: cased, type: adv.core.Cased}
      - {name: items, type: List<adv.core.Address>}
      - {name: by_key, type: "Map<String, adv.core.Address>"}
      - {name: shape, type: adv.core.Shape}
      - {name: maybe, type: Optional<adv.core.Address>}
      - {name: note, type: Optional<String>, naming: {wire: noteText}}
  - name: adv.core.Bare
    fields: []
  - name: adv.core.Opened
    fields:
      - {name: id, type: String}
  - name: adv.core.Noted
    fields:
      - {name: id, type: String}
commands:
  - name: adv.core.Submit
    input:
      - {name: swap, type: adv.core.Swap}
      - {name: cased, type: adv.core.Cased}
      - {name: items, type: List<adv.core.Address>}
      - {name: by_key, type: "Map<String, adv.core.Address>"}
      - {name: shape, type: adv.core.Shape}
      - {name: maybe, type: Optional<adv.core.Address>}
      - {name: note, type: Optional<String>, naming: {wire: noteText}}
    outcomes:
      - name: shaped
        emits: [adv.core.Shaped]
        payload:
          adv.core.Shaped:
            swap: input.swap
            cased: input.cased
            items: input.items
            by_key: input.by_key
            shape: input.shape
            maybe: input.maybe
            note: input.note
  - name: adv.core.Ping
    input: []
    outcomes:
      - name: pinged
        emits: [adv.core.Bare]
  - name: adv.core.Open
    input:
      - {name: id, type: String}
    outcomes:
      - name: opened
        emits: [adv.core.Opened]
        payload:
          adv.core.Opened: {id: input.id}
  - name: adv.core.Note
    input:
      - {name: id, type: String}
    outcomes:
      - name: noted
        emits: [adv.core.Noted]
        payload:
          adv.core.Noted: {id: input.id}
      - name: busy
        external: the notebook is full
        error: adv.core.Busy
errors:
  - name: adv.core.Busy
components:
  - component: adv-service
    owns: {domains: [adv.core]}
    accepts: {commands: [adv.core.Submit, adv.core.Ping, adv.core.Open, adv.core.Note]}
    publishes: {events: [adv.core.Shaped, adv.core.Bare, adv.core.Opened, adv.core.Noted]}
    reached_by: network
bindings:
  - id: note-on-open
    when:
      event: adv.core.Opened
    invoke:
      command: adv.core.Note
    mapping:
      id: event.id
    delivery: at_least_once
    on_failure: retry
"#;

const HARNESS: &str = r##"use std::cell::Cell;
use std::rc::Rc;

use adv_server::adv_service as surface;
use adv_server::{entry, http, json, wire};
use adv_system::SystemEvent;
use adv_types::behaviour::{ExternalCommand, Generated, TryContext};
use adv_types::core;
use adv_types::obligation::UnmetObligation;

#[derive(Clone, Copy)]
enum Mode {
    Free,
    Busy,
}

#[derive(Clone)]
struct Desk(Rc<Cell<Mode>>);

impl TryContext for Desk {
    fn try_external(&mut self, _command: ExternalCommand<'_>, outcome: &'static str) -> Result<bool, UnmetObligation> {
        Ok(matches!((outcome, self.0.get()), ("busy", Mode::Busy)))
    }
}

type System = adv_system::System<Generated<Desk>>;

fn parse(text: &str) -> json::Value {
    json::parse(text).expect("JSON")
}

fn system(mode: &Rc<Cell<Mode>>) -> System {
    adv_system::System::new(adv_service::AdvService::new(Generated::new(Desk(mode.clone()))))
}

fn show<T: std::fmt::Debug>(label: &str, read: Result<T, json::DecodeError>) {
    match read {
        Ok(value) => println!("dec {label} read {value:?}"),
        Err(error) => println!("dec {label} refused {error}"),
    }
}

fn listed(events: &[SystemEvent]) -> String {
    let rendered: Vec<String> = events.iter().map(wire::encode_system_event).collect();
    format!("[{}]", rendered.join(","))
}

fn kept(system: &mut System) -> usize {
    system.published().len() + system.invocations().len() + system.adv_service.drain_outbox().len()
}

fn request(routes: &[(String, String, String)], name: &str, body: &str) -> http::Request {
    let (_, method, path) = routes
        .iter()
        .find(|(declared, _, _)| declared == name)
        .unwrap_or_else(|| panic!("`{name}` is a route"));
    http::Request {
        method: method.clone(),
        path: path.clone(),
        query: String::new(),
        headers: Vec::new(),
        body: body.as_bytes().to_vec(),
    }
}

fn refusal(handled: &Result<json::Value, entry::Refused>) -> &'static str {
    match handled {
        Ok(_) => "answered",
        Err(entry::Refused::Undelivered(_)) => "undelivered",
        Err(entry::Refused::Unmet(_)) => "unmet",
        Err(entry::Refused::Input(_)) => "input",
        Err(entry::Refused::Unknown(_)) => "unknown",
        #[allow(unreachable_patterns)]
        Err(_) => "other",
    }
}

/// A payload of `adv.core.Shaped` with every member spelled as published, one of them replaced.
fn shaped(replace: &str, with: &str) -> String {
    let members = [
        ("swap", r#""swap":{"y":"1","z":"2"}"#),
        ("cased", r#""cased":{"Amount":"3","same":"s"}"#),
        ("items", r#""items":[{"postalCode":"p","lines":[]}]"#),
        ("by_key", r#""by_key":{"k":{"postalCode":"q","lines":[]}}"#),
        ("shape", r#""shape":{"kind":"home","value":{"postalCode":"r","lines":[]}}"#),
        ("maybe", r#""maybe":{"postalCode":"m","lines":[]}"#),
        ("note", r#""noteText":"n""#),
    ];
    let body: Vec<&str> = members
        .iter()
        .map(|(name, spelled)| if *name == replace { with } else { spelled })
        .filter(|spelled| !spelled.is_empty())
        .collect();
    format!("{{{}}}", body.join(","))
}

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let routes: Vec<(String, String, String)> = arguments
        .chunks(3)
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect();

    // 1. Model-name refusals through every shape that reaches a wire-named field.
    let event = |label: &str, text: &str| show(label, wire::decode_event_adv_core_shaped(&parse(text), "payload"));
    event("published", &shaped("", ""));
    event("swap-model-x", &shaped("swap", r#""swap":{"x":"1","z":"2"}"#));
    event("swap-wire-y-only", &shaped("swap", r#""swap":{"y":"1","z":"2","x":"9"}"#));
    event("cased-lower", &shaped("cased", r#""cased":{"amount":"3","same":"s"}"#));
    event("items-model", &shaped("items", r#""items":[{"postalCode":"p","lines":[]},{"post_code":"p","lines":[]}]"#));
    event("map-model", &shaped("by_key", r#""by_key":{"k":{"post_code":"q","lines":[]}}"#));
    event("union-model", &shaped("shape", r#""shape":{"kind":"home","value":{"post_code":"r","lines":[]}}"#));
    event("optional-struct-model", &shaped("maybe", r#""maybe":{"post_code":"m","lines":[]}"#));
    event("optional-absent", &shaped("maybe", ""));
    event("note-null-model", &shaped("note", r#""note":null"#));
    event("note-both", &shaped("note", r#""noteText":"n","note":"o""#));
    event("note-wire-null", &shaped("note", r#""noteText":null"#));
    show("command-swap-model-x", wire::decode_command_adv_core_submit(&parse(&shaped("swap", r#""swap":{"x":"1","z":"2"}"#)), "body"));
    show("command-published", wire::decode_command_adv_core_submit(&parse(&shaped("", "")), "body"));

    // 2. Field-less records: the event (corrected) and the command input beside it.
    show("bare-array", wire::decode_event_adv_core_bare(&parse("[]"), "payload"));
    show("bare-object", wire::decode_event_adv_core_bare(&parse("{}"), "payload"));
    show("ping-text", wire::decode_command_adv_core_ping(&parse(r#""text""#), "body"));
    show("ping-null", wire::decode_command_adv_core_ping(&parse("null"), "body"));
    let mode = Rc::new(Cell::new(Mode::Free));
    let mut pinged = system(&mode);
    let (handled, events) = surface::handle_with_events(&mut pinged, "adv.core.Ping", parse("[1,2]"));
    println!("ping-handled {} {}", refusal(&handled), listed(&events));

    // 3. An event published through a component's own port before the call, still in its
    // outbox, is not published while the call runs.
    let mut outboxed = system(&mode);
    outboxed.adv_service.open(core::Open { id: "x".to_owned() }).expect("opens");
    let (handled, events) = surface::handle_with_events(&mut outboxed, "adv.core.Ping", parse("{}"));
    println!("log outboxed {} {} kept={}", refusal(&handled), listed(&events), kept(&mut outboxed));

    // 4. A pumped but untaken event and a held retry when the call starts: the count taken at
    // the start skips exactly the untaken event, and the held retry's cascade is the call's.
    mode.set(Mode::Busy);
    let mut pumped = system(&mode);
    pumped.adv_service.open(core::Open { id: "x".to_owned() }).expect("opens");
    let _ = pumped.pump();
    let before = listed(pumped.published());
    mode.set(Mode::Free);
    let (answered, events) = surface::dispatch_with_events(&mut pumped, &request(&routes, "adv.core.Open", r#"{"id":"b"}"#));
    println!("log pumped before={before} {} {} kept={}", answered.status, listed(&events), kept(&mut pumped));

    // 5. A refused call while something is on the log leaves it there and lists nothing; the
    // next call that runs takes it and still lists only its own.
    let mut stale = system(&mode);
    stale.redeliver(&SystemEvent::Opened(core::Opened { id: "r".to_owned() })).expect("redelivers");
    let (handled, events) = surface::handle_with_events(&mut stale, "adv.core.Open", parse(r#"{"id":7}"#));
    println!("log refused-stale {} {} onlog={}", refusal(&handled), listed(&events), stale.published().len());
    let (handled, events) = surface::handle_with_events(&mut stale, "adv.core.Open", parse(r#"{"id":"s"}"#));
    println!("log after-stale {} {} kept={}", refusal(&handled), listed(&events), kept(&mut stale));
    let _ = routes.len();
}
"##;

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("spec.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("spec.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn route_table(ir: &EssIr) -> Vec<String> {
    let component = ir.components().values().next().expect("one component");
    let mut table = Vec::new();
    for route in http::routes(ir, component) {
        let name = match route.serves {
            Served::Command(handle) => ir.command(handle).name.to_string(),
            Served::View(handle) => ir.view(handle).name.to_string(),
        };
        table.extend([name, route.method.as_str().to_owned(), route.path]);
    }
    table
}

fn write(synthesis: &Synthesis, directory: &Path) {
    let _ = std::fs::remove_dir_all(directory);
    for (relative, artifact) in &synthesis.artifacts {
        let destination = directory.join(relative);
        std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
        std::fs::write(&destination, &artifact.contents).expect("write");
    }
}

fn cargo(directory: &Path, arguments: &[String]) -> Output {
    let output = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
        .args(arguments)
        .current_dir(directory)
        .env_remove("CARGO_TARGET_DIR")
        .env_remove("CARGO_ENCODED_RUSTFLAGS")
        .env("RUSTFLAGS", "-D warnings")
        .env("CARGO_INCREMENTAL", "0")
        .output()
        .expect("cargo runs");
    eprintln!(
        "cargo {arguments:?} in {}\n{}",
        directory.display(),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

/// The harness's output lines, built and run once per test binary.
fn lines() -> &'static [String] {
    static LINES: std::sync::OnceLock<Vec<String>> = std::sync::OnceLock::new();
    LINES.get_or_init(|| {
        let ir = ir();
        let synthesis = synthesize_laid_out(&ir, Target::Rust, OutputLayout::Workspace)
            .expect("the model synthesizes");
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("adversary-c-event-wire-2-{}", std::process::id()));
        write(&synthesis, &root.join("adv"));
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = ["adv-server", "adv-system", "adv-service", "adv-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../adv/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"adversary-c-harness-2\"\nversion = \"0.0.0\"\nedition = \
                 \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), HARNESS).expect("write");
        let target = root.join("target");
        let mut arguments: Vec<String> = [
            "run",
            "--offline",
            "--quiet",
            "--target-dir",
            target.to_str().expect("UTF-8"),
            "--",
        ]
        .iter()
        .map(|argument| (*argument).to_owned())
        .collect();
        arguments.extend(route_table(&ir));
        let ran = cargo(&harness, &arguments);
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        let _ = std::fs::remove_dir_all(&root);
        eprintln!("{stdout}");
        assert!(
            ran.status.success(),
            "the harness builds with -D warnings and runs"
        );
        stdout.lines().map(str::to_owned).collect()
    })
}

fn line(prefix: &str) -> &'static str {
    lines()
        .iter()
        .find_map(|line| line.strip_prefix(prefix))
        .unwrap_or_else(|| panic!("no line `{prefix}…` in {:#?}", lines()))
}

/// Guard: a model name that is another field's wire name is that field's member and is never
/// refused, and the published spelling of every nested shape reads.
#[test]
fn adversary_c2_a_model_name_that_is_another_fields_wire_name_is_read_as_that_field() {
    let published = line("dec published ");
    assert!(published.starts_with("read "), "{published}");
    assert!(
        published.contains("swap: Swap { x: \"1\", y: \"2\" }"),
        "y is x's wire name, z is y's: {published}"
    );
    assert!(
        line("dec command-published ").starts_with("read "),
        "{}",
        line("dec command-published ")
    );
    assert!(
        line("dec optional-absent ").starts_with("read "),
        "{}",
        line("dec optional-absent ")
    );
    assert!(
        line("dec note-wire-null ").starts_with("read "),
        "{}",
        line("dec note-wire-null ")
    );
}

/// Guard: the model name of a renamed field is refused at its own path wherever the record sits:
/// directly, case-only renamed, in a list element, a map value, a union variant's content and an
/// optional struct, and in a command input as in an event payload.
#[test]
fn adversary_c2_model_names_are_refused_at_their_own_path_at_every_depth() {
    for (label, path) in [
        ("swap-model-x", "payload.swap.x"),
        ("cased-lower", "payload.cased.amount"),
        ("items-model", "payload.items.1.post_code"),
        ("map-model", "payload.by_key.k.post_code"),
        ("union-model", "payload.shape.value.post_code"),
        ("optional-struct-model", "payload.maybe.post_code"),
        ("note-both", "payload.note"),
        ("command-swap-model-x", "body.swap.x"),
    ] {
        let got = line(&format!("dec {label} "));
        assert!(
            got.starts_with(&format!("refused {path}: ")),
            "{label}: {got}"
        );
    }
    assert_eq!(
        line("dec swap-wire-y-only "),
        "refused payload.swap.x: expected no member: `x` is sent as `y`, found a string"
    );
}

/// Guard: an optional field's model name sent as `null` is still its model name, refused at its
/// path (the published schema is closed), not read as the field's absence.
#[test]
fn adversary_c2_an_optional_fields_model_name_sent_as_null_is_refused() {
    let got = line("dec note-null-model ");
    assert!(got.starts_with("refused payload.note: "), "{got}");
}

/// Guard for correction 3: a field-less event's payload must be an object, an array included.
#[test]
fn adversary_c2_a_fieldless_event_refuses_an_array_payload() {
    assert!(
        line("dec bare-array ").starts_with("refused payload: "),
        "{}",
        line("dec bare-array ")
    );
    assert!(line("dec bare-object ").starts_with("read "));
}

/// The documented edge: an event a caller published through the component's own port before
/// the call is still in the component's outbox, no pump collected it, and the call lists it as
/// its own, ahead of the call's own event. The generated docs tell the caller to pump after a
/// direct port call; this guard pins what happens when the caller does not.
#[test]
fn adversary_c2_an_event_still_in_an_outbox_is_listed_as_the_calls() {
    assert_eq!(
        line("log outboxed "),
        "answered [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"x\"}},\
         {\"event\":\"adv.core.Bare\",\"payload\":{}},\
         {\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"x\"}}] kept=0"
    );
}

/// Guard against an off-by-one in the start-of-call count: an untaken, already pumped event is
/// skipped exactly, and what an earlier call's held retry publishes during this call is listed
/// after the call's own events.
#[test]
fn adversary_c2_the_start_count_skips_exactly_the_untaken_events() {
    assert_eq!(
        line("log pumped "),
        "before=[{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"x\"}}] 202 \
         [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"b\"}},\
         {\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"b\"}},\
         {\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"x\"}}] kept=0"
    );
}

/// Guard: a call refused before its port leaves what is on the log there and lists nothing; the
/// next call that runs takes it off the log and lists only its own.
#[test]
fn adversary_c2_a_refused_call_leaves_the_log_and_the_next_lists_only_its_own() {
    assert_eq!(line("log refused-stale "), "input [] onlog=1");
    assert_eq!(
        line("log after-stale "),
        "answered [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"s\"}},\
         {\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"s\"}}] kept=0"
    );
}
