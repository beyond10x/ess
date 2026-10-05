//! The generated Go runtime checks an invariant reading `defined()` over an `Optional` aggregate
//! (suite/26, beyond10x/ess#176) and gives the reference verdicts (beyond10x/ess#188).
//!
//! The target is `tests/defined_over_optional_aggregates.rs`'s, recorded once and replayed to the
//! Go runtime, over the specification as written and over the one whose `resumed` forgets to say
//! it clears the metrics — where only the invariant catches the mutant.

mod support_go;

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::{
    ir::EssIr,
    refs::{CommandRef, OutcomeRef},
    resolve::compile,
    source::SourceMap,
};
use ess_conformance::{synthesize::synthesize, target::*, ConformanceSuite};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::node::Node;

const QUEUE: &str = include_str!("fixtures/defined-over-optional-aggregates.yaml");
const AFTER_RESUMED: &str = "demo.queue.Queue/invariant/after/demo.queue.ResumeQueue/resumed";

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("queue.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

fn suite_of(text: &str) -> ConformanceSuite {
    let synthesis = synthesize(&ir(text));
    assert!(synthesis.refusals.is_empty(), "{:#?}", synthesis.refusals);
    assert_eq!(synthesis.suite.provenance.suite_version.major(), 34);
    synthesis.suite
}

#[test]
fn go_gives_the_reference_verdict_over_optional_aggregates() {
    let forgetful = QUEUE.replace("        sets: {metrics: {cleared: true}}\n", "");
    assert_ne!(forgetful, QUEUE);
    for (label, text) in [("written", QUEUE.to_owned()), ("forgetful", forgetful)] {
        let suite = suite_of(&text);
        for mutant in [Mutant::None, Mutant::KeepsMetricsOnResume] {
            let verdicts = support_go::assert_parity(
                &format!("defined-{label}-{mutant:?}").to_lowercase(),
                &suite,
                Queues::new(mutant),
            );
            let wrong = support_go::not_passed(&verdicts);
            match (label, mutant) {
                ("written", Mutant::None) => assert!(wrong.is_empty(), "{verdicts:?}"),
                ("forgetful", Mutant::KeepsMetricsOnResume) => {
                    assert_eq!(wrong, [AFTER_RESUMED], "{verdicts:?}");
                }
                _ => assert!(!wrong.is_empty(), "{label} {mutant:?}: {verdicts:?}"),
            }
        }
    }
}

/// One way a queue implementation could treat its metrics.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mutant {
    None,
    /// `resume` moves the queue and leaves the metrics it was paused with.
    KeepsMetricsOnResume,
}

type Row = BTreeMap<String, Node>;

struct Queues {
    mutant: Mutant,
    rows: RefCell<Vec<Row>>,
    minted: Cell<u64>,
}

impl Queues {
    fn new(mutant: Mutant) -> Self {
        Self {
            mutant,
            rows: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

impl ConformanceTarget for Queues {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("queues-fixture", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let token = ess_primitives::consistency::ConsistencyToken::new(format!(
            "seq:{}",
            self.minted.get()
        ))
        .unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        // The row a command acts on where it is in `from`, and otherwise the declared conflict —
        // naming the state it is in, where there is a row at all.
        let find = |rows: &mut Vec<Row>, from: &str| {
            let id = request.input["queue_id"].clone();
            let index = rows.iter().position(|row| row["queue_id"] == id);
            let state = index.map(|index| rows[index]["state"].clone());
            match (index, state) {
                (Some(index), Some(state)) if state == Node::Text(from.into()) => Ok((index, id)),
                (_, state) => {
                    let error =
                        DeclaredErrorValue::new("demo.queue.QueueStateConflict".parse().unwrap());
                    let error = match state {
                        Some(state) => error.with("state", state),
                        None => error,
                    };
                    Err(Box::new(
                        SemanticCommandResult::took(outcome(&command, "wrong-state"))
                            .with_error(error),
                    ))
                }
            }
        };
        let result = match command.to_string().as_str() {
            "demo.queue.OpenQueue" => {
                let id = Node::Text(format!("00000000-0000-4000-8000-{:012}", self.minted.get()));
                let mut row = Row::new();
                row.insert("queue_id".into(), id.clone());
                row.insert("state".into(), Node::Text("Running".into()));
                rows.push(row);
                SemanticCommandResult::took(outcome(&command, "opened")).emitting(
                    ObservedEvent::new("demo.queue.QueueOpened".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            "demo.queue.PauseQueue" => {
                let (index, id) = match find(&mut rows, "Running") {
                    Ok(found) => found,
                    Err(refused) => return Ok(refused.with_consistency(token)),
                };
                rows[index].insert("state".into(), Node::Text("Paused".into()));
                rows[index].insert("metrics".into(), request.input["metrics"].clone());
                SemanticCommandResult::took(outcome(&command, "paused")).emitting(
                    ObservedEvent::new("demo.queue.QueuePaused".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            "demo.queue.ResumeQueue" => {
                let (index, id) = match find(&mut rows, "Paused") {
                    Ok(found) => found,
                    Err(refused) => return Ok(refused.with_consistency(token)),
                };
                rows[index].insert("state".into(), Node::Text("Running".into()));
                if self.mutant != Mutant::KeepsMetricsOnResume {
                    rows[index].insert("metrics".into(), Node::Null);
                }
                SemanticCommandResult::took(outcome(&command, "resumed")).emitting(
                    ObservedEvent::new("demo.queue.QueueResumed".parse().unwrap())
                        .with("queue_id", id),
                )
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, request: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        match request.view.to_string().as_str() {
            "demo.queue.QueueById" => Ok(SemanticViewResult::of(self.rows.borrow().clone())),
            other => Err(TargetError::unsupported("view", other)),
        }
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
