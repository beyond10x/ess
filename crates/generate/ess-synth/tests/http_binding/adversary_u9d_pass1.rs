//! Adversary, unit U9d, pass 1: `body: response.<field>` over every kind a response field can
//! have, and `errors:` beside the binding's refusals, sent to both generated servers, and held
//! against the `OpenAPI` document both embed
//! (`story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status`).
//!
//! `Place`'s response gains a wire-named `String`, a `List`, a `Map`, a struct holding an
//! `Optional`, a newtype, an `Integer` past 2^53, a `Boolean` that is `false` and a `Json` that is
//! `null`. Each gets its own success outcome on its own `2xx` answering that one field bare, and
//! `errors` answers the one error outcome `422`.

use std::collections::BTreeMap;

use ess_synth::{synthesize_bound, OutputLayout, Synthesis, Target};
use serde_json::{json, Value};

use super::super::shared::{self, Answer};
use super::adversary_k6_pass1::{ir_of, rust_run_with};

/// `(anchor, replacement)` in the shared model, each anchor found exactly once.
const MODEL_EDITS: &[(&str, &str)] = &[
    (
        "  - {name: shop.items.ReturnUri, kind: newtype, of: String}\n",
        "  - {name: shop.items.ReturnUri, kind: newtype, of: String}
  - name: shop.items.Detail
    kind: struct
    fields:
      - {name: label, type: String}
      - {name: hint, type: Optional<String>}
",
    ),
    (
        "    response:
      - {name: code, type: String}
    outcomes:
      - name: refused
        when: count > 5
        error: shop.items.OutOfStock
      - name: placed
        returns: true
",
        "    response:
      - {name: code, type: String}
      - {name: version, type: String, naming: {wire: etag}}
      - {name: tags, type: List<String>}
      - {name: counts, type: \"Map<String, Integer>\"}
      - {name: detail, type: shop.items.Detail}
      - {name: item, type: shop.items.ItemId}
      - {name: size, type: Integer}
      - {name: flag, type: Boolean}
      - {name: extra, type: Json}
    outcomes:
      - name: refused
        when: count > 50
        error: shop.items.OutOfStock
      - name: tagged
        when: count == 2
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: untagged
        when: count == 3
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: counted
        when: count == 4
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: detailed
        when: count == 5
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: etagged
        when: count == 6
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: itemed
        when: count == 7
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: sized
        when: count == 8
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: flagged
        when: count == 9
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: nulled
        when: count == 10
        returns: true
        emits: [shop.items.Touched]
        payload:
          shop.items.Touched: {item_id: input.item_id}
      - name: placed
        returns: true
",
    ),
];

/// The `Place` route: every success answers one field bare, `errors` answers `refused`.
const ROUTE_EDITS: &[(&str, &str)] = &[(
    "    outcomes:
      placed:
        status: 201
        body: response
        headers:
          Cache-Control: {literal: no-store}
          X-Item: {source: input.item_id}
          X-Code: {source: response.code}
      refused:
        status: 409
        body: {error: error}
        headers:
          X-Error: {source: error.wire}
",
    "    outcomes:
      placed: {status: 201, body: response.code}
      tagged: {status: 200, body: response.tags}
      untagged: {status: 202, body: response.tags}
      counted: {status: 203, body: response.counts}
      detailed: {status: 206, body: response.detail}
      etagged:
        status: 207
        body: response.version
        headers:
          ETag: {source: response.version}
      itemed: {status: 208, body: response.item}
      sized: {status: 226, body: response.size}
      flagged: {status: 230, body: response.flag}
      nulled: {status: 231, body: response.extra}
    errors:
      status: 422
      body: {error: error}
      headers:
        X-Error: {source: error.wire}
",
)];

/// The `Place` behaviour both harnesses implement: the outcome by `count`, and one response.
const RUST_PLACE: &str = r#"impl PlaceBehavior for Ports {
    fn place(&mut self, input: items::Place) -> Result<items::PlaceOutcome, UnmetObligation> {
        if input.count == 0 {
            return Err(UnmetObligation { capability: "command-behaviour", source: "shop.items.Place" });
        }
        if input.count > 50 {
            return Ok(items::PlaceOutcome::Refused { error: items::OutOfStock });
        }
        let tags: Vec<String> = if input.count == 3 { Vec::new() } else { vec!["b".to_owned(), "a".to_owned()] };
        let mut counts = std::collections::BTreeMap::new();
        counts.insert("b".to_owned(), 2);
        counts.insert("a".to_owned(), 1);
        let response = items::PlaceResponse {
            code: format!("C-{}", input.item_id.0),
            version: "v7".to_owned(),
            tags,
            counts,
            detail: items::Detail { label: "L".to_owned(), hint: None },
            item: input.item_id.clone(),
            size: 9007199254740993,
            flag: false,
            extra: shop_types::json::Value::Null,
        };
        let touched = items::Touched { item_id: input.item_id };
        Ok(match input.count {
            2 => items::PlaceOutcome::Tagged { response, touched },
            3 => items::PlaceOutcome::Untagged { response, touched },
            4 => items::PlaceOutcome::Counted { response, touched },
            5 => items::PlaceOutcome::Detailed { response, touched },
            6 => items::PlaceOutcome::Etagged { response, touched },
            7 => items::PlaceOutcome::Itemed { response, touched },
            8 => items::PlaceOutcome::Sized { response, touched },
            9 => items::PlaceOutcome::Flagged { response, touched },
            10 => items::PlaceOutcome::Nulled { response, touched },
            _ => items::PlaceOutcome::Placed { response, touched },
        })
    }
}

