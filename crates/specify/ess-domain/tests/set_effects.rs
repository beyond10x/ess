//! Set effects over filtered instances (ess/16; beyond10x/ess#167, #175,
//! `docs/design/set-effects-over-filtered-instances.md`).
//!
//! * `instances: {where: <predicate>}` on a `moves:` or `updates:` outcome: every stored row of the
//!   entity the predicate selects, read with `input.<field>` operands. Beside `instance:`, or on a
//!   `creates:`/`deletes:`, it is refused.
//! * `{count: changed}` in `payload:`: how many rows such an outcome changed, into an `Integer`.
//! * `affects:` on an outcome with one subject: a list of `{entity, where, sets}` effects on other
//!   rows, whose `where` reads `input.<field>` and `subject.<field>` (the subject before the
//!   outcome).
//!
//! Below `ess/16` each is refused with `unsupported_format_version`.

use ess_domain::command::{PayloadSource, RawOutcome};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects.yaml");

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .map_err(|error| ValidationErrors::from(error_of(&error.to_string())))?;
    Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
}

fn error_of(text: &str) -> ess_primitives::error::ValidationError {
    ess_primitives::error::ValidationError::new(
        ValidationCode::ConflictingDeclaration,
        "parse",
        text.to_owned(),
    )
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

fn assert_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) {
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == code && error.to_string().contains(needle)),
        "expected {code:?} mentioning `{needle}`, got:\n{errors}"
    );
}

fn edited(from: &str, to: &str) -> String {
    assert!(MODEL.contains(from), "the fixture holds:\n{from}");
    MODEL.replacen(from, to, 1)
}

const ENDED: &str = "        moves: demo.desk.Session.end
        instances: {where: team == input.team}
";

const INVITE_AFFECTS: &str = "        affects:
          - entity: demo.desk.Session
            where: team == subject.team
            sets:
              on_hold: true
";

#[test]
fn a_set_move_a_set_update_and_a_secondary_effect_are_admitted_under_ess_16() {
    let spec = admitted(MODEL);
    let command = |name: &str| {
        spec.commands()
            .values()
            .find(|command| command.name.to_string() == name)
            .unwrap_or_else(|| panic!("{name} is declared"))
    };
    let ended = &command("demo.desk.EndTeam").outcomes[0];
    assert!(ended.subject.is_none(), "a set subject is not one instance");
    let set = ended
        .set_effects
        .instances
        .as_ref()
        .expect("`instances:` is kept");
    assert_eq!(set.entity.to_string(), "demo.desk.Session");
    assert_eq!(set.effect.transition(), Some("end"));
    assert_eq!(set.filter.to_string(), "team == input.team");
    assert_eq!(
        ended
            .payload
            .values()
            .next()
            .and_then(|fields| fields.get("ended")),
        Some(&PayloadSource::ChangedCount)
    );
    let noted = &command("demo.desk.NoteTeam").outcomes[0];
    assert_eq!(
        noted
            .set_effects
            .instances
            .as_ref()
            .map(|set| set.effect.verb()),
        Some("updates")
    );
    let invited = &command("demo.desk.Invite").outcomes[0];
    assert!(invited.subject.is_some());
    let [affect] = invited.set_effects.affects.as_slice() else {
        panic!("one secondary effect: {:#?}", invited.set_effects.affects)
    };
    assert_eq!(affect.entity.to_string(), "demo.desk.Session");
    assert_eq!(affect.filter.to_string(), "team == subject.team");
    assert!(affect.sets.contains_key("on_hold"));
}

#[test]
fn the_written_form_round_trips() {
    let spec = admitted(MODEL);
    for command in spec.commands().values() {
        for outcome in &command.outcomes {
            let written = serde_yaml::to_string(&RawOutcome::from(outcome.clone()))
                .expect("an outcome serializes");
            let read: RawOutcome =
                serde_yaml::from_str(&written).unwrap_or_else(|error| panic!("{error}\n{written}"));
            let back = ess_domain::command::Outcome::try_from(read)
                .unwrap_or_else(|errors| panic!("{errors}\n{written}"));
            assert_eq!(&back, outcome, "{written}");
        }
    }
}

