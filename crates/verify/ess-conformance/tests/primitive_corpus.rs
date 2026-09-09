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
