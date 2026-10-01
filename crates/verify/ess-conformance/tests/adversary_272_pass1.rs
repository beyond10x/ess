//! Adversary, pass 1, for beyond10x/ess#272: an aggregate view whose creating command carries a
//! `when_related:` guard is witnessed instead of refused with `ESS-SYNTH-017`.
//!
//! The unit's own fixture (`aggregate_related_guard.rs`) holds one ownership (an assay belongs to
//! its sample) and, under the predicate guard, one view: the one keyed on the very input the
//! predicate compares. These cases move off that point:
//!
//! - the predicate guard with an input-set key the predicate does not compare (`operator`), and
//!   with no key at all (an ungrouped count) — the acceptance statement's "an input-set key and a
//!   `when_related` guard";
//! - the assay owned by the study the predicate compares, rather than by the sample;
//! - the predicate guard beside a key copied from the guarded row's owner link (#270 into #257);
//! - and, against a hand-written target, the mutants the brief names: a target that skips the
//!   guard, one that merges groups across state, one that counts one more, on shapes the unit's
//!   fixture does not hold (a multi-state source, several aggregates over one source, the
//!   assay owned by the study).
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
/// nobody received, and for a sample of another study than the input names. The assay records
/// who ran it (`operator`), from the input. Four views count assays: per study and state, per
/// operator, in total, and per sample and study.
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
";

const BY_STUDY: &str = "demo.lab.AssaysByStudy";
const BY_OPERATOR: &str = "demo.lab.AssaysByOperator";
const COUNT: &str = "demo.lab.AssayCount";
const BY_SAMPLE_STUDY: &str = "demo.lab.AssaysBySampleStudy";
const UNKNOWN_SAMPLE: &str = "demo.lab.Start/outcome/unknown-sample";
const OTHER_STUDY: &str = "demo.lab.Start/outcome/other-study";

const OTHER_STUDY_BRANCH: &str = "      - name: other-study
        when_related: {via: input.sample_id, predicate: study_id != input.study_id}
        error: demo.lab.OtherStudy
        payload:
          demo.lab.OtherStudy: {sample_id: input.sample_id, study_id: input.study_id}
";

/// One variation of [`LAB`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[allow(clippy::struct_excessive_bools)]
struct Shape {
    /// The `other-study` predicate guard is present (else `exists: false` alone).
    predicate: bool,
    /// `study_id` is copied from the sample the guard reads (#257), not taken from the input.
    copies: bool,
    /// The assay is owned by its study (`via: study_id`), not by its sample.
    owned_by_study: bool,
    /// The assay has a second state, `Done`, reached by `demo.lab.Finish`.
    finishes: bool,
}

const PREDICATE: Shape = Shape {
    predicate: true,
    copies: false,
    owned_by_study: false,
    finishes: false,
};
const EXISTS: Shape = Shape {
    predicate: false,
    ..PREDICATE
};

impl Shape {
    fn model(self) -> String {
        let mut text = LAB.to_owned();
        if !self.predicate {
            text = text.replace(OTHER_STUDY_BRANCH, "");
        }
        if self.copies {
            text = text.replace(
                "study_id: input.study_id, operator: input.operator}",
                "study_id: {related: {via: sample_id, field: study_id}}, operator: \
                 input.operator}",
            );
        }
        if self.owned_by_study {
            text = text
                .replace(
                    "    relations:\n      - {name: assays, kind: owns, target: demo.lab.Assay, \
                     cardinality: many, via: sample_id}\n",
                    "",
                )
                .replace(
                    "      - {name: samples, kind: owns, target: demo.lab.Sample, cardinality: \
                     many, via: study_id}\n",
                    "      - {name: samples, kind: owns, target: demo.lab.Sample, cardinality: \
                     many, via: study_id}\n      - {name: assays, kind: owns, target: \
                     demo.lab.Assay, cardinality: many, via: study_id}\n",
                );
        }
        if self.finishes {
            text = text
                .replace(
                    "    lifecycle: {initial: Running, states: [Running], terminal: [Running]}\n",
                    "    lifecycle:
      initial: Running
      states: [Running, Done]
      terminal: [Done]
      transitions:
        - {name: finish, from: [Running], to: Done}
",
                )
                .replace(
                    "commands:\n",
                    "commands:
  - name: demo.lab.Finish
    input:
      - {name: assay_id, type: demo.lab.AssayId}
    outcomes:
      - name: finished
        moves: demo.lab.Assay.finish
        instance: assay_id
        emits: [demo.lab.AssayFinished]
        payload: {demo.lab.AssayFinished: {assay_id: input.assay_id}}
",
                )
                .replace(
                    "events:\n",
                    "events:
  - name: demo.lab.AssayFinished
    fields:
      - {name: assay_id, type: demo.lab.AssayId}
",
                );
        }
        text
    }
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("lab.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn synthesis(shape: Shape) -> Synthesis {
    ess_conformance::synthesize::synthesize(&ir(&shape.model()))
}

/// The `ESS-SYNTH-017` refusals naming one of `views`, rendered.
fn refused(result: &Synthesis, views: &[&str]) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| {
            views
                .iter()
                .any(|view| rendered.contains(&format!("`{view}/aggregate`")))
        })
        .collect()
}

