//! A `when_subject` branch whose predicate is an entity invariant's upper bound, declared before a
//! second `when_subject` branch, is synthesized (beyond10x/ess#251).
//!
//! `Revise` declares `exhausted` (`revision >= MAX`) before `stale` (`revision >= input.revision`),
//! and the entity holds `revision lte MAX`. Through 0.42.0 the `exhausted` witness sent
//! `revision: MAX + 1`, an input that also refutes `stale`. Since inputs are held to the entity
//! invariants of the branches they can reach (0.43.0, beyond10x/ess#234), that input is never
//! sent, and synthesis refused `exhausted` (ESS-SYNTH-003): it still asked for an input refuting
//! the later-declared `stale`. Held-state branches answer in declaration order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order"), so on a row at
//! the bound `exhausted` answers whatever `stale` says, and only earlier-declared branches need
//! refuting.
#![allow(clippy::too_many_lines)]

use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, refs::OutcomeRef, resolve::compile, source::SourceMap};
use ess_conformance::{
    report::{ConformanceStatus, Status},
    synthesize::{synthesize, Synthesis},
    target::*,
    AdmittedSuite, Runner,
};
use ess_domain::{
    command::OutcomeName,
    spec::{RawSpecFile, Specification},
    system::Source,
};
use ess_primitives::{consistency::ConsistencyToken, node::Node};

/// The invariant's upper bound: 2^53 - 1, the largest integer a binary64 carries exactly.
const MAX: i64 = 9_007_199_254_740_991;

/// The #251 reproduction, as one document.
const POLICY: &str = r"format: ess/15
system: repro
version: v1
domain: repro.policy
summary: Minimal reproduction of an ESS-SYNTH-003 refusal on a stored-revision bound.
entities:
  - name: repro.policy.Policy
    identity: {name: policy_id, type: Uuid}
    fields:
      - {name: revision, type: Integer}
    invariants:
      - revision: {gte: 1, lte: 9007199254740991}
    lifecycle:
      initial: Configured
      states: [Configured]
      terminal: [Configured]
      transitions:
        - {name: revise, from: [Configured], to: Configured}
commands:
  - name: repro.policy.Configure
    input:
      - {name: policy_id, type: Uuid}
      - {name: revision, type: Integer}
    outcomes:
      - name: configured
        creates: repro.policy.Policy
        instance: policy_id
        emits: [repro.policy.Configured]
        payload:
          repro.policy.Configured:
            policy_id: input.policy_id
        sets:
          revision: input.revision
  - name: repro.policy.Revise
    input:
      - {name: policy_id, type: Uuid}
      - {name: revision, type: Integer}
    outcomes:
      - name: exhausted
        when_subject:
          predicate: revision >= 9007199254740991
        error: repro.policy.Exhausted
      - name: stale
        when_subject:
          predicate: revision >= input.revision
        error: repro.policy.Exhausted
      - name: revised
        moves: repro.policy.Policy.revise
        instance: policy_id
        emits: [repro.policy.Revised]
        payload:
          repro.policy.Revised:
            policy_id: input.policy_id
        sets:
          revision: input.revision
events:
  - name: repro.policy.Configured
    fields: [{name: policy_id, type: Uuid}]
  - name: repro.policy.Revised
    fields: [{name: policy_id, type: Uuid}]
errors:
  - name: repro.policy.Exhausted
    fields: [{name: policy_id, type: Uuid}]
views:
  - name: repro.policy.Policies
    source: repro.policy.Policy
    consistency: read_your_writes
    fields:
      - {name: policy_id, type: Uuid}
      - {name: revision, type: Integer}
      - {name: state, type: repro.policy.Policy.State}
actors:
  - name: repro.policy.Host
    may: [repro.policy.Configure, repro.policy.Revise]
";

const EXHAUSTED: &str = "repro.policy.Revise/outcome/exhausted";

/// The seven scenarios 0.40.0 through 0.42.0 synthesized for the reproduction.
const EXPECTED: [&str; 7] = [
    "repro.policy.Configure/outcome/configured",
    "repro.policy.Policy/invariant/after/repro.policy.Configure/configured",
    "repro.policy.Policy/invariant/after/repro.policy.Revise/revised",
    "repro.policy.Policy/transition/revise/by/repro.policy.Revise/revised",
    EXHAUSTED,
    "repro.policy.Revise/outcome/revised",
    "repro.policy.Revise/outcome/stale",
];

fn compiled(text: &str) -> Synthesis {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("policy.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    let ir: EssIr = compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"));
    synthesize(&ir)
}

