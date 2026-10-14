//! `trimmed:` (beyond10x/ess#532, ess/25, `docs/design/value-expressions.md` § E10) in conformance:
//! synthesis witnesses both sides of a trimmed guard so that a target that does not trim, trims too
//! little, or trims Unicode spaces fails a scenario, and the Rust, Go and TypeScript evaluators
//! answer the shared `trimmed-text.json` table alike. A guard is decided at synthesis, so no suite
//! carries the construct, and every runner's admission refuses a suite that does.
use super::support_scratch;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::{
    synthesize, AdmittedSuite, ConformanceScenario, ConformanceSuite, ScenarioStep, ScenarioValue,
};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{trim_ascii_whitespace, Predicate, Truth};
use serde_json::{json, Value};

const VECTORS: &str =
    include_str!("../../../../specify/ess-primitives/tests/vectors/trimmed-text.json");

const LITERALS: [&str; 3] = ["NEW", "HOLD", "DONE"];

/// The issue's command, in both polarities: `accepting` takes the trimmed, folded membership as
/// the guard of its accepting branch; otherwise the refusal is `not:` of it, as filed.
fn model(accepting: bool) -> String {
    let membership = "{reason: {trimmed: {in_ignore_case: [NEW, HOLD, DONE]}}}";
    let outcomes = if accepting {
        format!(
            "      - name: closed
        when: {membership}
        emits: [promo.codes.Closed]
        payload:
          promo.codes.Closed:
            reason: input.reason
      - name: unknown-reason
        error: promo.codes.UnknownReason
"
        )
    } else {
        format!(
            "      - name: unknown-reason
        when: {{not: {membership}}}
        error: promo.codes.UnknownReason
      - name: closed
        emits: [promo.codes.Closed]
        payload:
          promo.codes.Closed:
            reason: input.reason
"
        )
    };
    format!(
        "format: ess/25
system: promo
version: v1
domain: promo.codes
errors:
  - name: promo.codes.UnknownReason
    summary: The reason is not a known reason.
    fields: []
events:
  - name: promo.codes.Closed
    fields:
      - {{name: reason, type: String}}
commands:
  - name: promo.codes.Close
    input:
      - {{name: reason, type: String}}
    outcomes:
{outcomes}"
    )
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let specification = Specification::assemble([(Source::new("promo.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model validates:\n{errors}\n{text}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the model resolves:\n{diagnostics}"))
}

fn scenario<'a>(suite: &'a ConformanceSuite, id: &str) -> &'a ConformanceScenario {
    let found = suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id);
    let Some((_, scenario)) = found else {
        panic!(
            "no scenario {id} in {:?}",
            suite
                .scenarios
                .keys()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        )
    };
    scenario
}

/// Every `reason` the scenario's `Close` steps send, in step order.
fn reasons(scenario: &ConformanceScenario) -> Vec<String> {
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.to_string() == "promo.codes.Close" =>
            {
                Some(input.clone())
            }
            _ => None,
        })
        .map(|input: BTreeMap<String, ScenarioValue>| {
            match input.get("reason").and_then(ScenarioValue::as_literal) {
                Some(Node::Text(text)) => text.clone(),
                other => panic!("not a literal text: {other:?}"),
            }
        })
        .collect()
}

/// Whether the implementation's rule — trim ASCII whitespace, then compare without ASCII case —
/// accepts `reason`.
fn accepted(reason: &str) -> bool {
    let trimmed = trim_ascii_whitespace(reason);
    LITERALS
        .iter()
        .any(|literal| trimmed.eq_ignore_ascii_case(literal))
}

