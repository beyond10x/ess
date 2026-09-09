//! Adversarial cases for `story:review-primitive-semantics`, wave 21 pass 1.
//!
//! Each case drives the implementation against a document the unit wrote about itself:
//! `docs/design/review-primitive-semantics.md` and
//! `crates/specify/ess-primitives/tests/vectors/primitive-semantics.json`. The design page says
//! "one grammar, three implementations, one corpus"; these are the vectors the corpus does not
//! carry, on which the three do not agree, plus the round trip the page's `Integer` row promises
//! and the serializer does not deliver.

use std::path::{Path, PathBuf};
use std::process::Command;

use ess_conformance::{AdmittedSuite, Holds};
use ess_domain::Primitive;
use ess_primitives::node::Node;
use serde_json::{json, Value};

const ADMISSION_JS: &str = include_str!("../assets/coverage-admission.js");

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
        other => panic!("{other} is not a primitive"),
    }
}

/// A scratch root inside the coordinator's assigned directory, never the shared temp dir.
fn directory(label: &str) -> PathBuf {
    let path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../../target/review-boundaries-21/scratch")
        .join(format!("divergence-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a working directory");
    path
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

func TestExtraCorpus(t *testing.T) {
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
			t.Errorf("Go %s: primitive(%q, %v) admitted=%v, the design page says %v",
				vector.Name, vector.Kind, vector.Value, admitted, vector.Admitted)
		}
	}
}
"#;

const JS_HARNESS: &str = r"import {readFileSync} from 'node:fs'
import {primitiveAdmits} from './admission.js'
const corpus = JSON.parse(readFileSync(new URL('./vectors.json', import.meta.url), 'utf8'))
const failures = []
for (const vector of corpus.admission) {
  const admitted = primitiveAdmits(vector.kind, vector.value)
  if (admitted !== vector.admitted) {
    failures.push(`${vector.name}: primitiveAdmits(${vector.kind}, ${JSON.stringify(vector.value)}) = ${admitted}, the design page says ${vector.admitted}`)
  }
}
if (failures.length > 0) { console.error(failures.join('\n')); process.exit(1) }
console.log(`${corpus.admission.length} vectors agree`)
";

/// Asks the Rust admitter, the Go runtime and the browser adapter about one vector list.
///
/// Returns one line per lane that answered something other than the vector's `admitted`.
fn three_lanes(label: &str, vectors: &Value) -> Vec<String> {
    let mut divergences = Vec::new();
    let corpus = json!({ "admission": vectors.clone() });
    let corpus_text = serde_json::to_string_pretty(&corpus).unwrap();

    for vector in corpus["admission"].as_array().unwrap() {
        let node: Node =
            serde_json::from_value(vector["value"].clone()).expect("a vector is a node");
        let holds = Holds::Primitive {
            kind: primitive_named(vector["kind"].as_str().unwrap()),
        };
        let admitted = holds.admits(&node);
        if admitted != vector["admitted"].as_bool().unwrap() {
            divergences.push(format!(
                "Rust {}: primitive_value({}, {}) admitted={}, the design page says {}",
                vector["name"], vector["kind"], vector["value"], admitted, vector["admitted"]
            ));
        }
    }

    let root = directory(&format!("{label}-go"));
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(
        root.join("go.mod"),
        "module primitivedivergence\n\ngo 1.24\n",
    )
    .unwrap();
    std::fs::write(package.join("vectors.json"), &corpus_text).unwrap();
    std::fs::write(package.join("corpus_test.go"), GO_CORPUS_TEST).unwrap();
    let go = Command::new("go")
        .args(["test", "-count=1", "./...", "-run", "^TestExtraCorpus$"])
        .current_dir(&root)
        .output()
        .expect("the required Go toolchain executes");
    if !go.status.success() {
        divergences.push(
            String::from_utf8_lossy(&go.stdout)
                .lines()
                .filter(|line| line.contains("Go "))
                .map(str::trim)
                .collect::<Vec<_>>()
                .join("\n"),
        );
    }

    let root = directory(&format!("{label}-js"));
    std::fs::write(root.join("admission.js"), ADMISSION_JS).unwrap();
    std::fs::write(root.join("vectors.json"), &corpus_text).unwrap();
    std::fs::write(root.join("harness.mjs"), JS_HARNESS).unwrap();
    let node_bin = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let js = Command::new(&node_bin)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("the required Node toolchain executes; set ESS_NODE to name it");
    if !js.status.success() {
        divergences.push(String::from_utf8_lossy(&js.stderr).trim().to_owned());
    }
    divergences
}

