//! Adversary, pass 1, `story:feature-request-278` (beyond10x/ess#278).
//!
//! The unit's tests run the #278 suite against the model interpreter, which reports the scenarios
//! that matter `Unsupported` (a stored-field guard). These run the same suites against a
//! hand-written `Rollout` target that answers in the precedence order
//! (`docs/design/cross-record-and-stored-field-guards.md`, "The precedence order": accepting
//! branches in declaration order, then `wrong_state` where the selected branch's move does not
//! start from the held state), and against mutants of it: one answering the last declared of two
//! overlapping branches, one refusing an overlap as ambiguous. They vary the #278 shape where the
//! unit's tests do not: a non-moving twin declared after the mixed branch (the third pass of
//! `refusal_input` relies on the mixed branch's stored guard holding on the row), a non-moving twin
//! declared before it, and the twin written `result in [Healthy]` rather than `result == Healthy`.

use std::cell::{Cell, RefCell};
use std::collections::BTreeMap;

use ess_compiler::ir::EssIr;
use ess_compiler::refs::{CommandRef, OutcomeRef};
use ess_compiler::{resolve::compile, source::SourceMap};
use ess_conformance::report::Status;
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::target::*;
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::{command::OutcomeName, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::node::Node;

const HEAD: &str = r"format: ess/19
system: demo
version: v1
domain: demo.rollout
summary: A deployment held for promotion unless it promotes itself.
types:
  - {name: demo.rollout.Result, kind: enum, variants: [Healthy, Regression]}
entities:
  - name: demo.rollout.Deployment
    identity: {name: deployment_id, type: Uuid}
    fields:
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
    lifecycle:
      initial: Canary
      states: [Canary, Held, Promoted, RolledBack]
      terminal: [Promoted, RolledBack]
      transitions:
        - {name: hold, from: [Canary], to: Held}
        - {name: promote, from: [Canary], to: Promoted}
        - {name: rollback, from: [Canary], to: RolledBack}
        - {name: approve, from: [Held], to: Promoted}
        - {name: reject, from: [Held], to: RolledBack}
errors:
  - name: demo.rollout.Conflict
    summary: wrong state
    fields:
      - {name: state, type: demo.rollout.Deployment.State}
events:
  - name: demo.rollout.Moved
    fields: [{name: deployment_id, type: Uuid}]
actors:
  - name: demo.rollout.Engine
    may: [demo.rollout.Deploy, demo.rollout.RecordResult, demo.rollout.Decide]
commands:
  - name: demo.rollout.Deploy
    input:
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
    outcomes:
      - name: deployed
        creates: demo.rollout.Deployment
        instance: deployment_id
        sets:
          auto_promote: input.auto_promote
          automatic_rollback: input.automatic_rollback
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: {generated: true}}
  - name: demo.rollout.RecordResult
    input:
      - {name: deployment_id, type: Uuid}
      - {name: result, type: demo.rollout.Result}
    outcomes:
";