#[test]
fn synthesis_witnesses_trimmed_case_folded_membership() {
    for accepting in [true, false] {
        let synthesis = synthesize(&compiled(&model(accepting)));
        assert!(
            synthesis.refusals.is_empty(),
            "{:?}",
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
        );
        let suite = synthesis.suite;
        let closed = reasons(scenario(&suite, "promo.codes.Close/outcome/closed"));
        let refused = reasons(scenario(&suite, "promo.codes.Close/outcome/unknown-reason"));
        assert!(
            !closed.is_empty() && !refused.is_empty(),
            "{closed:?} {refused:?}"
        );
        for reason in &closed {
            assert!(accepted(reason), "{accepting}: closed is sent {reason:?}");
        }
        for reason in &refused {
            assert!(
                !accepted(reason),
                "{accepting}: the refusal is sent {reason:?}"
            );
        }
        // The accepting side: a literal padded with ASCII spaces and a tab, in another ASCII case,
        // which a target that does not trim, or does not fold, refuses.
        assert!(
            closed.iter().any(|reason| {
                let trimmed = trim_ascii_whitespace(reason);
                reason.starts_with(" \t") && reason.ends_with(' ') && !LITERALS.contains(&trimmed)
            }),
            "{accepting}: no padded, case-changed witness in {closed:?}"
        );
        // And the literal between VT FF and CR LF, which a trim of spaces and tabs alone, or
        // Rust's `trim_ascii`, leaves.
        assert!(
            closed
                .iter()
                .any(|reason| reason.starts_with("\u{b}\u{c}") && reason.ends_with("\r\n")),
            "{accepting}: no VT FF / CR LF witness in {closed:?}"
        );
        // The refusing side: the literal with an inner space, and the literal between two U+00A0,
        // which a target trimming Unicode spaces accepts and so fails.
        for literal in LITERALS {
            let (first, rest) = literal.split_at(1);
            for expected in [format!("{first} {rest}"), format!("\u{a0}{literal}\u{a0}")] {
                assert!(
                    refused.contains(&expected),
                    "{accepting}: the refusal is not sent {expected:?}: {refused:?}"
                );
            }
        }
        // A guard is decided at synthesis: the suite carries no `trimmed`, keeps its format, and
        // admits.
        let canonical = suite.to_canonical_json().expect("the suite serialises");
        assert!(!canonical.contains("trimmed"), "{canonical}");
        AdmittedSuite::from_json(&canonical).expect("the synthesized suite admits");
    }
}

fn suite(version: &str, predicate: &Value) -> Value {
    let mut suite = json!({
        "provenance": {"suite_version": version, "system": "demo", "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"demo.orders/authored/rows": {"purpose": "Check a trimmed filter over rows",
            "steps": [{"step": "expect_view", "view": "demo.orders.Rows",
                "expectation": {"expect": "satisfies", "predicate": predicate}}],
            "source": []}}
    });
    // From suite/34 a suite states that every scenario starts empty; the refusal under test must
    // be the `trimmed` one, not this structural one.
    let major: u32 = version
        .rsplit('/')
        .next()
        .and_then(|major| major.parse().ok())
        .expect("a suite version ends in its major");
    if major >= 34 {
        suite["provenance"]["scenario_initial_state"] = json!("empty");
    }
    suite
}

const REFUSED: &str =
    "`trimmed` text operators are decided at synthesis, and no suite format carries one";

#[derive(serde::Deserialize)]
struct Corpus {
    trimmed: Vec<Vector>,
}

#[derive(serde::Deserialize)]
struct Vector {
    name: String,
    #[serde(default)]
    value: Option<Value>,
    operator: Value,
    truth: String,
}

/// The Go runtime's admission refusing a suite carrying `trimmed`, and its evaluator over every
/// vector of the table.
const GO_TEST: &str = r#"package essconform

import (
	"encoding/json"
	"os"
	"strings"
	"testing"
)

func TestTrimmedSuiteRefused(t *testing.T) {
	raw, err := os.ReadFile("trimmed48.json")
	if err != nil {
		t.Fatal(err)
	}
	_, err = admitSuite(string(raw))
	if err == nil || !strings.Contains(err.Error(), "no suite format carries one") {
		t.Errorf("suite/48 carrying trimmed: got %v", err)
	}
}

