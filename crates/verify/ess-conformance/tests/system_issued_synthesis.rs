//! Synthesis awaits a command the system issues itself, and never drives it (beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`, *Which branches a scenario can claim*, unit U-4).
//!
//! An entry waits; the system's schedulers assign it to a free worker of their own choosing, and
//! expire it. A system auditor records an audit row, which no existing record addresses. A caller
//! adds workers and enqueues entries through a served component.
//!
//! | branch | claimed as |
//! |---|---|
//! | `Assign/assigned`, `Expire/expired` (accepting) | `await_outcome`, the expectations the branch owes, the decided input read as `{kind: awaited}` |
//! | `Assign/dialler-refused` (`external:`) | `configure_external_outcome`, `await_outcome`, `expect_outcome`, `expect_error` |
//! | `Assign/wrong-state`, `Expire/not-waiting`, `Expire/unknown-entry` (the record decides) | `expect_not_issued`, the subject unchanged |
//! | `Assign/no-worker`, `Assign/busy` (`when_related:`, the input decides) | no scenario; `ESS-SYNTH-024` names the branch |
//! | `Record/recorded` (`creates:`) | no scenario; `ESS-SYNTH-023` |
//!
//! Every suite this writes passes on the interpreted model, which answers the await steps (U-5).

use std::collections::{BTreeMap, BTreeSet};

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{
    ActorRef, ConformanceScenario, EventRef, Holds, LeafShape, PayloadShape, ScenarioId,
    ScenarioStep, ScenarioValue, ViewExpectation,
};
use ess_conformance::synthesize::{synthesize, Synthesis};
use ess_conformance::{AdmittedSuite, Runner};
use ess_domain::binding::periodic::Period;
use ess_domain::types::Primitive;
use ess_domain::{spec::RawSpecFile, system::Source, QualifiedName, Specification};

const ASSIGN: &str = "queueing.entry.Assign";
const EXPIRE: &str = "queueing.entry.Expire";
const RECORD: &str = "queueing.entry.Record";
const CALLER: &str = "queueing.entry.Caller";

/// The queue, with `system` written on each actor the system is, and `await_within:` on each
/// system-issued command where `bound` is set.
#[allow(clippy::too_many_lines)] // One model, read top to bottom.
fn model(system: &str, bound: &str) -> String {
    format!(
        "format: ess/27
system: queueing
version: v1
domain: queueing.entry
types:
  - {{name: queueing.entry.EntryId, kind: newtype, of: Uuid}}
  - {{name: queueing.entry.WorkerId, kind: newtype, of: Uuid}}
  - {{name: queueing.entry.AuditId, kind: newtype, of: Uuid}}
entities:
  - name: queueing.entry.Worker
    identity: {{name: worker_id, type: queueing.entry.WorkerId}}
    fields:
      - {{name: free, type: Boolean}}
    lifecycle: {{initial: Ready, states: [Ready], terminal: [Ready]}}
  - name: queueing.entry.Entry
    identity: {{name: entry_id, type: queueing.entry.EntryId}}
    fields:
      - {{name: worker, type: Optional<queueing.entry.WorkerId>}}
    lifecycle:
      initial: Waiting
      states: [Waiting, Assigned, Expired]
      terminal: [Assigned, Expired]
      transitions:
        - {{name: assign, from: [Waiting], to: Assigned}}
        - {{name: expire, from: [Waiting], to: Expired}}
  - name: queueing.entry.Audit
    identity: {{name: audit_id, type: queueing.entry.AuditId}}
    fields: []
    lifecycle: {{initial: Kept, states: [Kept], terminal: [Kept]}}
errors:
  - {{name: queueing.entry.EntryStateConflict, summary: The entry is not waiting., fields: []}}
  - {{name: queueing.entry.NoWorker, summary: No worker carries the identity., fields: []}}
  - {{name: queueing.entry.WorkerBusy, summary: The worker is busy., fields: []}}
  - {{name: queueing.entry.DiallerRefused, summary: The dialler refused the call., fields: []}}
  - {{name: queueing.entry.NoEntry, summary: No entry carries the identity., fields: []}}
events:
  - name: queueing.entry.WorkerAdded
    fields:
      - {{name: worker_id, type: queueing.entry.WorkerId}}
  - name: queueing.entry.Enqueued
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
  - name: queueing.entry.Expired
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
  - name: queueing.entry.Assigned
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
  - name: queueing.entry.Recorded
    fields:
      - {{name: audit_id, type: queueing.entry.AuditId}}
actors:
  - name: queueing.entry.Auditor
{system}    may: [queueing.entry.Record]
  - name: queueing.entry.Caller
    may: [queueing.entry.AddWorker, queueing.entry.Enqueue]
  - name: queueing.entry.Dispatcher
{system}    may: [queueing.entry.Assign]
  - name: queueing.entry.Scheduler
{system}    may: [queueing.entry.Assign, queueing.entry.Expire]
views:
  - name: queueing.entry.Entries
    source: queueing.entry.Entry
    consistency: read_your_writes
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: Optional<queueing.entry.WorkerId>}}
      - {{name: state, type: queueing.entry.Entry.State}}
