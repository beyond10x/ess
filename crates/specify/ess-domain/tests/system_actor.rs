//! An actor that is the system itself, `system: true`, and the bound a system-issued command
//! declares, `await_within:` (`ess/25`, beyond10x/ess#542,
//! `docs/design/system-issued-commands.md`).
//!
//! What validation decides: a command is system-issued exactly when it has at least one grant and
//! every actor granting it is a system actor; `await_within:` only on such a command; a system
//! actor carries no `attributes:`; no binding invokes a system-issued command; and below `ess/25`
//! both keys are refused naming `ess/25`.

use ess_domain::QualifiedName;

fn spec(body: &str) -> Result<ess_domain::Specification, String> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|e| e.to_string())?;
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("queueing.yaml"), raw)])
        .map_err(|e| e.to_string())
}

fn refused(body: &str) -> String {
    match spec(body) {
        Ok(_) => panic!("the document validates:\n{body}"),
        Err(errors) => errors,
    }
}

fn name(value: &str) -> QualifiedName {
    QualifiedName::new(value).expect("a valid name")
}

/// An entry waits and the system's scheduler assigns it. `actors` and `assign` are spliced in
/// verbatim: the actor list, and keys added to `queueing.entry.Assign` before its `input:`.
fn model(format: &str, actors: &str, assign: &str, extra: &str) -> String {
    format!(
        "format: {format}
system: queueing
version: v1
domain: queueing.entry
types:
  - {{name: queueing.entry.EntryId, kind: newtype, of: Uuid}}
  - {{name: queueing.entry.WorkerId, kind: newtype, of: String}}
entities:
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
events:
  - name: queueing.entry.Enqueued
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
  - name: queueing.entry.Assigned
    fields:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
actors:
{actors}commands:
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
{assign}    input:
      - {{name: entry_id, type: queueing.entry.EntryId}}
      - {{name: worker, type: queueing.entry.WorkerId}}
    outcomes:
      - name: assigned
        moves: queueing.entry.Entry.assign
        instance: entry_id
        emits: [queueing.entry.Assigned]
        sets: {{worker: input.worker}}
        payload:
          queueing.entry.Assigned: {{entry_id: input.entry_id, worker: input.worker}}
      - {{name: wrong-state, wrong_state: true, error: queueing.entry.EntryStateConflict}}
{extra}"
    )
}

const CALLER: &str = "  - name: queueing.entry.Caller
    may: [queueing.entry.Enqueue]
";

const SCHEDULER: &str = "  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign]
";

const WITHIN: &str = "    await_within: PT5S\n";

#[test]
fn a_system_actor_validates_at_ess_25() {
    let body = model("ess/25", &format!("{CALLER}{SCHEDULER}"), WITHIN, "");
    let spec = spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
    let scheduler = &spec.actors()[&name("queueing.entry.Scheduler")];
    assert!(scheduler.system, "the actor is the system itself");
    assert!(!spec.actors()[&name("queueing.entry.Caller")].system);
    let assign = &spec.commands()[&name("queueing.entry.Assign")];
    assert_eq!(
        assign
            .await_within
            .map(ess_domain::binding::periodic::Period::seconds),
        Some(5),
        "the bound is the period written"
    );
}

#[test]
fn a_command_granted_only_to_system_actors_is_system_issued() {
    let body = model("ess/25", &format!("{CALLER}{SCHEDULER}"), "", "");
    let spec = spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
    assert!(spec.is_system_issued(&name("queueing.entry.Assign")));
    assert!(
        !spec.is_system_issued(&name("queueing.entry.Enqueue")),
        "granted to an ordinary actor, so driven"
    );
}

#[test]
fn a_command_granted_to_no_actor_is_unchanged() {
    let body = model(
        "ess/25",
        "  - name: queueing.entry.Scheduler\n    system: true\n",
        "",
        "",
    );
    let spec = spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
    for command in ["queueing.entry.Assign", "queueing.entry.Enqueue"] {
        assert!(
            !spec.is_system_issued(&name(command)),
            "{command} has no grant, and a command no actor issues is not the system's"
        );
    }
}

#[test]
fn mixed_grants_stay_driven() {
    let operator = "  - name: queueing.entry.Operator
    may: [queueing.entry.Assign]
";
    let actors = format!("{CALLER}{SCHEDULER}{operator}");
    let body = model("ess/25", &actors, "", "");
    let spec = spec(&body).unwrap_or_else(|errors| panic!("{errors}"));
    assert!(
        !spec.is_system_issued(&name("queueing.entry.Assign")),
        "granted to the system and to an operator, so an ordinary, driven command"
    );

    let errors = refused(&model("ess/25", &actors, WITHIN, ""));
    assert!(errors.contains("conflicting_declaration"), "{errors}");
    assert!(errors.contains("await_within"), "{errors}");
    assert!(
        errors.contains("queueing.entry.Operator"),
        "the refusal names the grant that makes the command driven: {errors}"
    );
}

