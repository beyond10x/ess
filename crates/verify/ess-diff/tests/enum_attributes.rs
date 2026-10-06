//! `ess verify diff` over typed enum attributes (`ess/23`, beyond10x/ess#450).
//!
//! A value change that moves a guard's lowered membership is the guard's own change,
//! `outcome-condition-changed`, classified as that change always is. An attribute declared, removed
//! or revalued is reported through the vocabulary every released delta format already has,
//! `unclassified-changed`: no change of an attributed enum passes as no change at all. Classifying
//! a moved lowered set as breaking needs a delta format this cut does not introduce.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_diff::compatibility::Compatibility;
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

/// The rules system with `Operator`'s `types:` entry as given; `AddRule` decides by
/// `operator.takes_number` when `guarded`.
fn model(operator: &str, guarded: bool) -> EssIr {
    let (numeric, textual) = if guarded {
        (
            "operator.takes_number == true",
            "operator.takes_number == false",
        )
    } else {
        (
            "{operator: {any_of: [GreaterThan, LessThan]}}",
            "{operator: {any_of: [Contains]}}",
        )
    };
    let text = format!(
        "format: ess/23
system: demo
version: v1
domain: demo.rules
types:
{operator}events:
  - name: demo.rules.RuleAdded
    fields:
      - {{name: operator, type: demo.rules.Operator}}
commands:
  - name: demo.rules.AddRule
    input:
      - {{name: operator, type: demo.rules.Operator}}
    outcomes:
      - name: numeric
        when: {numeric}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{operator: input.operator}}
      - name: textual
        when: {textual}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{operator: input.operator}}
"
    );
    let raw = RawSpecFile::parse(&text).expect("parses");
    let spec = Specification::assemble([(Source::new("rules.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("{errors}"))
}

fn operator(less_than_takes_number: bool, attributes: bool) -> String {
    if !attributes {
        return "  - name: demo.rules.Operator
    kind: enum
    variants: [GreaterThan, LessThan, Contains]
"
        .to_owned();
    }
    format!(
        "  - name: demo.rules.Operator
    kind: enum
    attributes:
      - {{name: takes_number, type: Boolean}}
    variants:
      - {{name: GreaterThan, attributes: {{takes_number: true}}}}
      - {{name: LessThan, attributes: {{takes_number: {less_than_takes_number}}}}}
      - {{name: Contains, attributes: {{takes_number: false}}}}
"
    )
}

/// Each change of the classified delta, as `kind` and verdict.
fn changes(before: &EssIr, after: &EssIr) -> Vec<(String, Compatibility)> {
    let delta = ess_diff::classified(before, after).expect("one system");
    let verdicts = delta.compatibility().expect("classified").to_vec();
    delta
        .changes()
        .iter()
        .zip(verdicts)
        .map(|(change, verdict)| (change.kind().to_owned(), verdict.verdict()))
        .collect()
}

#[test]
fn enum_attribute_diff_reported() {
    // A value change that moves the lowered guard set: the guard moved, as a guard change, and the
    // change is not compatible.
    let found = changes(
        &model(&operator(true, true), true),
        &model(&operator(false, true), true),
    );
    assert!(
        found
            .iter()
            .any(|(kind, verdict)| kind == "outcome-condition-changed"
                && *verdict != Compatibility::Compatible),
        "the moved guard is reported: {found:#?}"
    );
    assert!(
        found.iter().any(|(kind, _)| kind.contains("unclassified")),
        "the revalued attribute is reported: {found:#?}"
    );

    // Declared and removed: the same guards, written by hand on one side.
    let declared = changes(
        &model(&operator(true, false), false),
        &model(&operator(true, true), true),
    );
    assert_ne!(declared.len(), 0, "declaring attributes is a change");
    assert!(
        declared.iter().all(|(kind, _)| !kind.contains("condition")),
        "the lowered guards equal the hand-written ones: {declared:#?}"
    );
    let removed = changes(
        &model(&operator(true, true), true),
        &model(&operator(true, false), false),
    );
    assert_ne!(removed.len(), 0, "removing attributes is a change");

    // Nothing moved, nothing reported.
    assert_eq!(
        changes(
            &model(&operator(true, true), true),
            &model(&operator(true, true), true)
        ),
        Vec::new()
    );
}
