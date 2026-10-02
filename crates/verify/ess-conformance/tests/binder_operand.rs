//! A quantifier binder on the right of a comparison means the binder in every runner
//! (beyond10x/ess#289).
//!
//! A synthesized suite carries an entity's invariants as predicates, and the Go and TypeScript
//! runners read them back with their own parsers. The Rust reader now resolves a bare word naming a
//! binder in scope to the binder; a runner that still read it as text would hold the same suite to
//! a different invariant. The Go vectors are `fixtures/binder-operand.go`, and
//! `src/ts/predicate.test.ts` answers the same ones in TypeScript.

use ess_conformance::AdmittedSuite;
use ess_primitives::{
    facts::{FactStore, FactValue},
    node::Node,
    predicate::{Predicate, Truth},
};
use serde_json::{json, Value};

const DISJOINT: &str = r#"{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": "tag != b"}}}}"#;
const OPERATOR: &str = r#"{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": {"tag": {"ne": "b"}}}}}}"#;
const QUOTED: &str = r#"{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": "tag != \"b\""}}}}"#;
const SHORTHAND: &str = r#"{"forall": {"in": "tags", "as": "tag", "that": {"forall": {"in": "banned", "as": "b", "that": {"tag": "b"}}}}}"#;
const OUT_OF_SCOPE: &str = r#"{"all": [{"forall": {"in": "banned", "as": "b", "that": "b != x"}}, {"forall": {"in": "tags", "as": "tag", "that": "tag != b"}}]}"#;

/// A case: its name, the predicate, the two lists it reads, and the expected truth.
type Vector = (
    &'static str,
    &'static str,
    &'static [&'static str],
    &'static [&'static str],
    Truth,
);

/// The vectors `fixtures/binder-operand.go` and `predicate.test.ts` answer too.
const VECTORS: &[Vector] = &[
    ("disjoint lists", DISJOINT, &["x", "y"], &["z"], Truth::True),
    ("a shared item", DISJOINT, &["x"], &["x"], Truth::False),
    (
        "a shared item named like the binder",
        DISJOINT,
        &["x", "b"],
        &["b"],
        Truth::False,
    ),
    (
        "the operator spelling, disjoint",
        OPERATOR,
        &["x"],
        &["z"],
        Truth::True,
    ),
    (
        "the operator spelling, shared",
        OPERATOR,
        &["x"],
        &["x"],
        Truth::False,
    ),
    ("a quoted word is text", QUOTED, &["x"], &["x"], Truth::True),
    (
        "a quoted word is text, matching",
        QUOTED,
        &["b"],
        &["z"],
        Truth::False,
    ),
    (
        "the shorthand is text",
        SHORTHAND,
        &["b"],
        &["z"],
        Truth::True,
    ),
    (
        "the shorthand is text, not matching",
        SHORTHAND,
        &["x"],
        &["x"],
        Truth::False,
    ),
    (
        "out of scope is text",
        OUT_OF_SCOPE,
        &["b"],
        &["z"],
        Truth::False,
    ),
    (
        "out of scope is text, not matching",
        OUT_OF_SCOPE,
        &["x"],
        &["z"],
        Truth::True,
    ),
];

fn list(store: &mut FactStore, name: &str, items: &[&str]) {
    store.set_path(&format!("{name}.count"), FactValue::count(items.len()));
    for (index, item) in items.iter().enumerate() {
        store.set_path(&format!("{name}.{index}"), FactValue::text(*item));
    }
}

#[test]
fn issue_289_the_rust_reader_answers_the_shared_vectors() {
    for (name, document, tags, banned, want) in VECTORS {
        let node: Node = serde_json::from_str(document).expect("json");
        let predicate = Predicate::from_node(&node).expect("parses");
        let mut store = FactStore::new();
        list(&mut store, "tags", tags);
        list(&mut store, "banned", banned);
        assert_eq!(predicate.evaluate(&store), *want, "{name}: {document}");
    }
}

fn document(predicate: &Value) -> String {
    json!({
        "provenance": {"suite_version": "ess-conformance/4", "system": "binder",
            "specification_version": "v1", "spec_digest": "a".repeat(64), "contract_digest": "b".repeat(64)},
        "scenarios": {"binder.core/authored/operand": {"purpose": "Compare two binders",
            "steps": [{"step": "expect_view", "view": "binder.core.Rows",
                "expectation": {"expect": "satisfies", "predicate": predicate}}], "source": []}}
    })
    .to_string()
}

