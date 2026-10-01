//! Adversary, pass 2, for beyond10x/ess#272: an aggregate view whose creating command carries a
//! `when_related:` guard is witnessed instead of refused with `ESS-SYNTH-017`.
//!
//! Pass 1 moved the views off the one the unit's fixture holds. These cases attack what the
//! correction added:
//!
//! - the refusals it introduced, each against a shape whose creating outcome the same synthesis
//!   witnesses — a row owned through another input than the one the guard compares, and two
//!   inputs compared with the guarded row's owner link;
//! - a view keyed on the very input the guard reads (`sample_id`), whose rows the arrangement
//!   gives one fresh related row each, so no group holds more than one row;
//! - the ungrouped count, asserted only as a change, against targets that miscount.
use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{
    report::Status, scenario::ErrorRef, synthesize::Synthesis, target::*, AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

/// A study owns its samples and a sample its assays; starting an assay is refused for a sample
/// nobody received and for a sample of another study than the input names. Five views count
/// assays: per study and state, per operator, in total, per sample and study, and per sample.
const LAB: &str = "format: ess/19
system: demo
version: v1
domain: demo.lab
types:
  - {name: demo.lab.StudyId, kind: newtype, of: Uuid}
  - {name: demo.lab.SampleId, kind: newtype, of: Uuid}
  - {name: demo.lab.AssayId, kind: newtype, of: Uuid}
entities:
  - name: demo.lab.Study
    identity: {name: study_id, type: demo.lab.StudyId}
    fields:
      - {name: title, type: String}
    relations:
      - {name: samples, kind: owns, target: demo.lab.Sample, cardinality: many, via: study_id}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
  - name: demo.lab.Sample
    identity: {name: sample_id, type: demo.lab.SampleId}
    fields:
      - {name: study_id, type: demo.lab.StudyId}
    relations:
      - {name: assays, kind: owns, target: demo.lab.Assay, cardinality: many, via: sample_id}
    lifecycle: {initial: Received, states: [Received], terminal: [Received]}
  - name: demo.lab.Assay
    identity: {name: assay_id, type: demo.lab.AssayId}
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: study_id, type: demo.lab.StudyId}
      - {name: operator, type: String}
    lifecycle: {initial: Running, states: [Running], terminal: [Running]}
errors:
  - name: demo.lab.SampleNotFound
    summary: No such sample.
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
  - name: demo.lab.OtherStudy
    summary: The sample belongs to another study.
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: study_id, type: demo.lab.StudyId}
events:
  - name: demo.lab.StudyOpened
    fields:
      - {name: study_id, type: demo.lab.StudyId}
  - name: demo.lab.SampleReceived
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: study_id, type: demo.lab.StudyId}
  - name: demo.lab.AssayStarted
    fields:
      - {name: assay_id, type: demo.lab.AssayId}
commands:
  - name: demo.lab.Open
    input:
      - {name: title, type: String}
    outcomes:
      - name: opened
        creates: demo.lab.Study
        instance: study_id
        sets: {title: input.title}
        emits: [demo.lab.StudyOpened]
        payload:
          demo.lab.StudyOpened: {study_id: {generated: true}}
  - name: demo.lab.Receive
    input:
      - {name: study_id, type: demo.lab.StudyId}
    outcomes:
      - name: received
        creates: demo.lab.Sample
        instance: sample_id
        sets: {study_id: input.study_id}
        emits: [demo.lab.SampleReceived]
        payload:
          demo.lab.SampleReceived: {sample_id: {generated: true}, study_id: input.study_id}
  - name: demo.lab.Start
    input:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: study_id, type: demo.lab.StudyId}
      - {name: operator, type: String}
    outcomes:
      - name: unknown-sample
        when_related: {via: input.sample_id, exists: false}
        error: demo.lab.SampleNotFound
        payload:
          demo.lab.SampleNotFound: {sample_id: input.sample_id}
      - name: other-study
        when_related: {via: input.sample_id, predicate: study_id != input.study_id}
        error: demo.lab.OtherStudy
        payload:
          demo.lab.OtherStudy: {sample_id: input.sample_id, study_id: input.study_id}
      - name: started
        creates: demo.lab.Assay
        instance: assay_id
        sets: {sample_id: input.sample_id, study_id: input.study_id, operator: input.operator}
        emits: [demo.lab.AssayStarted]
        payload:
          demo.lab.AssayStarted: {assay_id: {generated: true}}
