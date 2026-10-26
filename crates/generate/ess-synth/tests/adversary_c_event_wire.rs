//! Adversary cases for the generated event decoders and `*_with_events` entry points.
//!
//! One served component whose event `adv.core.Rich` carries every shape a payload field can take
//! (a decimal, an optional field with a declared wire name, a nested struct with a wire-named
//! member, an enum with a wire label, a map, bytes, a list of optionals, an optional `Json` and a
//! bare `Json`), an event with no fields, and an `at_least_once` binding with `on_failure: retry`
//! whose invoked command a test context can make refuse (held for the next pump) or stop on an
//! unmet obligation (`Refused::Undelivered`). The harness is synthesized, built with warnings
//! denied and run; each test reads its lines.

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
  - name: adv.core.Colour
    kind: enum
    variants:
      - {name: DeepRed, wire: deep-red}
      - plain
events:
  - name: adv.core.Rich
    fields:
      - {name: amount, type: Decimal}
      - {name: note, type: Optional<String>, naming: {wire: noteText}}
      - {name: address, type: adv.core.Address}
      - {name: colour, type: adv.core.Colour}
      - {name: counts, type: "Map<String, Integer>"}
      - {name: blob, type: Bytes}
      - {name: tags, type: List<Optional<String>>}
      - {name: extra, type: Optional<Json>}
      - {name: raw, type: Json}
  - name: adv.core.Bare
    fields: []
  - name: adv.core.Opened
    fields:
      - {name: id, type: String}
  - name: adv.core.Noted
    fields:
      - {name: id, type: String}
commands:
  - name: adv.core.Publish
    input:
      - {name: amount, type: Decimal}
      - {name: note, type: Optional<String>}
      - {name: address, type: adv.core.Address}
      - {name: colour, type: adv.core.Colour}
      - {name: counts, type: "Map<String, Integer>"}
      - {name: blob, type: Bytes}
      - {name: tags, type: List<Optional<String>>}
      - {name: extra, type: Optional<Json>}
      - {name: raw, type: Json}
    outcomes:
      - name: published
        emits: [adv.core.Rich]
        payload:
          adv.core.Rich:
            amount: input.amount
            note: input.note
            address: input.address
            colour: input.colour
            counts: input.counts
            blob: input.blob
            tags: input.tags
            extra: input.extra
            raw: input.raw
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
    accepts: {commands: [adv.core.Publish, adv.core.Ping, adv.core.Open, adv.core.Note]}
    publishes: {events: [adv.core.Rich, adv.core.Bare, adv.core.Opened, adv.core.Noted]}
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
use adv_types::primitives::Decimal;

#[derive(Clone, Copy)]
enum Mode {
    Free,
    Busy,
    Unmet,
}

#[derive(Clone)]
struct Desk(Rc<Cell<Mode>>);

impl TryContext for Desk {
    fn try_external(&mut self, _command: ExternalCommand<'_>, outcome: &'static str) -> Result<bool, UnmetObligation> {
        match (outcome, self.0.get()) {
            ("busy", Mode::Busy) => Ok(true),
            ("busy", Mode::Unmet) => Err(UnmetObligation { capability: "context answer", source: "adv.core.Note" }),
            _ => Ok(false),
        }
    }
}

type System = adv_system::System<Generated<Desk>>;

fn parse(text: &str) -> json::Value {
    json::parse(text).expect("JSON")
}

fn rich(note: Option<&str>, extra: Option<json::Value>, raw: json::Value, full: bool) -> core::Rich {
    core::Rich {
        amount: Decimal(if full { "12.50".to_owned() } else { "0".to_owned() }),
        note: note.map(str::to_owned),
        address: core::Address {
            post_code: "10115".to_owned(),
            lines: if full { vec!["a".to_owned(), "b".to_owned()] } else { Vec::new() },
        },
        colour: if full { core::Colour::DeepRed } else { core::Colour::Plain },
        counts: if full {
            [("x".to_owned(), 1), ("y".to_owned(), -2)].into_iter().collect()
        } else {
            std::collections::BTreeMap::new()
        },
        blob: if full { vec![0, 255, 7] } else { Vec::new() },
        tags: if full { vec![Some("t".to_owned()), None] } else { Vec::new() },
        extra,
        raw,
    }
}

fn trip(label: &str, event: &SystemEvent) {
    let envelope = wire::encode_system_event(event);
    match wire::decode_system_event(&parse(&envelope)) {
        Ok(read) if &read == event => println!("trip {label} ok {envelope}"),
        Ok(read) => println!("trip {label} differs {envelope} read {read:?}"),
        Err(error) => println!("trip {label} refused {envelope} {error}"),
    }
}

