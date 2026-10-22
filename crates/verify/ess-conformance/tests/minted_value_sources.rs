//! A minted value stored present, and a stored value read from the outcome's own payload
//! (`ess/27`, beyond10x/ess#545, `docs/design/minted-value-sources.md`).
//!
//! `SelectAgent` mints a `connect_seq`, publishes it and stores it on an `Optional` field; an
//! invariant requires the field present once the entry is `Bridged`. With `{generated: true}` the
//! interpreted target leaves the field absent and the next transition breaks the invariant. With
//! `{generated: true, present: true}` it stores a value, and with `{payload: <Event>.<field>}` the
//! stored value is the published one, which a synthesized scenario then requires of the row.
use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::{
    interpret::Interpreted, report::Status, synthesize::synthesize, target::*, AdmittedSuite,
    ConformanceSuite, Runner,
};
use ess_domain::{
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const MODEL: &str = include_str!("fixtures/minted-value-sources.yaml");
const BRIDGED: &str = "demo.routing.BridgeEntry/outcome/bridged";
const SELECTED: &str = "demo.routing.SelectAgent/outcome/selected";

fn edit(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "fixture edit did not land: {from}");
    text.replacen(from, to, 1)
}

/// The model whose `sets:` reads the value its payload minted.
fn read_from_payload() -> String {
    edit(
        MODEL,
        "          connect_seq: {generated: true, present: true}\n",
        "          connect_seq: {payload: demo.routing.EntryAgentSelected.connect_seq}\n",
    )
}

/// The model as an author who left `present: true` off the stored mint writes it.
fn absent_mint() -> String {
    edit(
        MODEL,
        "          connect_seq: {generated: true, present: true}\n",
        "          connect_seq: {generated: true}\n",
    )
}