"#;

const GO_PLACE: &str = r#"func (p *ports) Place(input items.Place) (items.PlaceOutcome, *obligation.UnmetObligation) {
	if input.Count == 0 {
		return nil, &obligation.UnmetObligation{Capability: "command-behaviour", Source: "shop.items.Place"}
	}
	if input.Count > 50 {
		return items.PlaceOutcomeRefused{}, nil
	}
	tags := []string{"b", "a"}
	if input.Count == 3 {
		tags = []string{}
	}
	response := items.PlaceResponse{
		Code:    "C-" + input.ItemId.Value(),
		Version: "v7",
		Tags:    tags,
		Counts:  map[string]int64{"b": 2, "a": 1},
		Detail:  items.Detail{Label: "L"},
		Item:    input.ItemId,
		Size:    9007199254740993,
		Flag:    false,
		Extra:   primitives.Json{},
	}
	touched := items.Touched{ItemId: input.ItemId}
	switch input.Count {
	case 2:
		return items.PlaceOutcomeTagged{Response: response, Touched: touched}, nil
	case 3:
		return items.PlaceOutcomeUntagged{Response: response, Touched: touched}, nil
	case 4:
		return items.PlaceOutcomeCounted{Response: response, Touched: touched}, nil
	case 5:
		return items.PlaceOutcomeDetailed{Response: response, Touched: touched}, nil
	case 6:
		return items.PlaceOutcomeEtagged{Response: response, Touched: touched}, nil
	case 7:
		return items.PlaceOutcomeItemed{Response: response, Touched: touched}, nil
	case 8:
		return items.PlaceOutcomeSized{Response: response, Touched: touched}, nil
	case 9:
		return items.PlaceOutcomeFlagged{Response: response, Touched: touched}, nil
	case 10:
		return items.PlaceOutcomeNulled{Response: response, Touched: touched}, nil
	}
	return items.PlaceOutcomePlaced{Response: response, Touched: touched}, nil
}

"#;

