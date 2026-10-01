//! An aggregate view whose creating command is guarded by a related row (`when_related:` on a
//! sibling branch) is witnessed (beyond10x/ess#272).
//!
//! Every row the aggregate scenario creates is created through the guarded command, so each needs
//! the related row its guard reads — arranged, and the input pointed at it — as the outcome
//! scenarios of that command already arrange it (#211, #270, #271). Three shapes: an `exists:
//! false` guard alone, a predicate guard comparing the related row with the input beside it, and a
//! group key the creating command copies from the very row its guard reads (#257). The last reads
//! the sample's link to its study, so the value is a study the arrangement creates; a fourth shape
//! copies it with no guard at all, which was refused the same way.
//!
//! Each scenario is run against a hand-written target that answers the model, which must pass it,
//! and against mutants that count or group wrongly, each of which must fail it. The guard's own
//! refusal branches keep their own scenarios, which the same target passes.
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
    ScenarioStep, ScenarioValue,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{facts::FactValue, node::Node};

/// A study owns its samples, a sample owns its assays, and starting an assay is refused for a
/// sample nobody received. The view counts assays per study and state.
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
";

const VIEW: &str = "demo.lab.AssaysByStudy";
/// A view that does not read the input a predicate guard compares.
const BY_OPERATOR: &str = "demo.lab.AssaysByOperator";
/// An ungrouped view, which reads no input at all.
const COUNT: &str = "demo.lab.AssayCount";
const AGGREGATE: &str = "demo.lab.AssaysByStudy/aggregate";
const UNKNOWN_SAMPLE: &str = "demo.lab.Start/outcome/unknown-sample";
const OTHER_STUDY: &str = "demo.lab.Start/outcome/other-study";

/// Which guard the creating command carries, and where its group key comes from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Shape {
    /// `exists: false` alone; the key comes from the input.
    ExistsOnly,
    /// A predicate guard beside it compares the sample's study with the study the input names.
    Predicate,
    /// The group key is copied from the sample the guard reads (`{related: …}`, #257).
    Copied,
    /// As `Copied`, with no guard: the key read is still the sample's link to its study, an
    /// identity the arrangement creates rather than a value the pattern chooses.
    CopiedUnguarded,
    /// The predicate guard beside a key copied from the sample's link to its study (#270 into
    /// #257): the study the input names must be the one the copied sample is filed under.
    CopiedPredicate,
    /// The predicate guard, with the assay owned by the study the predicate compares rather than
    /// by its sample: the row's own owner and the one the input names are one study.
    PredicateOwnedByStudy,
}

impl Shape {
    const ALL: [Self; 6] = [
        Self::ExistsOnly,
        Self::Predicate,
        Self::Copied,
        Self::CopiedUnguarded,
        Self::CopiedPredicate,
        Self::PredicateOwnedByStudy,
    ];

    fn model(self) -> String {
        let predicate = |text: &str| {
            text.replace(
                "      - name: started\n",
                "      - name: other-study
        when_related: {via: input.sample_id, predicate: study_id != input.study_id}
        error: demo.lab.OtherStudy
        payload:
          demo.lab.OtherStudy: {sample_id: input.sample_id, study_id: input.study_id}
      - name: started\n",
            )
        };
        match self {
            Self::ExistsOnly => LAB.to_owned(),
            Self::Predicate => predicate(LAB),
            Self::CopiedPredicate => predicate(&Self::Copied.model()),
            Self::PredicateOwnedByStudy => predicate(LAB)
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
                ),
            Self::Copied => LAB.replace(
                "sets: {sample_id: input.sample_id, study_id: input.study_id, operator: input.operator}",
                "sets: {sample_id: input.sample_id, study_id: {related: {via: sample_id, field: \
                 study_id}}, operator: input.operator}",
            ),
            Self::CopiedUnguarded => Self::Copied.model().replace(
                "      - name: unknown-sample
        when_related: {via: input.sample_id, exists: false}
        error: demo.lab.SampleNotFound
        payload:
          demo.lab.SampleNotFound: {sample_id: input.sample_id}\n",
                "",
            ),
        }
    }

    fn copies(self) -> bool {
        matches!(
            self,
            Self::Copied | Self::CopiedUnguarded | Self::CopiedPredicate
        )
    }

    fn compares(self) -> bool {
        matches!(
            self,
            Self::Predicate | Self::CopiedPredicate | Self::PredicateOwnedByStudy
        )
    }

    fn guarded(self) -> bool {
        self != Self::CopiedUnguarded
    }
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

/// The `ESS-SYNTH-017` refusals of the view, rendered.
fn unwitnessed(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .filter(|refusal| refusal.code().to_string() == "ESS-SYNTH-017")
        .map(ToString::to_string)
        .filter(|rendered| rendered.contains(VIEW))
        .collect()
}