/// A `Bytes` spelling the Go runtime admits and the other two admitters refuse.
///
/// `paddedBase64` in `src/go/runtime.go` admits `=` anywhere in the final two positions rather
/// than only as a suffix, so `"AA=A"` — padding followed by a data character — passes there and
/// fails `ess_primitives::facts::is_padded_base64`, `BASE64_PATTERN` and the browser adapter's
/// regular expression. The design page says the three are one grammar; the corpus carries no
/// vector that puts `=` anywhere but last, so nothing notices.
#[test]
fn the_three_admitters_agree_about_padding_that_is_not_a_suffix() {
    let divergences = three_lanes(
        "bytes",
        &json!([
            {"name": "bytes-padding-followed-by-data", "kind": "bytes", "value": "AA=A", "admitted": false},
            {"name": "bytes-padding-followed-by-data-long", "kind": "bytes", "value": "AAAAAA=A", "admitted": false},
            {"name": "bytes-two-pads-then-data", "kind": "bytes", "value": "A==A", "admitted": false}
        ]),
    );
    assert!(
        divergences.is_empty(),
        "one grammar, three implementations:\n{}",
        divergences.join("\n")
    );
}

/// The top of the range the design page declares `Primitive::Integer` admits.
///
/// `docs/design/review-primitive-semantics.md` gives `Integer` the abstract value "an exact
/// integer in `[i64::MIN, i64::MAX]`". Rust now admits `i64::MAX`; the Go runtime's `integral`
/// and the browser adapter's `primitiveAdmits` both compare a float64 against `2^63` and refuse
/// it, along with every integer above `9223372036854775296`.
#[test]
fn the_three_admitters_agree_about_the_top_of_the_declared_integer_range() {
    let divergences = three_lanes(
        "integer",
        &json!([
            {"name": "integer-i64-max", "kind": "integer", "value": 9_223_372_036_854_775_807_i64, "admitted": true},
            {"name": "integer-i64-min", "kind": "integer", "value": -9_223_372_036_854_775_808_i64, "admitted": true}
        ]),
    );
    assert!(
        divergences.is_empty(),
        "one grammar, three implementations:\n{}",
        divergences.join("\n")
    );
}

/// A `Node` is the same node after its own serializer has written it and its own reader read it.
///
/// The design page's `Integer` row promises an exact integer in `[i64::MIN, i64::MAX]`, and
/// `node.rs::from_value` was changed in this unit precisely so an integer token arrives exact.
/// `Serialize` then writes the binary64, so the value that comes back is a different `Number` —
/// unequal to the one written, by the same `PartialEq` this unit rewrote.
#[test]
fn a_node_number_is_the_number_it_was_after_a_round_trip_through_its_own_serializer() {
    for text in [
        "9007199254740993",
        "9223372036854775807",
        "-9223372036854775808",
    ] {
        let node: Node = serde_json::from_str(text).expect("an integer token is a node");
        let written = serde_json::to_string(&node).expect("a node serialises");
        let read: Node = serde_json::from_str(&written).expect("what was written is readable");
        assert_eq!(
            read, node,
            "{text} was written as {written} and came back a different number"
        );
    }
}

/// An `Integer` payload admitted once is admitted again after the suite has been written.
///
/// A suite is emitted, persisted and re-read; the same document must be admitted the same way
/// both times. `i64::MAX` is admitted as an `Integer` on the way in and refused on the way back,
/// because the serializer writes the binary64 and the reader then sees a value outside `i64`.
#[test]
fn an_integer_payload_is_admitted_the_same_way_before_and_after_it_is_written() {
    let holds = Holds::Primitive {
        kind: Primitive::Integer,
    };
    let node: Node = serde_json::from_str("9223372036854775807").expect("an integer token");
    assert!(holds.admits(&node), "i64::MAX is an Integer on the way in");
    let written = serde_json::to_string(&node).expect("a node serialises");
    let read: Node = serde_json::from_str(&written).expect("what was written is readable");
    assert!(
        holds.admits(&read),
        "i64::MAX was written as {written} and is no longer an Integer when read back"
    );
}