views:
  - name: demo.lab.AssaysByStudy
    source: demo.lab.Assay
    consistency: read_your_writes
    group_by: [study_id, state]
    fields:
      - {name: study_id, type: demo.lab.StudyId}
      - {name: state, type: demo.lab.Assay.State}
      - {name: assays, type: Integer, aggregate: {count: {}}}
  - name: demo.lab.AssaysByOperator
    source: demo.lab.Assay
    consistency: read_your_writes
    group_by: [operator]
    fields:
      - {name: operator, type: String}
      - {name: assays, type: Integer, aggregate: {count: {}}}
  - name: demo.lab.AssayCount
    source: demo.lab.Assay
    consistency: read_your_writes
    fields:
      - {name: assays, type: Integer, aggregate: {count: {}}}
  - name: demo.lab.AssaysBySampleStudy
    source: demo.lab.Assay
    consistency: read_your_writes
    group_by: [sample_id, study_id]
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: study_id, type: demo.lab.StudyId}
      - {name: assays, type: Integer, aggregate: {count: {}}}
  - name: demo.lab.AssaysBySample
    source: demo.lab.Assay
    consistency: read_your_writes
    group_by: [sample_id]
    fields:
      - {name: sample_id, type: demo.lab.SampleId}
      - {name: assays, type: Integer, aggregate: {count: {}}}
";

const OTHER_STUDY_BRANCH: &str = "      - name: other-study
        when_related: {via: input.sample_id, predicate: study_id != input.study_id}
        error: demo.lab.OtherStudy
        payload:
          demo.lab.OtherStudy: {sample_id: input.sample_id, study_id: input.study_id}
";

const BY_STUDY: &str = "demo.lab.AssaysByStudy";
const BY_OPERATOR: &str = "demo.lab.AssaysByOperator";
const COUNT: &str = "demo.lab.AssayCount";
const BY_SAMPLE_STUDY: &str = "demo.lab.AssaysBySampleStudy";
const BY_SAMPLE: &str = "demo.lab.AssaysBySample";
const VIEWS: [&str; 5] = [BY_STUDY, BY_OPERATOR, COUNT, BY_SAMPLE_STUDY, BY_SAMPLE];
const STARTED: &str = "demo.lab.Start/outcome/started";

/// [`LAB`] with the predicate guard (`true`) or with `exists: false` alone.
fn lab(predicate: bool) -> String {
    if predicate {
        LAB.to_owned()
    } else {
        LAB.replace(OTHER_STUDY_BRANCH, "")
    }
}

/// [`LAB`] with the assay owned by a bench, named by its own input `bench_id`, rather than by its
/// sample. The predicate still compares `input.study_id` with the sample's study: the assay's
/// owner is a bench and the guard's is a study, two entities and two inputs.
fn bench() -> String {
    LAB.replace(
        "    relations:\n      - {name: assays, kind: owns, target: demo.lab.Assay, cardinality: \
         many, via: sample_id}\n",
        "",
    )
    .replace(
        "  - name: demo.lab.Assay\n",
        "  - name: demo.lab.Bench
    identity: {name: bench_id, type: demo.lab.BenchId}
    fields:
      - {name: label, type: String}
    relations:
      - {name: assays, kind: owns, target: demo.lab.Assay, cardinality: many, via: bench_id}
    lifecycle: {initial: Ready, states: [Ready], terminal: [Ready]}
  - name: demo.lab.Assay\n",
    )
    .replace(
        "      - {name: operator, type: String}\n    lifecycle: {initial: Running",
        "      - {name: operator, type: String}\n      - {name: bench_id, type: \
         demo.lab.BenchId}\n    lifecycle: {initial: Running",
    )
    .replace(
        "  - {name: demo.lab.AssayId, kind: newtype, of: Uuid}\n",
        "  - {name: demo.lab.AssayId, kind: newtype, of: Uuid}\n  - {name: demo.lab.BenchId, \
         kind: newtype, of: Uuid}\n",
    )
    .replace(
        "events:\n",
        "events:\n  - name: demo.lab.BenchInstalled\n    fields:\n      - {name: bench_id, type: \
         demo.lab.BenchId}\n",
    )
    .replace(
        "commands:\n",
        "commands:
  - name: demo.lab.Install
    input:
      - {name: label, type: String}
    outcomes:
      - name: installed
        creates: demo.lab.Bench
        instance: bench_id
        sets: {label: input.label}
        emits: [demo.lab.BenchInstalled]
        payload:
          demo.lab.BenchInstalled: {bench_id: {generated: true}}\n",
    )
    .replace(
        "      - {name: operator, type: String}\n    outcomes:",
        "      - {name: operator, type: String}\n      - {name: bench_id, type: \
         demo.lab.BenchId}\n    outcomes:",
    )
    .replace(
        "sets: {sample_id: input.sample_id, study_id: input.study_id, operator: input.operator}",
        "sets: {sample_id: input.sample_id, study_id: input.study_id, operator: input.operator, \
         bench_id: input.bench_id}",
    )
}

