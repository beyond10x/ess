//! An `existing_instance:` outcome beside `when_related:` guards on one input, one of them a
//! list-membership guard over a field the related row's creator sets from its own input
//! (`not: {exists: {in: entries, as: entry, that: entry.action == input.action}}`), reported by an
//! adopter: the existence family fixes the input first — the identity is sent twice — and the
//! related row is then searched toward it, so its list has to hold the action that input names.
//!
//! `tests/fixtures/existence-list-membership.yaml`: `Project` creates a catalogue from its input,
//! `Select` refuses an unknown catalogue, an action the catalogue does not list and a revision
//! other than the catalogue's, and otherwise creates a selection once under its identity.

use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::ScenarioStep;
use ess_conformance::{synthesize::Synthesis, AdmittedSuite, Runner, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/existence-list-membership.yaml");
const EXISTS: &str = "demo.pick.Select/outcome/selection-exists";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("existence-list-membership.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| format!("{}: {}", refusal.cause.code(), refusal.cause))
        .collect()
}

fn statuses(result: &Synthesis, model: &EssIr) -> BTreeMap<String, Status> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model.clone()))
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| (run.scenario.to_string(), run.status))
        .collect()
}

fn literal<'v>(input: &'v BTreeMap<String, ScenarioValue>, field: &str) -> Option<&'v Node> {
    match input.get(field)? {
        ScenarioValue::Literal { value } => Some(value),
        _ => None,
    }
}

/// The inputs of every send of `command` in the scenario `id`.
fn sends(result: &Synthesis, id: &str, command: &str) -> Vec<BTreeMap<String, ScenarioValue>> {
    let Some((_, scenario)) = result
        .suite
        .scenarios
        .iter()
        .find(|(key, _)| key.to_string() == id)
    else {
        panic!("{id} is synthesized; refusals: {:#?}", refusals(result));
    };
    scenario
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command: sent,
                input,
                ..
            } if sent.to_string() == command => Some(input.clone()),
            _ => None,
        })
        .collect()
}

#[test]
fn an_existence_outcome_passes_a_related_list_membership_guard() {
    let model = ir(MODEL);
    let result = ess_conformance::synthesize::synthesize(&model);
    let refused = refusals(&result);
    assert!(
        !refused
            .iter()
            .any(|refusal| refusal.contains("demo.pick.Select")),
        "{refused:#?}"
    );
    let selects = sends(&result, EXISTS, "demo.pick.Select");
    assert_eq!(selects.len(), 2, "the identity is sent twice: {selects:#?}");
    let action = literal(&selects[0], "action").expect("the first send names an action");
    let revision = literal(&selects[0], "revision").expect("the first send names a revision");
    let projects = sends(&result, EXISTS, "demo.pick.Project");
    assert!(
        projects.iter().any(|project| {
            let listed = matches!(literal(project, "entries"), Some(Node::Seq(entries))
                if entries.iter().any(|entry| matches!(entry, Node::Map(fields)
                    if fields.get("action") == Some(action))));
            listed && literal(project, "revision") == Some(revision)
        }),
        "a catalogue lists {action:?} at revision {revision:?}: {projects:#?}"
    );
    let passed = statuses(&result, &model);
    assert_eq!(passed.get(EXISTS), Some(&Status::Passed), "{passed:#?}");
    assert!(
        passed.values().all(|status| *status == Status::Passed),
        "{passed:#?}"
    );
}
