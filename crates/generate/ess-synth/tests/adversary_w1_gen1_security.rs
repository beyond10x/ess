//! Security pass for ESS W1 GEN1 (beyond10x/ess#707,
//! `story:the-generated-rust-server-refuses-what-its-openapi-refuses`).
//!
//! One generated Rust server, built and driven by a harness, answers two kinds of question. Its
//! emitted `json` module's `Timestamp` reader is held to the `format: date-time` the served
//! document asserts, over a seeded corpus, and to linear work and no panic on hostile input. Its
//! `dispatch` is driven over a model that reaches a closed object at positions the unit's own test
//! does not: a struct as a direct field, a struct inside it, a map value, a union branch and its
//! content, a renamed field, a path-carried route, and a newtype over `Timestamp`.

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

// ---- the emitted reader, on its own ---------------------------------------------------------------

/// A fixed-seed xorshift, so every run reads the same corpus.
struct Rng(u64);

impl Rng {
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn below(&mut self, n: usize) -> usize {
        usize::try_from(self.next() % u64::try_from(n).expect("fits")).expect("fits")
    }
}

const SEEDS: &[&str] = &[
    "2026-10-29T12:00:00Z",
    "2016-12-31T23:59:60Z",
    "2016-12-31T18:29:60-05:30",
    "2017-01-01T00:29:60+00:30",
    "2024-02-29T00:00:00.5+14:00",
    "1998-12-31T15:59:60.123-08:00",
    "0000-01-01t00:00:00z",
    "9999-12-31T23:59:59.999999999+23:59",
];

const ALPHABET: &[char] = &[
    '0', '1', '2', '3', '5', '6', '9', '-', ':', 'T', 't', 'Z', 'z', '+', '.', ' ', 'e', '\u{e9}',
    '\u{663}', '\u{ff12}', '\u{0}',
];

/// Hand-picked cases: the JSON Schema Test Suite's `date-time` cases and the edges of the grammar.
const HAND: &[&str] = &[
    "1963-06-19T08:30:06.283185Z",
    "1963-06-19T08:30:06Z",
    "1937-01-01T12:00:27.87+00:20",
    "1990-12-31T15:59:50.123-08:00",
    "1998-12-31T23:59:60Z",
    "1998-12-31T15:59:60.123-08:00",
    "1998-12-31T23:59:61Z",
    "1998-12-31T23:58:60Z",
    "1998-12-31T22:59:60Z",
    "1990-02-31T15:59:59.123-08:00",
    "1990-12-31T15:59:59-24:00",
    "1963-06-19T08:30:06.28123+01:00Z",
    "06/19/1963 08:30:06 PST",
    "1963-06-19t08:30:06.283185z",
    "2013-350T01:01:01",
    "1963-6-19T08:30:06.283185Z",
    "1963-06-1T08:30:06.283185Z",
    "1963-06-1\u{9ea}T00:00:00Z",
    "1963-06-11T0\u{9ea}:00:00Z",
    "",
    "2026-10-29T12:00:00",
    "2026-10-29T12:00:00.Z",
    "2026-10-29T12:00:00,5Z",
    "2026-10-29T12:00:00+0200",
    "2026-10-29T12:00:00+02",
    "2026-10-29T12:00Z",
    "2026-10-29T24:00:00Z",
    "2026-10-29T12:60:00Z",
    "2026-13-01T00:00:00Z",
    "2026-00-01T00:00:00Z",
    "2026-01-00T00:00:00Z",
    "1900-02-29T00:00:00Z",
    "2000-02-29T00:00:00Z",
    "2026-10-29T12:00:00Z ",
    " 2026-10-29T12:00:00Z",
    "2026-10-29 12:00:00Z",
    "2026-10-29T12:00:00-00:00",
    "2026-10-29T12:00:00+23:60",
    "2026-10-29T12:00:00+24:00",
    "2026-10-29T12:00:00ZZ",
    "2026-10-29T12:00:00+02:00Z",
    "2026-10-29TT2:00:00Z",
    "2016-12-31T23:59:60+00:00",
    "2017-01-01T00:59:60+01:00",
    "2016-12-31T23:59:60-00:01",
];