/// [`LAB`] with a second input, `lead_study_id`, compared with the sample's study by a second
/// predicate branch. `started` is selected where both inputs name the study the sample is filed
/// under — one owner, named twice.
fn two_inputs() -> String {
    LAB.replace(
        "      - {name: operator, type: String}\n    outcomes:",
        "      - {name: operator, type: String}\n      - {name: lead_study_id, type: \
         demo.lab.StudyId}\n    outcomes:",
    )
    .replace(
        "      - name: started\n",
        "      - name: other-lead
        when_related: {via: input.sample_id, predicate: study_id != input.lead_study_id}
        error: demo.lab.OtherStudy
        payload:
          demo.lab.OtherStudy: {sample_id: input.sample_id, study_id: input.lead_study_id}
      - name: started\n",
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("lab.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(text: &str) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(text))
}

fn aggregate(view: &str) -> String {
    format!("{view}/aggregate")
}

/// The `ESS-SYNTH-017` refusals of the aggregate scenarios, rendered.
fn refused(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains("/aggregate`"))
        .collect()
}

fn has(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

// ---------------------------------------------------------------------------------------------
// The refusals the correction introduced, each against a shape the same synthesis witnesses.
// ---------------------------------------------------------------------------------------------

/// `guard_owner` refuses a row whose owner link is filled from another input than the guard
/// compares: "one row is created under one owner". Here the assay's owner is a bench and the
/// study the guard compares is no owner of the assay at all; the synthesis witnesses
/// `Start/outcome/started` itself, on a bench and a sample filed under the study the input names.
/// The acceptance statement, with an input-set key (`operator`, `bench_id`, `study_id`) or none.
#[test]
fn a_row_owned_through_another_input_than_the_guard_compares_is_witnessed() {
    let result = synthesis(&bench());
    assert!(
        has(&result, STARTED),
        "the creating outcome is not witnessed"
    );
    let found = refused(&result);
    assert!(found.is_empty(), "{found:#?}");
}

/// `guard_link` refuses a command comparing two inputs with the guarded row's owner link: "one
/// row is filed under one owner". Both inputs naming that one owner selects `started`, and the
/// synthesis witnesses `Start/outcome/started` that way.
#[test]
fn two_inputs_compared_with_the_owner_link_are_witnessed() {
    let result = synthesis(&two_inputs());
    assert!(
        has(&result, STARTED),
        "the creating outcome is not witnessed"
    );
    let found = refused(&result);
    assert!(found.is_empty(), "{found:#?}");
}

// ---------------------------------------------------------------------------------------------
// A hand-written target answering the model, and the mutants it can carry.
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Reports a count of one for every group: counts distinct groups, not rows.
    CountsOne,
    /// Counts one assay fewer than the group holds (never below zero).
    CountsOneFewer,
    /// Counts one assay more than the group holds.
    CountsOneMore,
    /// Counts only the assays of the study the latest assay was started for.
    LatestStudyOnly,
}

type Row = BTreeMap<String, Node>;

struct Lab {
    predicate: bool,
    mutant: Mutant,
    /// Each sample, and the study it was received for.
    samples: RefCell<Vec<(Node, Node)>>,
    assays: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn number(n: usize) -> Node {
    match FactValue::parse_literal(&n.to_string()) {
        FactValue::Number(number) => Node::Number(number),
        other => panic!("{other:?}"),
    }
}

fn error(name: &str) -> DeclaredErrorValue {
    DeclaredErrorValue::new(name.parse::<ErrorRef>().unwrap())
}

impl Lab {
    fn new(predicate: bool, mutant: Mutant) -> Self {
        Self {
            predicate,
            mutant,
            samples: RefCell::default(),
            assays: RefCell::default(),
            minted: Cell::new(0),
        }
    }

    fn mint(&self) -> Node {
        self.minted.set(self.minted.get() + 1);
        Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()))
    }

    fn view(&self, name: &str) -> Vec<Row> {
        let keys: &[&str] = match name {
            BY_STUDY => &["study_id", "state"],
            BY_OPERATOR => &["operator"],
            COUNT => &[],
            BY_SAMPLE_STUDY => &["sample_id", "study_id"],
            BY_SAMPLE => &["sample_id"],
            _ => return Vec::new(),
        };
        let assays = self.assays.borrow();
        let latest = assays.last().map(|assay| assay["study_id"].clone());
        let mut groups: Vec<(Vec<Node>, Vec<Row>)> = Vec::new();
        for assay in assays.iter() {
            if self.mutant == Mutant::LatestStudyOnly && Some(&assay["study_id"]) != latest.as_ref()
            {
                continue;
            }
            let key: Vec<Node> = keys.iter().map(|key| assay[*key].clone()).collect();
            match groups.iter_mut().find(|(held, _)| *held == key) {
                Some((_, members)) => members.push(assay.clone()),
                None => groups.push((key, vec![assay.clone()])),
            }
        }
        if keys.is_empty() && groups.is_empty() {
            groups.push((Vec::new(), Vec::new()));
        }
        groups
            .into_iter()
            .map(|(_, members)| {
                let held = members.len();
                let count = match self.mutant {
                    Mutant::CountsOne => usize::from(held > 0),
                    Mutant::CountsOneFewer => held.saturating_sub(1),
                    Mutant::CountsOneMore => held + 1,
                    Mutant::None | Mutant::LatestStudyOnly => held,
                };
                let mut row = Row::from([("assays".to_owned(), number(count))]);
                for field in keys {
                    row.insert(
                        (*field).to_owned(),
                        members
                            .first()
                            .map_or(Node::Null, |member| member[*field].clone()),
                    );
                }
                row
            })
            .collect()
    }

    fn start(&self, command: &CommandRef, input: &Row) -> SemanticCommandResult {
        let named = input.get("sample_id").cloned().unwrap_or(Node::Null);
        let sent = input.get("study_id").cloned().unwrap_or(Node::Null);
        let held = self
            .samples
            .borrow()
            .iter()
            .find(|(id, _)| *id == named)
            .map(|(_, study)| study.clone());
        let Some(held) = held else {
            return SemanticCommandResult::took(outcome(command, "unknown-sample"))
                .with_error(error("demo.lab.SampleNotFound").with("sample_id", named));
        };
        if self.predicate && held != sent {
            return SemanticCommandResult::took(outcome(command, "other-study")).with_error(
                error("demo.lab.OtherStudy")
                    .with("sample_id", named)
                    .with("study_id", sent),
            );
        }
        let id = self.mint();
        self.assays.borrow_mut().push(Row::from([
            ("assay_id".to_owned(), id.clone()),
            ("sample_id".to_owned(), named),
            ("study_id".to_owned(), sent),
            (
                "operator".to_owned(),
                input.get("operator").cloned().unwrap_or(Node::Null),
            ),
            ("state".to_owned(), Node::Text("Running".to_owned())),
        ]));
        SemanticCommandResult::took(outcome(command, "started")).emitting(
            ObservedEvent::new("demo.lab.AssayStarted".parse().unwrap()).with("assay_id", id),
        )
    }
}

