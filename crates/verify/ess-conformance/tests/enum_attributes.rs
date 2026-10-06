//! An enum attribute read in a guard (`ess/23`, beyond10x/ess#450) compiles to the membership the
//! author could have written by hand, and synthesis witnesses both sides of it.

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const ATTRIBUTED: &str = "  - name: demo.rules.Operator
    kind: enum
    attributes:
      - {name: takes_number, type: Boolean}
    variants:
      - {name: GreaterThan, attributes: {takes_number: true}}
      - {name: LessThan, attributes: {takes_number: true}}
      - {name: Contains, attributes: {takes_number: false}}
";
const BARE: &str = "  - name: demo.rules.Operator
    kind: enum
    variants: [GreaterThan, LessThan, Contains]
";

fn model(types: &str, numeric: &str, textual: &str) -> String {
    format!(
        "format: ess/23
system: demo
version: v1
domain: demo.rules
types:
{types}events:
  - name: demo.rules.RuleAdded
    fields:
      - {{name: operator, type: demo.rules.Operator}}
  - name: demo.rules.TextRuleAdded
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
        emits: [demo.rules.TextRuleAdded]
        payload:
          demo.rules.TextRuleAdded: {{operator: input.operator}}
"
    )
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("rules.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// The compiled command, as JSON.
fn command(ir: &EssIr) -> serde_json::Value {
    let command = ir
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.rules.AddRule")
        .expect("AddRule");
    serde_json::to_value(command).expect("serialises")
}

#[test]
fn enum_attribute_guard_lowers_to_membership() {
    let attributed = ir_of(&model(
        ATTRIBUTED,
        "operator.takes_number == true",
        "operator.takes_number == false",
    ));
    let written = ir_of(&model(
        BARE,
        "{operator: {any_of: [GreaterThan, LessThan]}}",
        "{operator: {any_of: [Contains]}}",
    ));
    assert_eq!(
        command(&attributed),
        command(&written),
        "the attribute guard compiles to the membership written by hand"
    );

    // Synthesis witnesses both sides, and the interpreter passes them.
    let synthesis = ess_conformance::synthesize::synthesize(&attributed);
    let document: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    let scenarios = document["scenarios"].as_object().expect("scenarios");
    for outcome in ["numeric", "textual"] {
        let id = format!("demo.rules.AddRule/outcome/{outcome}");
        assert!(
            scenarios.contains_key(&id),
            "{id} is witnessed: {:#?}",
            scenarios.keys()
        );
    }
    let numeric = &scenarios["demo.rules.AddRule/outcome/numeric"]["steps"][0]["input"]["operator"];
    assert!(
        numeric["value"] == "GreaterThan" || numeric["value"] == "LessThan",
        "the numeric witness takes a number: {numeric}"
    );
    let textual = &scenarios["demo.rules.AddRule/outcome/textual"]["steps"][0]["input"]["operator"];
    assert_eq!(textual["value"], "Contains", "{textual}");

    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(attributed))
        .into_report();
    assert_ne!(report.scenarios.len(), 0, "not empty");
    for result in &report.scenarios {
        assert_eq!(result.status, Status::Passed, "{}", result.scenario);
    }
}