/// Every string of the corpus: the hand cases, the seeds, seeded edits of the seeds, and seeded
/// strings of no particular shape.
fn corpus() -> Vec<String> {
    let mut rng = Rng(0x0707_2026_1102_0001);
    let mut out: Vec<String> = HAND
        .iter()
        .chain(SEEDS)
        .map(|text| (*text).to_owned())
        .collect();
    for _ in 0..40_000 {
        let mut chars: Vec<char> = SEEDS[rng.below(SEEDS.len())].chars().collect();
        for _ in 0..=rng.below(3) {
            let index = rng.below(chars.len().max(1));
            match rng.below(4) {
                0 if !chars.is_empty() => chars[index] = ALPHABET[rng.below(ALPHABET.len())],
                1 => chars.insert(index.min(chars.len()), ALPHABET[rng.below(ALPHABET.len())]),
                2 if !chars.is_empty() => {
                    chars.remove(index);
                }
                _ => {
                    let other = rng.below(chars.len().max(1));
                    if !chars.is_empty() {
                        chars.swap(index, other);
                    }
                }
            }
        }
        out.push(chars.into_iter().collect());
    }
    for _ in 0..20_000 {
        let length = rng.below(32);
        out.push(
            (0..length)
                .map(|_| ALPHABET[rng.below(ALPHABET.len())])
                .collect(),
        );
    }
    out
}

/// The emitted `Timestamp` reader admits exactly the strings the served `format: date-time` admits,
/// as a map key and as a value, and refuses at the path it was given; the harness read the whole
/// corpus without panicking.
#[test]
fn the_emitted_timestamp_reader_admits_exactly_what_format_date_time_admits() {
    let validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&json!({"type": "string", "format": "date-time"}))
        .expect("the schema compiles");
    let corpus = corpus();
    let read = &run().stamps;
    assert_eq!(read.len(), corpus.len(), "the harness read every string");
    let mut disagreements = Vec::new();
    let (mut admitted, mut refused) = (0, 0);
    let mut oracle_errors = 0;
    for (text, (value, key, at)) in corpus.iter().zip(read) {
        let validated = validator.is_valid(&Value::String(text.clone()));
        let document = validated && digits_where_rfc3339_puts_them(text);
        if validated != document {
            oracle_errors += 1;
        }
        if !*value && at != "body.t" {
            disagreements.push(format!("{text:?}: refused at `{at}`"));
        }
        if value != key {
            disagreements.push(format!("{text:?}: value {value} key {key}"));
        }
        if *value != document {
            disagreements.push(format!(
                "{text:?}: the reader {} it, `format: date-time` {} it",
                if *value { "admits" } else { "refuses" },
                if document { "admits" } else { "refuses" }
            ));
        }
        if document {
            admitted += 1;
        } else {
            refused += 1;
        }
    }
    assert!(
        disagreements.is_empty(),
        "{} disagreement(s) over {} strings, the first: {:#?}",
        disagreements.len(),
        corpus.len(),
        &disagreements[..disagreements.len().min(20)]
    );
    assert!(
        admitted > 1_000 && refused > 1_000,
        "the corpus reaches both sides: {admitted} / {refused}"
    );
    eprintln!(
        "jsonschema 0.52.1 admitted {oracle_errors} string(s) with a non-digit at a digit position"
    );
}

/// `jsonschema` 0.52.1 reads two digits with a SWAR test (`keywords/format.rs:382-387`) that
/// takes `+`, `-` and `.` for digits, so it admits `202+-10-29T12:00:00Z`. Its verdict stands only
/// where every position RFC 3339 spells with a digit holds an ASCII digit.
fn digits_where_rfc3339_puts_them(text: &str) -> bool {
    let bytes = text.as_bytes();
    let all = |from: usize, to: usize| {
        bytes
            .get(from..to)
            .is_some_and(|part| part.iter().all(u8::is_ascii_digit))
    };
    let fixed = [(0, 4), (5, 7), (8, 10), (11, 13), (14, 16), (17, 19)]
        .iter()
        .all(|&(from, to)| all(from, to));
    let offset = matches!(bytes.last(), Some(b'Z' | b'z'))
        || (bytes.len() >= 5
            && all(bytes.len() - 5, bytes.len() - 3)
            && all(bytes.len() - 2, bytes.len()));
    fixed && offset
}

/// Hostile sizes do linear work in the emitted module: a fraction of four million digits is read
/// and refused once cut; an object of two hundred thousand members each named by a declared field
/// is admitted, and one more undeclared is refused at its path, built and parsed alike.
#[test]
fn hostile_sizes_are_read_in_linear_time() {
    let hostile = &run().hostile;
    assert_eq!(
        hostile[..5],
        ["true", "false", "true", "body.d", "body.zz"],
        "{hostile:?}"
    );
    let millis: u128 = hostile[5].parse().expect("milliseconds");
    eprintln!("hostile sizes read in {millis} ms");
    assert!(millis < 20_000, "{millis} ms");
}