func TestTrimmedCorpus(t *testing.T) {
	raw, err := os.ReadFile("trimmed-text.json")
	if err != nil {
		t.Fatal(err)
	}
	var corpus struct {
		Trimmed []map[string]json.RawMessage `json:"trimmed"`
	}
	if err := json.Unmarshal(raw, &corpus); err != nil {
		t.Fatal(err)
	}
	if len(corpus.Trimmed) < 25 {
		t.Fatalf("the corpus selected %d trimmed vectors", len(corpus.Trimmed))
	}
	for _, vector := range corpus.Trimmed {
		var name, expected string
		var operator any
		for key, into := range map[string]any{"name": &name, "operator": &operator, "truth": &expected} {
			if err := json.Unmarshal(vector[key], into); err != nil {
				t.Fatalf("%s: %v", key, err)
			}
		}
		node := map[string]any{"source": map[string]any{"trimmed": operator}}
		leaf, err := fromNode(node)
		if err != nil {
			t.Fatalf("%s: %v", name, err)
		}
		if !strings.HasPrefix(leaf.String(), "trimmed(source ") {
			t.Errorf("%s renders as %s", name, leaf.String())
		}
		source := factSource{}
		if value, ok := vector["value"]; ok {
			var decoded any
			if err := json.Unmarshal(value, &decoded); err != nil {
				t.Fatal(err)
			}
			source["source"] = decoded
		}
		before := source["source"]
		want := map[string]truth{"true": truthTrue, "false": truthFalse, "unknown": truthUnknown}[expected]
		if got := leaf.evaluate(source); got != want {
			t.Errorf("Go %s: %v over %v: got %v, corpus says %s", name, node, source, got, expected)
		}
		if got := (predicate{kind: "not", body: &leaf}).evaluate(source); got != want.not() {
			t.Errorf("Go %s: not %v: got %v", name, node, got)
		}
		if source["source"] != before {
			t.Errorf("Go %s: the source was changed", name)
		}
	}
}
"#;

fn repository_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("the workspace root exists")
}

/// Whether `name` runs; Go spells its version query `go version`, every other tool `--version`.
fn tool(name: &str) -> bool {
    Command::new(name)
        .arg(if name == "go" { "version" } else { "--version" })
        .output()
        .is_ok_and(|output| output.status.success())
}

#[test]
fn trimmed_guard_runs_alike_on_rust_go_and_typescript() {
    let carried = json!({"reason": {"trimmed": {"in_ignore_case": ["NEW", "DONE"]}}});
    rust_answers_the_table(&carried);
    go_answers_the_table(&carried);
    typescript_answers_the_table();
}

/// Rust: the interpreted target's evaluator over the table, and every Rust admission refusing a
/// suite that carries `carried`.
fn rust_answers_the_table(carried: &Value) {
    let corpus: Corpus = serde_json::from_str(VECTORS).expect("the table reads");
    assert!(corpus.trimmed.len() >= 25);
    let path = FactPath::new("source").unwrap();
    for vector in &corpus.trimmed {
        let mut facts = FactStore::new();
        match &vector.value {
            None => {}
            Some(Value::String(text)) => facts.set(path.clone(), FactValue::text(text)),
            Some(Value::Bool(value)) => facts.set(path.clone(), FactValue::Bool(*value)),
            Some(Value::Number(number)) => facts.set(
                path.clone(),
                FactValue::Number(Number::new(number.as_f64().unwrap()).unwrap()),
            ),
            Some(other) => panic!("{}: {other}", vector.name),
        }
        let node: Node =
            serde_json::from_value(json!({"source": {"trimmed": vector.operator}})).unwrap();
        let want = match vector.truth.as_str() {
            "true" => Truth::True,
            "false" => Truth::False,
            _ => Truth::Unknown,
        };
        assert_eq!(
            Predicate::from_node(&node).unwrap().evaluate(&facts),
            want,
            "Rust {}",
            vector.name
        );
    }

    // Every Rust admission refuses a suite carrying the construct, at every major.
    for version in [
        "ess-conformance/20",
        "ess-conformance/40",
        "ess-conformance/48",
    ] {
        let error = AdmittedSuite::from_json(&suite(version, carried).to_string())
            .err()
            .unwrap_or_else(|| panic!("{version} admits a suite carrying trimmed"));
        assert!(error.to_string().contains(REFUSED), "{version}: {error}");
    }
}