fn refusals(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .refusals
        .iter()
        .map(|refusal| format!("{}: {refusal}", refusal.cause.code()))
        .collect()
}

fn scenario_ids(synthesis: &Synthesis) -> BTreeSet<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn integer(node: &Node) -> i64 {
    let Node::Number(number) = node else {
        panic!("an integer: {node:?}")
    };
    #[allow(clippy::cast_possible_truncation)]
    let whole = number.get() as i64;
    whole
}

/// The `revision` literals the scenario `id` sends, in step order, per command (`Configure` or
/// `Revise`), read off the suite's JSON.
fn sent(synthesis: &Synthesis, id: &str) -> Vec<(String, i64)> {
    let json = serde_json::to_value(&synthesis.suite).unwrap();
    let scenario = &json["scenarios"][id];
    assert!(scenario.is_object(), "no scenario {id}");
    scenario["steps"]
        .as_array()
        .unwrap()
        .iter()
        .filter(|step| step["step"] == "execute_command")
        .filter_map(|step| {
            let command = step["command"].as_str()?.rsplit('.').next()?.to_owned();
            let value = step["input"]["revision"]["value"].as_f64()?;
            #[allow(clippy::cast_possible_truncation)]
            Some((command, value as i64))
        })
        .collect()
}

/// What `Revise` answers for a row holding `stored` and an input carrying `input`.
type Decide = fn(stored: i64, input: i64) -> &'static str;

/// The model's own answer: `exhausted` first, then `stale`, then `revised`.
fn declared(stored: i64, input: i64) -> &'static str {
    if stored >= MAX {
        "exhausted"
    } else if stored >= input {
        "stale"
    } else {
        "revised"
    }
}

/// A hand-written in-memory target of the model, answering `Revise` with `decide`.
struct Policies {
    decide: Decide,
    rows: RefCell<Vec<BTreeMap<String, Node>>>,
    sequence: Cell<u64>,
}

impl Policies {
    fn new(decide: Decide) -> Self {
        Self {
            decide,
            rows: RefCell::default(),
            sequence: Cell::new(0),
        }
    }
}

impl ConformanceTarget for Policies {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("policies-251", "1"))
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
        let command = request.command.clone();
        let took = |name: &str| {
            SemanticCommandResult::took(OutcomeRef::new(
                command.clone(),
                OutcomeName::new(name).unwrap(),
            ))
        };
        let event = |name: &str, id: Node| {
            ObservedEvent::new(format!("repro.policy.{name}").parse().unwrap())
                .with("policy_id", id)
        };
        let id = request.input["policy_id"].clone();
        let revision = request.input["revision"].clone();
        let mut rows = self.rows.borrow_mut();
        let result = match command.to_string().as_str() {
            "repro.policy.Configure" => {
                let mut row = BTreeMap::new();
                row.insert("policy_id".to_owned(), id.clone());
                row.insert("revision".to_owned(), revision);
                row.insert("state".to_owned(), Node::Text("Configured".into()));
                rows.push(row);
                took("configured").emitting(event("Configured", id))
            }
            "repro.policy.Revise" => {
                let Some(row) = rows.iter_mut().find(|row| row["policy_id"] == id) else {
                    self.sequence.set(self.sequence.get() + 1);
                    return Ok(SemanticCommandResult::undeclared().with_consistency(
                        ConsistencyToken::new(format!("seq:{}", self.sequence.get())).unwrap(),
                    ));
                };
                match (self.decide)(integer(&row["revision"]), integer(&revision)) {
                    "revised" => {
                        row.insert("revision".to_owned(), revision);
                        took("revised").emitting(event("Revised", id))
                    }
                    refused => took(refused).with_error(DeclaredErrorValue::new(
                        "repro.policy.Exhausted".parse().unwrap(),
                    )),
                }
            }
            other => return Err(TargetError::unsupported("command", other)),
        };
        self.sequence.set(self.sequence.get() + 1);
        Ok(result.with_consistency(
            ConsistencyToken::new(format!("seq:{}", self.sequence.get())).unwrap(),
        ))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().clone()))
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

/// The scenarios of `synthesis` that do not pass against a target deciding `Revise` with `decide`.
fn failing(synthesis: &Synthesis, decide: Decide) -> Vec<String> {
    let suite = &synthesis.suite;
    let admitted = AdmittedSuite::from_suite(suite).unwrap();
    let report = Runner::for_suite(suite)
        .run_admitted(&admitted, &Policies::new(decide))
        .into_report();
    let failed: Vec<String> = report
        .scenarios
        .iter()
        .filter(|scenario| scenario.status != Status::Passed)
        .map(|scenario| format!("{}: {:?}", scenario.scenario, scenario.status))
        .collect();
    assert_eq!(
        report.status == ConformanceStatus::Passed,
        failed.is_empty()
    );
    failed
}