components:
  - component: queue-service
    owns: {{domains: [queueing.entry]}}
    accepts:
      commands: [queueing.entry.AddWorker, queueing.entry.Enqueue, queueing.entry.Assign,
                 queueing.entry.Expire, queueing.entry.Record]
    publishes:
      events: [queueing.entry.WorkerAdded, queueing.entry.Enqueued, queueing.entry.Expired,
               queueing.entry.Assigned, queueing.entry.Recorded]
    reached_by: network
commands:
  - name: queueing.entry.AddWorker
    input:
      - {{name: free, type: Boolean}}
    outcomes:
      - name: added
        creates: queueing.entry.Worker
        instance: worker_id
        emits: [queueing.entry.WorkerAdded]
        payload:
          queueing.entry.WorkerAdded: {{worker_id: {{generated: true}}}}
        sets:
          free: input.free
  - name: queueing.entry.Enqueue
    input: []
    outcomes:
      - name: enqueued
        creates: queueing.entry.Entry
        instance: entry_id
        emits: [queueing.entry.Enqueued]
        payload:
          queueing.entry.Enqueued: {{entry_id: {{generated: true}}}}
  - name: queueing.entry.Assign
{bound}    input:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
    outcomes:
      - name: no-worker
        when_related: {{via: input.worker, exists: false}}
        error: queueing.entry.NoWorker
      - name: busy
        when_related: {{via: input.worker, predicate: free == false}}
        error: queueing.entry.WorkerBusy
      - name: dialler-refused
        external: the dialler refuses the call
        error: queueing.entry.DiallerRefused
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        sets: {{worker: input.worker}}
        payload:
          queueing.entry.Assigned: {{entry_id: input.entry_id, worker: input.worker}}
      - {{name: wrong-state, wrong_state: true, error: queueing.entry.EntryStateConflict}}
  - name: queueing.entry.Expire
{bound}    input:
      - {{name: entry_id, type: queueing.entry.EntryId}}
    outcomes:
      - name: expired
        moves: queueing.entry.Entry.expire
        instance: entry_id
        emits: [queueing.entry.Expired]
        payload:
          queueing.entry.Expired: {{entry_id: input.entry_id}}
      - {{name: not-waiting, wrong_state: true, error: queueing.entry.EntryStateConflict}}
      - {{name: unknown-entry, unknown_instance: true, error: queueing.entry.NoEntry}}
  - name: queueing.entry.Record
    input: []
    outcomes:
      - name: recorded
        creates: queueing.entry.Audit
        instance: audit_id
        emits: [queueing.entry.Recorded]
        payload:
          queueing.entry.Recorded: {{audit_id: {{generated: true}}}}
"
    )
}

const SYSTEM: &str = "    system: true\n";
const BOUND: &str = "    await_within: PT5S\n";

fn ir_of(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).expect("the model parses");
    let spec = Specification::assemble([(Source::new("queueing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a valid name")
}

fn event(value: &str) -> EventRef {
    EventRef::new(name(value))
}

fn id(value: &str) -> ScenarioId {
    value.parse().expect("a scenario id")
}

fn scenario<'s>(synthesis: &'s Synthesis, value: &str) -> &'s ConformanceScenario {
    synthesis
        .suite
        .scenario(&id(value))
        .unwrap_or_else(|| panic!("`{value}` is synthesized: {:#?}", ids(synthesis)))
}

fn ids(synthesis: &Synthesis) -> Vec<String> {
    synthesis
        .suite
        .scenarios
        .keys()
        .map(ToString::to_string)
        .collect()
}

fn awaited(field: &str) -> ScenarioValue {
    ScenarioValue::Awaited {
        field: field.to_owned(),
    }
}

fn period(bounded: bool) -> Option<Period> {
    bounded.then(|| Period::parse("PT5S").unwrap())
}