const HELD: &str = r#"      - name: held-for-promotion
        when: result == Healthy
        when_subject:
          predicate: {any: ["not defined(auto_promote)", auto_promote == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
"#;

/// `HELD` with its input guard written as a one-element list: `result in [Healthy]`.
const HELD_IN: &str = r#"      - name: held-for-promotion
        when: {result: [Healthy]}
        when_subject:
          predicate: {any: ["not defined(auto_promote)", auto_promote == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
"#;

const PROMOTED: &str = r"      - name: promoted
        when: result == Healthy
        moves: demo.rollout.Deployment.promote
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
";

/// A twin of `HELD`'s input guard that moves nothing, so it answers in every held state.
const NOTED: &str = r"      - name: noted
        when: result == Healthy
        preserves: demo.rollout.Deployment
        instance: deployment_id
";

const TAIL: &str = r#"      - name: held-for-rollback
        when: result == Regression
        when_subject:
          predicate: {any: ["not defined(automatic_rollback)", automatic_rollback == false]}
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: rolled-back
        moves: demo.rollout.Deployment.rollback
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: wrong-state
        wrong_state: true
        error: demo.rollout.Conflict
  - name: demo.rollout.Decide
    input:
      - {name: deployment_id, type: Uuid}
      - {name: decision, type: demo.rollout.Result}
    outcomes:
      - name: promoted
        when: decision == Healthy
        moves: demo.rollout.Deployment.approve
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: rolled-back
        moves: demo.rollout.Deployment.reject
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
      - name: wrong-state
        wrong_state: true
        error: demo.rollout.Conflict
views:
  - name: demo.rollout.Deployments
    source: demo.rollout.Deployment
    consistency: read_your_writes
    fields:
      - {name: deployment_id, type: Uuid}
      - {name: auto_promote, type: Optional<Boolean>}
      - {name: automatic_rollback, type: Optional<Boolean>}
      - {name: state, type: demo.rollout.Deployment.State}
"#;

const HELD_SCENARIO: &str = "demo.rollout.RecordResult/outcome/held-for-promotion";
const HELD_REFUSES: &str = "demo.rollout.Deployment/state/Held/refuses/demo.rollout.RecordResult";

fn spec(branches: &[&str]) -> String {
    format!("{HEAD}{}{TAIL}", branches.concat())
}

/// Every deployment created with `auto_promote: true`: no row admits `held-for-promotion`.
fn pinned(text: &str) -> String {
    let pinned = text.replace(
        "          auto_promote: input.auto_promote\n",
        "          auto_promote: 'true'\n",
    );
    assert_ne!(pinned, text);
    pinned
}

/// The lifecycle without `promote`, for a `RecordResult` with no branch taking it.
fn unpromoted(text: &str) -> String {
    let unpromoted = text.replace(
        "        - {name: promote, from: [Canary], to: Promoted}\n",
        "",
    );
    assert_ne!(unpromoted, text);
    unpromoted
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    let spec = Specification::assemble([(Source::new("rollout.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}\n{text}"));
    compile(&spec, &SourceMap::new()).unwrap_or_else(|error| panic!("{error:?}"))
}

fn refusals(result: &Synthesis) -> Vec<String> {
    result
        .refusals
        .iter()
        .map(|refusal| {
            let scenario = refusal
                .scenario
                .as_ref()
                .map_or_else(String::new, ToString::to_string);
            format!("{scenario}: {}: {}", refusal.cause.code(), refusal.cause)
        })
        .collect()
}

fn has(result: &Synthesis, id: &str) -> bool {
    result
        .suite
        .scenarios
        .keys()
        .any(|key| key.to_string() == id)
}

// ---- a target answering in the precedence order -----------------------------------------------

#[derive(Clone, Copy, PartialEq, Eq)]
enum Guard {
    Healthy,
    Regression,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Row {
    /// `not defined(auto_promote) or auto_promote == false`.
    AutoPromoteOff,
    /// `not defined(automatic_rollback) or automatic_rollback == false`.
    RollbackOff,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Effect {
    /// A move from `Canary` to the named state.
    FromCanary(&'static str),
    /// Nothing moves: the branch answers in every held state.
    Preserves,
}

#[derive(Clone, Copy)]
struct Branch {
    name: &'static str,
    when: Guard,
    row: Option<Row>,
    effect: Effect,
}

const B_HELD: Branch = Branch {
    name: "held-for-promotion",
    when: Guard::Healthy,
    row: Some(Row::AutoPromoteOff),
    effect: Effect::FromCanary("Held"),
};
const B_PROMOTED: Branch = Branch {
    name: "promoted",
    when: Guard::Healthy,
    row: None,
    effect: Effect::FromCanary("Promoted"),
};
const B_NOTED: Branch = Branch {
    name: "noted",
    when: Guard::Healthy,
    row: None,
    effect: Effect::Preserves,
};
const B_ROLLBACK: Branch = Branch {
    name: "held-for-rollback",
    when: Guard::Regression,
    row: Some(Row::RollbackOff),
    effect: Effect::FromCanary("Held"),
};

/// How the target chooses among the accepting branches whose guards hold.
#[derive(Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// The first declared answers (the precedence order, step 5).
    FirstDeclared,
    /// Mutant: the last declared answers.
    LastDeclared,
    /// Mutant: two branches holding at once are refused as ambiguous.
    Unique,
}

struct Rollout {
    /// `RecordResult`'s guarded accepting branches, in declaration order.
    branches: Vec<Branch>,
    mode: Mode,
    rows: RefCell<BTreeMap<String, BTreeMap<String, Node>>>,
    events: RefCell<Vec<ObservedEvent>>,
    minted: Cell<u32>,
}

impl Rollout {
    fn new(branches: &[Branch], mode: Mode) -> Self {
        Self {
            branches: branches.to_vec(),
            mode,
            rows: RefCell::default(),
            events: RefCell::default(),
            minted: Cell::new(0),
        }
    }
}

fn outcome(command: &CommandRef, name: &str) -> OutcomeRef {
    OutcomeRef::new(command.clone(), OutcomeName::new(name).unwrap())
}

fn text(node: Option<&Node>) -> String {
    node.and_then(Node::as_text).unwrap_or_default().to_owned()
}

/// `not defined(field) or field == false` over a stored row.
fn off(row: &BTreeMap<String, Node>, field: &str) -> bool {
    match row.get(field) {
        None | Some(Node::Null) => true,
        Some(Node::Bool(value)) => !value,
        Some(Node::Text(value)) => value == "false",
        Some(_) => false,
    }
}

impl ConformanceTarget for Rollout {
    fn identity(&self) -> Result<ImplementationIdentity, TargetError> {
        Ok(ImplementationIdentity::new("adversary-278-rollout", "1"))
    }
    fn begin_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        self.rows.replace(BTreeMap::new());
        self.events.replace(Vec::new());
        Ok(())
    }
    fn end_scenario(&self, _: &ScenarioContext) -> Result<(), TargetError> {
        Ok(())
    }
    #[allow(clippy::too_many_lines)]
    fn execute_command(
        &self,
        request: SemanticCommandRequest,
    ) -> Result<SemanticCommandResult, TargetError> {
        self.minted.set(self.minted.get() + 1);
        let n = self.minted.get();
        let token = ess_primitives::consistency::ConsistencyToken::new(format!("seq:{n}")).unwrap();
        let command = request.command.clone();
        let mut rows = self.rows.borrow_mut();
        let moved = |id: &str| {
            let event = ObservedEvent::new("demo.rollout.Moved".parse().unwrap())
                .with("deployment_id", Node::Text(id.into()));
            self.events.borrow_mut().push(event.clone());
            event
        };
        let conflict = |state: &str| {
            SemanticCommandResult::took(outcome(&command, "wrong-state")).with_error(
                DeclaredErrorValue::new("demo.rollout.Conflict".parse().unwrap())
                    .with("state", Node::Text(state.to_owned())),
            )
        };
        let result = match command.to_string().as_str() {
            "demo.rollout.Deploy" => {
                let id = format!("00000000-0000-4000-8000-{n:012}");
                let mut row = BTreeMap::from([
                    ("deployment_id".to_owned(), Node::Text(id.clone())),
                    ("state".to_owned(), Node::Text("Canary".to_owned())),
                ]);
                for field in ["auto_promote", "automatic_rollback"] {
                    if let Some(value) = request.input.get(field) {
                        if *value != Node::Null {
                            row.insert(field.to_owned(), value.clone());
                        }
                    }
                }
                rows.insert(id.clone(), row);
                SemanticCommandResult::took(outcome(&command, "deployed")).emitting(moved(&id))
            }
            "demo.rollout.RecordResult" => {
                let id = text(request.input.get("deployment_id"));
                let Some(row) = rows.get_mut(&id) else {
                    // No record carries the identity: the model answers `wrong-state`.
                    return Ok(
                        SemanticCommandResult::took(outcome(&command, "wrong-state"))
                            .with_error(DeclaredErrorValue::new(
                                "demo.rollout.Conflict".parse().unwrap(),
                            ))
                            .with_consistency(token),
                    );
                };
                let result = text(request.input.get("result"));
                let state = text(row.get("state"));
                let holding: Vec<Branch> = self
                    .branches
                    .iter()
                    .copied()
                    .filter(|branch| {
                        let input = match branch.when {
                            Guard::Healthy => result == "Healthy",
                            Guard::Regression => result == "Regression",
                        };
                        let stored = match branch.row {
                            None => true,
                            Some(Row::AutoPromoteOff) => off(row, "auto_promote"),
                            Some(Row::RollbackOff) => off(row, "automatic_rollback"),
                        };
                        input && stored
                    })
                    .collect();
                let chosen = match (self.mode, holding.as_slice()) {
                    (_, []) => None,
                    (Mode::Unique, [_, _, ..]) => {
                        return Ok(conflict(&state).with_consistency(token));
                    }
                    (Mode::LastDeclared, [.., last]) => Some(*last),
                    (_, [first, ..]) => Some(*first),
                };
                match chosen {
                    Some(Branch {
                        name,
                        effect: Effect::Preserves,
                        ..
                    }) => SemanticCommandResult::took(outcome(&command, name)),
                    Some(Branch {
                        name,
                        effect: Effect::FromCanary(to),
                        ..
                    }) => {
                        if state == "Canary" {
                            row.insert("state".to_owned(), Node::Text(to.to_owned()));
                            SemanticCommandResult::took(outcome(&command, name))
                                .emitting(moved(&id))
                        } else {
                            conflict(&state)
                        }
                    }
                    None => {
                        if state == "Canary" {
                            row.insert("state".to_owned(), Node::Text("RolledBack".to_owned()));
                            SemanticCommandResult::took(outcome(&command, "rolled-back"))
                                .emitting(moved(&id))
                        } else {
                            conflict(&state)
                        }
                    }
                }
            }
            "demo.rollout.Decide" => {
                let id = text(request.input.get("deployment_id"));
                let Some(row) = rows.get_mut(&id) else {
                    // No record carries the identity: the model answers `wrong-state`.
                    return Ok(
                        SemanticCommandResult::took(outcome(&command, "wrong-state"))
                            .with_error(DeclaredErrorValue::new(
                                "demo.rollout.Conflict".parse().unwrap(),
                            ))
                            .with_consistency(token),
                    );
                };
                let state = text(row.get("state"));
                if state == "Held" {
                    let (name, to) = if text(request.input.get("decision")) == "Healthy" {
                        ("promoted", "Promoted")
                    } else {
                        ("rolled-back", "RolledBack")
                    };
                    row.insert("state".to_owned(), Node::Text(to.to_owned()));
                    SemanticCommandResult::took(outcome(&command, name)).emitting(moved(&id))
                } else {
                    conflict(&state)
                }
            }
            other => panic!("no command {other}"),
        };
        Ok(result.with_consistency(token))
    }
    fn query_view(&self, _: SemanticViewRequest) -> Result<SemanticViewResult, TargetError> {
        Ok(SemanticViewResult::of(self.rows.borrow().values().cloned()))
    }
    fn observe_events(
        &self,
        request: EventObservationRequest,
    ) -> Result<Vec<ObservedEvent>, TargetError> {
        let wanted = request.event.to_string();
        Ok(self
            .events
            .borrow()
            .iter()
            .filter(|event| format!("{event:?}").contains(&wanted))
            .cloned()
            .collect())
    }
    fn configure_external_outcome(&self, _: ExternalOutcomeControl) -> Result<(), TargetError> {
        panic!("nothing here is externally decided")
    }
    fn redeliver_event(&self, _: RedeliveryRequest) -> Result<(), TargetError> {
        panic!("no bindings")
    }
}

/// Every scenario of `result`'s suite run against `target`, with its status.
fn run(result: &Synthesis, target: &Rollout) -> Vec<(String, Status, String)> {
    let admitted =
        AdmittedSuite::from_suite(&result.suite).unwrap_or_else(|error| panic!("{error}"));
    Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, target)
        .into_report()
        .scenarios
        .into_iter()
        .map(|scenario| {
            let detail = format!("{scenario:?}");
            (scenario.scenario.to_string(), scenario.status, detail)
        })
        .collect()
}

fn not_passed(results: &[(String, Status, String)]) -> Vec<String> {
    results
        .iter()
        .filter(|(_, status, _)| *status != Status::Passed)
        .map(|(id, status, detail)| format!("{id}: {status:?}: {detail}"))
        .collect()
}

fn failed_ids(results: &[(String, Status, String)]) -> Vec<String> {
    results
        .iter()
        .filter(|(_, status, _)| *status == Status::Failed)
        .map(|(id, _, _)| id.clone())
        .collect()
}

// ---- 1. FirstDeclared semantics on the unit's own #278 shape ----------------------------------

/// Every scenario of the #278 suite passes a target that answers the first declared branch whose
/// guards hold.
#[test]
fn adversary_278_a_first_declared_target_passes_the_278_suite() {
    let result = synthesize(&ir(&spec(&[HELD, PROMOTED])));
    assert!(has(&result, HELD_SCENARIO), "{:#?}", refusals(&result));
    assert!(has(&result, HELD_REFUSES), "{:#?}", refusals(&result));
    let target = Rollout::new(&[B_HELD, B_PROMOTED, B_ROLLBACK], Mode::FirstDeclared);
    let results = run(&result, &target);
    assert_eq!(not_passed(&results), Vec::<String>::new());
}

/// A target answering the last declared of `held-for-promotion` and `promoted` fails the suite at
/// the branch #278 made synthesizable.
#[test]
fn adversary_278_a_last_declared_target_fails_the_278_suite() {
    let result = synthesize(&ir(&spec(&[HELD, PROMOTED])));
    let target = Rollout::new(&[B_HELD, B_PROMOTED, B_ROLLBACK], Mode::LastDeclared);
    let failed = failed_ids(&run(&result, &target));
    assert!(
        failed.iter().any(|id| id == HELD_SCENARIO),
        "the mutant passes {HELD_SCENARIO}; failed: {failed:#?}"
    );
}

/// A target refusing the overlap as ambiguous fails the suite, at the branch and at the
/// wrong-state scenario the third pass of `refusal_input` witnessed on the overlapping row.
#[test]
fn adversary_278_a_unique_target_fails_the_278_suite() {
    let result = synthesize(&ir(&spec(&[HELD, PROMOTED])));
    let target = Rollout::new(&[B_HELD, B_PROMOTED, B_ROLLBACK], Mode::Unique);
    let failed = failed_ids(&run(&result, &target));
    assert!(
        failed.iter().any(|id| id == HELD_SCENARIO),
        "the mutant passes {HELD_SCENARIO}; failed: {failed:#?}"
    );
}

// ---- 2. the third pass: a twin that moves nothing ---------------------------------------------

/// `noted` (`result == Healthy`, preserving) declared after `held-for-promotion` answers in every
/// held state wherever `held-for-promotion`'s stored guard does not hold. The wrong-state witness
/// in `Held` is then right only on a row whose `auto_promote` is unset or false: the third pass's
/// stored-guard condition is what keeps it there. A first-declared target passes every scenario.
#[test]
fn adversary_278_a_non_moving_twin_after_the_mixed_branch_passes_a_first_declared_target() {
    let result = synthesize(&ir(&unpromoted(&spec(&[HELD, NOTED]))));
    assert!(has(&result, HELD_SCENARIO), "{:#?}", refusals(&result));
    assert!(has(&result, HELD_REFUSES), "{:#?}", refusals(&result));
    let target = Rollout::new(&[B_HELD, B_NOTED, B_ROLLBACK], Mode::FirstDeclared);
    assert_eq!(not_passed(&run(&result, &target)), Vec::<String>::new());
    // And the scenario observes the difference: a target answering `noted` first fails it.
    let mutant = Rollout::new(&[B_HELD, B_NOTED, B_ROLLBACK], Mode::LastDeclared);
    let failed = failed_ids(&run(&result, &mutant));
    assert!(
        failed.iter().any(|id| id == HELD_REFUSES),
        "a target taking `noted` in Held passes {HELD_REFUSES}; failed: {failed:#?}"
    );
}

// ---- 4. `none of:` filtering ------------------------------------------------------------------

/// The input guards named after `and none of:` in `message`, and the condition before it.
fn none_of(message: &str) -> Vec<(String, Vec<String>)> {
    let mut found = Vec::new();
    let mut rest = message;
    while let Some(at) = rest.find(" and none of: ") {
        let own_start = rest[..at].rfind('`').map_or(0, |tick| tick + 1);
        let own = rest[own_start..at].to_owned();
        let list_start = at + " and none of: ".len();
        let list_end = rest[list_start..]
            .find('`')
            .map_or(rest.len(), |end| list_start + end);
        let list = &rest[list_start..list_end];
        let list = list.split(", on ").next().unwrap_or(list);
        found.push((
            own,
            list.split(", ").map(|it| it.trim().to_owned()).collect(),
        ));
        rest = &rest[list_start..];
    }
    found
}

/// Acceptance 2 (`c and none of: c, …`) where `c` is written `result in [Healthy]` and the twin
/// `result == Healthy`: the same guard, not the same text. The list must not name a guard every
/// input satisfying `c` satisfies.
#[test]
fn adversary_278_an_equivalent_twin_is_not_listed_against_its_own_guard() {
    let result = synthesize(&ir(&pinned(&spec(&[HELD_IN, PROMOTED]))));
    let mut contradictions = Vec::new();
    for message in refusals(&result) {
        for (own, list) in none_of(&message) {
            if own.contains("result in [Healthy]")
                && list.iter().any(|it| it == "result == Healthy")
            {
                contradictions.push(message.clone());
            }
        }
    }
    assert_eq!(contradictions, Vec::<String>::new());
}

/// The same twin unpinned: the mixed branch and its wrong-state scenario are synthesized, and a
/// first-declared target passes them.
#[test]
fn adversary_278_an_equivalent_twin_is_synthesized_and_passes() {
    let result = synthesize(&ir(&spec(&[HELD_IN, PROMOTED])));
    assert!(has(&result, HELD_SCENARIO), "{:#?}", refusals(&result));
    assert!(has(&result, HELD_REFUSES), "{:#?}", refusals(&result));
    let target = Rollout::new(&[B_HELD, B_PROMOTED, B_ROLLBACK], Mode::FirstDeclared);
    assert_eq!(not_passed(&run(&result, &target)), Vec::<String>::new());
}

/// `noted` declared *before* the mixed branch takes every `Healthy` input, so `Held`'s refusal
/// through `held-for-promotion` is refused. The diagnostic must not say the search needed a row
/// holding the mixed branch's stored guard: such a row exists in every state (no deployment is
/// pinned here), and what no candidate could do is refute `noted`. Dropping the equal twin from
/// `none of:` and naming the row instead hides that.
#[test]
fn adversary_278_an_earlier_twin_is_not_reported_as_a_missing_row() {
    let result = synthesize(&ir(&unpromoted(&spec(&[NOTED, HELD]))));
    let refused = refusals(&result);
    let held = refused
        .iter()
        .find(|refusal| refusal.starts_with(HELD_REFUSES));
    if let Some(held) = held {
        assert!(
            !held.contains("row in this state holding"),
            "the refusal blames a row a scenario arranges, not the twin declared first: {held}"
        );
    }
}

// ---- 3. a suite outside the 209: two input-only twins beside a stored-field branch ------------

/// `promoted` and `parked` both `result == Healthy`, neither reading the row, beside the stored
/// guard of `held-for-rollback`. `Order::Unique` finds no row selecting `promoted` alone, so the
/// fallback now witnesses it (the suite of such a model changed). The witness passes a
/// first-declared target, and a last-declared one fails it.
const PARKED: &str = r"      - name: parked
        when: result == Healthy
        moves: demo.rollout.Deployment.hold
        instance: deployment_id
        emits: [demo.rollout.Moved]
        payload:
          demo.rollout.Moved: {deployment_id: input.deployment_id}
";

const B_PARKED: Branch = Branch {
    name: "parked",
    when: Guard::Healthy,
    row: None,
    effect: Effect::FromCanary("Held"),
};

#[test]
fn adversary_278_input_only_twins_beside_a_stored_guard_follow_declaration_order() {
    let result = synthesize(&ir(&spec(&[PROMOTED, PARKED])));
    let promoted = "demo.rollout.RecordResult/outcome/promoted";
    assert!(has(&result, promoted), "{:#?}", refusals(&result));
    let target = Rollout::new(&[B_PROMOTED, B_PARKED, B_ROLLBACK], Mode::FirstDeclared);
    assert_eq!(not_passed(&run(&result, &target)), Vec::<String>::new());
    let mutant = Rollout::new(&[B_PROMOTED, B_PARKED, B_ROLLBACK], Mode::LastDeclared);
    let failed = failed_ids(&run(&result, &mutant));
    assert!(failed.iter().any(|id| id == promoted), "{failed:#?}");
}
