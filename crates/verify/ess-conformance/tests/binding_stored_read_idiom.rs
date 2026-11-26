//! The idiom beyond10x/ess#440 is declined in favour of: a binding maps the identity, and the
//! command it invokes reads the stored row. The binding guide's model validates, and synthesis
//! asserts the value read from the referenced row on the invoked command's event.

use std::path::PathBuf;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{spec::RawSpecFile, system::Source, Specification};

const HEADING: &str = "# Read stored state in the command a binding invokes";

/// The first fenced `yaml` block under `HEADING`, the title of the guide page the binding guide
/// moved the idiom to.
fn guide_model() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/binding-stored-state.md");
    let page = std::fs::read_to_string(&path).expect("the binding guide");
    let start = page
        .find(&format!("\n{HEADING}\n"))
        .unwrap_or_else(|| panic!("the guide has no `{HEADING}`"));
    let section = &page[start..];
    let open = section.find("```yaml\n").expect("a fenced model") + "```yaml\n".len();
    let close = section[open..].find("\n```").expect("a closed fence");
    section[open..=(open + close)].to_owned()
}

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("calls.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the guide's model validates: {errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

#[test]
fn binding_command_side_related_read_validates_and_asserts_value() {
    let model = guide_model();
    let ir = ir_of(&model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let document: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    let scenarios = document["scenarios"].as_object().expect("scenarios");
    let marked: Vec<(&String, &serde_json::Value)> = scenarios
        .iter()
        .filter(|(id, _)| id.contains("MarkLeg") && id.contains("marked"))
        .collect();
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_ne!(
        marked.len(),
        0,
        "a MarkLeg/marked scenario: {:#?} {refusals:#?}",
        scenarios.keys()
    );

    // Each scenario arranges the call with a `bridged` value, and asserts that value on the event
    // the invoked command emits: the read the binding could not make, made by the command.
    let mut asserted = 0;
    for (id, scenario) in &marked {
        let steps = scenario["steps"].as_array().expect("steps");
        let opened: Vec<&serde_json::Value> = steps
            .iter()
            .filter(|step| step["command"] == "demo.calls.OpenCall")
            .map(|step| &step["input"]["bridged"]["value"])
            .collect();
        for step in steps {
            if step["step"] == "expect_event" && step["event"] == "demo.calls.LegMarked" {
                let value = &step["payload"]["call_bridged"];
                assert!(
                    opened.contains(&value),
                    "{id}: `call_bridged` {value} is the arranged call's `bridged` {opened:?}"
                );
                asserted += 1;
            }
        }
    }
    assert!(
        asserted > 0,
        "LegMarked's `call_bridged` is asserted: {marked:#?}"
    );

    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir_of(&model)))
        .into_report();
    for result in &report.scenarios {
        if result.scenario.to_string().contains("MarkLeg") {
            assert_eq!(result.status, Status::Passed, "{}", result.scenario);
        }
    }
}

// ---- a command only a development build serves (beyond10x/ess#816 item 6) ----------------------

const BUILDS: &str = "### A command only some builds serve";

/// The `### A command only some builds serve` subsection of the components guide, up to the next
/// heading.
fn builds_subsection() -> String {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../../website/docs/guides/specify/bindings-and-components.md");
    let page = std::fs::read_to_string(&path).expect("the components guide");
    let start = page
        .find(&format!("\n{BUILDS}\n"))
        .unwrap_or_else(|| panic!("the guide has no `{BUILDS}`"))
        + 1;
    let mut end = page.len();
    let mut at = start + BUILDS.len();
    let mut fenced = false;
    for line in page[at..].split_inclusive('\n') {
        if line.starts_with("```") {
            fenced = !fenced;
        }
        if !fenced && line.starts_with('#') && at > start + BUILDS.len() {
            end = at;
            break;
        }
        at += line.len();
    }
    page[start..end].to_owned()
}

/// The first fenced `yaml` block of the subsection.
fn builds_model() -> String {
    let section = builds_subsection();
    let open = section.find("```yaml\n").expect("a fenced model") + "```yaml\n".len();
    let close = section[open..].find("\n```").expect("a closed fence");
    section[open..=(open + close)].to_owned()
}

#[test]
fn components_page_keeps_a_development_only_command_in_its_owner() {
    let section = builds_subsection();
    let _ = ir_of(&builds_model());
    for phrase in ["ESS-COMPONENT-004", "ESS-SYNTH-003", "the implementation's"] {
        assert!(
            section.contains(phrase),
            "the subsection says {phrase:?}:\n{section}"
        );
    }
}

#[test]
fn development_only_model_synthesizes_and_passes() {
    let model = builds_model();
    let ir = ir_of(&model);
    let synthesis = ess_conformance::synthesize::synthesize(&ir);
    let refusals: Vec<String> = synthesis.refusals.iter().map(ToString::to_string).collect();
    assert_eq!(refusals, Vec::<String>::new());
    let document: serde_json::Value =
        serde_json::from_str(&synthesis.suite.to_canonical_json().expect("serialises"))
            .expect("JSON");
    let scenarios = document["scenarios"].as_object().expect("scenarios");
    let arranged = scenarios
        .iter()
        .filter(|(id, _)| id.contains("SignOut"))
        .filter(|(_, scenario)| {
            scenario["steps"]
                .as_array()
                .expect("steps")
                .iter()
                .any(|step| step["command"] == "demo.access.LocalSignIn")
        })
        .count();
    assert!(
        arranged > 0,
        "a SignOut scenario arranges its session through LocalSignIn: {:#?}",
        scenarios.keys()
    );

    let admitted = AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|e| panic!("{e}"));
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(ir))
        .into_report();
    assert!(!report.scenarios.is_empty());
    for result in &report.scenarios {
        assert_eq!(result.status, Status::Passed, "{}", result.scenario);
    }
}

#[test]
fn a_second_component_accepting_the_development_command_is_refused() {
    let model = builds_model();
    let owner = "    accepts: {commands: [demo.access.LocalSignIn, demo.access.SignOut]}\n";
    assert_eq!(model.matches(owner).count(), 1, "the model moved");
    let split = model.replace(
        owner,
        "    accepts: {commands: [demo.access.SignOut]}\n    reached_by: network\n  - component: dev-sign-in\n    accepts: {commands: [demo.access.LocalSignIn]}\n",
    );
    let raw = RawSpecFile::parse(&split).expect("the model parses");
    let errors = Specification::assemble([(Source::new("split.yaml"), raw)])
        .map(|_| ())
        .expect_err("a second handler is refused")
        .to_string();
    assert!(
        errors.lines().any(|line| line
            .contains("[conflicting_declaration] component dev-sign-in.accepts.commands")),
        "{errors}"
    );
}