/// Go: the emitted runtime package over the same table, and its admission refusing a suite that
/// carries `carried`.
fn go_answers_the_table(carried: &Value) {
    assert!(tool("go"), "the Go toolchain this lane requires is on PATH");
    let root = support_scratch::Scratch::new(format!("ess-trimmed-go-{}", std::process::id()));
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    let emitted = synthesize(&compiled(&model(true))).suite;
    for artifact in ess_conformance::go::emit(&emitted).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module trimmed\n\ngo 1.24\n").unwrap();
    std::fs::write(
        package.join("trimmed48.json"),
        suite("ess-conformance/48", carried).to_string(),
    )
    .unwrap();
    std::fs::write(package.join("trimmed-text.json"), VECTORS).unwrap();
    std::fs::write(package.join("trimmed_test.go"), GO_TEST).unwrap();
    let output = Command::new("go")
        .args(["test", "-count=1", "-v", "./...", "-run", "^TestTrimmed"])
        .current_dir(&*root)
        .env("GOWORK", "off")
        .output()
        .expect("go test runs");
    let record = format!(
        "exit: {:?}\nstdout:\n{}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(output.status.success(), "{record}");
    for case in ["TestTrimmedSuiteRefused", "TestTrimmedCorpus"] {
        assert!(
            String::from_utf8_lossy(&output.stdout).contains(&format!("--- PASS: {case}")),
            "the Go case {case} ran rather than selecting nothing: {record}"
        );
    }
}

/// TypeScript: the runtime's own case over the same table, compiled from the sources.
fn typescript_answers_the_table() {
    if !tool("tsc") || !tool("node") {
        println!("skipped: no `tsc` or `node` on PATH, so the TypeScript runtime was not run");
        return;
    }
    let sources = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/ts");
    let out = repository_root()
        .join("target/trimmed-typescript")
        .join(std::process::id().to_string());
    let _ = std::fs::remove_dir_all(&out);
    std::fs::create_dir_all(&out).unwrap();
    let compiled = Command::new("tsc")
        .arg("--project")
        .arg(sources.join("tsconfig.json"))
        .arg("--noCheck")
        .arg("--outDir")
        .arg(&out)
        .output()
        .expect("tsc runs");
    let printed = String::from_utf8_lossy(&compiled.stdout).into_owned();
    for line in printed.lines().filter(|line| line.contains("error TS")) {
        assert!(
            line.contains("TS2688"),
            "tsc refused the sources:\n{printed}"
        );
    }
    std::fs::write(out.join("package.json"), "{\"type\":\"module\"}\n").unwrap();
    let run = Command::new("node")
        .arg("--test")
        .arg("--test-name-pattern")
        .arg("trimmed vector")
        .arg(out.join("predicate.test.js"))
        .current_dir(repository_root())
        .env(
            "ESS_PRIMITIVE_VECTORS",
            repository_root()
                .join("crates/specify/ess-primitives/tests/vectors/primitive-semantics.json"),
        )
        .output()
        .expect("node --test runs");
    let printed = format!(
        "{}{}",
        String::from_utf8_lossy(&run.stdout),
        String::from_utf8_lossy(&run.stderr)
    );
    let _ = std::fs::remove_dir_all(&out);
    assert!(run.status.success(), "{printed}");
    assert!(
        printed.lines().any(|line| line.trim() == "# pass 1")
            && printed.lines().any(|line| line.trim() == "# fail 0"),
        "the TypeScript trimmed case ran exactly once and passed:\n{printed}"
    );
}
