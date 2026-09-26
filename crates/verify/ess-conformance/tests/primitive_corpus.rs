//! The normative primitive corpus, answered by all three admission implementations.
//!
//! One document — `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`, the
//! corpus `docs/design/review-primitive-semantics.md` names — read here by
//! [`ess_conformance::Holds::admits`], by the Go conformance runtime's `primitive`, and by the
//! browser adapter's `primitiveAdmits`. Three languages, one table: a vector the three disagree
//! about fails here rather than in a target's test suite months later.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::{AdmittedSuite, Holds};
use ess_domain::Primitive;
use ess_primitives::facts::{FactValue, Number};
use ess_primitives::node::Node;
use serde_json::{json, Value};

const CORPUS: &str =
    include_str!("../../../specify/ess-primitives/tests/vectors/primitive-semantics.json");
const ADMISSION_JS: &str = include_str!("../assets/coverage-admission.js");

#[derive(serde::Deserialize)]
struct Vector {
    name: String,
    kind: String,
    value: Value,
    admitted: bool,
}

fn vectors() -> Vec<Vector> {
    #[derive(serde::Deserialize)]
    struct Corpus {
        admission: Vec<Vector>,
    }
    serde_json::from_str::<Corpus>(CORPUS)
        .expect("the corpus is readable")
        .admission
}

fn primitive_named(kind: &str) -> Primitive {
    match kind {
        "string" => Primitive::String,
        "boolean" => Primitive::Boolean,
        "integer" => Primitive::Integer,
        "decimal" => Primitive::Decimal,
        "timestamp" => Primitive::Timestamp,
        "duration" => Primitive::Duration,
        "uuid" => Primitive::Uuid,
        "bytes" => Primitive::Bytes,
        other => panic!("{other} is not a primitive this corpus names"),
    }
}

fn directory(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "ess-primitive-corpus-{label}-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a working directory");
    path
}

#[test]
fn rust_admission_answers_every_vector_the_corpus_states() {
    for vector in vectors() {
        let node: Node = serde_json::from_value(vector.value.clone()).expect("a corpus node");
        let holds = Holds::Primitive {
            kind: primitive_named(&vector.kind),
        };
        assert_eq!(
            holds.admits(&node),
            vector.admitted,
            "Rust {}: {}",
            vector.name,
            vector.value
        );
    }
}

/// The smallest suite/5 the Go emitter accepts, so the runtime compiles beside its suite.
fn minimal_suite() -> AdmittedSuite {
    let document = json!({
        "provenance": {
            "suite_version": "ess-conformance/5", "system": "example",
            "specification_version": "v1",
            "spec_digest": "a".repeat(64),
            "contract_digest": "a".repeat(64)
        },
        "scenarios": {
            "example.domain/authored/created": {"purpose": "Known candidate", "steps": [], "source": []}
        },
        "coverage": {
            "selection": {"scope": {"kind": "system"}, "origins": "authored", "filter": {"kind": "all"}},
            "knowledge": "complete_inventory", "generated": [],
            "authored": ["example.domain/authored/created"], "outside": [], "refused": [],
            "authored_sources": {"created.yaml": {
                "digest": format!("sha256:{}", "b".repeat(64)),
                "scenario": "example.domain/authored/created", "disposition": "accepted"
            }},
            "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 0}
        }
    });
    AdmittedSuite::from_json(&serde_json::to_string_pretty(&document).unwrap())
        .expect("the minimal suite is admitted")
}

/// The Go test that asks the runtime's own `primitive` about every vector.
const GO_CORPUS_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

type corpusVector struct {
	Name     string `json:"name"`
	Kind     string `json:"kind"`
	Value    any    `json:"value"`
	Admitted bool   `json:"admitted"`
}

