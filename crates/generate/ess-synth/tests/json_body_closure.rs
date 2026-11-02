//! The generated Rust server refuses what its `OpenAPI` document refuses (beyond10x/ess#707,
//! `story:the-generated-rust-server-refuses-what-its-openapi-refuses`).
//!
//! The model is the issue's toy shelf, widened to every position a `Timestamp` and a closed
//! object reach: a top-level `Timestamp`, an `Optional` one, a `List` of them, one inside a struct
//! that is a list element, one as a map key; a closed struct in a list, an open struct
//! (`undeclared_fields: ignored`) and a `Json` field, whose content the document leaves open. The
//! binding states `unknown: refuse` on one JSON route and leaves the other at its default, which is
//! `refuse` too, and overrides no refusal, so a malformed body is answered `400` with its detail.
//!
//! One harness drives the generated surface's own `dispatch` over the generated in-memory store
//! with every request of [`REQUESTS`], then reads both views back. Each body is also checked
//! against the request schema of the `openapi.json` that same server serves, with `format`
//! asserted, so a body the server refuses is one the document refuses and the other way round.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::ess_http;
use ess_gen::http::Binding;
use ess_synth::{synthesize_bound, OutputLayout, Target};
use serde_json::{json, Value};

const MODEL: &str = "format: ess/32
system: shelf
version: v1
domain: shelf.items
types:
  - {name: shelf.items.ItemId, kind: newtype, of: String}
  - {name: shelf.items.StampId, kind: newtype, of: String}
  - name: shelf.items.Label
    kind: struct
    fields:
      - {name: key, type: String}
      - {name: value, type: String}
      - {name: unit, type: Optional<String>}
      - {name: seen_at, type: Optional<Timestamp>}
  - name: shelf.items.Note
    kind: struct
    undeclared_fields: ignored
    fields:
      - {name: text, type: String}
entities:
  - name: shelf.items.Item
    identity: {name: item_id, type: shelf.items.ItemId}
    fields:
      - {name: created_at, type: Timestamp}
      - {name: due_at, type: Optional<Timestamp>}
      - {name: stamps, type: List<Timestamp>}
      - {name: labels, type: List<shelf.items.Label>}
      - {name: note, type: Optional<shelf.items.Note>}
      - {name: data, type: Optional<Json>}
    lifecycle: {initial: Shelved, states: [Shelved], terminal: [Shelved], transitions: []}
  - name: shelf.items.Stamp
    identity: {name: stamp_id, type: shelf.items.StampId}
    fields:
      - {name: stamped_at, type: Timestamp}
      - {name: marks, type: 'Map<Timestamp, String>'}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept], transitions: []}
events:
  - name: shelf.items.ItemAdded
    fields: [{name: item_id, type: shelf.items.ItemId}]
  - name: shelf.items.StampAdded
    fields: [{name: stamp_id, type: shelf.items.StampId}]
commands:
  - name: shelf.items.AddItem
    input:
      - {name: item_id, type: shelf.items.ItemId}
      - {name: created_at, type: Timestamp}
      - {name: due_at, type: Optional<Timestamp>}
      - {name: stamps, type: List<Timestamp>}
      - {name: labels, type: List<shelf.items.Label>}
      - {name: note, type: Optional<shelf.items.Note>}
      - {name: data, type: Optional<Json>}
    outcomes:
      - name: added
        creates: shelf.items.Item
        instance: item_id
        sets:
          created_at: input.created_at
          due_at: input.due_at
          stamps: input.stamps
          labels: input.labels
          note: input.note
          data: input.data
        emits: [shelf.items.ItemAdded]
        payload:
          shelf.items.ItemAdded: {item_id: input.item_id}
  - name: shelf.items.AddStamp
    input:
      - {name: stamp_id, type: shelf.items.StampId}
      - {name: stamped_at, type: Timestamp}
      - {name: marks, type: 'Map<Timestamp, String>'}
    outcomes:
      - name: added
        creates: shelf.items.Stamp
        instance: stamp_id
        sets: {stamped_at: input.stamped_at, marks: input.marks}
        emits: [shelf.items.StampAdded]
        payload:
          shelf.items.StampAdded: {stamp_id: input.stamp_id}
