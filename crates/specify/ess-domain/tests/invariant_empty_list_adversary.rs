//! Adversary pass on `sets: {field: []}` (ess/31): the empty list is a constant a branch writes
//! into a row, so it is held to what a literal written into a row is held to, and it is refused
//! below ess/31 in every position a `sets:` entry has.
//!
//! A literal a branch writes is checked against the entity's invariants at validation
//! (`literal_entity_invariants.rs`): one the literal makes false is refused as
//! `conflicting_declaration`. `[]` decides `tags.count >= 1` just as `0` decides `capacity > 0`.

use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<ess_domain::Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("{error}\n{body}"));
    ess_domain::Specification::assemble([(ess_domain::system::Source::new("jobs.yaml"), raw)])
}

/// A job with a required `tags` list under `invariant`, created with `tags` from `created`, and
/// retagged by an update writing `retagged`; a sweep over every job of a board writes `swept`
/// through `affects:`.
fn model(format: &str, invariant: &str, created: &str, retagged: &str, swept: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.jobs
types:
  - {{name: demo.jobs.JobId, kind: newtype, of: Uuid}}
  - {{name: demo.jobs.BoardId, kind: newtype, of: Uuid}}
entities:
  - name: demo.jobs.Board
    identity: {{name: board_id, type: demo.jobs.BoardId}}
    fields: []
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
  - name: demo.jobs.Job
    identity: {{name: job_id, type: demo.jobs.JobId}}
    fields:
      - {{name: board, type: demo.jobs.BoardId}}
      - {{name: tags, type: List<String>}}
{invariant}    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
events:
  - name: demo.jobs.BoardOpened
    fields: [{{name: board_id, type: demo.jobs.BoardId}}]
  - name: demo.jobs.JobAdded
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
  - name: demo.jobs.JobRetagged
    fields: [{{name: job_id, type: demo.jobs.JobId}}]
  - name: demo.jobs.BoardSwept
    fields: [{{name: board_id, type: demo.jobs.BoardId}}]
commands:
  - name: demo.jobs.OpenBoard
    input: []
    outcomes:
      - name: opened
        creates: demo.jobs.Board
        instance: board_id
        emits: [demo.jobs.BoardOpened]
        payload: {{demo.jobs.BoardOpened: {{board_id: {{generated: true}}}}}}
  - name: demo.jobs.AddJob
    input:
      - {{name: board, type: demo.jobs.BoardId}}
      - {{name: tags, type: List<String>}}
    outcomes:
      - name: added
        creates: demo.jobs.Job
        instance: job_id
        sets: {{board: input.board, tags: {created}}}
        emits: [demo.jobs.JobAdded]
        payload: {{demo.jobs.JobAdded: {{job_id: {{generated: true}}}}}}
  - name: demo.jobs.Retag
    input:
      - {{name: job_id, type: demo.jobs.JobId}}
      - {{name: tags, type: List<String>}}
    outcomes:
      - name: retagged
        updates: demo.jobs.Job
        instance: job_id
        sets: {{tags: {retagged}}}
        emits: [demo.jobs.JobRetagged]
        payload: {{demo.jobs.JobRetagged: {{job_id: input.job_id}}}}
  - name: demo.jobs.Sweep
    input:
      - {{name: board_id, type: demo.jobs.BoardId}}
      - {{name: tags, type: List<String>}}
    outcomes:
      - name: swept
        updates: demo.jobs.Board
        instance: board_id
        emits: [demo.jobs.BoardSwept]
        payload: {{demo.jobs.BoardSwept: {{board_id: input.board_id}}}}
        affects:
          - entity: demo.jobs.Job
            where: board == subject.board_id
            sets: {{tags: {swept}}}
"
    )
}

const AT_LEAST_ONE: &str = "    invariants:\n      - tags.count >= 1\n";
const SOME_URGENT: &str =
    "    invariants:\n      - {exists: {in: tags, as: t, that: t == urgent}}\n";

fn conflicting(errors: &ValidationErrors) -> Vec<String> {
    errors
        .as_slice()
        .iter()
        .filter(|error| error.code == ValidationCode::ConflictingDeclaration)
        .map(ToString::to_string)
        .collect()
}

/// Control: the invariant is admissible, and a creator reading the list from its input is the
/// run's to decide, so the model validates.
#[test]
fn adversary_control_the_invariants_validate_with_input_sources() {
    for invariant in [AT_LEAST_ONE, SOME_URGENT] {
        let text = model(
            "ess/31",
            invariant,
            "input.tags",
            "input.tags",
            "input.tags",
        );
        if let Err(errors) = assemble(&text) {
            panic!("the control must validate: {errors}\n{text}");
        }
    }
}

/// `tags: []` on a creator under `tags.count >= 1` writes a row no run can hold: every request
/// taking `added` breaks the invariant. A literal doing the same is refused at validation.
#[test]
fn adversary_an_empty_list_creation_breaking_an_entity_invariant_is_refused() {
    for invariant in [AT_LEAST_ONE, SOME_URGENT] {
        let text = model("ess/31", invariant, "[]", "input.tags", "input.tags");
        let errors = assemble(&text).err().unwrap_or_else(|| {
            panic!(
                "`tags: []` under `{}` must be refused:\n{text}",
                invariant.trim()
            )
        });
        let conflicting = conflicting(&errors);
        assert!(
            conflicting.iter().any(|error| error.contains("tags")),
            "a `conflicting_declaration` naming `tags`, as a literal gets: {errors}"
        );
    }
}

/// The same on an update branch, which `literal_entity_invariants.rs` refuses for a literal.
#[test]
fn adversary_an_empty_list_update_breaking_an_entity_invariant_is_refused() {
    let text = model("ess/31", AT_LEAST_ONE, "input.tags", "[]", "input.tags");
    let errors = assemble(&text)
        .err()
        .unwrap_or_else(|| panic!("`tags: []` on an update under `tags.count >= 1`:\n{text}"));
    assert!(
        conflicting(&errors)
            .iter()
            .any(|error| error.contains("tags")),
        "{errors}"
    );
}

/// Below ess/31 `[]` names ess/31 in every `sets:` position: a creator, an update and an
/// `affects:` entry.
#[test]
fn adversary_an_empty_list_below_ess_31_names_ess_31_in_every_sets_position() {
    for (label, created, retagged, swept) in [
        ("creator", "[]", "input.tags", "input.tags"),
        ("update", "input.tags", "[]", "input.tags"),
        ("affects", "input.tags", "input.tags", "[]"),
    ] {
        for format in ["ess/25", "ess/30"] {
            let text = model(format, "", created, retagged, swept);
            let errors = assemble(&text)
                .err()
                .unwrap_or_else(|| panic!("{label} at {format}: `[]` must be refused:\n{text}"));
            assert!(
                errors.as_slice().iter().any(|error| error.code
                    == ValidationCode::UnsupportedFormatVersion
                    && error.to_string().contains("ess/31")),
                "{label} at {format}: {errors}"
            );
        }
    }
}

/// From ess/31 `[]` is admitted on an update and an `affects:` entry as on a creator.
#[test]
fn adversary_an_empty_list_on_update_and_affects_is_admitted_at_ess_31() {
    let text = model("ess/31", "", "[]", "[]", "[]");
    if let Err(errors) = assemble(&text) {
        panic!("{errors}\n{text}");
    }
}