func TestPrimitiveCorpus(t *testing.T) {
	raw, err := os.ReadFile("vectors.json")
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		Admission []corpusVector `json:"admission"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	if len(corpus.Admission) == 0 {
		t.Fatal("the corpus selected no vector")
	}
	for _, vector := range corpus.Admission {
		admitted := primitive(vector.Kind, vector.Value) == ""
		if admitted != vector.Admitted {
			t.Errorf("Go %s: primitive(%q, %v) admitted=%v, corpus says %v",
				vector.Name, vector.Kind, vector.Value, admitted, vector.Admitted)
		}
	}
}
"#;

#[test]
fn the_go_runtime_answers_every_vector_the_corpus_states() {
    let root = directory("go");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module primitivecorpus\n\ngo 1.24\n").unwrap();
    std::fs::write(package.join("vectors.json"), CORPUS).unwrap();
    std::fs::write(package.join("corpus_test.go"), GO_CORPUS_TEST).unwrap();

    let output = Command::new("go")
        .args(["test", "-count=1", "./...", "-run", "^TestPrimitiveCorpus$"])
        .current_dir(&root)
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("go.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
}

/// The Go test that asks the runtime's predicate evaluator to order every `text_orderings` pair.
///
/// Through `parseLeaf` and `evaluate` rather than through `compare` alone, so the vector reaches
/// the ordering the way a `when:` or a view filter does (ess#94).
const GO_TEXT_ORDER_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

type textOrdering struct {
	Name     string `json:"name"`
	Left     string `json:"left"`
	Right    string `json:"right"`
	Ordering string `json:"ordering"`
}

func TestTextOrderingCorpus(t *testing.T) {
	raw, err := os.ReadFile("vectors.json")
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		TextOrderings []textOrdering `json:"text_orderings"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	if len(corpus.TextOrderings) < 8 {
		t.Fatalf("the corpus selected %d text orderings", len(corpus.TextOrderings))
	}
	for _, vector := range corpus.TextOrderings {
		less, greater := vector.Ordering == "less", vector.Ordering == "greater"
		for op, holds := range map[string]bool{"<": less, "<=": !greater, ">": greater, ">=": !less} {
			expression := "caller " + op + " \"" + vector.Right + "\""
			leaf, err := parseLeaf(expression)
			if err != nil {
				t.Fatalf("%s: %s: %v", vector.Name, expression, err)
			}
			got := leaf.evaluate(factSource{"caller": vector.Left})
			if got != truthOf(holds) {
				t.Errorf("Go %s: %q %s: got %v, corpus says %v", vector.Name, vector.Left, expression, got, holds)
			}
		}
	}
}
"#;

#[test]
fn the_go_runtime_orders_every_text_pair_the_corpus_states_by_its_bytes() {
    let root = directory("go-text");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module textcorpus\n\ngo 1.24\n").unwrap();
    std::fs::write(package.join("vectors.json"), CORPUS).unwrap();
    std::fs::write(package.join("text_order_test.go"), GO_TEXT_ORDER_TEST).unwrap();

    let output = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestTextOrderingCorpus$",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("go.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("--- PASS: TestTextOrderingCorpus"),
        "the Go case ran rather than selecting nothing: {record}"
    );
}

/// The Go test that asks the runtime's predicate reader and evaluator every `text_matches` vector
/// (beyond10x/ess#95), with the admission and meaning a suite reaches them through.
///
/// A `null` vector is bound, as the Go flattener binds one (`bindFact`'s `default:`), so the lane
/// that does bind it is the lane that has to answer `Unknown` for it.
const GO_TEXT_MATCH_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