/// #251's acceptance: the reproduction synthesizes the seven scenarios it did through 0.42.0 and
/// refuses nothing.
#[test]
fn issue_251_the_reproduction_synthesizes_seven_scenarios_and_no_refusal() {
    let synthesis = compiled(POLICY);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let expected: BTreeSet<String> = EXPECTED.iter().map(|id| (*id).to_owned()).collect();
    assert_eq!(scenario_ids(&synthesis), expected);
}

/// #251's acceptance: `exhausted` is witnessed on a row at the invariant's bound, with an input the
/// invariant admits — so `stale` holds there too and the declared order decides.
#[test]
fn issue_251_exhausted_is_witnessed_at_the_bound() {
    let synthesis = compiled(POLICY);
    let steps = sent(&synthesis, EXHAUSTED);
    let configured: Vec<i64> = steps
        .iter()
        .filter(|(command, _)| command == "Configure")
        .map(|(_, value)| *value)
        .collect();
    assert_eq!(configured, [MAX], "{steps:?}");
    let (_, revised) = steps
        .iter()
        .rev()
        .find(|(command, _)| command == "Revise")
        .unwrap_or_else(|| panic!("{steps:?}"));
    assert!(
        (1..=MAX).contains(revised),
        "the witness sends a revision the invariant refuses: {steps:?}"
    );
}

/// A hand-written target answering in declaration order passes the whole suite; one answering
/// `stale` before `exhausted`, or never `exhausted`, fails it.
#[test]
fn issue_251_the_suite_passes_a_correct_target_and_fails_order_mutants() {
    let synthesis = compiled(POLICY);
    assert!(synthesis.refusals.is_empty(), "{:#?}", refusals(&synthesis));
    let failed = failing(&synthesis, declared);
    assert!(failed.is_empty(), "the correct target failed: {failed:#?}");
    let mutants: [(&str, Decide); 2] = [
        ("stale before exhausted", |stored, input| {
            if stored >= input {
                "stale"
            } else if stored >= MAX {
                "exhausted"
            } else {
                "revised"
            }
        }),
        ("never exhausted", |stored, input| {
            if stored >= input {
                "stale"
            } else {
                "revised"
            }
        }),
    ];
    let survived: Vec<&str> = mutants
        .iter()
        .filter(|(_, decide)| failing(&synthesis, *decide).is_empty())
        .map(|(name, _)| *name)
        .collect();
    assert!(survived.is_empty(), "mutants survived: {survived:?}");
}

/// The other half of the class: a held-state branch declared *before* the branch under test is
/// still refuted. With `stale` declared first, every `exhausted` witness must send an input above
/// the row, which the invariant refuses at the bound — so `exhausted` is never witnessed on a row
/// where `stale` would answer first.
#[test]
fn issue_251_an_earlier_declared_held_state_branch_is_still_refuted() {
    let exhausted = "      - name: exhausted\n        when_subject:\n          predicate: revision >= 9007199254740991\n        error: repro.policy.Exhausted\n";
    let stale = "      - name: stale\n        when_subject:\n          predicate: revision >= input.revision\n        error: repro.policy.Exhausted\n";
    let swapped = POLICY.replace(
        &format!("{exhausted}{stale}"),
        &format!("{stale}{exhausted}"),
    );
    assert_ne!(swapped, POLICY);
    let synthesis = compiled(&swapped);
    if scenario_ids(&synthesis).contains(EXHAUSTED) {
        let steps = sent(&synthesis, EXHAUSTED);
        let stored = steps
            .iter()
            .filter(|(command, _)| command == "Configure")
            .map(|(_, value)| *value)
            .next_back()
            .unwrap_or_else(|| panic!("{steps:?}"));
        let (_, input) = steps
            .iter()
            .rev()
            .find(|(command, _)| command == "Revise")
            .unwrap_or_else(|| panic!("{steps:?}"));
        assert!(
            stored < *input,
            "`exhausted` witnessed on a row the earlier `stale` claims: {steps:?}"
        );
    } else {
        assert!(
            refusals(&synthesis)
                .iter()
                .any(|refusal| refusal.contains(EXHAUSTED) && refusal.contains("ESS-SYNTH-003")),
            "{:#?}",
            refusals(&synthesis)
        );
    }
}