/// A member named twice: the decoders read the first (`member_at`), so the value stored is the one
/// the `Timestamp` reader validated, whichever of the two is malformed.
#[test]
fn a_member_named_twice_is_read_at_its_first_and_that_one_is_validated() {
    assert_eq!(run().duplicate, ["false", "true"]);
}

// ---- a generated server, at positions the unit's own test does not reach --------------------------

const MODEL: &str = "format: ess/32
system: rack
version: v1
domain: rack.items
types:
  - {name: rack.items.ItemId, kind: newtype, of: String}
  - {name: rack.items.When, kind: newtype, of: Timestamp}
  - name: rack.items.Size
    kind: struct
    fields:
      - {name: width, type: Integer}
      - {name: post_code, type: Optional<String>, naming: {wire: postalCode}}
  - name: rack.items.Label
    kind: struct
    fields:
      - {name: key, type: String}
      - {name: size, type: Optional<rack.items.Size>}
  - name: rack.items.Shape
    kind: union
    tag: kind
    variants:
      boxed: rack.items.Size
      named: String
entities:
  - name: rack.items.Item
    identity: {name: item_id, type: rack.items.ItemId}
    fields:
      - {name: label, type: rack.items.Label}
      - {name: sizes, type: 'Map<String, rack.items.Size>'}
      - {name: shape, type: rack.items.Shape}
      - {name: due_at, type: Optional<rack.items.When>}
    lifecycle: {initial: Kept, states: [Kept], terminal: [Kept], transitions: []}
events:
  - name: rack.items.ItemAdded
    fields: [{name: item_id, type: rack.items.ItemId}]
commands:
  - name: rack.items.AddItem
    input:
      - {name: item_id, type: rack.items.ItemId}
      - {name: label, type: rack.items.Label}
      - {name: sizes, type: 'Map<String, rack.items.Size>'}
      - {name: shape, type: rack.items.Shape}
      - {name: due_at, type: Optional<rack.items.When>}
    outcomes:
      - name: added
        creates: rack.items.Item
        instance: item_id
        sets:
          label: input.label
          sizes: input.sizes
          shape: input.shape
          due_at: input.due_at
        emits: [rack.items.ItemAdded]
        payload:
          rack.items.ItemAdded: {item_id: input.item_id}
views:
  - name: rack.items.AllItems
    source: rack.items.Item
    consistency: read_your_writes
    fields:
      - {name: item_id, type: rack.items.ItemId}
      - {name: label, type: rack.items.Label}
      - {name: sizes, type: 'Map<String, rack.items.Size>'}
      - {name: shape, type: rack.items.Shape}
      - {name: due_at, type: Optional<rack.items.When>}
actors:
  - name: rack.items.Clerk
    may: [rack.items.AddItem]
components:
  - component: rack-service
    reached_by: network
    owns: {domains: [rack.items]}
    accepts: {commands: [rack.items.AddItem]}
    publishes: {events: [rack.items.ItemAdded]}
";

const ROUTES: &str = "routes:
  - command: rack.items.AddItem
    method: post
    path: /items/{item_id}
    parameters: {item_id: input.item_id}
    request: {encoding: json, unknown: refuse}
  - {view: rack.items.AllItems, method: get, path: /items}
";

/// One request: label, path, body, and the member it is refused at, or `None` where it runs.
type Sent = (
    &'static str,
    &'static str,
    &'static str,
    Option<&'static str>,
);

const VALID: &str = r#"{"label":{"key":"k","size":{"width":1,"postalCode":"p"}},"sizes":{"a":{"width":2}},"shape":{"kind":"boxed","value":{"width":3}},"due_at":"2026-10-29T12:00:00Z"}"#;

