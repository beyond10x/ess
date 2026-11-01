//! Adversary pass 1 on D343 U4 (`story:short-names-combine-with-all-commands-and-all-views`):
//! under `--names short` a body root `<owner>.<member>` shortens to the owner's last segment and
//! the member, while declared type and event short names stay their last segment.

use std::collections::BTreeSet;

use ess_compiler::{resolve::compile, source::SourceMap};
use ess_domain::{spec::RawSpecFile, spec::Specification, system::Source};
use ess_gen::schema::ModelTypes;
use schema_contract::realize::Plan;

fn ir(files: &[(&str, &str)]) -> ess_compiler::ir::EssIr {
    let mut sources = SourceMap::new();
    for (name, text) in files {
        sources.insert(*name, *text);
    }
    let specification = Specification::assemble(files.iter().map(|(name, text)| {
        (
            Source::new(*name),
            RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{name} parses: {error}")),
        )
    }))
    .unwrap_or_else(|errors| panic!("assembles: {errors}"));
    compile(&specification, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

/// `--all-types --all-events --all-commands --all-views`.
fn everything(ir: &ess_compiler::ir::EssIr) -> ModelTypes {
    let mut roots: BTreeSet<String> = ir.types().keys().map(ToString::to_string).collect();
    roots.extend(ir.events().keys().map(ToString::to_string));
    roots.extend(ModelTypes::command_roots(ir));
    roots.extend(ModelTypes::view_roots(ir));
    ModelTypes::select(ir, &roots).unwrap_or_else(|errors| panic!("selects: {errors:?}"))
}

/// A declared type whose concept segment is `row` and an event whose concept segment is `input`.
/// The qualified-name grammar accepts a lower-case concept segment (`ess_domain::name`).
const LOWER_CONCEPTS: &str = r"format: ess/22
system: demo
version: v1
domain: demo.notes
types:
  - name: demo.notes.row
    kind: struct
    fields:
      - {name: title, type: String}
  - name: demo.notes.Holder
    kind: struct
    fields:
      - {name: row, type: demo.notes.row}
events:
  - name: demo.notes.input
    fields:
      - {name: title, type: String}
commands:
  - name: demo.notes.Create
    input:
      - {name: title, type: String}
    outcomes:
      - name: done
        emits: [demo.notes.input]
        payload:
          demo.notes.input: {title: input.title}
";

#[test]
fn a_declared_type_named_row_keeps_its_last_segment_under_all_types() {
    let ir = ir(&[("notes.yaml", LOWER_CONCEPTS)]);
    let short = Plan::from_model(&everything(&ir))
        .expect("plan")
        .with_short_names()
        .unwrap_or_else(|refused| panic!("refused: {refused}"));
    assert_eq!(
        short.declarations()["demo.notes.row"],
        "Row",
        "a declared type's short name is its last segment (story Decisions: type short names \
         unchanged); it is not a view's body root"
    );
}

#[test]
fn a_declared_type_is_named_alike_whether_selected_or_reached() {
    let ir = ir(&[("notes.yaml", LOWER_CONCEPTS)]);
    let reached = |roots: &[&str]| {
        let roots: BTreeSet<String> = roots.iter().map(|root| (*root).to_owned()).collect();
        Plan::from_model(&ModelTypes::select(&ir, &roots).expect("selects"))
            .expect("plan")
            .with_short_names()
            .expect("short names")
            .declarations()["demo.notes.row"]
            .clone()
    };
    assert_eq!(
        reached(&["demo.notes.Holder"]),
        reached(&["demo.notes.Holder", "demo.notes.row"]),
        "selecting a type explicitly must not rename it"
    );
}

#[test]
fn an_event_named_input_keeps_its_last_segment_under_all_events() {
    let ir = ir(&[("notes.yaml", LOWER_CONCEPTS)]);
    let short = Plan::from_model(&everything(&ir))
        .expect("plan")
        .with_short_names()
        .unwrap_or_else(|refused| panic!("refused: {refused}"));
    assert_eq!(
        short.declarations()["demo.notes.input"],
        "Input",
        "an event's short name is its last segment (story Decisions: event short names unchanged)"
    );
    assert_eq!(
        short.declarations()["demo.notes.Create.input"],
        "CreateInput"
    );
}

/// A body root whose short name equals a declared type's in another domain: still refused, naming
/// both. (The qualified names differ: `DemoOtherCreateInput` and `DemoNotesCreateInput`.)
const SHADOWED: &str = r"format: ess/22
system: demo
version: v1
domains: [demo.notes, demo.other]
domain: demo.notes
events:
  - name: demo.notes.Done
    fields:
      - {name: title, type: String}
commands:
  - name: demo.notes.Create
    input:
      - {name: title, type: String}
    outcomes:
      - name: done
        emits: [demo.notes.Done]
        payload:
          demo.notes.Done: {title: input.title}
";

const OTHER: &str = r"domain: demo.other
types:
  - name: demo.other.CreateInput
    kind: struct
    fields:
      - {name: title, type: String}
";

#[test]
fn a_body_root_shadowing_a_declared_type_is_refused_naming_both() {
    let ir = ir(&[("notes.yaml", SHADOWED), ("other.yaml", OTHER)]);
    let refused = Plan::from_model(&everything(&ir))
        .expect("plan")
        .with_short_names()
        .expect_err("CreateInput twice");
    let collision = refused
        .0
        .iter()
        .map(|finding| format!("{finding:?}"))
        .find(|finding| finding.contains("short_name_collision"))
        .unwrap_or_else(|| panic!("no short_name_collision in {refused}"));
    assert!(
        collision.contains("demo.other.CreateInput")
            && collision.contains("demo.notes.Create.input"),
        "{collision}"
    );
}

/// Lower-case, hyphenated and underscored owners, as the qualified-name grammar admits.
const OWNERS: &str = r"format: ess/22
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: title, type: String}
    lifecycle: {initial: Active, states: [Active], terminal: [Active], transitions: []}
events:
  - name: demo.notes.Done
    fields:
      - {name: note_id, type: demo.notes.NoteId}
commands:
  - name: demo.notes.archive-note
    input:
      - {name: note_id, type: demo.notes.NoteId}
    response:
      - {name: note_id, type: demo.notes.NoteId}
    outcomes:
      - name: done
        returns: true
        emits: [demo.notes.Done]
        payload:
          demo.notes.Done: {note_id: input.note_id}
views:
  - name: demo.notes.by_owner
    source: demo.notes.Note
    fields:
      - {name: note_id, type: demo.notes.NoteId}
";

#[test]
fn every_target_declares_body_roots_of_admitted_owner_spellings() {
    let ir = ir(&[("notes.yaml", OWNERS)]);
    let short = Plan::from_model(&everything(&ir))
        .expect("plan")
        .with_short_names()
        .unwrap_or_else(|refused| panic!("refused: {refused}"));
    let declarations = short.declarations();
    assert_eq!(
        declarations["demo.notes.archive-note.input"],
        "ArchiveNoteInput"
    );
    assert_eq!(
        declarations["demo.notes.archive-note.response"],
        "ArchiveNoteResponse"
    );
    assert_eq!(declarations["demo.notes.by_owner.row"], "ByOwnerRow");
    let rust = short.rust("notes").expect("rust").declarations;
    let go = short
        .go("notes", "example.invalid/notes")
        .expect("go")
        .declarations;
    let typescript = short.typescript().declarations;
    for name in ["ArchiveNoteInput", "ArchiveNoteResponse", "ByOwnerRow"] {
        assert!(rust.contains(&format!("pub struct {name} {{")), "{rust}");
        assert!(go.contains(&format!("type {name} struct")), "{go}");
        assert!(
            typescript.contains(&format!("export type {name} = {{")),
            "{typescript}"
        );
    }
    for output in [&rust, &go, &typescript] {
        assert!(!output.contains("DemoNotes"), "{output}");
    }
}