views:
  - name: shelf.items.AllItems
    source: shelf.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: shelf.items.ItemId}
      - {name: created_at, type: Timestamp}
      - {name: due_at, type: Optional<Timestamp>}
      - {name: stamps, type: List<Timestamp>}
      - {name: labels, type: List<shelf.items.Label>}
      - {name: note, type: Optional<shelf.items.Note>}
      - {name: data, type: Optional<Json>}
  - name: shelf.items.AllStamps
    source: shelf.items.Stamp
    consistency: read_your_writes
    fields:
      - {name: stamp_id, type: shelf.items.StampId}
      - {name: stamped_at, type: Timestamp}
      - {name: marks, type: 'Map<Timestamp, String>'}
actors:
  - name: shelf.items.Clerk
    may: [shelf.items.AddItem, shelf.items.AddStamp]
components:
  - component: shelf-service
    reached_by: network
    owns: {domains: [shelf.items]}
    accepts: {commands: [shelf.items.AddItem, shelf.items.AddStamp]}
    publishes: {events: [shelf.items.ItemAdded, shelf.items.StampAdded]}
";

/// `AddItem` states `unknown: refuse`; `AddStamp` leaves the request at its default.
const ROUTES: &str = "routes:
  - command: shelf.items.AddItem
    method: post
    path: /items
    request: {encoding: json, unknown: refuse}
  - {command: shelf.items.AddStamp, method: post, path: /stamps}
  - {view: shelf.items.AllItems, method: get, path: /items}
  - {view: shelf.items.AllStamps, method: get, path: /stamps}
";

/// One request: label, path, body, and the path of the member it is refused at, or `None` for a
/// body the server admits.
type Sent = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
);