/// `model` with its invariant replaced by one bridging always breaks and that reads no field.
fn other_invariant(model: &str) -> String {
    edit(
        model,
        "      - any: [{not: \"state == Bridged\"}, \"defined(connect_seq)\"]\n",
        "      - \"state != Bridged\"\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}"));
    let spec = Specification::assemble([(Source::new("routing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn run<T: ConformanceTarget>(suite: &ConformanceSuite, target: &T) -> BTreeMap<String, String> {
    let admitted = AdmittedSuite::from_suite(suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|run| {
            let passed = matches!(run.status, Status::Passed);
            (
                run.scenario.to_string(),
                if passed {
                    "passed".to_owned()
                } else {
                    format!("{:?} {:?}", run.status, run.checks)
                },
            )
        })
        .collect()
}

fn failing(statuses: &BTreeMap<String, String>) -> Vec<&str> {
    statuses
        .iter()
        .filter(|(_, status)| *status != "passed")
        .map(|(id, _)| id.as_str())
        .collect()
}

/// The issue's model, with the mint stored present: every synthesized scenario passes against the
/// interpreted target, the one that bridges a selected entry included.
#[test]
fn the_interpreter_keeps_a_present_mint_present_through_the_invariant() {
    let model = ir(MODEL);
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model));
    assert!(statuses.contains_key(BRIDGED), "{statuses:#?}");
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");
}

/// With `{generated: true}` alone the interpreter leaves the minted field absent and the bridge
/// breaks the invariant: the refusal says the model is wrong and names the repair,
/// `{generated: true, present: true}`, in a `help:` line.
#[test]
fn a_broken_invariant_over_an_absent_mint_names_present_true() {
    let model = ir(&absent_mint());
    let statuses = run(&synthesize(&model).suite, &Interpreted::for_model(model));
    let bridged = statuses.get(BRIDGED).expect("the bridge has its scenario");
    assert!(
        bridged.contains("violating its invariant"),
        "the bridge breaks the invariant: {bridged}"
    );
    let Some((_, help)) = bridged.split_once("help:") else {
        panic!("the refusal carries a help line: {bridged}");
    };
    assert!(
        help.contains("`connect_seq`")
            && help.contains("`demo.routing.SelectAgent/selected`")
            && help.contains("{generated: true, present: true}"),
        "the help names the field, the outcome and `present: true`: {bridged}"
    );
}

/// An invariant broken without reading an `Optional` field minted by `{generated: true}` gets no
/// such line: neither where the mint is absent but the invariant reads no field, nor where the mint
/// is present.
#[test]
fn a_broken_invariant_over_no_absent_mint_names_no_present_true() {
    for text in [other_invariant(&absent_mint()), other_invariant(MODEL)] {
        let model = ir(&text);
        let statuses = run(&synthesize(&model).suite, &Interpreted::for_model(model));
        let bridged = statuses.get(BRIDGED).expect("the bridge has its scenario");
        assert!(
            bridged.contains("violating its invariant"),
            "the bridge breaks the invariant: {bridged}"
        );
        assert!(
            !bridged.contains("help:") && !bridged.contains("present: true"),
            "no `present: true` line for an invariant that reads no absent mint: {bridged}"
        );
    }
}

/// A target whose rows carry another `connect_seq` than the one it published, or none.
struct StoresAnother(Interpreted, Stored);

/// What [`StoresAnother`] shows in place of the stored value.
#[derive(Clone, Copy)]
enum Stored {
    Other,
    Nothing,
}

impl ConformanceTarget for StoresAnother {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        self.0.identity()
    }
    fn begin_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.begin_scenario(scenario)
    }
    fn end_scenario(&self, scenario: &ScenarioContext) -> Result<(), TargetError> {
        self.0.end_scenario(scenario)
    }
    fn establish_entity(&self, request: EntitySetupRequest) -> Result<(), TargetError> {
        self.0.establish_entity(request)
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.0.execute_command(request)
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        let mut result = self.0.query_view(request)?;
        for row in &mut result.rows {
            if let Some(Node::Number(seq)) = row.get("connect_seq") {
                let other = seq.to_string().parse::<i64>().unwrap_or_default() + 1000;
                let shown = match self.1 {
                    Stored::Other => Node::Number(other.into()),
                    Stored::Nothing => Node::Null,
                };
                row.insert("connect_seq".to_owned(), shown);
            }
        }
        Ok(result)
    }
    fn configure_external_outcome(&self, r: ExternalOutcomeControl) -> Result<(), TargetError> {
        self.0.configure_external_outcome(r)
    }
    fn redeliver_event(&self, r: RedeliveryRequest) -> Result<(), TargetError> {
        self.0.redeliver_event(r)
    }
    fn observe_events(
        &self,
        r: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        self.0.observe_events(r)
    }
}

/// One mint feeds both: the interpreted target passes every scenario, and a target that stores a
/// value other than the one it published fails the scenario of the branch that minted it.
#[test]
fn a_stored_value_read_from_the_payload_equals_the_published_one() {
    let model = ir(&read_from_payload());
    let synthesis = synthesize(&model);
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    let statuses = run(&synthesis.suite, &Interpreted::for_model(model.clone()));
    assert!(statuses.contains_key(BRIDGED), "{statuses:#?}");
    assert!(failing(&statuses).is_empty(), "{statuses:#?}");

    let faulty = run(
        &synthesis.suite,
        &StoresAnother(Interpreted::for_model(model), Stored::Other),
    );
    assert!(
        failing(&faulty).contains(&SELECTED),
        "a row holding another value than the published one fails {SELECTED}: {faulty:#?}"
    );
}

/// A present mint is required of the row: a target that shows the minted field absent fails the
/// scenario of the branch that minted it, where it passed with `{generated: true}`.
#[test]
fn a_row_leaving_a_present_mint_absent_fails_the_minting_scenario() {
    let model = ir(MODEL);
    let suite = synthesize(&model).suite;
    let faulty = run(
        &suite,
        &StoresAnother(Interpreted::for_model(model), Stored::Nothing),
    );
    assert!(failing(&faulty).contains(&SELECTED), "{faulty:#?}");
}