fn system_issued(ir: &EssIr) -> BTreeSet<QualifiedName> {
    ir.commands()
        .values()
        .filter(|command| command.system_issued)
        .map(|command| command.name.clone())
        .collect()
}

fn is_system_actor(ir: &EssIr, actor: &ActorRef) -> bool {
    ir.actors()
        .values()
        .any(|declared| declared.name == *actor.name() && declared.system)
}

fn is_instance(value: &ScenarioValue, named: &str) -> bool {
    matches!(value, ScenarioValue::Instance { instance } if instance.to_string() == named)
}

fn names(step: &ScenarioStep, command_name: &str) -> bool {
    match step {
        ScenarioStep::AwaitOutcome { command, .. }
        | ScenarioStep::ExpectNotIssued { command, .. } => command.name() == &name(command_name),
        _ => false,
    }
}

/// Every send of a system-issued command in the suite: none, except a grant refusal sent as a
/// caller the system is not, which is refused before it runs.
fn assert_never_driven(ir: &EssIr, synthesis: &Synthesis) {
    let issued = system_issued(ir);
    for (scenario_id, scenario) in &synthesis.suite.scenarios {
        for (at, step) in scenario.steps.iter().enumerate() {
            let (ScenarioStep::ExecuteCommand {
                command: sent,
                actor,
                ..
            }
            | ScenarioStep::ExecuteCommandWithoutInput {
                command: sent,
                actor,
                ..
            }) = step
            else {
                continue;
            };
            if !issued.contains(sent.name()) {
                continue;
            }
            let refused = matches!(scenario_id, ScenarioId::Grant { .. })
                && actor
                    .as_ref()
                    .is_some_and(|actor| !is_system_actor(ir, actor))
                && matches!(
                    scenario.steps.get(at + 1),
                    Some(ScenarioStep::ExpectNotGranted { .. })
                );
            assert!(
                refused,
                "`{scenario_id}` drives the system-issued `{sent}` at step {at}: {step:?}"
            );
        }
    }
}

/// The declared input of `Assign`, as its await step carries it.
fn assign_shape() -> PayloadShape {
    let mut shape = PayloadShape::new();
    for field in ["entry_id", "worker"] {
        shape.insert(
            field,
            LeafShape::required(Holds::Primitive {
                kind: Primitive::Uuid,
            }),
        );
    }
    shape
}

/// The scenario's last await of `command`, and what follows it.
fn after_await<'s>(scenario: &'s ConformanceScenario, command_name: &str) -> &'s [ScenarioStep] {
    let at = scenario
        .steps
        .iter()
        .rposition(|step| {
            matches!(step, ScenarioStep::AwaitOutcome { .. }) && names(step, command_name)
        })
        .unwrap_or_else(|| panic!("an await of `{command_name}`: {:#?}", scenario.steps));
    &scenario.steps[at..]
}

fn not_issued_at(scenario: &ConformanceScenario, command_name: &str) -> usize {
    scenario
        .steps
        .iter()
        .position(|step| {
            matches!(step, ScenarioStep::ExpectNotIssued { .. }) && names(step, command_name)
        })
        .unwrap_or_else(|| {
            panic!(
                "an expect_not_issued of `{command_name}`: {:#?}",
                scenario.steps
            )
        })
}

fn run_on_the_interpreter(ir: EssIr, synthesis: &Synthesis) {
    let admitted = AdmittedSuite::from_suite(&synthesis.suite)
        .unwrap_or_else(|error| panic!("the synthesized suite is admitted: {error}"));
    let target = Interpreted::for_model(ir);
    let report = Runner::for_suite(admitted.suite())
        .run_admitted(&admitted, &target)
        .into_report();
    let failing: Vec<String> = report
        .scenarios
        .iter()
        .filter(|result| result.status != Status::Passed)
        .map(|result| format!("{} {:?}: {result:#?}", result.scenario, result.status))
        .collect();
    assert!(
        failing.is_empty(),
        "every synthesized scenario passes on the interpreted model:\n{}",
        failing.join("\n")
    );
    assert_eq!(
        report.scenarios.len(),
        synthesis.suite.scenarios.len(),
        "every scenario ran"
    );
}