/// Every command request the harness sends, in order. Each admitted body names a fresh identity,
/// and so does each refused one, so a refused request that wrote to the store shows as a row.
const REQUESTS: &[Sent] = &[
    // The issue's valid bodies, and wider ones: every RFC 3339 form `format: date-time` admits
    // (lower-case `t` and `z`, a fraction, an offset, a leap second at the last UTC minute), an
    // open struct carrying a member it does not declare, and a `Json` value carrying anything.
    (
        "item-valid",
        "/items",
        r#"{"item_id":"i1","created_at":"2026-10-29T12:00:00Z","stamps":[],"labels":[{"key":"x","value":"7","unit":"kg"}]}"#,
        None,
    ),
    (
        "item-valid-wide",
        "/items",
        r#"{"item_id":"i2","created_at":"2026-10-29t12:00:00.250+02:00","due_at":"2024-02-29T00:00:00-05:30","stamps":["2026-10-29T12:00:00z","2016-12-31T23:59:60Z"],"labels":[{"key":"x","value":"7","seen_at":"2016-12-31T18:29:60-05:30"}],"note":{"text":"t","colour":"red"},"data":{"any":{"thing":[1,"two"]},"extra":true}}"#,
        None,
    ),
    (
        "stamp-valid",
        "/stamps",
        r#"{"stamp_id":"s1","stamped_at":"2026-10-29T13:00:00-05:30","marks":{"2026-10-29T13:00:00Z":"start"}}"#,
        None,
    ),
    // A `Timestamp` that is not an RFC 3339 date-time, at every position.
    (
        "item-free-text",
        "/items",
        r#"{"item_id":"r1","created_at":"yesterday at noon","stamps":[],"labels":[]}"#,
        Some("body.created_at"),
    ),
    (
        "item-empty-text",
        "/items",
        r#"{"item_id":"r2","created_at":"","stamps":[],"labels":[]}"#,
        Some("body.created_at"),
    ),
    (
        "stamp-free-text",
        "/stamps",
        r#"{"stamp_id":"r3","stamped_at":"yesterday at noon","marks":{}}"#,
        Some("body.stamped_at"),
    ),
    (
        "item-optional-no-such-day",
        "/items",
        r#"{"item_id":"r4","created_at":"2026-10-29T12:00:00Z","due_at":"2026-02-29T00:00:00Z","stamps":[],"labels":[]}"#,
        Some("body.due_at"),
    ),
    (
        "item-list-space",
        "/items",
        r#"{"item_id":"r5","created_at":"2026-10-29T12:00:00Z","stamps":["2026-10-29T12:00:00Z","2026-10-29 12:00:00Z"],"labels":[]}"#,
        Some("body.stamps.1"),
    ),
    (
        "item-struct-no-offset",
        "/items",
        r#"{"item_id":"r6","created_at":"2026-10-29T12:00:00Z","stamps":[],"labels":[{"key":"x","value":"7","seen_at":"2026-10-29T12:00:00"}]}"#,
        Some("body.labels.0.seen_at"),
    ),
    (
        "stamp-leap-second-mid-day",
        "/stamps",
        r#"{"stamp_id":"r7","stamped_at":"2026-10-29T12:00:60Z","marks":{}}"#,
        Some("body.stamped_at"),
    ),
    (
        "stamp-map-key",
        "/stamps",
        r#"{"stamp_id":"r8","stamped_at":"2026-10-29T12:00:00Z","marks":{"tomorrow":"x"}}"#,
        Some("body.marks.tomorrow"),
    ),
    // A member the object's schema does not declare: in a list element, at the top level under a
    // stated `unknown: refuse`, and at the top level under the default.
    (
        "item-element-unknown",
        "/items",
        r#"{"item_id":"r9","created_at":"2026-10-27T12:00:00Z","stamps":[],"labels":[{"key":"x","value":"7","unti":"kg"}]}"#,
        Some("body.labels.0.unti"),
    ),
    (
        "item-top-unknown",
        "/items",
        r#"{"item_id":"r10","created_at":"2026-10-26T12:00:00Z","stamps":[],"labels":[],"colour":"red"}"#,
        Some("body.colour"),
    ),
    (
        "stamp-top-unknown",
        "/stamps",
        r#"{"stamp_id":"r11","stamped_at":"2026-10-29T13:00:00Z","marks":{},"extra":1}"#,
        Some("body.extra"),
    ),
];

/// The harness: every request line through the generated `dispatch`, one answer line each, then
/// both views.
const HARNESS: &str = r#"use shelf_server::http;
use shelf_server::shelf_service as surface;
use shelf_types::actor::{Actor, Caller};
use shelf_types::behaviour::Generated;

