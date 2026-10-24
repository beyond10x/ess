//! The native and Go runners agree that an `Optional` struct member left absent and one written
//! `null` are one value at any depth of a view assertion
//! (`story:a-null-member-matches-an-absent-optional-in-a-nested-assertion`).
//!
//! One authored scenario per pairing of how the row stores a `Quantity`'s `value` (absent, `null`,
//! `3`) with how a `contains` and an `at` assertion spell it, inside a list element and inside a
//! struct field. The Go runtime replays the transcript the Rust reference runner recorded against
//! the interpreted target, so a verdict that differs is a difference in how the runners compare a
//! row. The TypeScript runtime's `matches` is held to the same rule in `src/ts/runtime.test.ts`:
//! its transcript target does not answer an entity setup.

mod support_go;

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Source as AuthoredSource};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, AdmittedSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};

const MODEL: &str = r"
format: ess/1
system: demo
version: v1
domain: demo.usage
types:
  - name: demo.usage.Quantity
    kind: struct
    fields:
      - {name: kind, type: String}
      - {name: source, type: String}
      - {name: value, type: 'Optional<Integer>'}
entities:
  - name: demo.usage.Record
    identity: {name: record_id, type: Uuid}
    fields:
      - {name: quantities, type: 'List<demo.usage.Quantity>'}
      - {name: latest, type: demo.usage.Quantity}
    lifecycle:
      initial: Recorded
      states: [Recorded]
      terminal: [Recorded]
views:
  - name: demo.usage.Records
    source: demo.usage.Record
    consistency: read_your_writes
    order_by: [record_id asc]
    fields:
      - {name: record_id, type: Uuid}
      - {name: quantities, type: 'List<demo.usage.Quantity>'}
      - {name: latest, type: demo.usage.Quantity}
";

/// (stored tail, asserted tail, whether the scenario passes).
const CASES: [(&str, &str, bool); 8] = [
    ("", ", value: null", true),
    (", value: null", ", value: null", true),
    (", value: null", "", true),
    ("", "", true),
    ("", ", value: 3", false),
    (", value: null", ", value: 3", false),
    (", value: 3", ", value: null", false),
    (", value: 3", "", false),
];

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("usage.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn scenario(index: usize, stored: &str, asserted: &str) -> String {
    let quantity = |tail: &str| format!("{{kind: cached_input_tokens, source: unknown{tail}}}");
    format!(
        r"type: ess-scenario/2
domain: demo.usage
scenario: nested-optional-{index}
summary: An Optional member left unset is asserted as the authored spelling says.
arrange:
  - instance: usage
    entity: demo.usage.Record
    setup:
      identity: 00000000-0000-4000-8000-00000000000{index}
      fields: {{quantities: [{stored_quantity}], latest: {stored_quantity}}}
      state: Recorded
assert:
  - view: demo.usage.Records
    contains: {{record_id: {{$instance: usage}}, quantities: [{asserted_quantity}]}}
  - view: demo.usage.Records
    contains: {{record_id: {{$instance: usage}}, latest: {asserted_quantity}}}
  - view: demo.usage.Records
    at: {{row: first, fields: {{quantities: [{asserted_quantity}], latest: {asserted_quantity}}}}}
",
        stored_quantity = quantity(stored),
        asserted_quantity = quantity(asserted),
    )
}

fn written(status: Status) -> String {
    match status {
        Status::Passed => "passed",
        Status::Failed => "failed",
        Status::Error => "error",
        Status::Unsupported => "unsupported",
    }
    .to_owned()
}

#[test]
fn the_native_and_go_runners_agree_on_a_nested_null_and_an_absent_member() {
    let model = ir();
    let sources: Vec<AuthoredSource> = CASES
        .iter()
        .enumerate()
        .map(|(index, (stored, asserted, _))| {
            AuthoredSource::new(
                format!("case-{index}.yaml"),
                scenario(index, stored, asserted),
            )
        })
        .collect();
    let authored = compile_authored(&model, &sources);
    assert!(authored.is_complete(), "{:?}", authored.refusals);
    let mut suite = synthesize(&model).suite;
    suite.scenarios = authored.scenarios;
    let suite = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));

    let recorder = support_go::Recorder::new(Interpreted::for_model(model));
    let rust: BTreeMap<String, String> = Runner::for_suite(suite.suite())
        .run_admitted(&suite, &recorder)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), written(run.status)))
        .collect();
    let expected: BTreeMap<String, String> = CASES
        .iter()
        .enumerate()
        .map(|(index, (_, _, passes))| {
            (
                format!("demo.usage/authored/nested-optional-{index}"),
                if *passes { "passed" } else { "failed" }.to_owned(),
            )
        })
        .collect();
    assert_eq!(rust, expected, "Rust (left), expected (right)");

    let directory = support_go::package(
        "nested-optional",
        suite.suite(),
        &[support_go::TRANSCRIPT_TARGET],
    );
    let go = support_go::replay(&directory, &recorder, &[]);
    assert!(
        go.divergences.is_empty(),
        "the Go runtime sent requests the reference runner did not:\n{}\n{}",
        go.divergences.join("\n"),
        go.go.log
    );
    assert_eq!(
        go.go.outcomes, rust,
        "Go (left), Rust (right)\n{}",
        go.go.log
    );
}