/// `(label, count)` per success outcome, with the status its route declares and the value its
/// field holds.
const SUCCESSES: &[(&str, u16, &str)] = &[
    ("placed", 201, r#""C-i1""#),
    ("tagged", 200, r#"["b","a"]"#),
    ("untagged", 202, "[]"),
    ("counted", 203, r#"{"a":1,"b":2}"#),
    ("detailed", 206, r#"{"label":"L"}"#),
    ("etagged", 207, r#""v7""#),
    ("itemed", 208, r#""i1""#),
    ("sized", 226, "9007199254740993"),
    ("flagged", 230, "false"),
    ("nulled", 231, "null"),
];

fn count_of(label: &str) -> u32 {
    match label {
        "placed" => 1,
        "tagged" => 2,
        "untagged" => 3,
        "counted" => 4,
        "detailed" => 5,
        "etagged" => 6,
        "itemed" => 7,
        "sized" => 8,
        "flagged" => 9,
        "nulled" => 10,
        other => panic!("no count for {other}"),
    }
}

fn edited(text: &str, edits: &[(&str, &str)]) -> String {
    let mut text = text.to_owned();
    for (anchor, replacement) in edits {
        assert_eq!(
            text.matches(anchor).count(),
            1,
            "the fixture moved: {anchor}"
        );
        text = text.replace(anchor, replacement);
    }
    text
}

fn synthesized() -> &'static (Synthesis, Synthesis) {
    static BOTH: std::sync::OnceLock<(Synthesis, Synthesis)> = std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let ir = ir_of(&edited(shared::MODEL, MODEL_EDITS));
        let bound = shared::binding(&ir, &edited(shared::ROUTES, ROUTE_EDITS));
        let rust = synthesize_bound(&ir, Target::Rust, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Rust: {failure}"));
        let go = synthesize_bound(&ir, Target::Go, OutputLayout::Workspace, Some(&bound))
            .unwrap_or_else(|failure| panic!("synthesizes for Go: {failure}"));
        (rust, go)
    })
}

fn requests() -> Vec<[String; 7]> {
    let mut lines: Vec<[String; 7]> = SUCCESSES
        .iter()
        .map(|(label, _, _)| {
            [
                (*label).to_owned(),
                "y".to_owned(),
                "POST".to_owned(),
                "/places".to_owned(),
                String::new(),
                String::new(),
                format!(r#"{{"item_id":"i1","count":{}}}"#, count_of(label)),
            ]
        })
        .collect();
    for (label, caller, body) in [
        ("refused", "y", r#"{"item_id":"i1","count":99}"#),
        ("unfinished", "y", r#"{"item_id":"i1","count":0}"#),
        ("not-granted", "n", r#"{"item_id":"i1","count":1}"#),
        ("malformed", "y", r#"{"item_id":"i1","count":"x"}"#),
    ] {
        lines.push([
            label.to_owned(),
            caller.to_owned(),
            "POST".to_owned(),
            "/places".to_owned(),
            String::new(),
            String::new(),
            body.to_owned(),
        ]);
    }
    lines
}

fn scratch(label: &str) -> std::path::PathBuf {
    super::scratch(&format!("u9dadv-{label}"))
}

fn write_requests(path: &std::path::Path) {
    let lines: Vec<String> = requests().iter().map(|line| line.join("\t")).collect();
    std::fs::write(path, lines.join("\n")).expect("write");
}

/// [`super::HARNESS`] with its `Place` replaced by [`GO_PLACE`] and `primitives` imported.
fn go_harness() -> String {
    let start = super::HARNESS
        .find("func (p *ports) Place(")
        .expect("the Go harness's Place");
    let end = super::HARNESS
        .find("func (p *ports) ItemById(")
        .expect("the Go harness's ItemById");
    let text = format!(
        "{}{GO_PLACE}{}",
        &super::HARNESS[..start],
        &super::HARNESS[end..]
    );
    let anchor = "\t\"example.invalid/shop/types/obligation\"\n";
    assert_eq!(text.matches(anchor).count(), 1);
    text.replace(
        anchor,
        "\t\"example.invalid/shop/types/obligation\"\n\t\"example.invalid/shop/types/primitives\"\n",
    )
}

/// [`shared::RUST_HARNESS`] with its `PlaceBehavior` replaced by [`RUST_PLACE`].
fn rust_harness() -> String {
    let start = shared::RUST_HARNESS
        .find("impl PlaceBehavior for Ports {")
        .expect("the Rust harness's Place");
    let end = shared::RUST_HARNESS
        .find("impl ItemByIdQuery for Ports {")
        .expect("the Rust harness's ItemById");
    format!(
        "{}{RUST_PLACE}{}",
        &shared::RUST_HARNESS[..start],
        &shared::RUST_HARNESS[end..]
    )
}

fn go_run(synthesis: &Synthesis) -> BTreeMap<String, Answer> {
    let directory = scratch("module");
    shared::write(synthesis, &directory);
    std::fs::write(directory.join("server/http_binding_test.go"), go_harness()).expect("write");
    let file = scratch("requests.tsv");
    write_requests(&file);
    let ran = super::go(
        &directory,
        &file,
        &[
            "test",
            "-count=1",
            "-timeout",
            "120s",
            "-v",
            "-run",
            "TestHTTPBinding",
            "./server/",
        ],
    );
    let stdout = String::from_utf8_lossy(&ran.stdout).into_owned();
    assert!(
        ran.status.success(),
        "the Go harness builds and runs:\n{stdout}\n{}",
        String::from_utf8_lossy(&ran.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    let _ = std::fs::remove_file(&file);
    shared::parsed(
        stdout
            .lines()
            .filter_map(|line| line.strip_prefix("answer\t")),
    )
}

fn rust_run(synthesis: &Synthesis) -> BTreeMap<String, Answer> {
    let owned = requests();
    let borrowed: Vec<[&str; 7]> = owned
        .iter()
        .map(|line| std::array::from_fn(|index| line[index].as_str()))
        .collect();
    rust_run_with("u9dadv", synthesis, &borrowed, &rust_harness())
}

fn both() -> &'static (BTreeMap<String, Answer>, BTreeMap<String, Answer>) {
    static BOTH: std::sync::OnceLock<(BTreeMap<String, Answer>, BTreeMap<String, Answer>)> =
        std::sync::OnceLock::new();
    BOTH.get_or_init(|| {
        let (rust, go) = synthesized();
        (rust_run(rust), go_run(go))
    })
}

fn pair(label: &str) -> (&'static Answer, &'static Answer) {
    let (rust, go) = both();
    (
        rust.get(label)
            .unwrap_or_else(|| panic!("the Rust server answered `{label}`")),
        go.get(label)
            .unwrap_or_else(|| panic!("the Go server answered `{label}`")),
    )
}

fn parsed_body(answer: &Answer) -> Option<Value> {
    serde_json::from_str(&answer.body).ok()
}

/// Every bare field: the declared status and the field's value, alike from both servers.
#[test]
fn both_servers_answer_each_kind_of_field_bare() {
    let mut wrong = Vec::new();
    for (label, status, expected) in SUCCESSES {
        let expected: Value = serde_json::from_str(expected).expect("expected JSON");
        let (rust, go) = pair(label);
        for (target, answer) in [("rust", rust), ("go", go)] {
            if answer.status != *status || parsed_body(answer).as_ref() != Some(&expected) {
                wrong.push(format!(
                    "{label} ({target}): wanted {status} {expected}, got {answer:?}"
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{}", wrong.join("\n"));
}

/// The bodies' bytes, not only their parsed values, are the same from both servers.
#[test]
fn both_servers_write_the_same_bytes_for_each_bare_field() {
    let mut differing = Vec::new();
    for (label, _, _) in SUCCESSES {
        let (rust, go) = pair(label);
        if (rust.status, &rust.content_type, &rust.body) != (go.status, &go.content_type, &go.body)
        {
            differing.push(format!("{label}:\n  rust {rust:?}\n  go   {go:?}"));
        }
    }
    assert!(differing.is_empty(), "{}", differing.join("\n"));
}

/// `errors` answers the error outcome; the binding's standard refusals keep their own answers.
#[test]
fn errors_leaves_the_standard_refusals_their_answers() {
    for label in ["refused", "unfinished", "not-granted", "malformed"] {
        let (rust, go) = pair(label);
        assert_eq!(
            (rust.status, parsed_body(rust)),
            (go.status, parsed_body(go)),
            "{label}:\n  rust {rust:?}\n  go   {go:?}"
        );
    }
    let (rust, go) = pair("refused");
    for answer in [rust, go] {
        assert_eq!(answer.status, 422, "{answer:?}");
        assert_eq!(
            answer.json(),
            json!({"error": "out_of_stock"}),
            "{answer:?}"
        );
        assert_eq!(answer.header("X-Error"), Some("out_of_stock"), "{answer:?}");
    }
    for (label, status) in [
        ("unfinished", 503),
        ("not-granted", 403),
        ("malformed", 400),
    ] {
        let (rust, go) = pair(label);
        for answer in [rust, go] {
            assert_eq!(answer.status, status, "{label}: {answer:?}");
        }
    }
    let (rust, go) = pair("etagged");
    for answer in [rust, go] {
        assert_eq!(answer.header("ETag"), Some("v7"), "{answer:?}");
    }
}

/// The schema at `pointer` admits `value`: `$ref`, `type`, `items`, `properties`, `required`,
/// `additionalProperties`, `enum`, `const`, `oneOf`/`anyOf`/`allOf`. Anything else is reported
/// as not understood.
fn admits(document: &Value, schema: &Value, value: &Value, at: &str, failures: &mut Vec<String>) {
    if let Some(pointer) = schema.get("$ref").and_then(Value::as_str) {
        let key = pointer
            .strip_prefix("#/components/schemas/")
            .unwrap_or_else(|| panic!("{at}: a reference into components: {pointer}"));
        let target = &document["components"]["schemas"][key];
        assert!(!target.is_null(), "{at}: {pointer} resolves to nothing");
        admits(document, target, value, at, failures);
        return;
    }
    for (key, all) in [("allOf", true), ("anyOf", false), ("oneOf", false)] {
        if let Some(branches) = schema.get(key).and_then(Value::as_array) {
            let passing = branches
                .iter()
                .filter(|branch| {
                    let mut inner = Vec::new();
                    admits(document, branch, value, at, &mut inner);
                    inner.is_empty()
                })
                .count();
            let fine = if all {
                passing == branches.len()
            } else {
                passing >= 1
            };
            if !fine {
                failures.push(format!("{at}: {key} refuses {value}: {schema}"));
            }
        }
    }
    if let Some(types) = schema.get("type") {
        let names: Vec<&str> = match types {
            Value::String(one) => vec![one.as_str()],
            Value::Array(many) => many.iter().filter_map(Value::as_str).collect(),
            other => panic!("{at}: a type: {other}"),
        };
        let nullable = schema.get("nullable") == Some(&Value::Bool(true));
        let fits = names.iter().any(|name| match (*name, value) {
            ("string", Value::String(_))
            | ("boolean", Value::Bool(_))
            | ("array", Value::Array(_))
            | ("object", Value::Object(_))
            | ("null", Value::Null)
            | ("number", Value::Number(_)) => true,
            ("integer", Value::Number(number)) => number.is_i64() || number.is_u64(),
            _ => false,
        }) || (nullable && value.is_null());
        if !fits {
            failures.push(format!("{at}: type {types} refuses {value}"));
            return;
        }
    }
    if let Some(allowed) = schema.get("enum").and_then(Value::as_array) {
        if !allowed.contains(value) {
            failures.push(format!("{at}: enum {allowed:?} refuses {value}"));
        }
    }
    if let Some(constant) = schema.get("const") {
        if constant != value {
            failures.push(format!("{at}: const {constant} refuses {value}"));
        }
    }
    if let (Some(items), Value::Array(values)) = (schema.get("items"), value) {
        for (index, item) in values.iter().enumerate() {
            admits(document, items, item, &format!("{at}[{index}]"), failures);
        }
    }
    if let Value::Object(members) = value {
        let properties = schema.get("properties").and_then(Value::as_object);
        if let Some(required) = schema.get("required").and_then(Value::as_array) {
            for name in required.iter().filter_map(Value::as_str) {
                if !members.contains_key(name) {
                    failures.push(format!("{at}: required `{name}` missing from {value}"));
                }
            }
        }
        for (name, member) in members {
            match properties.and_then(|properties| properties.get(name)) {
                Some(property) => {
                    admits(
                        document,
                        property,
                        member,
                        &format!("{at}.{name}"),
                        failures,
                    );
                }
                None => match schema.get("additionalProperties") {
                    Some(Value::Bool(false)) => {
                        failures.push(format!("{at}: `{name}` is not a declared property"));
                    }
                    Some(extra @ Value::Object(_)) => {
                        admits(document, extra, member, &format!("{at}.{name}"), failures);
                    }
                    _ => {}
                },
            }
        }
    }
}

/// Each bare field's answer is admitted by the response schema the embedded document declares
/// for its status, which names that one outcome.
#[test]
fn the_embedded_openapi_admits_each_bare_field_both_servers_answer() {
    let (rust_synthesis, go_synthesis) = synthesized();
    let mut failures = Vec::new();
    for (target, synthesis) in [("rust", rust_synthesis), ("go", go_synthesis)] {
        let documents: Vec<Value> = synthesis
            .artifacts
            .iter()
            .filter(|(path, _)| path.ends_with(".openapi.json"))
            .map(|(_, artifact)| serde_json::from_str(&artifact.contents).expect("JSON"))
            .collect();
        assert_eq!(documents.len(), 1, "{target}");
        let document = &documents[0];
        let responses = &document["paths"]["/places"]["post"]["responses"];
        for (label, status, _) in SUCCESSES {
            let response = &responses[status.to_string()];
            assert_eq!(
                response["x-ess-outcomes"],
                json!([label]),
                "{target} {status}: {response}"
            );
            let schema = &response["content"]["application/json"]["schema"];
            assert!(!schema.is_null(), "{target} {status}: {response}");
            let (rust, go) = pair(label);
            for (server, answer) in [("rust", rust), ("go", go)] {
                let value = parsed_body(answer)
                    .unwrap_or_else(|| panic!("{label} ({server}) answers JSON: {answer:?}"));
                admits(
                    document,
                    schema,
                    &value,
                    &format!("{target} document, {server} {label}"),
                    &mut failures,
                );
            }
        }
        assert_eq!(
            responses["422"]["x-ess-outcomes"],
            json!(["refused"]),
            "{target}: {responses}"
        );
        let refused = parsed_body(pair("refused").0).expect("JSON");
        admits(
            document,
            &responses["422"]["content"]["application/json"]["schema"],
            &refused,
            &format!("{target} refused"),
            &mut failures,
        );
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
