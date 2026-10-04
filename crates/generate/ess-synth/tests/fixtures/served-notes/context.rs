use notebook_server::memory::MemoryPorts;
use notebook_types::behaviour::{Context, ExternalCommand, Generated, NoteStorage, TryContext};
use notebook_types::notes::{
    self,
    obligations::{AddNoteBehavior, ArchiveNoteBehavior, ProbeBehavior, RemoveNoteBehavior},
};

struct Legacy;
impl Context for Legacy {
    fn caller_author(&self) -> Option<String> {
        Some("legacy caller".into())
    }
    fn generate_string(&mut self) -> String {
        "legacy assigned".into()
    }
    fn external(&mut self, _: ExternalCommand<'_>, _: &'static str) -> bool {
        false
    }
}

#[test]
fn legacy_context_adapts_without_source_changes() {
    let mut legacy = Legacy;
    assert_eq!(
        legacy.try_caller_author().unwrap().as_deref(),
        Some("legacy caller")
    );
    assert_eq!(legacy.try_generate_string().unwrap(), "legacy assigned");
    assert!(!legacy
        .try_external(
            ExternalCommand::NotebookNotesProbe(&notes::Probe {}),
            "external"
        )
        .unwrap());
}

#[test]
fn missing_context_answers_are_typed_and_leave_storage_unchanged() {
    let mut ports = MemoryPorts::default();
    assert_eq!(
        ports.try_caller_author().unwrap_err().source,
        "caller attribute: author"
    );
    assert_eq!(
        ports.try_generate_string().unwrap_err().source,
        "assigned value: String"
    );
    assert_eq!(
        ports
            .try_external(
                ExternalCommand::NotebookNotesProbe(&notes::Probe {}),
                "external"
            )
            .unwrap_err()
            .source,
        "external branch answer"
    );
    let mut generated = Generated::new(ports.clone());
    let creation = generated.add_note(notes::AddNote {
        note_id: notes::NoteId(7),
        text: "seven".into(),
    });
    assert_eq!(creation.unwrap_err().source, "assigned value: String");
    assert!(
        ports.list().is_empty(),
        "late event assignment must not leave the created row"
    );
    ports.put(notes::NoteSnapshot {
        state: notes::NoteState::Active,
        data: notes::NoteData {
            note_id: notes::NoteId(7),
            text: "seven".into(),
        },
    });
    let moved = generated.archive_note(notes::ArchiveNote {
        note_id: notes::NoteId(7),
    });
    assert_eq!(moved.unwrap_err().source, "caller attribute: author");
    assert_eq!(
        ports.get(&notes::NoteId(7)).unwrap().state,
        notes::NoteState::Active
    );
    let removed = generated.remove_note(notes::RemoveNote {
        note_id: notes::NoteId(7),
    });
    assert_eq!(removed.unwrap_err().source, "caller attribute: author");
    assert_eq!(
        ports.list().len(),
        1,
        "late event assignment must not delete the row"
    );
    assert_eq!(
        generated.probe(notes::Probe {}).unwrap_err().source,
        "external branch answer"
    );
}
