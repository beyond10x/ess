//! The held lifecycle state as a value source, `{subject: state}` (ess/23, beyond10x/ess#458).
//!
//! A refusal's error field may say which state the record holds: `current: {subject: state}`, read
//! as the row was immediately before the outcome. It is admitted wherever `{subject: …}` already is
//! — an error payload, an event payload and `sets:` — from `ess/23`, typed as the entity's own
//! `State`, and refused where no row is read with the diagnostics `{subject: …}` already gives.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const MODEL: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/subject-state-source.yaml");

const PUBLISH_CONFLICT: &str = "command.demo.docs.PublishDoc.outcomes.conflict";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("docs.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> Vec<ValidationError> {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
        .as_slice()
        .to_vec()
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

#[test]
fn subject_state_source_validates_on_wrong_state() {
    let spec = accepted(MODEL);
    let command = &spec.commands()[&"demo.docs.PublishDoc".parse().unwrap()];
    let conflict = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "conflict")
        .unwrap();
    assert_eq!(
        conflict.error_payload["current"],
        ess_domain::command::PayloadSource::SubjectField {
            field: "state".to_owned()
        }
    );
}

#[test]
fn subject_state_source_refused_below_ess23() {
    // Only the publishing command's error reads it, so the refusal is that one source's.
    let text = replaced(MODEL, "format: ess/23\n", "format: ess/22\n");
    let text = replaced(
        &text,
        "          demo.docs.DocPublished: {doc_id: input.doc_id, from: {subject: state}}\n",
        "          demo.docs.DocPublished: {doc_id: input.doc_id, from: Draft}\n",
    );
    let text = replaced(
        &text,
        "        sets: {previous: {subject: state}}\n",
        "        sets: {previous: Published}\n",
    );
    let text = replaced(
        &text,
        "{doc_id: input.doc_id, current: {subject: state}, requested: Archived}",
        "{doc_id: input.doc_id, current: Published, requested: Archived}",
    );
    let errors = refused(&text);
    assert_eq!(errors.len(), 1, "{errors:#?}");
    let error = &errors[0];
    assert_eq!(
        error.code,
        ValidationCode::UnsupportedFormatVersion,
        "{error:#?}"
    );
    assert_eq!(
        error.location,
        format!("{PUBLISH_CONFLICT}.payload.demo.docs.StateConflict.current")
    );
    assert!(error.message.contains("ess/23"), "{error:#?}");
    assert!(error.message.contains("{subject: state}"), "{error:#?}");
}

#[test]
fn subject_state_source_refused_without_a_row() {
    // An input-guarded refusal is answered before any row is read.
    let guarded = replaced(
        MODEL,
        "      - {name: doc_id, type: demo.docs.DocId}\n    outcomes:\n      - name: published\n",
        "      - {name: doc_id, type: demo.docs.DocId}\n      - {name: hold, type: Boolean}\n    outcomes:\n      - name: held\n        when: hold == true\n        error: demo.docs.StateConflict\n        payload:\n          demo.docs.StateConflict: {doc_id: input.doc_id, current: {subject: state}, requested: Published}\n      - name: published\n",
    );
    let errors = refused(&guarded);
    assert!(
        errors.iter().any(|error| error.code == ValidationCode::UndeclaredReference
            && error.location
                == "command.demo.docs.PublishDoc.outcomes.held.payload.demo.docs.StateConflict.current"
            && error.message.contains("`{subject: …}`")),
        "{errors:#?}"
    );
    // `unknown_instance:` answers an identity no row carries.
    let unknown = replaced(
        MODEL,
        "        error: demo.docs.NoSuchDoc\n        payload:\n          demo.docs.NoSuchDoc: {doc_id: input.doc_id}\n",
        "        error: demo.docs.StateConflict\n        payload:\n          demo.docs.StateConflict: {doc_id: input.doc_id, current: {subject: state}, requested: Published}\n",
    );
    let errors = refused(&unknown);
    assert!(
        errors.iter().any(|error| error.code == ValidationCode::UndeclaredReference
            && error.location
                == "command.demo.docs.PublishDoc.outcomes.missing.payload.demo.docs.StateConflict.current"),
        "{errors:#?}"
    );
    // A creation has no row before it.
    let creating = replaced(
        MODEL,
        "  - name: demo.docs.DocCreated\n    fields:\n      - {name: doc_id, type: demo.docs.DocId}\n",
        "  - name: demo.docs.DocCreated\n    fields:\n      - {name: doc_id, type: demo.docs.DocId}\n      - {name: was, type: demo.docs.Doc.State}\n",
    );
    let creating = replaced(
        &creating,
        "          demo.docs.DocCreated: {doc_id: input.doc_id}\n",
        "          demo.docs.DocCreated: {doc_id: input.doc_id, was: {subject: state}}\n",
    );
    let errors = refused(&creating);
    assert!(
        errors.iter().any(|error| {
            error.code == ValidationCode::ConflictingDeclaration
            && error.location
                == "command.demo.docs.CreateDoc.outcomes.created.payload.demo.docs.DocCreated.was"
            && error.message.contains("`creates:`")
        }),
        "{errors:#?}"
    );
}

#[test]
fn subject_state_source_type_checked() {
    let text = replaced(
        MODEL,
        "{doc_id: input.doc_id, current: {subject: state}, requested: Published}",
        "{doc_id: {subject: state}, current: {subject: state}, requested: Published}",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.location
                    == format!("{PUBLISH_CONFLICT}.payload.demo.docs.StateConflict.doc_id")
                && error.message.contains("demo.docs.Doc.State")),
        "{errors:#?}"
    );
}

#[test]
fn subject_state_source_in_event_payload_and_sets() {
    let spec = accepted(MODEL);
    let archive = &spec.commands()[&"demo.docs.ArchiveDoc".parse().unwrap()];
    let archived = &archive.outcomes[0];
    assert_eq!(
        archived.sets["previous"],
        ess_domain::command::PayloadSource::SubjectField {
            field: "state".to_owned()
        }
    );
}

#[test]
fn commands_and_outcomes_page_states_the_source() {
    let page = std::fs::read_to_string(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("../../../website/docs/guides/specify/commands-and-outcomes.md"),
    )
    .unwrap();
    for phrase in [
        "`{subject: state}`",
        "`ess/23`",
        "the state before the move",
    ] {
        assert!(page.contains(phrase), "the page names {phrase}");
    }
    // At the current-state field, and in the list of sources.
    let current = page
        .find("An error field that describes the current state")
        .expect("the current-state paragraph");
    let next = page[current..]
        .find("\n\n")
        .map_or(page.len(), |end| current + end);
    assert!(
        page[current..next].contains("`{subject: state}`"),
        "the current-state paragraph names the source"
    );
}
