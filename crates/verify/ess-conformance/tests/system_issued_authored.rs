//! An authored act naming a command the system issues itself is awaited, never sent
//! (beyond10x/ess#542, `docs/design/system-issued-commands.md`, *Every lane*, unit U-6).
//!
//! | the act | compiles to |
//! |---|---|
//! | no `actor:`, an accepting `outcome:` (or none) | `await_outcome` on the identity input, the stated input partial; the claims after it as after any act |
//! | no `actor:`, an `external:` outcome | `configure_external_outcome`, then `await_outcome` and the claims |
//! | no `actor:`, a refusing outcome | `expect_not_issued` on the identity input, every event of the command unpublished |
//! | `actor:` a system actor | refused, `ESS-AUTHOR-044`: no caller can be the system |
//! | `actor:` another actor, `refused: not_granted` | driven, as before: an outside caller must not issue the system's step |
//! | no value for the identity input | refused, `ESS-AUTHOR-045`: there is no instance to await |

use std::collections::BTreeMap;

use ess_compiler::{ir::EssIr, resolve::compile, source::SourceMap};
use ess_conformance::authored::{compile as compile_authored, Authoring, Source};
use ess_conformance::interpret::Interpreted;
use ess_conformance::report::Status;
use ess_conformance::scenario::{ScenarioStep, ScenarioValue, SuiteProvenance};
use ess_conformance::{AdmittedSuite, ConformanceSuite, Runner};
use ess_domain::binding::periodic::Period;
use ess_domain::{spec::RawSpecFile, system::Source as SpecSource, Specification};
use ess_primitives::node::Node;

const ASSIGN: &str = "queueing.entry.Assign";
const WORKER: &str = "00000000-0000-4000-8000-0000000000aa";

/// A queue whose scheduler is the system itself; `await_within:` on `Assign` where `bound`.
fn model(bound: &str) -> String {
    format!(
        "format: ess/27
system: queueing
version: v1
domain: queueing.entry
types:
  - {{name: queueing.entry.EntryId, kind: newtype, of: Uuid}}
  - {{name: queueing.entry.WorkerId, kind: newtype, of: Uuid}}
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
      states: [Waiting, Assigned]
      terminal: [Assigned]
      transitions:
        - {{name: assign, from: [Waiting], to: Assigned}}
errors:
  - {{name: queueing.entry.EntryStateConflict, summary: The entry is not waiting., fields: []}}
  - {{name: queueing.entry.DiallerRefused, summary: The dialler refused the call., fields: []}}
events:
  - name: queueing.entry.WorkerAdded
    fields:
      - {{name: worker_id, type: queueing.entry.WorkerId}}
  - name: queueing.entry.Enqueued
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
  - name: queueing.entry.Assigned
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
actors:
  - name: queueing.entry.Caller
    may: [queueing.entry.AddWorker, queueing.entry.Enqueue]
  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign]
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
"
    )
}

