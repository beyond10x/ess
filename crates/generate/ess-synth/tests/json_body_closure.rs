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
//!
//! The harness is not built here: a test that runs cargo compiles a crate graph inside its lane
//! (`crates/edge/ess-xtask/tests/no_test_builds.rs`). `task json-body-harnesses` generates the
//! server from [`MODEL`] and [`BINDING`] with `ess generate synthesize --http` and builds the
//! harness once, before the test binaries run, and this test runs the binary [`HARNESS`] names.
//! Without it every case fails, naming the task.

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::path::PathBuf;
use std::process::Command;

use ess_compiler::{resolve::compile, source::SourceMap, EssIr};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_gen::ess_http;
use ess_gen::http::Binding;
use serde_json::{json, Value};

/// The model, read by `task json-body-harnesses` too, which generates the server the harness is
/// built against from the same file. It is named `.yaml.in` so no walker of the repository's
/// models reads it, as none did while it lived inside this file.
const MODEL: &str = include_str!("fixtures/json-body-closure/model.yaml.in");

/// The binding the server is generated with: `AddItem` states `unknown: refuse`; `AddStamp` leaves
/// the request at its default. Its `source_digest` is [`MODEL`]'s, which [`binding`] holds it to.
const BINDING: &str = include_str!("fixtures/json-body-closure/http.yaml.in");

/// The variable naming the harness `task json-body-harnesses` built
/// (`fixtures/json-body-closure/main.rs`).
const HARNESS: &str = "ESS_JSON_BODY_CLOSURE_HARNESS";

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

/// [`BINDING`], compiled against `ir`: it names `ir`'s source digest, so the server the harness
/// was built against was generated from [`MODEL`] and this binding.
fn binding(ir: &EssIr) -> Binding {
    assert!(
        BINDING.contains(&format!("  source_digest: sha256:{}\n", ir.source_digest())),
        "fixtures/json-body-closure/http.yaml.in pins MODEL's source digest, sha256:{}; write it \
         there and run `task json-body-harnesses` again",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(BINDING).expect("well formed");
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

/// The harness `task json-body-harnesses` built, from the path [`HARNESS`] names.
///
/// A file that is not executable is made so: an uploaded CI artifact loses its mode bits, and the
/// test shards receive the harness beside the nextest archives.
fn harness() -> PathBuf {
    let Some(path) = std::env::var_os(HARNESS).map(PathBuf::from) else {
        panic!(
            "{HARNESS} is not set: run `task json-body-harnesses` and set it to the binary that \
             task names, or run the tests through `task test`, which does both"
        );
    };
    let metadata = std::fs::metadata(&path).unwrap_or_else(|error| {
        panic!(
            "{HARNESS}={} names no harness ({error}): run `task json-body-harnesses`",
            path.display()
        )
    });
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mut permissions = metadata.permissions();
        if permissions.mode() & 0o111 == 0 {
            permissions.set_mode(permissions.mode() | 0o755);
            std::fs::set_permissions(&path, permissions).expect("the harness is made executable");
        }
    }
    #[cfg(not(unix))]
    let _ = metadata;
    path
}

/// The harness's answers by label, once per test binary.
fn answers() -> &'static BTreeMap<String, Answer> {
    static ANSWERS: std::sync::OnceLock<BTreeMap<String, Answer>> = std::sync::OnceLock::new();
    ANSWERS.get_or_init(|| {
        binding(&ir());
        let harness = harness();
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("json-body-closure-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        std::fs::create_dir_all(&root).expect("mkdir");
        let requests = root.join("requests.tsv");
        std::fs::write(&requests, request_lines()).expect("write");
        let ran = Command::new(&harness)
            .arg(&requests)
            .output()
            .expect("the harness runs");
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        eprintln!("{}\n{stdout}", String::from_utf8_lossy(&ran.stderr));
        let _ = std::fs::remove_dir_all(&root);
        assert!(ran.status.success(), "the harness runs");
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

/// The validator the agreement test holds the server to.
fn oracle(schema: &Value) -> jsonschema::Validator {
    schema_contract::uniqueness::options()
        .should_validate_formats(true)
        .build(schema)
        .unwrap_or_else(|error| panic!("the served document compiles as a schema: {error}"))
}

/// A `date-time` the oracle admits is one RFC 3339, and the servers' own `timestamp_at`, admit:
/// `+`, `-`, `.` and `x` at any of its 14 digit positions are refused.
#[test]
fn the_date_time_oracle_refuses_a_non_digit_at_every_digit_position() {
    const STAMP: &str = "2026-10-29T12:00:00Z";
    let validator = oracle(&json!({"type": "string", "format": "date-time"}));
    assert!(validator.is_valid(&json!(STAMP)), "{STAMP}");
    let mut admitted = Vec::new();
    let mut checked = 0;
    for position in (0..STAMP.len()).filter(|at| STAMP.as_bytes()[*at].is_ascii_digit()) {
        for replacement in ["+", "-", ".", "x"] {
            let mut text = STAMP.to_owned();
            text.replace_range(position..=position, replacement);
            checked += 1;
            if validator.is_valid(&json!(text)) {
                admitted.push(text);
            }
        }
    }
    assert_eq!(checked, 14 * 4, "every position and replacement was tried");
    assert!(admitted.is_empty(), "admitted: {admitted:?}");
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
        let validator = oracle(&schema);
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