#[test]
fn await_within_on_a_command_no_actor_grants_is_refused() {
    let errors = refused(&model("ess/25", CALLER, WITHIN, ""));
    assert!(errors.contains("conflicting_declaration"), "{errors}");
    assert!(
        errors.contains("command.queueing.entry.Assign.await_within"),
        "{errors}"
    );
}

#[test]
fn system_false_is_the_same_document_as_leaving_it_out() {
    let written = "  - name: queueing.entry.Scheduler
    system: false
    may: [queueing.entry.Assign]
";
    let left_out = "  - name: queueing.entry.Scheduler
    may: [queueing.entry.Assign]
";
    let written = spec(&model("ess/25", &format!("{CALLER}{written}"), "", ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
    let left_out = spec(&model("ess/25", &format!("{CALLER}{left_out}"), "", ""))
        .unwrap_or_else(|errors| panic!("{errors}"));
    assert_eq!(written.actors(), left_out.actors());
    assert!(!written.is_system_issued(&name("queueing.entry.Assign")));
    let rendered = serde_json::to_string(&written.actors()[&name("queueing.entry.Scheduler")])
        .expect("an actor serialises");
    assert!(!rendered.contains("system"), "{rendered}");
}

#[test]
fn system_below_ess_25_is_refused_naming_ess_25() {
    let errors = refused(&model("ess/24", &format!("{CALLER}{SCHEDULER}"), "", ""));
    assert!(errors.contains("unsupported_format_version"), "{errors}");
    assert!(
        errors.contains("actor.queueing.entry.Scheduler.system"),
        "{errors}"
    );
    assert!(errors.contains("ess/25"), "{errors}");
}

#[test]
fn await_within_below_ess_25_is_refused_naming_ess_25() {
    let actors = "  - name: queueing.entry.Scheduler
    may: [queueing.entry.Assign]
";
    let errors = refused(&model("ess/24", &format!("{CALLER}{actors}"), WITHIN, ""));
    assert!(errors.contains("unsupported_format_version"), "{errors}");
    assert!(
        errors.contains("command.queueing.entry.Assign.await_within"),
        "{errors}"
    );
    assert!(errors.contains("ess/25"), "{errors}");
}

#[test]
fn a_system_actor_carries_no_attributes() {
    let actors = "  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign]
    attributes:
      - {name: tenant, type: String}
";
    let errors = refused(&model("ess/25", &format!("{CALLER}{actors}"), "", ""));
    assert!(errors.contains("conflicting_declaration"), "{errors}");
    assert!(
        errors.contains("actor.queueing.entry.Scheduler.attributes"),
        "{errors}"
    );
}

#[test]
fn a_zero_bound_keeps_the_period_refusal() {
    let errors = refused(&model(
        "ess/25",
        &format!("{CALLER}{SCHEDULER}"),
        "    await_within: PT0S\n",
        "",
    ));
    assert!(errors.contains("PT0S"), "{errors}");
    let errors = refused(&model(
        "ess/25",
        &format!("{CALLER}{SCHEDULER}"),
        "    await_within: 5s\n",
        "",
    ));
    assert!(
        errors.contains("5s"),
        "another spelling is refused: {errors}"
    );
}

#[test]
fn a_binding_invoking_a_system_issued_command_is_refused() {
    let binding = "bindings:
  - id: assign-on-enqueue
    when: {event: queueing.entry.Enqueued}
    invoke: {command: queueing.entry.Assign}
    mapping:
      entry_id: event.entry_id
      worker: w-1
    delivery: at_most_once
    on_failure: drop
";
    let errors = refused(&model(
        "ess/25",
        &format!("{CALLER}{SCHEDULER}"),
        "",
        binding,
    ));
    assert!(errors.contains("conflicting_declaration"), "{errors}");
    assert!(
        errors.contains("binding.assign-on-enqueue.invoke"),
        "{errors}"
    );

    // The same binding over a driven command is ordinary.
    let operator = "  - name: queueing.entry.Operator
    may: [queueing.entry.Assign]
";
    spec(&model(
        "ess/25",
        &format!("{CALLER}{operator}"),
        "",
        binding,
    ))
    .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn a_system_actor_may_name_a_view_only_the_system_reads() {
    let actors = "  - name: queueing.entry.Scheduler
    system: true
    may: [queueing.entry.Assign, queueing.entry.Waiting]
";
    let view = "views:
  - name: queueing.entry.Waiting
    source: queueing.entry.Entry
    consistency: read_your_writes
    fields:
      - {name: entry_id, type: queueing.entry.EntryId}
";
    let spec = spec(&model("ess/25", &format!("{CALLER}{actors}"), "", view))
        .unwrap_or_else(|errors| panic!("{errors}"));
    assert!(spec.is_system_issued(&name("queueing.entry.Assign")));
}