fn aggregate(view: &str) -> String {
    format!("{view}/aggregate")
}

// ---------------------------------------------------------------------------------------------
// Acceptance: an input-set key (or none) beside the predicate guard.
// ---------------------------------------------------------------------------------------------

/// The acceptance statement: "With an input-set key and a `when_related` guard on the creating
/// command, `<view>/aggregate` is synthesized instead of ESS-SYNTH-017." `operator` is set from the
/// input, and the guard is the unit's own predicate shape; only the key differs from the fixture's
/// (`study_id`, the input the predicate compares). The ungrouped count has no key at all and the
/// same guard.
#[test]
fn a_predicate_guard_does_not_refuse_a_view_keyed_on_another_input() {
    let result = synthesis(PREDICATE);
    let found = refused(&result, &[BY_OPERATOR, COUNT]);
    assert!(found.is_empty(), "{found:#?}");
}

/// The assay belongs to the study the predicate compares (`via: study_id`), not to the sample:
/// the row's own owner and the owner the guard's input must name are one study. The key is
/// input-set (`study_id: input.study_id`).
#[test]
fn an_assay_owned_by_the_study_the_predicate_compares_is_witnessed() {
    let shape = Shape {
        owned_by_study: true,
        ..PREDICATE
    };
    let result = synthesis(shape);
    let found = refused(&result, &[BY_STUDY]);
    assert!(found.is_empty(), "{found:#?}");
}

/// The predicate guard beside a key copied from the guarded row's link to its owner (#270 into
/// #257): the unit witnesses each half alone (`Predicate`, `Copied` in its fixture).
#[test]
fn a_predicate_guard_beside_a_key_copied_from_the_owner_link_is_witnessed() {
    let shape = Shape {
        copies: true,
        ..PREDICATE
    };
    let result = synthesis(shape);
    let found = refused(&result, &[BY_STUDY]);
    assert!(found.is_empty(), "{found:#?}");
}

// ---------------------------------------------------------------------------------------------
// Counting mutants against a hand-written target.
// ---------------------------------------------------------------------------------------------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Starts an assay whatever sample the input names, and whatever its study.
    SkipsGuards,
    /// Groups `AssaysByStudy` by study alone, merging the states.
    MergesStates,
    /// Groups `AssaysBySampleStudy` by study alone, merging the samples.
    MergesSamples,
    /// Counts one assay more than each group holds.
    CountsOneMore,
}

type Row = BTreeMap<String, Node>;