#[test]
fn issue_289_the_generated_go_reader_answers_the_shared_vectors() {
    let predicate: Value = serde_json::from_str(DISJOINT).expect("json");
    let suite = AdmittedSuite::from_json(&document(&predicate)).expect("admitted");
    let directory =
        std::env::temp_dir().join(format!("ess-289-binder-operand-{}", std::process::id()));
    std::fs::create_dir_all(directory.join("essconform")).expect("directory");
    for artifact in ess_conformance::go::emit(suite.suite()).expect("emitted") {
        std::fs::write(directory.join(artifact.path), artifact.contents).expect("written");
    }
    std::fs::write(
        directory.join("go.mod"),
        "module example.invalid/binder\n\ngo 1.24\n",
    )
    .expect("go.mod");
    std::fs::write(
        directory.join("essconform/binder_test.go"),
        include_str!("fixtures/binder-operand.go"),
    )
    .expect("fixture");
    let result = std::process::Command::new("go")
        .args([
            "test",
            "./essconform",
            "-run",
            "TestBinderOperand",
            "-count=1",
            "-v",
        ])
        .env("GOWORK", "off")
        .current_dir(&directory)
        .output()
        .expect("go runs");
    let printed = format!(
        "{}\n{}",
        String::from_utf8_lossy(&result.stdout),
        String::from_utf8_lossy(&result.stderr)
    );
    let _ = std::fs::remove_dir_all(&directory);
    eprintln!("{printed}");
    assert!(
        result.status.success() && printed.contains("--- PASS: TestBinderOperand"),
        "Go binder operand regression: {}",
        result.status
    );
}

/// Two lists of text whose items may not be shared, compared binder against binder.
const SHARED_NOTHING: &str = r"format: ess/18
system: demo
version: v1
domain: demo.bundle
types:
  - {name: demo.bundle.BundleId, kind: newtype, of: Uuid}
entities:
  - name: demo.bundle.Bundle
    identity: {name: bundle_id, type: demo.bundle.BundleId}
    fields:
      - {name: tags, type: List<String>}
      - {name: banned, type: List<String>}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
    invariants:
      - forall:
          in: tags
          as: a
          that:
            forall: {in: banned, as: b, that: a != b}
events:
  - name: demo.bundle.Created
    fields:
      - {name: bundle_id, type: demo.bundle.BundleId}
actors:
  - name: demo.bundle.Operator
    may: [demo.bundle.CreateBundle]
commands:
  - name: demo.bundle.CreateBundle
    input:
      - {name: tags, type: List<String>}
      - {name: banned, type: List<String>}
    outcomes:
      - name: created
        creates: demo.bundle.Bundle
        instance: bundle_id
        emits: [demo.bundle.Created]
        payload:
          demo.bundle.Created: {bundle_id: {generated: true}}
        sets: {tags: input.tags, banned: input.banned}
views:
  - name: demo.bundle.Bundles
    source: demo.bundle.Bundle
    consistency: read_your_writes
    fields:
      - {name: bundle_id, type: demo.bundle.BundleId}
      - {name: tags, type: List<String>}
      - {name: banned, type: List<String>}
";

fn ir(text: &str) -> ess_compiler::ir::EssIr {
    let raw = ess_domain::spec::RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = ess_domain::spec::Specification::assemble([(
        ess_domain::system::Source::new("model.yaml"),
        raw,
    )])
    .unwrap_or_else(|errors| panic!("{errors}"));
    ess_compiler::resolve::compile(&spec, &ess_compiler::source::SourceMap::new())
        .unwrap_or_else(|error| panic!("{error:?}"))
}

fn texts(value: &ess_conformance::ScenarioValue) -> Vec<String> {
    match value {
        ess_conformance::ScenarioValue::Literal {
            value: Node::Seq(items),
        } => items
            .iter()
            .map(|item| match item {
                Node::Text(text) => text.clone(),
                other => panic!("a text item, not {other:?}"),
            })
            .collect(),
        other => panic!("a literal list, not {other:?}"),
    }
}

/// Synthesis holds the input it sends to the invariant read binder against binder: no candidate
/// sharing an item between the two lists is sent, a disjoint pair is, and the suite passes against
/// the interpreter of the same model. Read as `a != "b"`, the plain witness — the same text in both
/// lists — would have been sent.
#[test]
fn issue_289_synthesis_sends_only_disjoint_lists_and_the_suite_passes() {
    let model = ir(SHARED_NOTHING);
    let result = ess_conformance::synthesize::synthesize(&model);
    let mut sent = 0;
    for scenario in result.suite.scenarios.values() {
        for step in &scenario.steps {
            if let ess_conformance::ScenarioStep::ExecuteCommand { input, .. } = step {
                let (Some(tags), Some(banned)) = (input.get("tags"), input.get("banned")) else {
                    continue;
                };
                let (tags, banned) = (texts(tags), texts(banned));
                eprintln!("sent tags={tags:?} banned={banned:?}");
                assert!(
                    tags.iter().all(|tag| !banned.contains(tag)),
                    "a shared item was sent: tags={tags:?} banned={banned:?}"
                );
                sent += 1;
            }
        }
    }
    assert!(
        sent > 0,
        "synthesis sent no input at all: {:#?}",
        result.refusals
    );
    let admitted = AdmittedSuite::from_suite(&result.suite).expect("admitted");
    let report = ess_conformance::Runner::for_suite(&result.suite)
        .run_admitted(
            &admitted,
            &ess_conformance::interpret::Interpreted::for_model(ir(SHARED_NOTHING)),
        )
        .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| {
            matches!(
                scenario.status,
                ess_conformance::report::Status::Failed | ess_conformance::report::Status::Error
            )
        })
        .map(|scenario| format!("{scenario:#?}"))
        .collect();
    assert_eq!(failing, Vec::<String>::new());
}
