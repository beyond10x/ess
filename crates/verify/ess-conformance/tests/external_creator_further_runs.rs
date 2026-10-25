//! An external creating branch beside a `when_related: {exists: false}` guard sends every run of
//! its outcome scenario for the row the arrangement created.
//!
//! The fixture is the smallest shape of a downstream specification's render command: an
//! `exists: false` refusal over the patch the input names, then an external branch and a plain
//! branch that both create a render and copy an `Optional` input through `{input: …, else: 256}`.
//! The external branch's full invocation is arranged on a present patch (beyond10x/ess#464), but
//! the further run that leaves the `Optional` out was arranged as for a command reading no row: it
//! sent a literal patch identity no step created, `no-such-patch` answered, and
//! `RenderPatch/outcome/rendered-limited` failed on the model interpreter on every release from
//! 0.50.0 to 0.69.1. A further witness of such a branch is its siblings', as the plain branch's
//! already was, so the scenario keeps its full invocation alone.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::{AdmittedSuite, Runner, ScenarioStep, ScenarioValue};
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const MODEL: &str = include_str!("fixtures/external-creator-beside-related-guard.yaml");

const LIMITED: &str = "demo.studio.RenderPatch/outcome/rendered-limited";

fn ir() -> EssIr {
    let raw = RawSpecFile::parse(MODEL).unwrap_or_else(|error| panic!("{error}"));
    let mut texts = SourceMap::new();
    texts.insert("model.yaml".to_owned(), MODEL.to_owned());
    let spec = Specification::assemble(vec![(Source::new("model.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &texts).unwrap_or_else(|error| panic!("{error:?}"))
}

/// Every `patch_id` a send of `RenderPatch` in `scenario` carries.
fn sent_patches(steps: &[ScenarioStep]) -> Vec<ScenarioValue> {
    steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand { command, input, .. }
                if command.name().to_string() == "demo.studio.RenderPatch" =>
            {
                input.get("patch_id").cloned()
            }
            _ => None,
        })
        .collect()
}

#[test]
fn every_send_of_the_external_creator_names_an_arranged_patch() {
    let synthesis = ess_conformance::synthesize::synthesize(&ir());
    let (_, scenario) = synthesis
        .suite
        .scenarios
        .iter()
        .find(|(id, _)| id.to_string() == LIMITED)
        .unwrap_or_else(|| {
            panic!(
                "{LIMITED} is synthesized; refusals: {:#?}",
                synthesis
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .collect::<Vec<_>>()
            )
        });
    let sent = sent_patches(&scenario.steps);
    assert!(!sent.is_empty(), "{LIMITED} sends RenderPatch");
    let literal: Vec<&ScenarioValue> = sent
        .iter()
        .filter(|value| !matches!(value, ScenarioValue::Instance { .. }))
        .collect();
    assert!(
        literal.is_empty(),
        "{LIMITED} sends a patch no step created: {literal:?} of {sent:?}"
    );
}

#[test]
fn the_suite_passes_the_model_interpreter() {
    let model = ir();
    let synthesis = ess_conformance::synthesize::synthesize(&model);
    let admitted =
        AdmittedSuite::from_suite(&synthesis.suite).unwrap_or_else(|error| panic!("{error}"));
    let statuses: BTreeMap<String, Status> = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &Interpreted::for_model(model))
        .into_report()
        .scenarios
        .into_iter()
        .map(|result| (result.scenario.to_string(), result.status))
        .collect();
    assert_eq!(
        statuses.keys().map(String::as_str).collect::<Vec<_>>(),
        [
            "demo.studio.LoadPatch/outcome/loaded",
            "demo.studio.RenderPatch/outcome/no-such-patch",
            "demo.studio.RenderPatch/outcome/rendered",
            LIMITED,
        ],
        "every branch keeps its scenario"
    );
    let failing: Vec<_> = statuses
        .iter()
        .filter(|(_, status)| **status != Status::Passed)
        .collect();
    assert!(failing.is_empty(), "the interpreter fails {failing:?}");
}