fn assert_awaits_assignment(synthesis: &Synthesis, bounded: bool) {
    let assigned = scenario(synthesis, "queueing.entry.Assign/outcome/assigned");
    let tail = after_await(assigned, ASSIGN);
    let ScenarioStep::AwaitOutcome {
        instance,
        identity,
        within,
        input,
        shape,
        ..
    } = &tail[0]
    else {
        unreachable!()
    };
    assert!(
        is_instance(instance, "entry"),
        "the awaited instance is the entry the arrangement enqueued: {instance:?}"
    );
    assert_eq!(identity, "entry_id");
    assert_eq!(*within, period(bounded), "the bound is the declared one");
    assert!(
        input.is_empty(),
        "synthesis states no input the system decides: {input:?}"
    );
    assert_eq!(*shape, assign_shape(), "the answer's input is held by type");
    assert!(
        matches!(&tail[1], ScenarioStep::ExpectOutcome { outcome } if outcome.outcome.as_str() == "assigned"),
        "{:?}",
        tail[1]
    );
    let published = tail
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectEventValues {
                event: sent,
                payload,
                ..
            } if *sent == event("queueing.entry.Assigned") => Some(payload),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the published assignment is compared: {tail:#?}"));
    assert_eq!(
        published.get("worker"),
        Some(&awaited("worker")),
        "the worker published is the one the system reported issuing the command with"
    );
    assert!(
        published
            .get("entry_id")
            .is_some_and(|value| is_instance(value, "entry")),
        "{published:?}"
    );
    let row = tail
        .iter()
        .find_map(|step| match step {
            ScenarioStep::ExpectView {
                expectation: ViewExpectation::Contains { fields },
                ..
            } => Some(fields),
            _ => None,
        })
        .unwrap_or_else(|| panic!("the stored row is read back: {tail:#?}"));
    assert_eq!(
        row.get("worker"),
        Some(&awaited("worker")),
        "the stored worker is the one the system reported: {row:?}"
    );
}

#[test]
fn synthesis_awaits_a_system_issued_command_instead_of_driving_it() {
    let ir = ir_of(&model(SYSTEM, BOUND));
    let synthesis = synthesize(&ir);
    assert_never_driven(&ir, &synthesis);
    assert_awaits_assignment(&synthesis, true);
    assert_eq!(
        synthesis.suite.provenance.suite_version.major(),
        ess_conformance::system_issued::ORDINARY,
        "a suite that awaits is suite/52"
    );

    // An external refusal is an outcome the system reached: it tried, and the dialler refused.
    let refused = scenario(&synthesis, "queueing.entry.Assign/outcome/dialler-refused");
    let configured = refused
        .steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. }))
        .expect("the refusal is forced");
    let tail = after_await(refused, ASSIGN);
    assert!(
        refused.steps.len() - tail.len() > configured,
        "the force precedes the await"
    );
    assert!(
        matches!(&tail[1], ScenarioStep::ExpectOutcome { outcome } if outcome.outcome.as_str() == "dialler-refused")
    );
    assert!(
        tail.iter().any(|step| matches!(step, ScenarioStep::ExpectError { error, .. } if error.to_string() == "queueing.entry.DiallerRefused")),
        "{tail:#?}"
    );

    // Arranging a state the system reaches awaits it too.
    let expired = scenario(
        &synthesis,
        "queueing.entry.Entry/state/Expired/refuses/queueing.entry.Assign",
    );
    assert!(
        expired.steps.iter().any(|step| {
            matches!(step, ScenarioStep::AwaitOutcome { .. }) && names(step, EXPIRE)
        }),
        "{:#?}",
        expired.steps
    );

    // The grants: no admitted-grant scenario for a system actor, whose await is its witness; the
    // refusal of an outside caller is still driven, and sent as a caller the system is not.
    for scenario_id in synthesis.suite.scenarios.keys() {
        if let ScenarioId::GrantAdmitted { command, actor } = scenario_id {
            assert!(
                !system_issued(&ir).contains(command.name()),
                "`{scenario_id}` admits `{actor}` to a system-issued command"
            );
        }
    }
    let denied = scenario(&synthesis, "queueing.entry.Assign/grant/denied");
    let sent_as: Vec<&ActorRef> = denied
        .steps
        .iter()
        .filter_map(|step| match step {
            ScenarioStep::ExecuteCommand {
                command,
                actor: Some(actor),
                ..
            } if command.name() == &name(ASSIGN) => Some(actor),
            _ => None,
        })
        .collect();
    assert_eq!(
        sent_as,
        vec![&ActorRef::new(name(CALLER))],
        "the refusal is asked of an outside caller, not of another system actor"
    );

    run_on_the_interpreter(ir, &synthesis);
}