func TestTextMatchCorpus(t *testing.T) {
	raw, err := os.ReadFile("vectors.json")
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		TextMatches []map[string]json.RawMessage `json:"text_matches"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	if len(corpus.TextMatches) < 20 {
		t.Fatalf("the corpus selected %d text matches", len(corpus.TextMatches))
	}
	for _, vector := range corpus.TextMatches {
		var name, op, literal, expected string
		for key, into := range map[string]*string{"name": &name, "op": &op, "literal": &literal, "truth": &expected} {
			if err := json.Unmarshal(vector[key], into); err != nil {
				t.Fatalf("%s: %v", key, err)
			}
		}
		node := map[string]any{"caller": map[string]any{op: literal}}
		if err := admitPredicateVersion(node, 14); err != nil {
			t.Fatalf("%s: suite/14 refuses %v: %v", name, node, err)
		}
		if err := admitPredicateVersion(node, 13); err == nil {
			t.Errorf("%s: suite/13 admits %v", name, node)
		}
		leaf, err := fromNode(node)
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		source := factSource{}
		if value, present := vector["value"]; present {
			var decoded any
			if err := json.Unmarshal(value, &decoded); err != nil {
				t.Fatal(err)
			}
			source["caller"] = decoded
		}
		want := map[string]truth{"true": truthTrue, "false": truthFalse, "unknown": truthUnknown}[expected]
		if got := leaf.evaluate(source); got != want {
			t.Errorf("Go %s: %v over %v: got %v, corpus says %s", name, node, source, got, expected)
		}
		if got := (predicate{kind: "not", body: &leaf}).evaluate(source); got != want.not() {
			t.Errorf("Go %s: not %v: got %v", name, node, got)
		}
		if meaning, ok := predicateMeaning(node).([]any); !ok || meaning[0] != op {
			t.Errorf("%s: meaning of %v is %v, not its own operator", name, node, predicateMeaning(node))
		}
	}
	for _, operand := range []any{44.0, true, nil, []any{"a"}, map[string]any{"a": "b"}} {
		node := map[string]any{"caller": map[string]any{"starts_with": operand}}
		if err := admitPredicateVersion(node, 14); err == nil {
			t.Errorf("suite/14 admits the non-string operand %v", operand)
		}
	}
}
"#;

#[test]
fn the_go_runtime_answers_every_text_match_the_corpus_states() {
    let root = directory("go-text-match");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module textmatchcorpus\n\ngo 1.24\n").unwrap();
    std::fs::write(package.join("vectors.json"), CORPUS).unwrap();
    std::fs::write(package.join("text_match_test.go"), GO_TEXT_MATCH_TEST).unwrap();

    let output = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestTextMatchCorpus$",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("go.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("--- PASS: TestTextMatchCorpus"),
        "the Go case ran rather than selecting nothing: {record}"
    );
}

/// The Go test that asks the runtime's leaf reads every `text_lengths` vector (beyond10x/ess#104):
/// a bound fact wins; otherwise `<parent>.count` over bound text is its rune count. A `null` is
/// bound, as the Go flattener binds one, and has no length. Every leaf kind reads through the same
/// rule, and a quantifier over a text stays `Unknown`, because cardinality is not a length.
const GO_TEXT_LENGTH_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"testing"
)

func TestTextLengthCorpus(t *testing.T) {
	raw, err := os.ReadFile("vectors.json")
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		TextLengths []struct {
			Name     string          `json:"name"`
			Facts    map[string]any  `json:"facts"`
			Path     string          `json:"path"`
			Expected json.RawMessage `json:"expected"`
		} `json:"text_lengths"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	if len(corpus.TextLengths) < 8 {
		t.Fatalf("the corpus selected %d text lengths", len(corpus.TextLengths))
	}
	equals := func(path string, value float64) predicate {
		return predicate{kind: "compare", left: operand{path: path, isFact: true}, op: "==", right: operand{literal: value}}
	}
	for _, vector := range corpus.TextLengths {
		source := factSource{}
		for path, value := range vector.Facts {
			source[path] = value
		}
		var count float64
		if err := json.Unmarshal(vector.Expected, &count); err == nil {
			if got := equals(vector.Path, count).evaluate(source); got != truthTrue {
				t.Errorf("Go %s: %s == %v over %v: got %v", vector.Name, vector.Path, count, source, got)
			}
			if got := equals(vector.Path, count+1).evaluate(source); got != truthFalse {
				t.Errorf("Go %s: %s == %v over %v: got %v", vector.Name, vector.Path, count+1, source, got)
			}
			continue
		}
		if got := equals(vector.Path, 0).evaluate(source); got != truthUnknown {
			t.Errorf("Go %s: %s has no length over %v, got %v", vector.Name, vector.Path, source, got)
		}
	}
	bare := factSource{"keys": "abc"}
	bound := factSource{"keys": "abc", "keys.count": 3.0}
	leaves := []predicate{
		equals("keys.count", 3),
		{kind: "defined", path: "keys.count"},
		{kind: "truthy", path: "keys.count"},
		{kind: "any_of", path: "keys.count", values: []Node{3.0, 4.0}},
		{kind: "none_of", path: "keys.count", values: []Node{1.0, 2.0}},
	}
	for _, leaf := range leaves {
		if leaf.evaluate(bare) != leaf.evaluate(bound) || leaf.evaluate(bare) == truthUnknown {
			t.Errorf("Go %v: over text %v, over the bound count %v", leaf.kind, leaf.evaluate(bare), leaf.evaluate(bound))
		}
	}
	body := equals("k", 1)
	quantified := predicate{kind: "forall", over: "keys", bind: "k", body: &body}
	if got := quantified.evaluate(factSource{"keys": ""}); got != truthUnknown {
		t.Errorf("Go forall over a text: got %v", got)
	}
}
"#;