impl ConformanceTarget for Lab {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("lab-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.samples.replace(Vec::new());
        self.assays.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let result = match command.to_string().as_str() {
            "demo.lab.Open" => {
                let id = self.mint();
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.lab.StudyOpened".parse().unwrap())
                        .with("study_id", id),
                )
            }
            "demo.lab.Receive" => {
                let id = self.mint();
                let study = request.input.get("study_id").cloned().unwrap_or(Node::Null);
                self.samples.borrow_mut().push((id.clone(), study.clone()));
                SemanticCommandResult::took(outcome(&command, "received")).emitting(
                    ObservedEvent::new("demo.lab.SampleReceived".parse().unwrap())
                        .with("sample_id", id)
                        .with("study_id", study),
                )
            }
            "demo.lab.Start" => self.start(&command, &request.input),
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(
            self.view(request.view.to_string().as_str()),
        ))
    }
    fn observe_events(
        &self,
        _: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        Ok(Vec::new())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// The status of the scenario `id` from [`lab`]`(predicate)`'s suite against a target with
/// `mutant`.
fn status(predicate: bool, id: &str, mutant: Mutant) -> Status {
    let mut suite = synthesis(&lab(predicate)).suite;
    suite.scenarios.retain(|key, _| key.to_string() == id);
    assert_eq!(suite.scenarios.len(), 1, "no scenario {id}");
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Lab::new(predicate, mutant);
    Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios[0]
        .status
}

#[test]
fn every_view_of_the_fixture_is_witnessed_and_the_model_passes() {
    for predicate in [false, true] {
        let result = synthesis(&lab(predicate));
        let found = refused(&result);
        assert!(found.is_empty(), "predicate {predicate}: {found:#?}");
        for view in VIEWS {
            let id = aggregate(view);
            assert_eq!(
                status(predicate, &id, Mutant::None),
                Status::Passed,
                "predicate {predicate}: {id}"
            );
        }
    }
}

// ---------------------------------------------------------------------------------------------
// A view keyed on the input the guard reads.
// ---------------------------------------------------------------------------------------------

/// A view grouping by `sample_id`, the input the guard reads, is witnessed — but each row is
/// started on a sample of its own, so every group the scenario expects holds one row. A target
/// that reports one for every group, counting groups rather than rows, cannot fail it. The
/// pattern's rows sharing a key value would share one sample, as rows sharing a study already
/// share one study (`GuardLink` owners).
#[test]
fn a_view_keyed_on_the_guarded_row_catches_a_target_counting_groups_not_rows() {
    let mut missed = Vec::new();
    for predicate in [false, true] {
        for view in [BY_SAMPLE, BY_SAMPLE_STUDY] {
            let id = aggregate(view);
            for mutant in [Mutant::CountsOne, Mutant::CountsOneMore] {
                let got = status(predicate, &id, mutant);
                if got != Status::Failed {
                    missed.push((predicate, view, mutant, got));
                }
            }
        }
    }
    assert!(missed.is_empty(), "{missed:#?}");
}

// ---------------------------------------------------------------------------------------------
// The ungrouped count, asserted as a change.
// ---------------------------------------------------------------------------------------------

/// The ungrouped count is asserted only as the change the scenario's own rows make, so a constant
/// over-count is beyond it by design (`docs/design/aggregate-views.md`, "Scoping"; the same
/// assertion an unguarded command gets). What it does catch, under both guards: a count one
/// short, and a count of groups rather than rows. A count scoped to one study is beyond it too —
/// every row names one shared study — and the per-study view is what catches that.
#[test]
fn the_ungrouped_count_catches_what_a_change_can_show() {
    let id = aggregate(COUNT);
    for predicate in [false, true] {
        for mutant in [Mutant::CountsOneFewer, Mutant::CountsOne] {
            assert_eq!(
                status(predicate, &id, mutant),
                Status::Failed,
                "predicate {predicate}: {mutant:?}"
            );
        }
        assert_eq!(
            status(predicate, &aggregate(BY_STUDY), Mutant::LatestStudyOnly),
            Status::Failed,
            "predicate {predicate}: the per-study view misses a count scoped to one study"
        );
    }
}