/// One defect an implementation of the model could have.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// Counts every assay into one group.
    IgnoresKey,
    /// Groups assays by the sample they ran on rather than by study.
    GroupsBySample,
    /// Counts one assay more than the group holds.
    CountsOneMore,
    /// Copies the study of the sample received first, not the one the input names.
    CopiesFirstSample,
    /// Copies the study of the sample received last, not the one the input names.
    CopiesLatestSample,
    /// Counts only the assays of the study the latest assay was started for.
    LatestStudyOnly,
}

impl Mutant {
    /// The mutants that answer `shape` differently from the model.
    fn of(shape: Shape) -> Vec<Self> {
        let mut out = vec![Self::IgnoresKey, Self::GroupsBySample, Self::CountsOneMore];
        if shape.copies() {
            out.extend([Self::CopiesFirstSample, Self::CopiesLatestSample]);
        }
        out
    }
}

type Row = BTreeMap<String, Node>;

struct Lab {
    shape: Shape,
    mutant: Mutant,
    studies: RefCell<Vec<Node>>,
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

fn refusal(command: &CommandRef, name: &str, error: DeclaredErrorValue) -> SemanticCommandResult {
    SemanticCommandResult::took(outcome(command, name)).with_error(error)
}

fn error(name: &str) -> DeclaredErrorValue {
    DeclaredErrorValue::new(name.parse::<ErrorRef>().unwrap())
}

impl Lab {
    fn new(shape: Shape, mutant: Mutant) -> Self {
        Self {
            shape,
            mutant,
            studies: RefCell::default(),
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
        let shown: &[&str] = match name {
            VIEW => &["study_id", "state"],
            BY_OPERATOR => &["operator"],
            COUNT => &[],
            _ => return Vec::new(),
        };
        let mut groups: Vec<(Vec<Node>, Vec<Row>)> = Vec::new();
        let assays = self.assays.borrow();
        let latest = assays.last().map(|assay| assay["study_id"].clone());
        for assay in assays.iter() {
            if self.mutant == Mutant::LatestStudyOnly && Some(&assay["study_id"]) != latest.as_ref()
            {
                continue;
            }
            let key = match self.mutant {
                Mutant::IgnoresKey => Vec::new(),
                Mutant::GroupsBySample if !shown.is_empty() => vec![assay["sample_id"].clone()],
                _ => shown.iter().map(|field| assay[*field].clone()).collect(),
            };
            match groups.iter_mut().find(|(held, _)| *held == key) {
                Some((_, members)) => members.push(assay.clone()),
                None => groups.push((key, vec![assay.clone()])),
            }
        }
        if shown.is_empty() && groups.is_empty() {
            groups.push((Vec::new(), Vec::new()));
        }
        groups
            .into_iter()
            .map(|(_, members)| {
                let count = members.len() + usize::from(self.mutant == Mutant::CountsOneMore);
                let mut row = Row::from([("assays".to_owned(), number(count))]);
                for field in shown {
                    row.insert((*field).to_owned(), members[0][*field].clone());
                }
                row
            })
            .collect()
    }

    fn start(&self, command: &CommandRef, input: &Row) -> SemanticCommandResult {
        let named = input.get("sample_id").cloned().unwrap_or(Node::Null);
        let sent = input.get("study_id").cloned().unwrap_or(Node::Null);
        let samples = self.samples.borrow();
        let Some((_, held)) = samples.iter().find(|(id, _)| *id == named) else {
            if !self.shape.guarded() {
                return SemanticCommandResult::undeclared();
            }
            return refusal(
                command,
                "unknown-sample",
                error("demo.lab.SampleNotFound").with("sample_id", named),
            );
        };
        if self.shape.compares() && *held != sent {
            return refusal(
                command,
                "other-study",
                error("demo.lab.OtherStudy")
                    .with("sample_id", named)
                    .with("study_id", sent),
            );
        }
        let study = if self.shape.copies() {
            let read = match self.mutant {
                Mutant::CopiesFirstSample => samples.first(),
                Mutant::CopiesLatestSample => samples.last(),
                _ => Some(&(named.clone(), held.clone())),
            };
            read.map_or(Node::Null, |(_, study)| study.clone())
        } else {
            sent
        };
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
}

impl ConformanceTarget for Lab {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("lab-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.studies.replace(Vec::new());
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
                self.studies.borrow_mut().push(id.clone());
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

/// The status of each scenario named in `ids`, run from `shape`'s suite against a target with
/// `mutant`.
fn statuses(shape: Shape, mutant: Mutant, ids: &[&str]) -> BTreeMap<String, Status> {
    let mut suite = synthesis(&shape.model()).suite;
    suite
        .scenarios
        .retain(|id, _| ids.contains(&id.to_string().as_str()));
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

fn aggregate_status(shape: Shape, mutant: Mutant) -> Status {
    statuses(shape, mutant, &[AGGREGATE])[AGGREGATE]
}

#[test]
fn a_related_guard_on_the_creating_command_does_not_refuse_the_aggregate() {
    let mut refused = Vec::new();
    for shape in Shape::ALL {
        let result = synthesis(&shape.model());
        let found = unwitnessed(&result);
        if !found.is_empty() {
            refused.push((shape, found));
        }
    }
    assert!(refused.is_empty(), "{refused:#?}");
}

#[test]
fn every_row_starts_on_a_sample_the_scenario_received_before_it() {
    for shape in Shape::ALL {
        let suite = synthesis(&shape.model()).suite;
        let (_, scenario) = suite
            .scenarios
            .iter()
            .find(|(id, _)| id.to_string() == AGGREGATE)
            .unwrap_or_else(|| panic!("{shape:?}: no scenario {AGGREGATE}"));
        let mut captured = Vec::new();
        let mut started = 0;
        for step in &scenario.steps {
            match step {
                ScenarioStep::CaptureInstance {
                    instance, entity, ..
                } if entity.to_string() == "demo.lab.Sample" => captured.push(instance.clone()),
                ScenarioStep::ExecuteCommand { command, input, .. }
                    if command.to_string() == "demo.lab.Start" =>
                {
                    started += 1;
                    match &input["sample_id"] {
                        ScenarioValue::Instance { instance } => assert!(
                            captured.contains(instance),
                            "{shape:?}: `{instance}` is started on before it is received"
                        ),
                        other => panic!("{shape:?}: a start names no received sample: {other:?}"),
                    }
                }
                _ => {}
            }
        }
        assert!(started > 1, "{shape:?}: {started} rows");
    }
}

#[test]
fn the_view_as_specified_passes_and_every_miscount_fails() {
    let mut wrong = Vec::new();
    for shape in Shape::ALL {
        let status = aggregate_status(shape, Mutant::None);
        if status != Status::Passed {
            wrong.push((shape, Mutant::None, status));
        }
        for mutant in Mutant::of(shape) {
            let status = aggregate_status(shape, mutant);
            if status == Status::Passed {
                wrong.push((shape, mutant, status));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:?}");
}

#[test]
fn the_guards_refusals_keep_their_own_scenarios() {
    let exists = statuses(Shape::ExistsOnly, Mutant::None, &[UNKNOWN_SAMPLE]);
    assert_eq!(exists[UNKNOWN_SAMPLE], Status::Passed, "{exists:?}");
    let predicate = statuses(
        Shape::Predicate,
        Mutant::None,
        &[UNKNOWN_SAMPLE, OTHER_STUDY],
    );
    assert!(
        predicate.values().all(|status| *status == Status::Passed),
        "{predicate:?}"
    );
    let copied = statuses(Shape::Copied, Mutant::None, &[UNKNOWN_SAMPLE]);
    assert_eq!(copied[UNKNOWN_SAMPLE], Status::Passed, "{copied:?}");
}

/// A view that does not read the input a predicate guard compares — keyed on another input, or
/// ungrouped — is witnessed in every shape: every row names an arranged study, and the sample it
/// starts on is filed under it. The view as specified passes; a target grouping or counting
/// wrongly fails. Under a predicate the rows are spread over two studies, so a target counting
/// one study's assays alone fails the ungrouped count.
#[test]
fn views_not_reading_the_compared_input_are_witnessed() {
    let mut wrong = Vec::new();
    for shape in Shape::ALL {
        let result = synthesis(&shape.model());
        for (view, mutants) in [
            (
                BY_OPERATOR,
                &[
                    Mutant::IgnoresKey,
                    Mutant::GroupsBySample,
                    Mutant::CountsOneMore,
                ][..],
            ),
            (
                COUNT,
                if shape.compares() {
                    &[Mutant::LatestStudyOnly][..]
                } else {
                    &[][..]
                },
            ),
        ] {
            let id = format!("{view}/aggregate");
            if !result
                .suite
                .scenarios
                .keys()
                .any(|key| key.to_string() == id)
            {
                let refusals: Vec<String> = result
                    .refusals
                    .iter()
                    .map(ToString::to_string)
                    .filter(|rendered| rendered.contains(&id))
                    .collect();
                wrong.push(format!("{shape:?} {view}: refused {refusals:?}"));
                continue;
            }
            let honest = statuses(shape, Mutant::None, &[&id])[&id];
            if honest != Status::Passed {
                wrong.push(format!("{shape:?} {view}: the model is {honest:?}"));
            }
            for mutant in mutants {
                let status = statuses(shape, *mutant, &[&id])[&id];
                if status == Status::Passed {
                    wrong.push(format!("{shape:?} {view}: {mutant:?} passes"));
                }
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