#[test]
fn the_go_runtime_reads_every_text_length_the_corpus_states() {
    let root = directory("go-text-length");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module textlengthcorpus\n\ngo 1.24\n").unwrap();
    std::fs::write(package.join("vectors.json"), CORPUS).unwrap();
    std::fs::write(package.join("text_length_test.go"), GO_TEXT_LENGTH_TEST).unwrap();

    let output = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestTextLengthCorpus$",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("go.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("--- PASS: TestTextLengthCorpus"),
        "the Go case ran rather than selecting nothing: {record}"
    );
}

/// The corpus number a `numbers` vector builds, the way `ess-primitives/tests/primitive_corpus.rs`
/// builds it, and the spelling it was authored in.
fn corpus_number(from: &Value) -> (Number, String) {
    if let Some(value) = from.get("integer").and_then(Value::as_i64) {
        return (Number::from(value), value.to_string());
    }
    if let Some(text) = from.get("decimal").and_then(Value::as_str) {
        let number = FactValue::parse_literal(text)
            .as_number()
            .unwrap_or_else(|| panic!("{text} is a numeric literal"));
        return (number, text.to_owned());
    }
    let value = from["binary64"].as_f64().expect("a corpus binary64");
    (
        Number::new(value).expect("a finite binary64"),
        format!("{value}"),
    )
}

/// The same number as a decimal literal, where Rust reads that literal as this number.
fn literal_is(text: &str, number: Number) -> bool {
    FactValue::parse_literal(text).as_number() == Some(number)
}

/// Every Go type an implementation might return this number in, where the value that type carries
/// is the number Rust decides — each decided by Rust, not by the Go runtime under test.
///
/// `json.Number` is read as the token it spells, `float64` as its own value, and `float32` as its
/// shortest round-tripping decimal — the value the implementation wrote down. The integer types
/// carry every integer that fits them.
fn go_carriers(number: Number, authored: &str) -> Vec<Value> {
    let mut carriers = vec![json!({"type": "json.Number", "text": number.exact_text()})];
    if authored != number.exact_text() && literal_is(authored, number) {
        carriers.push(json!({"type": "json.Number", "text": authored}));
    }
    let binary = number.get();
    if Number::new(binary).ok() == Some(number) {
        carriers.push(json!({"type": "float64", "text": format!("{binary}")}));
    }
    #[allow(clippy::cast_possible_truncation)]
    let single = binary as f32;
    if single.is_finite() && literal_is(&format!("{single}"), number) {
        carriers.push(json!({"type": "float32", "text": format!("{single}")}));
    }
    if let Some(integer) = number.as_i64() {
        let text = integer.to_string();
        let mut fits = vec!["int64", "int"];
        if i32::try_from(integer).is_ok() {
            fits.push("int32");
        }
        if i16::try_from(integer).is_ok() {
            fits.push("int16");
        }
        if i8::try_from(integer).is_ok() {
            fits.push("int8");
        }
        if u64::try_from(integer).is_ok() {
            fits.extend(["uint64", "uint"]);
        }
        if u32::try_from(integer).is_ok() {
            fits.push("uint32");
        }
        if u16::try_from(integer).is_ok() {
            fits.push("uint16");
        }
        if u8::try_from(integer).is_ok() {
            fits.push("uint8");
        }
        carriers.extend(
            fits.into_iter()
                .map(|kind| json!({"type": kind, "text": text})),
        );
    }
    carriers
}

