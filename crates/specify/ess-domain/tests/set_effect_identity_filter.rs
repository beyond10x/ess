//! A set effect's filter reads the candidate row's identity from `ess/27` (beyond10x/ess#547,
//! `docs/design/set-effect-identity-filters.md`, D1 and D2).
//!
//! * The filter of `instances:` and of an `affects:` entry reads the identity bare, as a row-set
//!   selector already does; `state` stays unreadable there.
//! * Below `ess/27` such a read is refused as `unsupported_format_version` naming `ess/27`, alone.
//! * A pin over the subject's own entity whose operand is the subject's identity selects only the
//!   subject, which every entry excepts: `conflicting_declaration`.
//! * `instance:` without `each:` stays refused; its hint names the identity filter.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// Two entities, a call and a queue entry storing the call's identity in `call_id`; expiring the
/// entry ends the call it names.
const MODEL: &str = "format: ess/27
system: demo
version: v1
domain: demo.q
types:
  - {name: demo.q.CallId, kind: newtype, of: String}
  - {name: demo.q.EntryId, kind: newtype, of: String}
entities:
  - name: demo.q.Call
    identity: {name: call_id, type: demo.q.CallId}
    fields:
      - {name: label, type: String}
    lifecycle:
      initial: Waiting
      states: [Waiting, Ended]
      terminal: [Ended]
      transitions:
        - {name: end, from: [Waiting], to: Ended}
  - name: demo.q.Entry
    identity: {name: entry_id, type: demo.q.EntryId}
    fields:
      - {name: call_id, type: demo.q.CallId}
      - {name: previous, type: Optional<demo.q.EntryId>}
    lifecycle:
      initial: Queued
      states: [Queued, Expired]
      terminal: [Expired]
      transitions:
        - {name: expire, from: [Queued], to: Expired}
events:
  - name: demo.q.CallStarted
    fields: [{name: call_id, type: demo.q.CallId}]
  - name: demo.q.EntryQueued
    fields: [{name: entry_id, type: demo.q.EntryId}]
  - name: demo.q.EntryExpired
    fields: [{name: entry_id, type: demo.q.EntryId}]
errors:
  - name: demo.q.NotQueued
  - name: demo.q.NoCall
  - name: demo.q.CallEnded
commands:
  - name: demo.q.StartCall
    input: [{name: label, type: String}]
    outcomes:
      - name: started
        creates: demo.q.Call
        instance: call_id
        sets: {label: input.label}
        emits: [demo.q.CallStarted]
        payload: {demo.q.CallStarted: {call_id: {generated: true}}}
  - name: demo.q.EndCall
    input: [{name: call_id, type: demo.q.CallId}]
    outcomes:
      - name: ended
        moves: demo.q.Call.end
        instance: call_id
        emits: [demo.q.CallStarted]
        payload: {demo.q.CallStarted: {call_id: input.call_id}}
  - name: demo.q.QueueEntry
    input:
      - {name: call_id, type: demo.q.CallId}
      - {name: previous, type: Optional<demo.q.EntryId>}
    outcomes:
      - name: queued
        creates: demo.q.Entry
        instance: entry_id
        sets: {call_id: input.call_id, previous: input.previous}
        emits: [demo.q.EntryQueued]
        payload: {demo.q.EntryQueued: {entry_id: {generated: true}}}
  - name: demo.q.ExpireEntry
    input: [{name: entry_id, type: demo.q.EntryId}]
    outcomes:
      - name: expired
        moves: demo.q.Entry.expire
        instance: entry_id
        emits: [demo.q.EntryExpired]
        payload:
          demo.q.EntryExpired: {entry_id: input.entry_id}
        affects:
          - entity: demo.q.Call
            where: call_id == subject.call_id
            moves: demo.q.Call.end
      - {name: not-queued, wrong_state: true, error: demo.q.NotQueued}
";

const PINNED: &str = "            where: call_id == subject.call_id\n";

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the model parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("q.yaml"), raw)])
}