fn ir(bound: bool) -> EssIr {
    let text = model(if bound {
        "    await_within: PT5S\n"
    } else {
        ""
    });
    let raw = RawSpecFile::parse(&text).expect("the model parses");
    let spec = Specification::assemble([(SpecSource::new("queueing.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("the model is admitted: {errors}"));
    compile(&spec, &SourceMap::new()).expect("the model compiles")
}

/// An entry enqueued and captured as `entry`, then `acts`.
fn document(acts: &str) -> String {
    format!(
        "type: ess-scenario/1
domain: queueing.entry
scenario: the-system-assigns
summary: The system assigns a waiting entry
arrange:
  - instance: entry
    entity: queueing.entry.Entry
timeline:
  - at: 2026-01-05T09:00:00Z
    command: queueing.entry.AddWorker
    actor: queueing.entry.Caller
    input: {{free: true}}
    outcome: added
  - at: 2026-01-05T09:00:01Z
    command: queueing.entry.Enqueue
    actor: queueing.entry.Caller
    outcome: enqueued
    capture: {{instance: entry, event: queueing.entry.Enqueued, field: entry_id}}
{acts}"
    )
}

const AWAITED: &str = "  - at: 2026-01-05T09:00:02Z
    command: queueing.entry.Assign
    input: {entry_id: {$instance: entry}}
    outcome: assigned
    events:
      - event: queueing.entry.Assigned
        payload: {entry_id: {$instance: entry}}
";

const NOT_AGAIN: &str = "  - at: 2026-01-05T09:00:03Z
    command: queueing.entry.Assign
    input: {entry_id: {$instance: entry}}
    outcome: wrong-state
";

/// The document, in `ess-scenario/4` where an act expects `refused: not_granted`, which needs it.
fn authoring(ir: &EssIr, acts: &str) -> Authoring {
    let mut text = document(acts);
    if acts.contains("refused: not_granted") {
        text = text.replace("type: ess-scenario/1", "type: ess-scenario/4");
    }
    compile_authored(ir, &[Source::new("assigns.yaml", text)])
}

fn steps(ir: &EssIr, acts: &str) -> Vec<ScenarioStep> {
    let authoring = authoring(ir, acts);
    assert!(
        authoring.is_complete(),
        "{}",
        authoring
            .refusals
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
    authoring.scenarios.into_values().next().unwrap().steps
}

fn codes(ir: &EssIr, acts: &str) -> Vec<String> {
    authoring(ir, acts)
        .refusals
        .iter()
        .map(|refusal| refusal.cause.code().to_string())
        .collect()
}

fn entry() -> ScenarioValue {
    ScenarioValue::instance("entry".parse().unwrap())
}

fn sends_assign(steps: &[ScenarioStep]) -> bool {
    steps.iter().any(|step| {
        matches!(step, ScenarioStep::ExecuteCommand { command, .. }
            | ScenarioStep::ExecuteCommandWithoutInput { command, .. }
            if command.to_string() == ASSIGN)
    })
}

fn awaits(steps: &[ScenarioStep]) -> Vec<&ScenarioStep> {
    steps
        .iter()
        .filter(|step| matches!(step, ScenarioStep::AwaitOutcome { .. }))
        .collect()
}

#[test]
fn an_act_naming_a_system_issued_command_with_no_actor_is_awaited() {
    for bound in [false, true] {
        let ir = ir(bound);
        let steps = steps(&ir, AWAITED);
        assert!(!sends_assign(&steps), "never sent: {steps:#?}");
        let at = steps
            .iter()
            .position(|step| matches!(step, ScenarioStep::AwaitOutcome { .. }))
            .unwrap_or_else(|| panic!("an await: {steps:#?}"));
        let ScenarioStep::AwaitOutcome {
            command,
            instance,
            identity,
            within,
            input,
            shape,
        } = &steps[at]
        else {
            unreachable!()
        };
        assert_eq!(command.to_string(), ASSIGN);
        assert_eq!(instance, &entry());
        assert_eq!(identity, "entry_id");
        assert_eq!(*within, bound.then(|| Period::parse("PT5S").unwrap()));
        assert!(
            input.is_empty(),
            "nothing stated but the instance: {input:?}"
        );
        assert!(!shape.is_empty(), "the declared input is carried");
        assert!(
            matches!(&steps[at + 1], ScenarioStep::ExpectOutcome { outcome }
                if outcome.to_string() == format!("{ASSIGN}/assigned")),
            "{steps:#?}"
        );
        assert!(
            steps[at + 1..].iter().any(
                |step| matches!(step, ScenarioStep::ExpectEvent { event, .. }
                    | ScenarioStep::ExpectEventValues { event, .. }
                    if event.to_string() == "queueing.entry.Assigned")
            ),
            "the act's events compile as after any act: {steps:#?}"
        );
    }
}

#[test]
fn a_stated_input_of_an_awaited_act_is_carried_partial() {
    let ir = ir(false);
    let acts = AWAITED.replace(
        "input: {entry_id: {$instance: entry}}",
        &format!("input: {{entry_id: {{$instance: entry}}, worker: {WORKER}}}"),
    );
    let steps = steps(&ir, &acts);
    let [ScenarioStep::AwaitOutcome { input, .. }] = awaits(&steps)[..] else {
        panic!("one await: {steps:#?}");
    };
    assert_eq!(
        input,
        &BTreeMap::from([(
            "worker".to_owned(),
            ScenarioValue::literal(Node::Text(WORKER.to_owned()))
        )])
    );
}

#[test]
fn a_refusing_act_of_a_system_issued_command_expects_nothing_issued() {
    let ir = ir(true);
    let steps = steps(&ir, &format!("{AWAITED}{NOT_AGAIN}"));
    assert!(!sends_assign(&steps), "never sent: {steps:#?}");
    let last = steps.last().unwrap();
    let ScenarioStep::ExpectNotIssued {
        command,
        instance,
        within,
        unpublished,
    } = last
    else {
        panic!("the refusal ends the scenario as nothing issued: {steps:#?}");
    };
    assert_eq!(command.to_string(), ASSIGN);
    assert_eq!(instance, &entry());
    assert_eq!(*within, Some(Period::parse("PT5S").unwrap()));
    assert_eq!(
        unpublished
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["queueing.entry.Assigned"]
    );
}

#[test]
fn a_refusing_act_claiming_what_only_an_issued_command_answers_is_refused() {
    let ir = ir(false);
    for claim in [
        "    error: {name: queueing.entry.EntryStateConflict}\n".to_owned(),
        "    events:\n      - event: queueing.entry.Assigned\n".to_owned(),
    ] {
        let acts = format!("{AWAITED}{NOT_AGAIN}{claim}");
        assert_eq!(codes(&ir, &acts), ["ESS-AUTHOR-046"], "{claim}");
    }
    let acts = format!(
        "{AWAITED}{}",
        NOT_AGAIN.replace(
            "input: {entry_id: {$instance: entry}}",
            &format!("input: {{entry_id: {{$instance: entry}}, worker: {WORKER}}}"),
        )
    );
    assert_eq!(codes(&ir, &acts), ["ESS-AUTHOR-046"]);
}

#[test]
fn an_external_act_of_a_system_issued_command_forces_the_branch_then_awaits() {
    let ir = ir(false);
    let acts = "  - at: 2026-01-05T09:00:02Z
    command: queueing.entry.Assign
    input: {entry_id: {$instance: entry}}
    outcome: dialler-refused
    error: {name: queueing.entry.DiallerRefused}
";
    let steps = steps(&ir, acts);
    let configure = steps
        .iter()
        .position(|step| matches!(step, ScenarioStep::ConfigureExternalOutcome { .. }))
        .unwrap_or_else(|| panic!("a forced branch: {steps:#?}"));
    assert!(
        matches!(steps[configure + 1], ScenarioStep::AwaitOutcome { .. }),
        "{steps:#?}"
    );
    assert!(
        matches!(&steps[configure + 2], ScenarioStep::ExpectOutcome { outcome }
            if outcome.to_string() == format!("{ASSIGN}/dialler-refused")),
        "{steps:#?}"
    );
    assert!(
        steps
            .iter()
            .any(|step| matches!(step, ScenarioStep::ExpectError { .. })),
        "{steps:#?}"
    );
}

#[test]
fn an_act_sending_a_system_issued_command_as_the_system_is_refused() {
    let ir = ir(false);
    let acts = AWAITED.replace(
        "    input: {entry_id",
        "    actor: queueing.entry.Scheduler\n    input: {entry_id",
    );
    assert_eq!(codes(&ir, &acts), ["ESS-AUTHOR-044"]);
}

#[test]
fn an_outside_caller_refused_the_system_step_stays_driven() {
    let ir = ir(false);
    let acts = "  - at: 2026-01-05T09:00:02Z
    command: queueing.entry.Assign
    actor: queueing.entry.Caller
    input: {entry_id: {$instance: entry}, worker: 00000000-0000-4000-8000-0000000000aa}
    refused: not_granted
";
    let steps = steps(&ir, acts);
    assert!(sends_assign(&steps), "driven: {steps:#?}");
    assert!(awaits(&steps).is_empty(), "{steps:#?}");
    assert!(
        matches!(steps.last(), Some(ScenarioStep::ExpectNotGranted { .. })),
        "{steps:#?}"
    );
}

#[test]
fn an_awaited_act_without_its_instance_is_refused() {
    let ir = ir(false);
    let acts = AWAITED.replace(
        "input: {entry_id: {$instance: entry}}",
        &format!("input: {{worker: {WORKER}}}"),
    );
    assert_eq!(codes(&ir, &acts), ["ESS-AUTHOR-045"]);
    let acts = AWAITED.replace("    input: {entry_id: {$instance: entry}}\n", "");
    assert_eq!(codes(&ir, &acts), ["ESS-AUTHOR-045"]);
}

#[test]
fn an_authored_await_passes_on_the_interpreted_model() {
    for bound in [false, true] {
        let ir = ir(bound);
        let authoring = authoring(&ir, &format!("{AWAITED}{NOT_AGAIN}"));
        assert!(authoring.is_complete(), "{:?}", authoring.refusals);
        let mut suite = ConformanceSuite::new(SuiteProvenance::of(&ir));
        for (id, scenario) in authoring.scenarios {
            suite.insert(id, scenario).expect("one id");
        }
        suite.select_fresh_format();
        assert_eq!(
            suite.provenance.suite_version.to_string(),
            "ess-conformance/52"
        );
        let admitted = AdmittedSuite::from_suite(&suite).unwrap_or_else(|error| panic!("{error}"));
        let report = Runner::for_suite(admitted.suite())
            .run_admitted(&admitted, &Interpreted::for_model(ir))
            .into_report();
        for result in &report.scenarios {
            assert_eq!(result.status, Status::Passed, "{result:#?}");
        }
    }
}
