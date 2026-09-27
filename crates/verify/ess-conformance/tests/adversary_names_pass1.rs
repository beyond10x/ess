//! Adversary pass 1 against `story:field-names-underscore-and-newtype-map-keys`.
//!
//! The unit moved the Rust, Go and TypeScript readers of a suite's fact paths to "underscores, then
//! a letter" (beyond10x/ess#141), and guarded that with a scan of `src/go` and `src/ts`. The
//! browser coverage reader, `assets/coverage-admission.js`, reads the same suites and spells the
//! same grammar twice — once to admit a fact, once to decide whether a bare operand is a fact or a
//! literal — and was not moved. So a suite whose predicate reads `_url` is admitted by the Rust
//! reader and refused by the browser, and `x == _a.b` means a fact to Rust and a literal to the
//! browser.

use std::process::Command;

use ess_conformance::AdmittedSuite;
use ess_primitives::node::Node;
use ess_primitives::predicate::Predicate;
use serde_json::{json, Value};

const ADMISSION_JS: &str = include_str!("../assets/coverage-admission.js");

fn coverage(predicate: &Value) -> Value {
    json!({
        "provenance": {"suite_version": "ess-conformance/9", "system": "example",
            "specification_version": "v1",
            "spec_digest": "a".repeat(64), "contract_digest": "a".repeat(64)},
        "scenarios": {"example.domain/authored/created": {"purpose": "Known candidate",
            "steps": [{"step": "expect_view", "view": "example.All",
                "expectation": {"expect": "satisfies", "predicate": predicate}}],
            "source": []}},
        "coverage": {
            "selection": {"scope": {"kind": "system"}, "origins": "authored", "filter": {"kind": "all"}},
            "knowledge": "complete_inventory", "generated": [],
            "authored": ["example.domain/authored/created"], "outside": [], "refused": [],
            "authored_sources": {"created.yaml": {
                "digest": format!("sha256:{}", "b".repeat(64)),
                "scenario": "example.domain/authored/created", "disposition": "accepted"}},
            "counts": {"generated": 0, "authored": 1, "outside": 0, "refused": 0}
        }
    })
}

/// Runs the browser reader over each named document; each answer is its meaning, or the error.
fn browser(documents: &[(&str, Value)]) -> Value {
    let root = std::path::Path::new(env!("CARGO_TARGET_TMPDIR")).join(format!(
        "adversary-names-pass1-{}-{}",
        documents[0].0,
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    std::fs::create_dir_all(&root).unwrap();
    std::fs::write(root.join("admission.js"), ADMISSION_JS).unwrap();
    let mut names = Vec::new();
    for (name, document) in documents {
        std::fs::write(root.join(format!("{name}.json")), document.to_string()).unwrap();
        names.push(format!("{name:?}"));
    }
    std::fs::write(
        root.join("harness.mjs"),
        format!(
            "import {{readFileSync}} from 'node:fs'\nimport {{admitSuite}} from './admission.js'\n\
             const answer = async name => {{ try {{ const admitted = await admitSuite(readFileSync(name + '.json', 'utf8')); \
             return admitted.meaning['example.domain/authored/created'].steps[0].expectation.predicate }} \
             catch (error) {{ return String(error) }} }}\nconst out = {{}}\n\
             for (const name of [{}]) out[name] = await answer(name)\nconsole.log(JSON.stringify(out))\n",
            names.join(", ")
        ),
    )
    .unwrap();
    let node = std::env::var_os("ESS_NODE").unwrap_or_else(|| "node".into());
    let output = Command::new(&node)
        .arg("harness.mjs")
        .current_dir(&root)
        .output()
        .expect("the required Node toolchain executes; set ESS_NODE to name it");
    assert!(
        output.status.success(),
        "{}\n{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    serde_json::from_slice(&output.stdout).expect("JSON")
}

#[test]
fn a_predicate_on_an_underscore_field_is_admitted_by_rust_and_by_the_browser() {
    let structured = coverage(&json!({"_url": {"eq": "a"}}));
    let compact = coverage(&json!("_url == \"a\""));
    let control = coverage(&json!({"url": {"eq": "a"}}));
    for document in [&structured, &compact, &control] {
        AdmittedSuite::from_json(&document.to_string()).expect("the Rust reader admits it");
    }
    let answer = browser(&[
        ("structured", structured),
        ("compact", compact),
        ("control", control),
    ]);
    assert!(answer["control"].is_array(), "control: {answer}");
    assert!(
        answer["structured"].is_array(),
        "the browser refuses a structured predicate on `_url`: {answer}"
    );
    assert!(
        answer["compact"].is_array(),
        "the browser refuses a compact predicate on `_url`: {answer}"
    );
}

#[test]
fn a_bare_underscore_operand_means_the_same_to_rust_and_to_the_browser() {
    let rust = Predicate::from_node(&serde_json::from_value::<Node>(json!("x == _a.b")).unwrap())
        .expect("a predicate");
    let rust_reads_a_fact = format!("{rust:?}").contains("Fact(");
    assert!(rust_reads_a_fact, "Rust: {rust:?}");
    let answer = browser(&[("operand", coverage(&json!("x == _a.b")))]);
    assert_eq!(
        answer["operand"],
        json!(["compare", "x", "==", ["fact", "_a.b"]]),
        "Rust reads `_a.b` as a fact path and the browser as a literal"
    );
}

// ---- #143: "synthesizes a suite" -------------------------------------------------------------

fn checked(key: &str) -> String {
    format!(
        "format: ess/14\nsystem: demo\nversion: v1\ndomain: demo.orders\ntypes:\n  - {{name: \
         demo.orders.ItemId, kind: newtype, of: String}}\nevents:\n  - name: demo.orders.Checked\n    \
         fields:\n      - {{name: results, type: \"Map<{key}, Boolean>\"}}\nerrors:\n  - name: \
         demo.orders.Refused\n    fields: []\ncommands:\n  - name: demo.orders.Check\n    input:\n      \
         - {{name: results, type: \"Map<{key}, Boolean>\"}}\n    outcomes:\n      - name: checked\n        \
         emits: [demo.orders.Checked]\n        payload:\n          demo.orders.Checked:\n            \
         results: input.results\n      - name: refused\n        external: the checker is down\n        \
         error: demo.orders.Refused\n"
    )
}

fn suite_of(text: &str) -> ess_conformance::synthesize::Synthesis {
    use ess_compiler::{resolve::compile, source::SourceMap};
    use ess_domain::{spec::RawSpecFile, system::Source, Specification};
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let ir =
        compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"));
    ess_conformance::synthesize::synthesize(&ir)
}

#[test]
fn issue_143_synthesizes_the_same_suite_as_the_primitive_key() {
    let newtype = suite_of(&checked("demo.orders.ItemId"));
    let primitive = suite_of(&checked("String"));
    assert!(newtype.refusals.is_empty(), "{:#?}", newtype.refusals);
    assert!(
        !newtype.suite.scenarios.is_empty(),
        "a suite with scenarios"
    );
    assert_eq!(
        serde_json::to_value(&newtype.suite).unwrap(),
        serde_json::to_value(&primitive.suite).unwrap()
    );
}