const REQUESTS: &[Sent] = &[
    ("valid", "/items/v1", VALID, None),
    (
        "valid-named",
        "/items/v2",
        r#"{"label":{"key":"k"},"sizes":{},"shape":{"kind":"named","value":"n"}}"#,
        None,
    ),
    (
        "struct-field-unknown",
        "/items/r1",
        r#"{"label":{"key":"k","colour":"red"},"sizes":{},"shape":{"kind":"named","value":"n"}}"#,
        Some("body.label.colour"),
    ),
    (
        "nested-struct-unknown",
        "/items/r2",
        r#"{"label":{"key":"k","size":{"width":1,"height":2}},"sizes":{},"shape":{"kind":"named","value":"n"}}"#,
        Some("body.label.size.height"),
    ),
    (
        "map-value-unknown",
        "/items/r3",
        r#"{"label":{"key":"k"},"sizes":{"a":{"width":2,"depth":3}},"shape":{"kind":"named","value":"n"}}"#,
        Some("body.sizes.a.depth"),
    ),
    (
        "union-content-unknown",
        "/items/r4",
        r#"{"label":{"key":"k"},"sizes":{},"shape":{"kind":"boxed","value":{"width":3,"x":1}}}"#,
        Some("body.shape.value.x"),
    ),
    (
        "union-branch-unknown",
        "/items/r5",
        r#"{"label":{"key":"k"},"sizes":{},"shape":{"kind":"named","value":"n","extra":1}}"#,
        Some("body.shape.extra"),
    ),
    (
        "model-name",
        "/items/r6",
        r#"{"label":{"key":"k","size":{"width":1,"post_code":"p"}},"sizes":{},"shape":{"kind":"named","value":"n"}}"#,
        Some("body.label.size.post_code"),
    ),
    (
        "newtype-timestamp",
        "/items/r7",
        r#"{"label":{"key":"k"},"sizes":{},"shape":{"kind":"named","value":"n"},"due_at":"soon"}"#,
        Some("body.due_at"),
    ),
    (
        "path-member",
        "/items/r8",
        r#"{"item_id":"r8","label":{"key":"k"},"sizes":{},"shape":{"kind":"named","value":"n"}}"#,
        Some("body.item_id"),
    ),
];

/// An `Optional` member sent as `null`, which the decoder reads as absent.
const NULL_OPTIONAL: Sent = (
    "optional-null",
    "/items/n1",
    r#"{"label":{"key":"k","size":null},"sizes":{},"shape":{"kind":"named","value":"n"},"due_at":null}"#,
    None,
);

const HARNESS: &str = r##"use std::fmt::Write as _;

use rack_server::{http, json};
use rack_server::rack_service as surface;
use rack_types::actor::{Actor, Caller};
use rack_types::behaviour::Generated;

