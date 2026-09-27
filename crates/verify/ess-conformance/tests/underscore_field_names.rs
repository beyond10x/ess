//! A field whose name begins with an underscore synthesizes a suite (beyond10x/ess#141).
//!
//! Admitting `_url` in the specification is half the work: the suite reads a command input
//! through a fact path whose first segment is the field's name, and the Go and TypeScript runtimes
//! check the names and paths a suite carries before they run it. Each of those spelled the
//! field-name rule on its own, so each is held here to the one the specification publishes.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::ScenarioStep;
use ess_domain::types::Field;
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

/// #141's repro, with a command that takes the struct and the field, and a guard that reads it.
const MODEL: &str = r#"
format: ess/14
system: demo
version: v1
domain: demo.orders
types:
  - name: demo.orders.OrderView
    kind: struct
    fields:
      - {name: _url, type: Optional<String>}
      - {name: id, type: String}
events:
  - name: demo.orders.Linked
    fields:
      - {name: _url, type: String}
errors:
  - name: demo.orders.Unlinkable
    fields: []
commands:
  - name: demo.orders.Link
    input:
      - {name: _url, type: String}
      - {name: view, type: demo.orders.OrderView}
    outcomes:
      - name: linked
        when: _url != ""
        emits: [demo.orders.Linked]
        payload:
          demo.orders.Linked:
            _url: input._url
      - name: unlinkable
        error: demo.orders.Unlinkable
"#;

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("orders.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn issue_141_an_underscore_field_synthesizes_scenarios_and_no_refusal() {
    let result = ess_conformance::synthesize::synthesize(&ir(MODEL));
    assert!(result.refusals.is_empty(), "{:#?}", result.refusals);
    assert!(
        !result.suite.scenarios.is_empty(),
        "at least one scenario holds `_url`"
    );

    let inputs = result
        .suite
        .scenarios
        .values()
        .flat_map(|scenario| &scenario.steps)
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { input, .. } => Some(input),
            _ => None,
        })
        .collect::<Vec<_>>();
    assert!(!inputs.is_empty());
    for input in inputs {
        let rendered = serde_json::to_string(input).expect("serialises");
        assert!(
            rendered.contains("\"_url\""),
            "the input keeps its wire name: {rendered}"
        );
    }
}

#[test]
fn underscores_alone_or_before_a_digit_are_still_not_field_names() {
    for spelling in ["_", "_1"] {
        let text = MODEL.replace(
            "{name: _url, type: String}\n      - {name: view",
            &format!("{{name: {spelling:?}, type: String}}\n      - {{name: view"),
        );
        assert_ne!(text, MODEL, "the replacement found its target");
        let error = RawSpecFile::parse(&text).expect_err(spelling);
        assert!(
            error.to_string().contains("field name"),
            "{spelling:?}: {error}"
        );
    }
}

/// The literal the specification's field-name rule had before #141.
const OLD_FIELD_NAME: &str = "^[A-Za-z][A-Za-z0-9_]*$";
/// The literal the runtimes' fact-path grammar had before #141.
const OLD_FACT_PATH: &str = "^[A-Za-z][A-Za-z0-9_-]*(\\.[A-Za-z0-9_-]+)*$";
/// The runtimes' fact-path grammar: the Rust `FactPath` rule, first character included.
const FACT_PATH: &str = "^_*[A-Za-z][A-Za-z0-9_-]*(\\.[A-Za-z0-9_-]+)*$";

/// Every runtime source a synthesized Go or TypeScript suite package carries, and the browser
/// coverage player's scripts, which admit a replay by the same grammar.
fn runtime_sources() -> Vec<(String, String)> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let mut found = Vec::new();
    for directory in ["src/go", "src/ts", "assets"] {
        let mut entries = std::fs::read_dir(root.join(directory))
            .expect("the runtime directory exists")
            .map(|entry| entry.expect("an entry").path())
            .filter(|path| {
                path.extension().is_some_and(|extension| {
                    ["go", "ts", "js"].iter().any(|known| extension == *known)
                })
            })
            .collect::<Vec<_>>();
        entries.sort();
        for path in entries {
            let text = std::fs::read_to_string(&path).expect("readable");
            found.push((path.display().to_string(), text));
        }
    }
    assert!(found.len() > 4, "the runtime sources were found: {found:?}");
    found
}

#[test]
fn no_runtime_spells_the_field_name_rule_other_than_the_specification_does() {
    assert_eq!(Field::PATTERN, "^_*[A-Za-z][A-Za-z0-9_]*$");
    let mut stale = Vec::new();
    let mut current = 0;
    for (path, text) in runtime_sources() {
        for (number, line) in text.lines().enumerate() {
            if line.contains(OLD_FIELD_NAME) || line.contains(OLD_FACT_PATH) {
                stale.push(format!("{path}:{}: {}", number + 1, line.trim()));
            }
            current += line.matches(Field::PATTERN).count() + line.matches(FACT_PATH).count();
        }
    }
    assert!(
        stale.is_empty(),
        "a runtime refuses a field name the specification admits:\n{}",
        stale.join("\n")
    );
    // runtime.go ×4, replay.go, reading.go, reading.ts, runtime.ts, predicate.go, predicate.ts,
    // and coverage-admission.js twice: its fact grammar and its operand classifier.
    assert_eq!(
        current, 12,
        "every check this file knows of carries the published rule"
    );
}