struct Lab {
    shape: Shape,
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
    fn new(shape: Shape, mutant: Mutant) -> Self {
        Self {
            shape,
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
        let keys: &[&str] = match (name, self.mutant) {
            (BY_STUDY, Mutant::MergesStates) | (BY_SAMPLE_STUDY, Mutant::MergesSamples) => {
                &["study_id"]
            }
            (BY_STUDY, _) => &["study_id", "state"],
            (BY_OPERATOR, _) => &["operator"],
            (COUNT, _) => &[],
            (BY_SAMPLE_STUDY, _) => &["sample_id", "study_id"],
            _ => return Vec::new(),
        };
        let shown: &[&str] = match name {
            BY_STUDY => &["study_id", "state"],
            BY_OPERATOR => &["operator"],
            BY_SAMPLE_STUDY => &["sample_id", "study_id"],
            _ => &[],
        };
        let mut groups: Vec<(Vec<Node>, Vec<Row>)> = Vec::new();
        for assay in self.assays.borrow().iter() {
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
                let count = members.len() + usize::from(self.mutant == Mutant::CountsOneMore);
                let mut row = Row::from([("assays".to_owned(), number(count))]);
                for field in shown {
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
        let skips = self.mutant == Mutant::SkipsGuards;
        let held = self
            .samples
            .borrow()
            .iter()
            .find(|(id, _)| *id == named)
            .map(|(_, study)| study.clone());
        let Some(held) = held.or_else(|| skips.then_some(Node::Null)) else {
            return SemanticCommandResult::took(outcome(command, "unknown-sample"))
                .with_error(error("demo.lab.SampleNotFound").with("sample_id", named));
        };
        if self.shape.predicate && !skips && held != sent {
            return SemanticCommandResult::took(outcome(command, "other-study")).with_error(
                error("demo.lab.OtherStudy")
                    .with("sample_id", named)
                    .with("study_id", sent),
            );
        }
        let study = if self.shape.copies { held } else { sent };
        let id = self.mint();
        self.assays.borrow_mut().push(Row::from([
            ("assay_id".to_owned(), id.clone()),
            ("sample_id".to_owned(), named),
            ("study_id".to_owned(), study),
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

    fn finish(&self, command: &CommandRef, input: &Row) -> SemanticCommandResult {
        let named = input.get("assay_id").cloned().unwrap_or(Node::Null);
        let mut assays = self.assays.borrow_mut();
        let Some(assay) = assays.iter_mut().find(|assay| assay["assay_id"] == named) else {
            return SemanticCommandResult::undeclared();
        };
        if assay["state"] != Node::Text("Running".to_owned()) {
            return SemanticCommandResult::undeclared();
        }
        assay.insert("state".to_owned(), Node::Text("Done".to_owned()));
        SemanticCommandResult::took(outcome(command, "finished")).emitting(
            ObservedEvent::new("demo.lab.AssayFinished".parse().unwrap()).with("assay_id", named),
        )
    }
}

impl ConformanceTarget for Lab {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("lab-adversary", "1"))
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
            "demo.lab.Finish" => self.finish(&command, &request.input),
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

/// The status of each scenario of `ids`, run from `shape`'s suite against `mutant`. Every id must
/// be synthesized: a scenario that is missing is reported, not silently passed.
fn statuses(shape: Shape, mutant: Mutant, ids: &[String]) -> BTreeMap<String, Status> {
    let mut suite = synthesis(shape).suite;
    suite
        .scenarios
        .retain(|id, _| ids.contains(&id.to_string()));
    assert_eq!(
        suite.scenarios.len(),
        ids.len(),
        "{shape:?}: every scenario of {ids:?} is synthesized"
    );
    let admitted = AdmittedSuite::from_suite(&suite).unwrap();
    let target = Lab::new(shape, mutant);
    Runner::for_suite(&suite)
        .run_admitted(&admitted, &target)
        .into_report()
        .scenarios
        .iter()
        .map(|scenario| (scenario.scenario.to_string(), scenario.status))
        .collect()
}

/// The brief's question: the aggregate does not catch a target that skips the guard; the guard's
/// own scenarios must. Each guarded shape the unit witnesses, against a target starting an assay
/// on any sample of any study.
#[test]
fn a_target_skipping_the_guard_fails_the_guards_own_scenarios() {
    let mut missed = Vec::new();
    for (shape, guards) in [
        (EXISTS, vec![UNKNOWN_SAMPLE]),
        (PREDICATE, vec![UNKNOWN_SAMPLE, OTHER_STUDY]),
        (
            Shape {
                copies: true,
                ..EXISTS
            },
            vec![UNKNOWN_SAMPLE],
        ),
    ] {
        let ids: Vec<String> = guards.iter().map(|id| (*id).to_owned()).collect();
        let honest = statuses(shape, Mutant::None, &ids);
        assert!(
            honest.values().all(|status| *status == Status::Passed),
            "{shape:?}: {honest:?}"
        );
        for (id, status) in statuses(shape, Mutant::SkipsGuards, &ids) {
            if status != Status::Failed {
                missed.push((shape, id));
            }
        }
    }
    assert!(missed.is_empty(), "{missed:#?}");
}

/// Shapes the unit witnesses that its own fixture does not run: a second state on the source
/// (the key is state plus the owner link), several aggregates over one source, and the exists
/// guard on an assay owned by the study. The honest target passes each; each mutant fails it.
#[test]
fn every_miscount_fails_on_the_shapes_the_fixture_does_not_hold() {
    let multi_state_copied = Shape {
        copies: true,
        finishes: true,
        ..EXISTS
    };
    let multi_state_predicate = Shape {
        finishes: true,
        ..PREDICATE
    };
    let owned_by_study = Shape {
        owned_by_study: true,
        ..EXISTS
    };
    let cases: [(Shape, &str, &[Mutant]); 5] = [
        (
            multi_state_copied,
            BY_STUDY,
            &[Mutant::MergesStates, Mutant::CountsOneMore],
        ),
        (
            multi_state_predicate,
            BY_STUDY,
            &[Mutant::MergesStates, Mutant::CountsOneMore],
        ),
        (
            PREDICATE,
            BY_SAMPLE_STUDY,
            &[Mutant::MergesSamples, Mutant::CountsOneMore],
        ),
        (EXISTS, BY_OPERATOR, &[Mutant::CountsOneMore]),
        (owned_by_study, BY_STUDY, &[Mutant::CountsOneMore]),
    ];
    let mut wrong = Vec::new();
    for (shape, view, mutants) in cases {
        let id = aggregate(view);
        let honest = statuses(shape, Mutant::None, std::slice::from_ref(&id))[&id];
        if honest != Status::Passed {
            wrong.push((shape, view, Mutant::None, honest));
        }
        for mutant in mutants {
            let status = statuses(shape, *mutant, std::slice::from_ref(&id))[&id];
            if status != Status::Failed {
                wrong.push((shape, view, *mutant, status));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