#[test]
fn synthesis_awaits_without_a_declared_bound() {
    let ir = ir_of(&model(SYSTEM, ""));
    let synthesis = synthesize(&ir);
    assert_never_driven(&ir, &synthesis);
    assert_awaits_assignment(&synthesis, false);
    let refusal = scenario(
        &synthesis,
        "queueing.entry.Entry/state/Assigned/refuses/queueing.entry.Assign",
    );
    let ScenarioStep::ExpectNotIssued { within, .. } =
        &refusal.steps[not_issued_at(refusal, ASSIGN)]
    else {
        unreachable!()
    };
    assert_eq!(*within, None, "the runner's own deadline bounds it");
    run_on_the_interpreter(ir, &synthesis);
}

#[test]
fn a_refusal_of_a_system_issued_command_expects_nothing_within_the_bound() {
    let ir = ir_of(&model(SYSTEM, BOUND));
    let synthesis = synthesize(&ir);

    // The record decides: an assigned entry is not assigned again, and nothing it publishes is.
    let refusal = scenario(
        &synthesis,
        "queueing.entry.Entry/state/Assigned/refuses/queueing.entry.Assign",
    );
    let at = not_issued_at(refusal, ASSIGN);
    let ScenarioStep::ExpectNotIssued {
        instance,
        within,
        unpublished,
        ..
    } = &refusal.steps[at]
    else {
        unreachable!()
    };
    assert!(is_instance(instance, "entry"), "{instance:?}");
    assert_eq!(*within, period(true));
    assert_eq!(*unpublished, vec![event("queueing.entry.Assigned")]);
    let before = &refusal.steps[..at];
    assert!(
        before
            .iter()
            .any(|step| matches!(step, ScenarioStep::AwaitOutcome { .. }) && names(step, ASSIGN)),
        "the arrangement awaits the first assignment: {before:#?}"
    );
    assert!(
        before
            .iter()
            .any(|step| matches!(step, ScenarioStep::SnapshotCompleteSubject { .. })),
        "{before:#?}"
    );
    let after = &refusal.steps[at + 1..];
    assert!(
        after
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectCompleteSubjectUnchanged { .. })),
        "the subject is unchanged: {after:#?}"
    );
    for step in after {
        assert!(
            !matches!(
                step,
                ScenarioStep::ExpectOutcome { .. }
                    | ScenarioStep::ExpectError { .. }
                    | ScenarioStep::ExpectNoError
                    | ScenarioStep::ExpectEvent { .. }
                    | ScenarioStep::ExpectEventValues { .. }
                    | ScenarioStep::ExpectNoEvent { .. }
                    | ScenarioStep::ExpectNoEvents
            ),
            "nothing reads a command result nobody produced: {step:?}"
        );
    }

    // No record carries the identity: nothing is expired.
    let unknown = scenario(&synthesis, "queueing.entry.Expire/outcome/unknown-entry");
    let ScenarioStep::ExpectNotIssued {
        instance,
        unpublished,
        ..
    } = &unknown.steps[not_issued_at(unknown, EXPIRE)]
    else {
        unreachable!()
    };
    assert!(
        matches!(instance, ScenarioValue::Literal { .. }),
        "an identity no record carries: {instance:?}"
    );
    assert_eq!(*unpublished, vec![event("queueing.entry.Expired")]);

    run_on_the_interpreter(ir, &synthesis);
}

#[test]
fn a_refusal_the_input_decides_and_a_created_record_are_refused_by_name() {
    let synthesis = synthesize(&ir_of(&model(SYSTEM, BOUND)));

    // The input decides: the system chooses the worker, so no scenario claims a refusal another
    // choice would avoid. Each is refused by name.
    let refused: BTreeMap<String, String> = synthesis
        .refusals
        .iter()
        .filter_map(|refusal| {
            refusal
                .scenario
                .as_ref()
                .map(|scenario| (scenario.to_string(), refusal.code().to_string()))
        })
        .collect();
    for branch in ["no-worker", "busy"] {
        let scenario_id = format!("{ASSIGN}/outcome/{branch}");
        assert!(
            synthesis.suite.scenario(&id(&scenario_id)).is_none(),
            "`{scenario_id}` is not synthesized"
        );
        assert_eq!(
            refused.get(&scenario_id).map(String::as_str),
            Some("ESS-SYNTH-024"),
            "{refused:#?}"
        );
        let text = synthesis
            .refusals
            .iter()
            .find(|refusal| refusal.scenario.as_ref() == Some(&id(&scenario_id)))
            .map(ToString::to_string)
            .unwrap_or_default();
        assert!(
            text.contains(&format!("`{ASSIGN}/{branch}`")) && text.contains("when_related"),
            "the refusal names the branch and what decides it: {text}"
        );
    }

    // A system-issued command that creates its record has no instance to await.
    let recorded = format!("{RECORD}/outcome/recorded");
    assert!(synthesis.suite.scenario(&id(&recorded)).is_none());
    assert_eq!(
        refused.get(&recorded).map(String::as_str),
        Some("ESS-SYNTH-023"),
        "{refused:#?}"
    );
}