fn main() {
    let requests = std::fs::read_to_string(std::env::args().nth(1).expect("the requests")).expect("read");
    let mut system = shelf_system::System::new(shelf_service::ShelfService::new(Generated::new(
        shelf_server::memory::MemoryPorts::default(),
    )));
    let clerk = Caller { actor: Actor::ALL[0] };
    for line in requests.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        let [label, method, path, body] = fields[..] else {
            panic!("a request line has four fields: {line}");
        };
        let request = http::Request {
            method: method.to_owned(),
            path: path.to_owned(),
            query: String::new(),
            headers: vec![("content-type".to_owned(), http::JSON.to_owned())],
            body: body.as_bytes().to_vec(),
        };
        let answered = surface::dispatch(&mut system, Some(&clerk), &request);
        println!("{label}\t{}\t{}", answered.status, answered.body.replace('\n', " "));
    }
}
"#;

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("shelf.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("shelf.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn binding(ir: &EssIr) -> Binding {
    let document = format!(
        "type: ess-http/1\nspecification:\n  system: shelf\n  version: v1\n  source_digest: \
         sha256:{}\n{ROUTES}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&document).expect("well formed");
    let compiled = ess_http::compile(&spec, ir, &ess_gen::http::derived(ir))
        .unwrap_or_else(|refused| panic!("the binding compiles: {refused}"));
    Binding::new(compiled).unwrap_or_else(|refused| panic!("every route has a path: {refused}"))
}

/// One answer the harness printed.
#[derive(Debug)]
struct Answer {
    status: u16,
    body: String,
}

impl Answer {
    fn json(&self) -> Value {
        serde_json::from_str(&self.body)
            .unwrap_or_else(|error| panic!("the body is JSON ({error}): {}", self.body))
    }
}

/// Every request line the harness reads: the commands, then both views and the served document.
fn request_lines() -> String {
    let mut lines: Vec<String> = REQUESTS
        .iter()
        .map(|(label, path, body, _)| format!("{label}\tPOST\t{path}\t{body}"))
        .collect();
    lines.push("items\tGET\t/items\t".to_owned());
    lines.push("stamps\tGET\t/stamps\t".to_owned());
    lines.push("openapi\tGET\t/openapi.json\t".to_owned());
    lines.join("\n")
}

/// The harness's answers by label, once per test binary.
fn answers() -> &'static BTreeMap<String, Answer> {
    static ANSWERS: std::sync::OnceLock<BTreeMap<String, Answer>> = std::sync::OnceLock::new();
    ANSWERS.get_or_init(|| {
        let ir = ir();
        let synthesis = synthesize_bound(
            &ir,
            Target::Rust,
            OutputLayout::Workspace,
            Some(&binding(&ir)),
        )
        .unwrap_or_else(|failure| panic!("the bound model synthesizes: {failure}"));
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("json-body-closure-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (relative, artifact) in &synthesis.artifacts {
            let destination = root.join("shelf").join(relative);
            std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
            std::fs::write(&destination, &artifact.contents).expect("write");
        }
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = [
            "shelf-server",
            "shelf-system",
            "shelf-service",
            "shelf-types",
        ]
        .iter()
        .fold(String::new(), |mut out, name| {
            let _ = writeln!(out, "{name} = {{ path = \"../shelf/crates/{name}\" }}");
            out
        });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"json-body-closure-harness\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), HARNESS).expect("write");
        let requests = root.join("requests.tsv");
        std::fs::write(&requests, request_lines()).expect("write");
        let ran = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args(["run", "--offline", "--quiet", "--target-dir"])
            .arg(root.join("target"))
            .arg("--")
            .arg(&requests)
            .current_dir(&harness)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        eprintln!("{}\n{stdout}", String::from_utf8_lossy(&ran.stderr));
        let _ = std::fs::remove_dir_all(&root);
        assert!(ran.status.success(), "the harness builds and runs");
        stdout
            .lines()
            .map(|line| {
                let mut fields = line.splitn(3, '\t');
                let label = fields.next().expect("a label").to_owned();
                let status = fields
                    .next()
                    .and_then(|status| status.parse().ok())
                    .expect("a status");
                let body = fields.next().unwrap_or_default().to_owned();
                (label, Answer { status, body })
            })
            .collect()
    })
}

fn answered(label: &str) -> &'static Answer {
    answers()
        .get(label)
        .unwrap_or_else(|| panic!("the harness answered `{label}`"))
}

/// The rows a view answered, keyed by their identity member.
fn rows(label: &str, identity: &str) -> BTreeMap<String, Value> {
    let answer = answered(label);
    assert_eq!(answer.status, 200, "{label}: {}", answer.body);
    answer.json()["rows"]
        .as_array()
        .unwrap_or_else(|| panic!("{label} answers rows: {}", answer.body))
        .iter()
        .map(|row| {
            (
                row[identity].as_str().expect("an identity").to_owned(),
                row.clone(),
            )
        })
        .collect()
}