/// The table the Go case reads: every corpus number in its carriers, and every ordered pair of
/// corpus numbers with the ordering `Number::cmp` gives it.
fn carrier_table() -> Value {
    #[derive(serde::Deserialize)]
    struct Numbers {
        numbers: Vec<NumberVector>,
    }
    #[derive(serde::Deserialize)]
    struct NumberVector {
        name: String,
        from: Value,
    }
    let vectors = serde_json::from_str::<Numbers>(CORPUS)
        .expect("the corpus is readable")
        .numbers;
    let mut numbers: Vec<(String, Number, String)> = vectors
        .into_iter()
        .map(|vector| {
            let (number, authored) = corpus_number(&vector.from);
            (vector.name, number, authored)
        })
        .collect();
    // Beyond the corpus, and still decided by Rust: the edges of the rule a Go runtime has to draw
    // itself — a binary64 above 2^63 is its shortest decimal, an integer token is exact while its
    // digits fit an i128 and the binary64 once they do not, and `.0` keeps an integer exact.
    for (name, from) in [
        ("ten-to-23-as-binary64", json!({"binary64": 1e23})),
        ("ten-to-23", json!({"decimal": "100000000000000000000000"})),
        ("ten-to-23-with-an-exponent", json!({"decimal": "1e23"})),
        (
            "i128-max",
            json!({"decimal": "170141183460469231731687303715884105727"}),
        ),
        (
            "two-to-127-overflows-i128-digits",
            json!({"decimal": "170141183460469231731687303715884105728"}),
        ),
        (
            "minus-two-to-127-overflows-i128-digits",
            json!({"decimal": "-170141183460469231731687303715884105728"}),
        ),
        (
            "two-to-53-plus-one-with-a-point",
            json!({"decimal": "9007199254740993.0"}),
        ),
        ("a-small-decimal", json!({"decimal": "1e-7"})),
    ] {
        let (number, authored) = corpus_number(&from);
        numbers.push((name.to_owned(), number, authored));
    }
    let mut pairs = Vec::new();
    for (left, a, _) in &numbers {
        for (right, b, _) in &numbers {
            let ordering = match a.cmp(b) {
                std::cmp::Ordering::Less => -1,
                std::cmp::Ordering::Equal => 0,
                std::cmp::Ordering::Greater => 1,
            };
            pairs.push(json!({"left": left, "right": right, "ordering": ordering}));
        }
    }
    json!({
        "numbers": numbers.iter().map(|(name, number, authored)| json!({
            "name": name,
            "display": number.exact_text(),
            "integer": number.as_i64().is_some(),
            "carriers": go_carriers(*number, authored),
        })).collect::<Vec<_>>(),
        "pairs": pairs,
    })
}