/// A model in which no command is system-issued — no system actor, or one whose commands another
/// actor is granted too — gets the suite it got before the await vocabulary, byte for byte apart
/// from the digest of the model it was synthesized from.
#[test]
fn an_old_suite_keeps_its_bytes() {
    // Every command a system actor holds is granted to the caller too, so each stays driven.
    let mixed = |system: &str| {
        model(system, "").replace(
            "    may: [queueing.entry.AddWorker, queueing.entry.Enqueue]",
            "    may: [queueing.entry.AddWorker, queueing.entry.Enqueue, queueing.entry.Assign, \
             queueing.entry.Expire, queueing.entry.Record]",
        )
    };
    let with = synthesize(&ir_of(&mixed(SYSTEM)));
    let without = synthesize(&ir_of(&mixed("")));
    assert!(!ess_conformance::system_issued::used_by(&with.suite));
    assert_eq!(
        with.suite.provenance.suite_version,
        without.suite.provenance.suite_version
    );
    let unpinned = |synthesis: &Synthesis| {
        let mut suite = synthesis.suite.clone();
        suite.provenance = without.suite.provenance.clone();
        suite.to_canonical_json().unwrap()
    };
    assert_eq!(unpinned(&with), unpinned(&without));
    let printed = |synthesis: &Synthesis| {
        (
            synthesis
                .refusals
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
            synthesis
                .notes
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>(),
        )
    };
    assert_eq!(printed(&with), printed(&without));

    // No system actor at all: every command is driven, and the input-decided refusals the system
    // would choose around keep their scenarios.
    let plain = synthesize(&ir_of(&model("", "")));
    assert!(!ess_conformance::system_issued::used_by(&plain.suite));
    for branch in ["no-worker", "busy"] {
        assert!(plain
            .suite
            .scenario(&id(&format!("{ASSIGN}/outcome/{branch}")))
            .is_some());
    }
}

/// The codes of the refusals a coverage inventory of `text` carries, and its suite major.
fn covered(text: &str) -> (BTreeSet<String>, u32, bool) {
    let input = ess_conformance::coverage_build::build(
        &ir_of(text),
        &[],
        ess_conformance::coverage::Scope::System,
        ess_conformance::coverage::Origins::Generated,
    )
    .unwrap_or_else(|error| panic!("the inventory is admitted: {error}"));
    let selected = input.selected();
    let codes = selected
        .coverage()
        .expect("a coverage suite")
        .refused
        .iter()
        .map(|refusal| refusal.code.clone())
        .collect();
    let suite = selected.suite();
    (
        codes,
        suite.provenance.suite_version.major(),
        ess_conformance::system_issued::used_by(suite),
    )
}

/// The coverage inventory carries both refusals beside the awaited scenarios, as coverage suite/53,
/// and carries them there even where nothing is awaited.
#[test]
fn the_coverage_inventory_carries_the_awaits_and_both_refusals() {
    let (codes, major, awaits) = covered(&model(SYSTEM, BOUND));
    assert!(awaits);
    assert_eq!(major, ess_conformance::system_issued::COVERAGE);
    for code in ["ESS-SYNTH-023", "ESS-SYNTH-024"] {
        assert!(codes.contains(code), "{code}: {codes:?}");
    }

    // Only the auditor is the system: its one command creates a record, nothing is awaited, and
    // the refusal alone selects the coverage major that carries it.
    let audited = model("", "").replace(
        "  - name: queueing.entry.Auditor\n    may:",
        "  - name: queueing.entry.Auditor\n    system: true\n    may:",
    );
    let (codes, major, awaits) = covered(&audited);
    assert!(!awaits);
    assert!(codes.contains("ESS-SYNTH-023"), "{codes:?}");
    assert_eq!(major, ess_conformance::system_issued::COVERAGE);
}