#[test]
fn each_construct_is_refused_below_ess_16() {
    let below = MODEL.replacen("format: ess/16", "format: ess/15", 1);
    let errors = refused(&below);
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "instances",
    );
    assert_code(&errors, ValidationCode::UnsupportedFormatVersion, "affects");
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "{count: changed}",
    );
    assert!(
        errors.as_slice().iter().all(|error| !matches!(
            error.code,
            ValidationCode::EmptyDeclaration | ValidationCode::MissingDeclaration
        )),
        "the format is refused without a cascade:\n{errors}"
    );
}

#[test]
fn instance_and_instances_together_are_refused() {
    let errors = refused(&edited(
        ENDED,
        "        moves: demo.desk.Session.end
        instance: team
        instances: {where: team == input.team}
",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "instances");
}

#[test]
fn instances_on_a_creation_or_a_deletion_is_refused() {
    let errors = refused(&edited(
        ENDED,
        "        creates: demo.desk.Session
        instances: {where: team == input.team}
",
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "instances");
    let errors = refused(&edited(
        ENDED,
        "        deletes: demo.desk.Session
        instances: {where: team == input.team}
",
    ));
    assert_code(&errors, ValidationCode::UnsupportedConstruct, "instances");
}

#[test]
fn instances_without_a_verb_is_refused() {
    let errors = refused(&edited(
        ENDED,
        "        instances: {where: team == input.team}
",
    ));
    assert_code(&errors, ValidationCode::MissingDeclaration, "instances");
}

#[test]
fn the_filter_reads_declared_stored_fields_and_declared_inputs() {
    let errors = refused(&edited(
        "instances: {where: team == input.team}\n        emits: [demo.desk.TeamEnded]",
        "instances: {where: squad == input.team}\n        emits: [demo.desk.TeamEnded]",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("squad")),
        "{errors}"
    );
    let errors = refused(&edited(
        "instances: {where: team == input.team}\n        emits: [demo.desk.TeamEnded]",
        "instances: {where: team == input.squad}\n        emits: [demo.desk.TeamEnded]",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("squad")),
        "{errors}"
    );
}

#[test]
fn a_source_reading_one_subject_is_refused_under_instances_by_name() {
    let errors = refused(&edited(
        "        sets:\n          note: input.note\n  - name: demo.desk.NoteTeam",
        "        sets:\n          note: {subject: team}\n  - name: demo.desk.NoteTeam",
    ));
    assert_code(&errors, ValidationCode::UnsupportedConstruct, "{subject:");
}

#[test]
fn a_count_outside_a_set_outcome_is_refused_by_name() {
    let errors = refused(&edited(
        "demo.desk.SessionParked: {session_id: input.session_id}",
        "demo.desk.SessionParked: {session_id: {count: changed}}",
    ));
    assert_code(
        &errors,
        ValidationCode::UnsupportedConstruct,
        "{count: changed}",
    );
}

#[test]
fn a_count_into_a_field_that_is_not_an_integer_is_refused() {
    let errors = refused(&edited(
        "demo.desk.TeamEnded: {team: input.team, ended: {count: changed}}",
        "demo.desk.TeamEnded: {team: {count: changed}, ended: 0}",
    ));
    assert_code(&errors, ValidationCode::TypeMismatch, "{count: changed}");
}

#[test]
fn a_count_in_sets_is_refused_by_name() {
    let errors = refused(&edited(
        "        sets:\n          note: input.note\n  - name: demo.desk.NoteTeam",
        "        sets:\n          note: {count: changed}\n  - name: demo.desk.NoteTeam",
    ));
    assert_code(
        &errors,
        ValidationCode::UnsupportedConstruct,
        "{count: changed}",
    );
}

#[test]
fn affects_needs_one_subject() {
    // Beside `instances:`.
    let errors = refused(&edited(
        ENDED,
        &format!("{ENDED}        affects:\n          - {{entity: demo.desk.Session, where: team == input.team, sets: {{on_hold: true}}}}\n"),
    ));
    assert_code(&errors, ValidationCode::UnsupportedConstruct, "affects");
    // With no subject at all.
    let errors = refused(&edited(
        "        updates: demo.desk.Session\n        instance: session_id\n        emits: [demo.desk.Invited]",
        "        emits: [demo.desk.Invited]",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("affects")),
        "{errors}"
    );
}

#[test]
fn a_move_inside_affects_is_refused_by_name() {
    let errors = refused(&edited(
        INVITE_AFFECTS,
        "        affects:
          - entity: demo.desk.Session
            where: team == subject.team
            moves: demo.desk.Session.park
",
    ));
    assert_code(&errors, ValidationCode::UnsupportedConstruct, "moves");
}

#[test]
fn an_affects_filter_reads_the_subject_by_its_declared_fields() {
    let errors = refused(&edited(
        "where: team == subject.team",
        "where: team == subject.squad",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("squad")),
        "{errors}"
    );
}

#[test]
fn an_affects_entity_must_be_declared_and_its_sets_must_name_its_fields() {
    let errors = refused(&edited(
        "          - entity: demo.desk.Session\n            where: team == subject.team",
        "          - entity: demo.desk.Nothing\n            where: team == subject.team",
    ));
    assert_code(
        &errors,
        ValidationCode::UndeclaredReference,
        "demo.desk.Nothing",
    );
    let errors = refused(&edited(
        "            sets:\n              on_hold: true",
        "            sets:\n              held: true",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("held")),
        "{errors}"
    );
}

#[test]
fn an_affects_source_reading_one_subject_is_refused_by_name() {
    let errors = refused(&edited(
        "            sets:\n              on_hold: true",
        "            sets:\n              on_hold: {subject: on_hold}",
    ));
    assert_code(&errors, ValidationCode::UnsupportedConstruct, "{subject:");
}

#[test]
fn a_set_move_satisfies_the_lifecycle_cause_of_its_transition() {
    // `end` is taken by nothing but the set move once `Close` is gone, and the model is admitted.
    let close = MODEL
        .find("  - name: demo.desk.Close\n")
        .expect("the fixture declares Close");
    let after = close
        + MODEL[close..]
            .find("  - name: demo.desk.EndTeam\n")
            .expect("EndTeam follows Close");
    let without =
        format!("{}{}", &MODEL[..close], &MODEL[after..]).replace(" demo.desk.Close,", "");
    let spec = admitted(&without);
    assert!(spec
        .commands()
        .values()
        .flat_map(|command| &command.outcomes)
        .filter_map(|outcome| outcome.subject.as_ref())
        .all(|subject| subject.effect.transition() != Some("end")));
}

#[test]
fn a_set_move_naming_an_undeclared_transition_is_refused() {
    let errors = refused(&edited(
        ENDED,
        "        moves: demo.desk.Session.close\n        instances: {where: team == input.team}\n",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("close")),
        "{errors}"
    );
}

#[test]
fn a_set_outcome_under_a_condition_reading_one_subject_is_refused() {
    let errors = refused(&edited(
        ENDED,
        &format!("{ENDED}        when_subject_state: Open\n"),
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("instances")
                || error.to_string().contains("when_subject_state")),
        "{errors}"
    );
}

#[test]
fn a_refusal_changing_a_set_of_rows_is_refused() {
    let errors = refused(&edited(
        "        instances: {where: team == input.team}\n        emits: [demo.desk.TeamEnded]\n        payload:\n          demo.desk.TeamEnded: {team: input.team, ended: {count: changed}}\n        sets:\n          note: input.note\n",
        "        instances: {where: team == input.team}\n        error: demo.desk.NotOpen\n",
    ));
    assert_code(&errors, ValidationCode::RefusalMutatedState, "instances");
}