/// beyond10x/ess#101: a payload the implementation returned as `int64` or `json.Number` never
/// equalled the suite's number, so a conforming implementation failed until it returned `float64`.
/// Every comparison the Go runtime makes must answer by value, as Rust's `Number` does, whatever
/// Go numeric type carries it.
#[test]
fn the_go_runtime_compares_every_number_carrier_by_the_value_rust_decides() {
    let table = carrier_table();
    let numbers = table["numbers"].as_array().expect("numbers");
    let equal_pairs = table["pairs"]
        .as_array()
        .expect("pairs")
        .iter()
        .filter(|pair| pair["ordering"] == json!(0))
        .count();
    assert!(
        equal_pairs > numbers.len(),
        "the corpus holds numbers that are one value under two spellings"
    );
    let types: std::collections::BTreeSet<&str> = numbers
        .iter()
        .flat_map(|number| number["carriers"].as_array().expect("carriers"))
        .map(|carrier| carrier["type"].as_str().expect("a type"))
        .collect();
    assert_eq!(
        types.len(),
        13,
        "every Go numeric carrier is asked: {types:?}"
    );

    let root = directory("go-numbers");
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module numbercarriers\n\ngo 1.24\n").unwrap();
    std::fs::write(
        package.join("number-carriers.json"),
        serde_json::to_string_pretty(&table).unwrap(),
    )
    .unwrap();
    std::fs::write(
        package.join("number_carriers_test.go"),
        include_str!("fixtures/number-carriers.go"),
    )
    .unwrap();

    let output = Command::new("go")
        .args([
            "test",
            "-count=1",
            "-v",
            "./...",
            "-run",
            "^TestEveryNumberCarrier|^TestSuiteExpectationsMatchWhateverTypeCarriesTheNumber$",
        ])
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("go.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
    let stdout = String::from_utf8_lossy(&output.stdout);
    for case in [
        "TestEveryNumberCarrierComparesByTheValueRustDecides",
        "TestEveryNumberCarrierIsAdmittedAsTheKindItHolds",
        "TestSuiteExpectationsMatchWhateverTypeCarriesTheNumber",
    ] {
        assert!(
            stdout.contains(&format!("--- PASS: {case}")),
            "{case} ran rather than selecting nothing: {record}"
        );
    }
}

/// The harness that asks the browser adapter's own export about every vector.
const JS_HARNESS: &str = r"import {readFileSync} from 'node:fs'
import {primitiveAdmits} from './admission.js'
const corpus = JSON.parse(readFileSync(new URL('./vectors.json', import.meta.url), 'utf8'))
const failures = []
if (corpus.admission.length === 0) failures.push('the corpus selected no vector')
for (const vector of corpus.admission) {
  const admitted = primitiveAdmits(vector.kind, vector.value)
  if (admitted !== vector.admitted) {
    failures.push(`${vector.name}: primitiveAdmits(${vector.kind}, ${JSON.stringify(vector.value)}) = ${admitted}, corpus says ${vector.admitted}`)
  }
}
if (failures.length > 0) { console.error(failures.join('\n')); process.exit(1) }
console.log(`${corpus.admission.length} vectors agree`)
";

#[test]
fn the_browser_adapter_answers_every_vector_the_corpus_states() {
    let root = directory("js");
    std::fs::write(root.join("admission.js"), ADMISSION_JS).unwrap();
    std::fs::write(root.join("vectors.json"), CORPUS).unwrap();
    std::fs::write(root.join("harness.mjs"), JS_HARNESS).unwrap();

    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(&node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("the required Node toolchain executes; set ESS_NODE to name it");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("node.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
}

/// A payload value that its own declared shape refuses is refused at replay admission, in both
/// languages, or the browser and the runner disagree about one document.
#[test]
fn a_payload_that_contradicts_its_shape_is_refused_by_both_admitters() {
    let step = |value: Value| {
        json!([{
            "step": "expect_event", "event": "example.Created",
            "payload": {"id": value},
            "shape": {"id": {"holds": "primitive", "kind": "uuid"}}
        }])
    };
    let suite = |steps: Value| {
        json!({
            "provenance": {
                "suite_version": "ess-conformance/5", "system": "example",
                "specification_version": "v1",
                "spec_digest": "a".repeat(64), "contract_digest": "a".repeat(64)
            },
            "scenarios": {
                "example.domain/authored/created": {"purpose": "Known candidate", "steps": steps, "source": []}
            },
            "coverage": {
                "selection": {"scope": {"kind": "system"}, "origins": "authored", "filter": {"kind": "all"}},
                "knowledge": "complete_inventory", "generated": [],
                "authored": ["example.domain/authored/created"], "outside": [], "refused": [],
                "authored_sources": {"created.yaml": {
                    "digest": format!("sha256:{}", "b".repeat(64)),
                    "scenario": "example.domain/authored/created", "disposition": "accepted"
                }},
                "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 0}
            }
        })
    };

    let good =
        serde_json::to_string_pretty(&suite(step(json!("0f8fad5b-d9cb-469f-a165-70867728950e"))))
            .unwrap();
    let bad = serde_json::to_string_pretty(&suite(step(json!("x")))).unwrap();

    assert!(
        AdmittedSuite::from_json(&good).is_ok(),
        "a canonical uuid payload is still admitted"
    );
    assert!(
        AdmittedSuite::from_json(&bad).is_err(),
        "a payload the shape refuses is refused at admission"
    );

    let root = directory("payload");
    std::fs::write(root.join("admission.js"), ADMISSION_JS).unwrap();
    std::fs::write(root.join("good.json"), &good).unwrap();
    std::fs::write(root.join("bad.json"), &bad).unwrap();
    std::fs::write(
        root.join("harness.mjs"),
        r"import {readFileSync} from 'node:fs'
import {admitSuite} from './admission.js'
const answer = async name => { try { await admitSuite(readFileSync(name, 'utf8')); return true } catch (error) { return String(error) } }
console.log(JSON.stringify({good: await answer('good.json'), bad: await answer('bad.json')}))
",
    )
    .unwrap();
    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(&node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("the required Node toolchain executes; set ESS_NODE to name it");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    std::fs::write(root.join("node.log"), &record).unwrap();
    assert!(output.status.success(), "{record}");
    let answer: Value = serde_json::from_slice(&output.stdout).expect("the harness prints JSON");
    assert_eq!(answer["good"], json!(true), "{record}");
    assert_ne!(answer["bad"], json!(true), "{record}");
}

/// Kept beside the two toolchain lanes: a corpus nothing reads is a corpus nothing checks.
#[test]
fn the_corpus_is_the_one_document_all_three_readers_open() {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../specify/ess-primitives/tests/vectors/primitive-semantics.json");
    assert_eq!(
        std::fs::read_to_string(&path).expect("the corpus is at the path the design page names"),
        CORPUS,
        "the compiled-in corpus is the file on disk"
    );
    assert!(vectors().len() >= 30, "the corpus is not a token corpus");
}
