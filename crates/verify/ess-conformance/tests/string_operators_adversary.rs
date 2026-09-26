//! Adversarial cases for `starts_with`, `ends_with` and `contains` (beyond10x/ess#95), pass 1.
//!
//! Each case states what the binding design (`docs/design/string-predicate-operators.md`) or the
//! Rust lane answers, and asks another lane or another path for the same answer.

use std::path::PathBuf;
use std::process::Command;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_conformance::{ScenarioStep, ViewExpectation};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use serde_json::json;

fn scratch(label: &str) -> PathBuf {
    let path =
        std::env::temp_dir().join(format!("ess-b1b-adversary-{label}-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("a working directory");
    path
}

fn compiled(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the fixture parses");
    let specification = Specification::assemble([(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the fixture validates:\n{errors}"));
    compile(&specification, &SourceMap::new())
        .unwrap_or_else(|diagnostics| panic!("the fixture resolves:\n{diagnostics}"))
}

/// The smallest coverage suite the Go emitter accepts, so the runtime compiles beside it.
fn minimal_suite() -> ess_conformance::AdmittedSuite {
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
    ess_conformance::AdmittedSuite::from_json(&serde_json::to_string_pretty(&document).unwrap())
        .expect("the minimal suite is admitted")
}

/// Runs one Go test function inside the emitted runtime package and returns its record.
fn go_test(label: &str, source: &str, name: &str) -> (bool, String) {
    let root = scratch(label);
    let package = root.join("essconform");
    std::fs::create_dir_all(&package).unwrap();
    for artifact in ess_conformance::go::emit(minimal_suite().suite()).expect("the suite emits") {
        std::fs::write(root.join(&artifact.path), artifact.contents).unwrap();
    }
    std::fs::write(root.join("go.mod"), "module adversary\n\ngo 1.24\n").unwrap();
    std::fs::write(package.join("adversary_test.go"), source).unwrap();
    let output = Command::new("go")
        .args(["test", "-count=1", "-v", "./...", "-run"])
        .arg(format!("^{name}$"))
        .current_dir(&root)
        .env("GOWORK", "off")
        .output()
        .expect("the required Go toolchain executes");
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    let record = format!(
        "exit: {:?}\nstdout:\n{stdout}\nstderr:\n{}",
        output.status.code(),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        stdout.contains(&format!("--- PASS: {name}"))
            || stdout.contains(&format!("--- FAIL: {name}")),
        "the Go case ran rather than selecting nothing: {record}"
    );
    let _ = std::fs::remove_dir_all(&root);
    (output.status.success(), record)
}

/// Pairs of operands Rust reads as two different string-operator literals.
const DISTINCT: [(&str, &str); 5] = [
    ("+44", "44"),
    (" urgent", "urgent"),
    ("'x'", "x"),
    ("0", "0.0"),
    ("1e2", "100"),
];

const GO_MEANING_TEST: &str = r#"package essconform

import (
	"reflect"
	"testing"
)

func TestTextMatchMeaningKeepsTheOperandVerbatim(t *testing.T) {
	pairs := [][2]string{{"+44", "44"}, {" urgent", "urgent"}, {"'x'", "x"}, {"0", "0.0"}, {"1e2", "100"}}
	for _, op := range []string{"starts_with", "ends_with", "contains"} {
		for _, pair := range pairs {
			left := map[string]any{"caller": map[string]any{op: pair[0]}}
			right := map[string]any{"caller": map[string]any{op: pair[1]}}
			if reflect.DeepEqual(predicateMeaning(left), predicateMeaning(right)) {
				t.Errorf("Go gives %s %q and %s %q one meaning, %v; Rust's lineage check tells them apart", op, pair[0], op, pair[1], predicateMeaning(left))
			}
		}
	}
}
"#;

/// The Go coverage-lineage check compares a child scenario with its parent through
/// `predicateMeaning`, whose string-operator arm reads the operand through `meaningScalar`: it trims
/// spaces, strips one layer of quotes and reads numeric text as a number. Rust compares the typed
/// scenario, where the operand is the text it spells (design, *Syntax*). So a child that swaps
/// `starts_with: "+44"` for `starts_with: "44"` is one scenario to Go and two to Rust.
#[test]
fn go_coverage_lineage_reads_a_string_operand_verbatim_as_rust_does() {
    for (left, right) in DISTINCT {
        for op in ["starts_with", "ends_with", "contains"] {
            let read = |text: &str| {
                Predicate::from_node(
                    &serde_json::from_value::<Node>(json!({"caller": {op: text}})).unwrap(),
                )
                .unwrap()
            };
            assert_ne!(
                read(left),
                read(right),
                "Rust tells {left:?} and {right:?} apart"
            );
        }
    }
    let (passed, record) = go_test(
        "meaning",
        GO_MEANING_TEST,
        "TestTextMatchMeaningKeepsTheOperandVerbatim",
    );
    assert!(passed, "{record}");
}

const GO_CONJOINED_TEST: &str = r#"package essconform

import "testing"

func TestTextMatchConjoinsEveryOperatorOfOneMapping(t *testing.T) {
	node := map[string]any{"caller": map[string]any{"starts_with": "+", "ends_with": "0"}}
	if err := admitPredicateEnvelope(node, 0); err != nil {
		t.Fatalf("admission refuses %v: %v", node, err)
	}
	if err := admitPredicateVersion(node, 14); err != nil {
		t.Fatalf("suite/14 refuses %v: %v", node, err)
	}
	leaf, err := fromNode(node)
	if err != nil {
		t.Fatal(err)
	}
	if got := leaf.evaluate(factSource{"caller": "+441"}); got != truthFalse {
		t.Errorf("Go answers %v for %v over +441; Rust conjoins both operators and answers false", got, node)
	}
}
"#;

/// Rust conjoins every operator of one mapping (`from_constraint`; the page's
/// `sku: {starts_with: "A", ends_with: "0"}` example). The Go reader admits the same mapping and
/// then evaluates only the first string operator its fixed key list finds.
#[test]
fn go_evaluates_every_string_operator_of_one_mapping_as_rust_does() {
    let rust = Predicate::from_node(
        &serde_json::from_value::<Node>(json!({"caller": {"starts_with": "+", "ends_with": "0"}}))
            .unwrap(),
    )
    .unwrap();
    assert!(
        matches!(&rust, Predicate::All(children) if children.len() == 2),
        "{rust:?}"
    );
    let (passed, record) = go_test(
        "conjoined",
        GO_CONJOINED_TEST,
        "TestTextMatchConjoinsEveryOperatorOfOneMapping",
    );
    assert!(passed, "{record}");
}

const PUBLISHED: &str = r#"format: ess/8
system: shop
version: v1
domain: shop.order
types:
  - name: shop.order.Sku
    kind: newtype
    of: String
    invariants:
      - value: {contains: "-"}
entities:
  - name: shop.order.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
    invariants:
      - not: {sku: {contains: " "}}
    lifecycle:
      initial: Open
      states: [Open, Closed]
      terminal: [Closed]
      transitions:
        - {name: close, from: [Open], to: Closed}
errors:
  - name: shop.order.Refused
    summary: The order was not placed.
  - name: shop.order.NotOpen
    summary: The order is not open.
commands:
  - name: shop.order.PlaceOrder
    input:
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
    outcomes:
      - name: placed
        when: {sku: {starts_with: "SKU-"}}
        creates: shop.order.Order
        instance: order_id
        emits: [shop.order.OrderPlaced]
        payload:
          shop.order.OrderPlaced:
            order_id: {generated: true}
      - name: refused
        error: shop.order.Refused
  - name: shop.order.CloseOrder
    input:
      - {name: order_id, type: Uuid}
    outcomes:
      - name: closed
        moves: shop.order.Order.close
        instance: order_id
        emits: [shop.order.OrderClosed]
        payload:
          shop.order.OrderClosed:
            order_id: input.order_id
      - name: wrong-state
        wrong_state: true
        error: shop.order.NotOpen
events:
  - name: shop.order.OrderPlaced
    fields:
      - {name: order_id, type: Uuid}
  - name: shop.order.OrderClosed
    fields:
      - {name: order_id, type: Uuid}
views:
  - name: shop.order.OrderById
    source: shop.order.Order
    consistency: read_your_writes
    fields:
      - {name: order_id, type: Uuid}
      - {name: quantity, type: Integer}
      - {name: sku, type: shop.order.Sku}
      - {name: state, type: shop.order.Order.State}
"#;

fn satisfied(suite: &ess_conformance::ConformanceSuite) -> Vec<String> {
    let mut found = Vec::new();
    for scenario in suite.scenarios.values() {
        for step in &scenario.steps {
            if let ScenarioStep::ExpectView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            }
            | ScenarioStep::EventuallyView {
                expectation: ViewExpectation::Satisfies { predicate },
                ..
            } = step
            {
                found.push(serde_json::to_string(predicate).unwrap());
            }
        }
    }
    found
}

/// An entity invariant and a newtype invariant written with string operators reach the suite as
/// `satisfies` expectations — the newtype's re-rooted from `value` onto the view field — so the
/// synthesized suite takes `/14` and its coverage counterpart `/15`. Nothing else builds a coverage
/// suite through `coverage_build` with the construct in it.
#[test]
fn published_string_invariants_reach_the_suite_rebased_and_take_the_new_pair() {
    let ir = compiled(PUBLISHED);
    let synthesis = ess_conformance::synthesize(&ir);
    let predicates = satisfied(&synthesis.suite);
    assert!(
        predicates
            .iter()
            .any(|p| p.contains(r#"{"sku":{"contains":"-"}}"#)),
        "the newtype invariant is asserted at the field that holds it: {predicates:?} {:?}",
        synthesis
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
    );
    assert!(
        predicates
            .iter()
            .any(|p| p.contains(r#"{"not":{"sku":{"contains":" "}}}"#)),
        "the entity invariant is asserted: {predicates:?}"
    );
    assert_eq!(synthesis.suite.provenance.suite_version.major(), 14);
    let covered = ess_conformance::coverage_build::build(
        &ir,
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .expect("coverage builds");
    assert_eq!(
        covered.selected().suite().provenance.suite_version.major(),
        15
    );
}

fn unguarded(invariant: &str) -> String {
    format!(
        r"format: ess/8
system: routing
version: v1
domain: routing.calls
types:
  - name: routing.calls.PhoneNumber
    kind: newtype
    of: String
    invariants:
      - {invariant}
events:
  - name: routing.calls.Registered
    fields: []
commands:
  - name: routing.calls.Register
    input:
      - {{name: number, type: routing.calls.PhoneNumber}}
    outcomes:
      - name: registered
        emits: [routing.calls.Registered]
"
    )
}

fn register_refusals(model: &str) -> Vec<String> {
    let ir = compiled(model);
    let synthesis = ess_conformance::synthesize(&ir);
    synthesis
        .refusals
        .iter()
        .map(ToString::to_string)
        .filter(|refusal| refusal.contains("routing.calls.Register"))
        .collect()
}

/// The design's own motivating type, `PhoneNumber` with `value: {starts_with: "+"}`, taken by a
/// command that does not guard on it. Rule 3's invariant-composed candidates are built only for a
/// path some guard reads, so the base witness `number` — which the type refuses — is the only value
/// tried, and the one scenario the command has is refused.
#[test]
fn a_string_invariant_on_an_input_no_guard_reads_is_still_witnessed() {
    let refused = register_refusals(&unguarded(r#"value: {starts_with: "+"}"#));
    assert!(refused.is_empty(), "{refused:?}");
}

/// Control for the case above, with an invariant spelled in pre-#95 vocabulary. Red here means the
/// gap is the witness's class, which this unit's construct makes reachable with the design's own
/// type, rather than something the string operators alone introduced.
#[test]
fn control_an_any_of_invariant_on_an_input_no_guard_reads_is_still_witnessed() {
    let refused = register_refusals(&unguarded(r"value: {any_of: [uk, us]}"));
    assert!(refused.is_empty(), "{refused:?}");
}

/// The composed layouts take `P` = the *first* positive `starts_with` literal (design, *Witness*).
/// When a later positive prefix extends it, every layout begins with the shorter one, no single
/// literal carries the suffix too, and a satisfiable guard (`AB0` satisfies it) has no witness.
#[test]
fn two_positive_prefixes_where_the_second_extends_the_first_are_still_witnessed() {
    let model = r#"format: ess/8
system: routing
version: v1
domain: routing.calls
events:
  - name: routing.calls.Routed
    fields: []
errors:
  - name: routing.calls.Refused
    fields: []
commands:
  - name: routing.calls.Register
    input:
      - {name: sku, type: String}
    outcomes:
      - name: matched
        when:
          all:
            - sku: {starts_with: "A"}
            - sku: {starts_with: "AB"}
            - sku: {ends_with: "0"}
        emits: [routing.calls.Routed]
      - name: other
        error: routing.calls.Refused
"#;
    let refused = register_refusals(model);
    assert!(refused.is_empty(), "{refused:?}");
}