fn admitted(body: &str) -> Specification {
    assemble(body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn edited(body: &str, from: &str, to: &str) -> String {
    assert!(body.contains(from), "the model holds:\n{from}");
    body.replacen(from, to, 1)
}

/// The one refusal of `errors`, which must be the only one.
fn alone(errors: &ValidationErrors) -> &ess_primitives::error::ValidationError {
    let [only] = errors.as_slice() else {
        panic!("exactly one refusal, got:\n{errors}");
    };
    only
}

/// The filter of `ExpireEntry`'s first `affects:` entry, as the specification holds it.
fn affect_filter(spec: &Specification, command: &str) -> ess_primitives::predicate::Predicate {
    let command = spec
        .commands()
        .values()
        .find(|candidate| candidate.name.to_string() == command)
        .expect("the command is declared");
    command.outcomes[0].set_effects.affects[0]
        .filter
        .clone()
        .expect("a `where:` entry")
}

/// Whether `predicate` reads the bare root `root` as a fact, rather than holding it as text.
fn reads_root(predicate: &ess_primitives::predicate::Predicate, root: &str) -> bool {
    predicate
        .fact_paths()
        .iter()
        .any(|path| path.to_string() == root)
}

#[test]
fn an_affects_filter_pinning_the_identity_to_a_subject_field_validates_at_ess_27() {
    let spec = admitted(MODEL);
    let filter = affect_filter(&spec, "demo.q.ExpireEntry");
    assert_eq!(filter.to_string(), "call_id == subject.call_id");
    assert!(reads_root(&filter, "call_id"), "{filter:?}");
    // Either side: the identity written on the right reads the candidate's identity too, not the
    // text `call_id`, as in a row-set selector.
    let swapped = admitted(&edited(
        MODEL,
        PINNED,
        "            where: subject.call_id == call_id\n",
    ));
    let filter = affect_filter(&swapped, "demo.q.ExpireEntry");
    assert!(
        reads_root(&filter, "call_id"),
        "the bare identity on the right is a read of the row: {filter:?}"
    );
}

#[test]
fn an_affects_filter_pinning_the_identity_to_an_input_validates_at_ess_27() {
    let model = edited(
        MODEL,
        "    input: [{name: entry_id, type: demo.q.EntryId}]\n    outcomes:\n      - name: expired",
        "    input:\n      - {name: entry_id, type: demo.q.EntryId}\n      - {name: call, type: \
         demo.q.CallId}\n    outcomes:\n      - name: expired",
    );
    let model = edited(&model, PINNED, "            where: call_id == input.call\n");
    let spec = admitted(&model);
    let filter = affect_filter(&spec, "demo.q.ExpireEntry");
    assert_eq!(filter.to_string(), "call_id == input.call");
    assert!(reads_root(&filter, "call_id"), "{filter:?}");
}

/// `ForgetEntry`: every entry whose identity the input names expires; zero rows is accepted.
const FORGET: &str = "  - name: demo.q.ForgetEntry
    input: [{name: entry_id, type: demo.q.EntryId}]
    outcomes:
      - name: forgotten
        moves: demo.q.Entry.expire
        instances: {where: entry_id == input.entry_id}
        emits: [demo.q.EntryExpired]
        payload:
          demo.q.EntryExpired: {entry_id: input.entry_id}
";

#[test]
fn an_instances_filter_reading_the_identity_validates_at_ess_27() {
    let spec = admitted(&format!("{MODEL}{FORGET}"));
    let command = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.q.ForgetEntry")
        .expect("declared");
    let set = command.outcomes[0]
        .set_effects
        .instances
        .as_ref()
        .expect("`instances:` is kept");
    assert_eq!(set.filter.to_string(), "entry_id == input.entry_id");
    assert!(reads_root(&set.filter, "entry_id"), "{:?}", set.filter);
}

#[test]
fn the_identity_in_a_set_effect_filter_below_ess_27_is_refused_alone_naming_ess_27() {
    let below = MODEL.replacen("format: ess/27", "format: ess/26", 1);
    let errors = refused(&below);
    let only = alone(&errors);
    assert_eq!(
        only.code,
        ValidationCode::UnsupportedFormatVersion,
        "{errors}"
    );
    assert_eq!(
        only.location, "command.demo.q.ExpireEntry.outcomes.expired.affects[0].where",
        "{errors}"
    );
    assert!(
        only.message.contains("`call_id`")
            && only.message.contains("identity of `demo.q.Call`")
            && only.message.contains("ess/27"),
        "{errors}"
    );
    assert!(
        only.hint
            .as_deref()
            .is_some_and(|hint| hint.contains("format: ess/27")),
        "{errors}"
    );

    let below = format!("{MODEL}{FORGET}")
        .replacen("format: ess/27", "format: ess/26", 1)
        .replacen(
            "        affects:\n          - entity: demo.q.Call\n            where: call_id == \
             subject.call_id\n            moves: demo.q.Call.end\n",
            "",
            1,
        );
    let errors = refused(&below);
    let only = alone(&errors);
    assert_eq!(
        only.code,
        ValidationCode::UnsupportedFormatVersion,
        "{errors}"
    );
    assert_eq!(
        only.location, "command.demo.q.ForgetEntry.outcomes.forgotten.instances.where",
        "{errors}"
    );
    assert!(
        only.message.contains("`entry_id`") && only.message.contains("ess/27"),
        "{errors}"
    );
}

#[test]
fn a_pin_naming_the_subject_on_its_own_entity_is_refused() {
    let own = |filter: &str| {
        edited(
            MODEL,
            "          - entity: demo.q.Call\n            where: call_id == subject.call_id\n            moves: demo.q.Call.end\n",
            &format!(
                "          - entity: demo.q.Entry\n            where: {filter}\n            moves: \
                 demo.q.Entry.expire\n"
            ),
        )
    };
    for filter in [
        "entry_id == subject.entry_id",
        "subject.entry_id == entry_id",
        "entry_id == input.entry_id",
        "{all: [entry_id == subject.entry_id, call_id == subject.call_id]}",
    ] {
        let errors = refused(&own(filter));
        let only = alone(&errors);
        assert_eq!(
            only.code,
            ValidationCode::ConflictingDeclaration,
            "{filter}: {errors}"
        );
        assert_eq!(
            only.location, "command.demo.q.ExpireEntry.outcomes.expired.affects[0].where",
            "{filter}: {errors}"
        );
        assert!(
            only.message.contains("only the subject"),
            "{filter}: {errors}"
        );
        assert!(
            only.hint
                .as_deref()
                .is_some_and(|hint| hint.contains("sets:")),
            "{filter}: {errors}"
        );
    }
    // The same entity pinned to another row the subject names is admitted: it is not the subject.
    admitted(&own("entry_id == subject.previous"));
}

#[test]
fn state_in_a_set_effect_filter_stays_unobservable() {
    let errors = refused(&edited(
        MODEL,
        PINNED,
        "            where: {all: [call_id == subject.call_id, state == Waiting]}\n",
    ));
    let only = alone(&errors);
    assert_eq!(only.code, ValidationCode::UnobservableFact, "{errors}");
    assert!(only.message.contains("`state`"), "{errors}");
}

#[test]
fn instance_without_each_hints_the_identity_filter() {
    let errors = refused(&edited(
        MODEL,
        PINNED,
        "            instance: subject.call_id\n",
    ));
    let refusal = errors
        .as_slice()
        .iter()
        .find(|error| error.code == ValidationCode::ConflictingDeclaration)
        .unwrap_or_else(|| panic!("`instance:` without `each:` is refused: {errors}"));
    assert_eq!(
        refusal.location,
        "command.demo.q.ExpireEntry.outcomes.expired.affects[0].instance"
    );
    assert!(
        refusal.hint.as_deref().is_some_and(|hint| hint
            .contains("to change one existing row by its identity, write `where: <identity> == subject.<field>`")
            && hint.contains("ess/27")),
        "{errors}"
    );
}

#[test]
fn related_guards_compose_with_a_pinned_entry() {
    let spec = admitted(&edited(
        MODEL,
        "      - name: expired\n",
        "      - name: no-call\n        when_related: {via: call_id, exists: false}\n        \
         error: demo.q.NoCall\n      - name: call-ended\n        when_related: {via: call_id, \
         predicate: state != Waiting}\n        error: demo.q.CallEnded\n      - name: expired\n",
    ));
    let command = spec
        .commands()
        .values()
        .find(|candidate| candidate.name.to_string() == "demo.q.ExpireEntry")
        .expect("declared");
    let expired = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "expired")
        .expect("the accepting branch");
    assert_eq!(
        expired.set_effects.affects[0]
            .filter
            .as_ref()
            .map(ToString::to_string)
            .as_deref(),
        Some("call_id == subject.call_id")
    );
}
