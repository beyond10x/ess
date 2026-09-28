//! Adversary pass 1 on `story:set-effects-over-filtered-instances` (ess/16; beyond10x/ess#167, #175):
//! refusals the unit's own suite does not state.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/set-effects.yaml");

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body).map_err(|error| {
        ValidationErrors::from(ess_primitives::error::ValidationError::new(
            ValidationCode::ConflictingDeclaration,
            "parse",
            error.to_string(),
        ))
    })?;
    Specification::assemble([(ess_domain::system::Source::new("desk.yaml"), raw)])
}

fn refused(body: &str) -> ValidationErrors {
    match assemble(body) {
        Ok(_) => panic!("the model is refused:\n{body}"),
        Err(errors) => errors,
    }
}

fn edited(text: &str, from: &str, to: &str) -> String {
    assert!(text.contains(from), "the model holds:\n{from}");
    text.replacen(from, to, 1)
}

fn below_16(text: &str) -> String {
    edited(text, "format: ess/16", "format: ess/15")
}

fn has_code(errors: &ValidationErrors, code: ValidationCode, needle: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(needle))
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

/// Under `ess/15` the construct is not in the language: whatever else is wrong with how it was
/// written, the reader is told the format first. The invariants: "A new construct is refused under
/// an earlier header as `unsupported_format_version`".
#[test]
fn instance_beside_instances_under_ess_15_is_refused_as_the_format() {
    let text = below_16(&edited(
        MODEL,
        ENDED,
        "        moves: demo.desk.Session.end
        instance: team
        instances: {where: team == input.team}
",
    ));
    let errors = refused(&text);
    assert!(
        has_code(
            &errors,
            ValidationCode::UnsupportedFormatVersion,
            "EndTeam.outcomes.ended.instances"
        ),
        "`instances:` under ess/15 is refused as unsupported_format_version, got:\n{errors}"
    );
}

#[test]
fn instances_without_a_verb_under_ess_15_is_refused_as_the_format() {
    let text = below_16(&edited(
        MODEL,
        ENDED,
        "        instances: {where: team == input.team}
",
    ));
    let errors = refused(&text);
    assert!(
        has_code(
            &errors,
            ValidationCode::UnsupportedFormatVersion,
            "EndTeam.outcomes.ended.instances"
        ),
        "`instances:` under ess/15 is refused as unsupported_format_version, got:\n{errors}"
    );
}

#[test]
fn a_move_inside_affects_under_ess_15_is_refused_as_the_format() {
    let text = below_16(&edited(
        MODEL,
        INVITE_AFFECTS,
        "        affects:
          - entity: demo.desk.Session
            where: team == subject.team
            moves: demo.desk.Session.park
",
    ));
    let errors = refused(&text);
    assert!(
        has_code(&errors, ValidationCode::UnsupportedFormatVersion, "affects"),
        "`affects:` under ess/15 is refused as unsupported_format_version, got:\n{errors}"
    );
}

/// Every selected row coming to hold one identity is a contradiction: a set effect's `sets:` may not
/// write the entity's identity.
#[test]
fn an_affects_entry_setting_the_identity_of_every_selected_row_is_refused() {
    let text = edited(
        MODEL,
        "            sets:\n              on_hold: true",
        "            sets:\n              on_hold: true\n              session_id: input.session_id",
    );
    let errors = refused(&text);
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("session_id")),
        "{errors}"
    );
}

#[test]
fn a_set_update_setting_the_identity_of_every_selected_row_is_refused() {
    let text = edited(
        MODEL,
        "  - name: demo.desk.NoteTeam
    input:
      - {name: team, type: demo.desk.Team}
      - {name: note, type: demo.desk.Note}",
        "  - name: demo.desk.NoteTeam
    input:
      - {name: team, type: demo.desk.Team}
      - {name: note, type: demo.desk.Note}
      - {name: session_id, type: demo.desk.SessionId}",
    );
    let text = edited(
        &text,
        "        sets:\n          note: input.note\n  - name: demo.desk.Invite",
        "        sets:\n          note: input.note\n          session_id: input.session_id\n  - name: demo.desk.Invite",
    );
    let errors = refused(&text);
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("session_id")),
        "{errors}"
    );
}

/// `{count: changed}` inside an `affects:` entry's `sets:` is refused by name.
#[test]
fn a_count_in_an_affects_entry_is_refused_by_name() {
    let text = edited(
        MODEL,
        "            sets:\n              on_hold: true",
        "            sets:\n              on_hold: true\n              note: {count: changed}",
    );
    let errors = refused(&text);
    assert!(
        has_code(
            &errors,
            ValidationCode::UnsupportedConstruct,
            "{count: changed}"
        ),
        "{errors}"
    );
}