fn main() {
    let requests = std::fs::read_to_string(std::env::args().nth(1).expect("the requests")).expect("read");
    let corpus = std::fs::read_to_string(std::env::args().nth(2).expect("the corpus")).expect("read");
    for (index, text) in corpus.lines().enumerate() {
        let held = json::Value::Text(text.to_owned());
        let value = json::timestamp_at(&held, "body.t", "an RFC 3339 instant");
        let key = json::key_timestamp(text, "body.k");
        let at = value.as_ref().err().map(|error| error.at.clone()).unwrap_or_default();
        println!("stamp\t{index}\t{}\t{}\t{at}", value.is_ok(), key.is_ok());
    }
    let started = std::time::Instant::now();
    let long = format!("2026-10-29T12:00:00.{}Z", "9".repeat(4_000_000));
    let long_ok = json::timestamp_at(&json::Value::Text(long.clone()), "t", "x").is_ok();
    let cut = format!("{}X", &long[..long.len() - 1]);
    let cut_ok = json::timestamp_at(&json::Value::Text(cut), "t", "x").is_ok();
    let declared = ["a", "b", "c"];
    let mut members = vec![("a".to_owned(), json::Value::Null); 200_000];
    let many_ok = json::closed_at(&json::Value::Object(members.clone()), "body", &declared).is_ok();
    members.push(("d".to_owned(), json::Value::Null));
    let last_at = json::closed_at(&json::Value::Object(members), "body", &declared)
        .err()
        .map(|error| error.at)
        .unwrap_or_default();
    let mut text = String::from("{");
    for index in 0..100_000 {
        let _ = write!(text, "\"a\":{index},");
    }
    text.push_str("\"zz\":1}");
    let parsed = json::parse(&text).expect("JSON");
    let parsed_at = json::closed_at(&parsed, "body", &declared)
        .err()
        .map(|error| error.at)
        .unwrap_or_default();
    println!(
        "hostile\t{long_ok}\t{cut_ok}\t{many_ok}\t{last_at}\t{parsed_at}\t{}",
        started.elapsed().as_millis()
    );
    let first_bad = json::parse(r#"{"t":"yesterday","t":"2026-10-29T12:00:00Z"}"#).expect("JSON");
    let first = json::timestamp_at(json::member_at(&first_bad, "body", "t").expect("present"), "body.t", "x");
    let last_bad = json::parse(r#"{"t":"2026-10-29T12:00:00Z","t":"yesterday"}"#).expect("JSON");
    let last = json::timestamp_at(json::member_at(&last_bad, "body", "t").expect("present"), "body.t", "x");
    println!("duplicate\t{}\t{}", first.is_ok(), last == Ok("2026-10-29T12:00:00Z"));
    let mut system = rack_system::System::new(rack_service::RackService::new(Generated::new(
        rack_server::memory::MemoryPorts::default(),
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
"##;

fn ir() -> EssIr {
    let spec = Specification::assemble([(
        Source::new("rack.yaml"),
        RawSpecFile::parse(MODEL).expect("well formed"),
    )])
    .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("rack.yaml", MODEL);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn binding(ir: &EssIr) -> Binding {
    let document = format!(
        "type: ess-http/1\nspecification:\n  system: rack\n  version: v1\n  source_digest: \
         sha256:{}\n{ROUTES}",
        ir.source_digest()
    );
    let spec = ess_http::HttpSpec::from_yaml(&document).expect("well formed");
    let compiled = ess_http::compile(&spec, ir, &ess_gen::http::derived(ir))
        .unwrap_or_else(|refused| panic!("the binding compiles: {refused}"));
    Binding::new(compiled).unwrap_or_else(|refused| panic!("every route has a path: {refused}"))
}

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

/// What the harness printed: each request's answer, each corpus string's reading (value admitted,
/// key admitted, refusal path), the hostile-size line and the duplicate-member line.
struct Run {
    answers: BTreeMap<String, Answer>,
    stamps: Vec<(bool, bool, String)>,
    hostile: Vec<String>,
    duplicate: Vec<String>,
}

#[allow(clippy::too_many_lines)]
fn run() -> &'static Run {
    static RUN: std::sync::OnceLock<Run> = std::sync::OnceLock::new();
    RUN.get_or_init(|| {
        let ir = ir();
        let synthesis = synthesize_bound(
            &ir,
            Target::Rust,
            OutputLayout::Workspace,
            Some(&binding(&ir)),
        )
        .unwrap_or_else(|failure| panic!("the bound model synthesizes: {failure}"));
        let root = PathBuf::from(env!("CARGO_TARGET_TMPDIR"))
            .join(format!("adversary-w1-gen1-security-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        for (relative, artifact) in &synthesis.artifacts {
            let destination = root.join("rack").join(relative);
            std::fs::create_dir_all(destination.parent().expect("a parent")).expect("mkdir");
            std::fs::write(&destination, &artifact.contents).expect("write");
        }
        let harness = root.join("harness");
        std::fs::create_dir_all(harness.join("src")).expect("mkdir");
        let dependencies = ["rack-server", "rack-system", "rack-service", "rack-types"]
            .iter()
            .fold(String::new(), |mut out, name| {
                let _ = writeln!(out, "{name} = {{ path = \"../rack/crates/{name}\" }}");
                out
            });
        std::fs::write(
            harness.join("Cargo.toml"),
            format!(
                "[package]\nname = \"adversary-w1-gen1-security-harness\"\nversion = \"0.0.0\"\n\
                 edition = \"2021\"\n\n[dependencies]\n{dependencies}\n[workspace]\n"
            ),
        )
        .expect("write");
        std::fs::write(harness.join("src/main.rs"), HARNESS).expect("write");
        let mut lines: Vec<String> = REQUESTS
            .iter()
            .chain(std::iter::once(&NULL_OPTIONAL))
            .map(|(label, path, body, _)| format!("{label}\tPOST\t{path}\t{body}"))
            .collect();
        lines.push("items\tGET\t/items\t".to_owned());
        lines.push("openapi\tGET\t/openapi.json\t".to_owned());
        let requests = root.join("requests.tsv");
        std::fs::write(&requests, lines.join("\n")).expect("write");
        let stamps = root.join("corpus.txt");
        std::fs::write(
            &stamps,
            corpus().iter().fold(String::new(), |mut out, text| {
                out.push_str(text);
                out.push('\n');
                out
            }),
        )
        .expect("write");
        let ran = Command::new(std::env::var_os("CARGO").expect("Cargo supplies its executable"))
            .args(["run", "--offline", "--quiet", "--target-dir"])
            .arg(root.join("target"))
            .arg("--")
            .arg(&requests)
            .arg(&stamps)
            .current_dir(&harness)
            .env_remove("CARGO_TARGET_DIR")
            .env_remove("CARGO_ENCODED_RUSTFLAGS")
            .env("CARGO_INCREMENTAL", "0")
            .output()
            .expect("cargo runs");
        let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
        let stderr = String::from_utf8_lossy(&ran.stderr).into_owned();
        let _ = std::fs::remove_dir_all(&root);
        assert!(
            ran.status.success(),
            "the harness builds and runs: {stderr}"
        );
        let mut out = Run {
            answers: BTreeMap::new(),
            stamps: Vec::new(),
            hostile: Vec::new(),
            duplicate: Vec::new(),
        };
        for line in stdout.lines() {
            let fields: Vec<&str> = line.split('\t').collect();
            match fields[0] {
                "stamp" => out.stamps.push((
                    fields[2] == "true",
                    fields[3] == "true",
                    fields[4].to_owned(),
                )),
                "hostile" => out.hostile = fields[1..].iter().map(|f| (*f).to_owned()).collect(),
                "duplicate" => {
                    out.duplicate = fields[1..].iter().map(|f| (*f).to_owned()).collect();
                }
                label => {
                    let mut rest = line.splitn(3, '\t').skip(1);
                    let status = rest
                        .next()
                        .and_then(|status| status.parse().ok())
                        .expect("a status");
                    let body = rest.next().unwrap_or_default().to_owned();
                    out.answers
                        .insert(label.to_owned(), Answer { status, body });
                }
            }
        }
        for (label, answer) in &out.answers {
            if label != "openapi" {
                eprintln!("{label}\t{}\t{}", answer.status, answer.body);
            }
        }
        out
    })
}

fn answered(label: &str) -> &'static Answer {
    run()
        .answers
        .get(label)
        .unwrap_or_else(|| panic!("the harness answered `{label}`"))
}

/// Whether the served document's request schema for this route admits the body, `format` asserted.
fn document_admits(path: &str, body: &str) -> bool {
    let document = answered("openapi").json();
    let at = &document["paths"]["/items/{item_id}"]["post"]["requestBody"]["content"]
        ["application/json"]["schema"];
    assert!(at["$ref"].is_string(), "{path}: {at}");
    let mut schema = document.clone();
    schema["$ref"] = at["$ref"].clone();
    let validator = jsonschema::draft202012::options()
        .should_validate_formats(true)
        .build(&schema)
        .unwrap_or_else(|error| panic!("the served document compiles as a schema: {error}"));
    validator.is_valid(&serde_json::from_str::<Value>(body).expect("JSON"))
}

/// At every closed position the model reaches, an undeclared member is refused `400` naming its
/// path, a non-RFC 3339 instant behind a newtype is too, nothing refused is stored, and the served
/// document agrees with each answer.
#[test]
fn every_closed_position_refuses_its_undeclared_member_as_the_document_does() {
    let mut failures = String::new();
    for (label, path, body, refused_at) in REQUESTS {
        let answer = answered(label);
        let server_admits = (200..300).contains(&answer.status);
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
            None if !server_admits => {
                let _ = writeln!(
                    failures,
                    "{label}: expected 2xx, got {} {}",
                    answer.status, answer.body
                );
            }
            None => {}
        }
        if document_admits(path, body) != server_admits {
            let _ = writeln!(
                failures,
                "{label}: the document {} it, the server answers {} {}",
                if server_admits { "refuses" } else { "admits" },
                answer.status,
                answer.body
            );
        }
    }
    let items = answered("items");
    assert_eq!(items.status, 200, "{}", items.body);
    let rows = items.json()["rows"].as_array().cloned().unwrap_or_default();
    let mut ids: Vec<String> = rows
        .iter()
        .map(|row| row["item_id"].as_str().unwrap_or_default().to_owned())
        .collect();
    ids.sort();
    if ids != ["n1", "v1", "v2"] {
        let _ = writeln!(
            failures,
            "stored rows {ids:?}, expected only the admitted bodies"
        );
    }
    let mut sent: Value = serde_json::from_str(VALID).expect("JSON");
    sent["item_id"] = json!("v1");
    let v1 = rows.iter().find(|row| row["item_id"] == json!("v1"));
    if v1 != Some(&sent) {
        let _ = writeln!(failures, "v1 reads back as {v1:?}, sent {sent}");
    }
    assert!(failures.is_empty(), "{failures}");
}