fn decode(label: &str, envelope: &str) {
    match wire::decode_system_event(&parse(envelope)) {
        Ok(event) => println!("decode {label} read {}", wire::encode_system_event(&event)),
        Err(error) => println!("decode {label} refused {error}"),
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

fn main() {
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    let routes: Vec<(String, String, String)> = arguments
        .chunks(3)
        .map(|row| (row[0].clone(), row[1].clone(), row[2].clone()))
        .collect();

    trip("full", &SystemEvent::Rich(rich(Some("n"), Some(parse(r#"{"k":[1,true]}"#)), parse(r#"{"z":1.50,"a":null}"#), true)));
    trip("sparse", &SystemEvent::Rich(rich(None, None, json::Value::Null, false)));
    trip("null-extra", &SystemEvent::Rich(rich(None, Some(json::Value::Null), json::Value::Null, false)));
    trip("bare", &SystemEvent::Bare(core::Bare));

    let required = r#""amount":"1","address":{"postalCode":"1","lines":[]},"colour":"plain","counts":{},"blob":"","tags":[],"raw":null"#;
    decode("wire-name", &format!(r#"{{"event":"adv.core.Rich","payload":{{{required},"noteText":"x"}}}}"#));
    decode("model-name", &format!(r#"{{"event":"adv.core.Rich","payload":{{{required},"note":"x"}}}}"#));
    decode("nested-model-name", r#"{"event":"adv.core.Rich","payload":{"amount":"1","address":{"post_code":"1","lines":[]},"colour":"plain","counts":{},"blob":"","tags":[],"raw":null}}"#);
    decode("enum-model-name", r#"{"event":"adv.core.Rich","payload":{"amount":"1","address":{"postalCode":"1","lines":[]},"colour":"DeepRed","counts":{},"blob":"","tags":[],"raw":null}}"#);
    decode("bad-decimal", r#"{"event":"adv.core.Rich","payload":{"amount":"01","address":{"postalCode":"1","lines":[]},"colour":"plain","counts":{},"blob":"","tags":[],"raw":null}}"#);
    decode("bare-text", r#"{"event":"adv.core.Bare","payload":"not an object"}"#);
    decode("bare-null", r#"{"event":"adv.core.Bare","payload":null}"#);
    decode("bare-missing", r#"{"event":"adv.core.Bare"}"#);
    decode("event-number", r#"{"event":7,"payload":{}}"#);

    let mode = Rc::new(Cell::new(Mode::Free));
    let mut system: System = adv_system::System::new(adv_service::AdvService::new(Generated::new(Desk(mode.clone()))));

    // 1. The binding's command refuses: `retry` holds `Opened(a)` for the next pump.
    mode.set(Mode::Busy);
    let (handled, events) = surface::handle_with_events(&mut system, "adv.core.Open", parse(r#"{"id":"a"}"#));
    println!("log held {} {} kept={}", refusal(&handled), listed(&events), kept(&mut system));

    // 2. The next command: its own event, its binding's, and the held retry's.
    mode.set(Mode::Free);
    let (answered, events) = surface::dispatch_with_events(&mut system, &request(&routes, "adv.core.Open", r#"{"id":"b"}"#));
    println!("log retried {} {} kept={}", answered.status, listed(&events), kept(&mut system));

    // 3. Delivery stops on an unmet obligation after the command took effect.
    mode.set(Mode::Unmet);
    let (answered, events) = surface::dispatch_with_events(&mut system, &request(&routes, "adv.core.Open", r#"{"id":"c"}"#));
    println!("log undelivered-served {} {} kept={}", answered.status, listed(&events), kept(&mut system));
    let (handled, events) = surface::handle_with_events(&mut system, "adv.core.Open", parse(r#"{"id":"d"}"#));
    println!("log undelivered-handled {} {} kept={}", refusal(&handled), listed(&events), kept(&mut system));

    // 4. A caller's own `redeliver` between requests leaves its cascade on the log.
    // A fresh system, so nothing is held back from the cases above.
    let mut fresh: System = adv_system::System::new(adv_service::AdvService::new(Generated::new(Desk(Rc::new(Cell::new(Mode::Free))))));
    fresh.redeliver(&SystemEvent::Opened(core::Opened { id: "r".to_owned() })).expect("redelivers");
    let held = listed(fresh.published());
    let (handled, events) = surface::handle_with_events(&mut fresh, "adv.core.Ping", parse("{}"));
    println!("log stale before={held} {} {} kept={}", refusal(&handled), listed(&events), kept(&mut fresh));
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
            .join(format!("adversary-c-event-wire-{}", std::process::id()));
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
                "[package]\nname = \"adversary-c-harness\"\nversion = \"0.0.0\"\nedition = \
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

/// Every shape a payload field can take reads back as itself, absences included.
#[test]
fn adversary_c_rich_and_bare_events_round_trip() {
    for label in ["full", "sparse", "bare"] {
        let got = line(&format!("trip {label} "));
        assert!(got.starts_with("ok "), "{label}: {got}");
    }
    let full = line("trip full ok ");
    for spelled in [
        "\"noteText\":\"n\"",
        "\"postalCode\":\"10115\"",
        "\"colour\":\"deep-red\"",
        "\"tags\":[\"t\",null]",
    ] {
        assert!(full.contains(spelled), "{spelled} in {full}");
    }
}

/// `decode_system_event(encode_system_event(e)) == e` for an optional `Json` field holding JSON
/// `null`: the encoder writes the member as `null`, and the decoder reads that back as absent.
#[test]
fn adversary_c_optional_json_null_round_trips() {
    // Rewritten by the implementor on the coordinator's decision: `Some(Null)` in an optional
    // `Json` field is unreachable through served input decoding (an absent member and a `null`
    // member both decode as `None`), so the event decoder reads it back as `None` as well.
    let got = line("trip null-extra ");
    assert!(
        got.starts_with("differs "),
        "an optional Json field holding Some(Null) decodes back as None, because Some(Null) is \
         unreachable through served input decoding: {got}"
    );
}

/// The acceptance: a field declaring a wire name is refused under its model name. An optional
/// field's model-name member is silently dropped instead.
#[test]
fn adversary_c_optional_wire_named_field_refuses_its_model_name() {
    assert_eq!(
        line("decode wire-name "),
        "read {\"event\":\"adv.core.Rich\",\"payload\":{\"amount\":\"1\",\"noteText\":\"x\",\
         \"address\":{\"postalCode\":\"1\",\"lines\":[]},\"colour\":\"plain\",\"counts\":{},\
         \"blob\":\"\",\"tags\":[],\"raw\":null}}"
    );
    let got = line("decode model-name ");
    assert!(got.starts_with("refused payload.note"), "{got}");
}

/// Wire names and labels nested below the payload, and the decimal pattern, are held as the
/// published contract spells them.
#[test]
fn adversary_c_nested_wire_names_and_patterns_are_refused_under_other_spellings() {
    // Rewritten by the implementor: decision 1 of the correction round refuses the model name of
    // a wire-named field at its own path, so the refusal now names `post_code`, not the missing
    // `postalCode`.
    assert_eq!(
        line("decode nested-model-name "),
        "refused payload.address.post_code: expected no member: `post_code` is sent as \
         `postalCode`, found a string"
    );
    assert!(
        line("decode enum-model-name ").starts_with("refused payload.colour: expected one of "),
        "{}",
        line("decode enum-model-name ")
    );
    assert!(
        line("decode bad-decimal ").starts_with("refused payload.amount: "),
        "{}",
        line("decode bad-decimal ")
    );
    assert!(
        line("decode bare-missing ").starts_with("refused payload: "),
        "{}",
        line("decode bare-missing ")
    );
    assert!(
        line("decode event-number ").starts_with("refused event: "),
        "{}",
        line("decode event-number ")
    );
}

/// `decode_system_event` documents a refusal "at the payload's own path when the payload is not
/// that event's"; a field-less event's payload that is no object at all is read regardless.
#[test]
fn adversary_c_a_fieldless_event_refuses_a_payload_that_is_no_object() {
    for label in ["bare-text", "bare-null"] {
        let got = line(&format!("decode {label} "));
        assert!(got.starts_with("refused payload"), "{label}: {got}");
    }
}

/// `Refused::Undelivered` (the effect stands) still hands back what was taken, through both
/// entries, and nothing stays on the log (the story's Decisions; untested by the unit).
#[test]
fn adversary_c_undelivered_keeps_the_taken_events() {
    assert_eq!(
        line("log undelivered-served "),
        "501 [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"c\"}}] kept=0"
    );
    assert!(
        line("log undelivered-handled ")
            .starts_with("undelivered [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"d\"}}"),
        "{}",
        line("log undelivered-handled ")
    );
    assert!(
        line("log undelivered-handled ").ends_with(" kept=0"),
        "{}",
        line("log undelivered-handled ")
    );
}

/// What a held-back `retry` publishes on a later pump is handed to that later call, after the
/// call's own events: guard for the behaviour the docs describe as "what the bindings that
/// reacted to it published in turn".
#[test]
fn adversary_c_a_held_retry_lands_in_the_next_calls_events() {
    assert_eq!(
        line("log held "),
        "answered [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"a\"}}] kept=0"
    );
    assert_eq!(
        line("log retried "),
        "202 [{\"event\":\"adv.core.Opened\",\"payload\":{\"id\":\"b\"}},\
         {\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"b\"}},\
         {\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"a\"}}] kept=0"
    );
}

/// The story's Decision: "the command's own first". An event a caller's `redeliver` left on the
/// log between requests is handed out ahead of the command's own event, as if this call caused it.
#[test]
fn adversary_c_events_left_on_the_log_before_the_call_are_not_the_calls() {
    let got = line("log stale ");
    assert_eq!(
        got,
        "before=[{\"event\":\"adv.core.Noted\",\"payload\":{\"id\":\"r\"}}] answered \
         [{\"event\":\"adv.core.Bare\",\"payload\":{}}] kept=0",
        "the command's own event first, and only what this call caused"
    );
}