/// Each refused body is answered `400` naming the path of the offending member; each admitted one
/// runs.
#[test]
fn a_body_its_openapi_refuses_is_answered_400_naming_the_member() {
    let mut failures = String::new();
    for (label, _, _, refused_at) in REQUESTS {
        let answer = answered(label);
        match refused_at {
            Some(at) => {
                let named = answer
                    .json()
                    .get("refused")
                    .and_then(Value::as_str)
                    .is_some_and(|detail| detail.starts_with(&format!("{at}: ")));
                if answer.status != 400 || !named {
                    let _ = writeln!(
                        failures,
                        "{label}: expected 400 naming `{at}`, got {} {}",
                        answer.status, answer.body
                    );
                }
            }
            None => {
                if answer.status != 202 {
                    let _ = writeln!(
                        failures,
                        "{label}: expected 202, got {} {}",
                        answer.status, answer.body
                    );
                }
            }
        }
    }
    assert!(failures.is_empty(), "{failures}");
}

/// A refused body wrote nothing; each admitted body's row reads back as it was sent, except the
/// member the open struct ignores.
#[test]
fn an_admitted_body_reads_back_unchanged_and_a_refused_one_is_never_stored() {
    let items = rows("items", "item_id");
    let stamps = rows("stamps", "stamp_id");
    assert_eq!(
        items.keys().chain(stamps.keys()).collect::<Vec<_>>(),
        ["i1", "i2", "s1"],
        "only the admitted bodies were stored"
    );
    let sent = |label: &str| -> Value {
        let (_, _, body, _) = REQUESTS
            .iter()
            .find(|(sent, ..)| *sent == label)
            .expect("sent");
        serde_json::from_str(body).expect("JSON")
    };
    assert_eq!(items["i1"], sent("item-valid"));
    let mut wide = sent("item-valid-wide");
    wide["note"]
        .as_object_mut()
        .expect("an object")
        .remove("colour");
    assert_eq!(items["i2"], wide);
    assert_eq!(stamps["s1"], sent("stamp-valid"));
}

/// The server and the document it serves agree on every body: the request schema of the route in
/// the served `openapi.json`, with `format` asserted, refuses exactly the bodies the server
/// answers `400`.
#[test]
fn the_server_refuses_exactly_what_its_served_openapi_refuses() {
    let document = answered("openapi").json();
    let mut failures = String::new();
    let (mut refused, mut admitted) = (0, 0);
    for (label, path, body, _) in REQUESTS {
        let at = &document["paths"][*path]["post"]["requestBody"]["content"]["application/json"]
            ["schema"];
        assert!(at["$ref"].is_string(), "{label}: {at}");
        let mut schema = document.clone();
        schema["$ref"] = at["$ref"].clone();
        let validator = jsonschema::draft202012::options()
            .should_validate_formats(true)
            .build(&schema)
            .unwrap_or_else(|error| panic!("the served document compiles as a schema: {error}"));
        let value: Value = serde_json::from_str(body).expect("JSON");
        let document_admits = validator.is_valid(&value);
        let server_admits = answered(label).status != 400;
        if document_admits {
            admitted += 1;
        } else {
            refused += 1;
        }
        if document_admits != server_admits {
            let _ = writeln!(
                failures,
                "{label}: the document {} it, the server answers {} {}",
                if document_admits { "admits" } else { "refuses" },
                answered(label).status,
                answered(label).body
            );
        }
    }
    assert!(failures.is_empty(), "{failures}");
    assert_eq!((admitted, refused), (3, 11), "every body was compared");
    assert_eq!(
        document["components"]["schemas"]["shelf.items.Label"]["additionalProperties"],
        json!(false),
        "the list element is closed in the served document"
    );
    assert_eq!(
        document["components"]["schemas"]["shelf.items.Note"]["additionalProperties"],
        json!(true),
        "the struct declaring `undeclared_fields: ignored` is open in the served document"
    );
}
