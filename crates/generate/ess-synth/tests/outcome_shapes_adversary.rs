//! Adversarial cases for the typed constructors `into:` (ess/15) adds to the generated Rust entity.

use std::fmt::Write as _;

use ess_compiler::ir::EssIr;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_synth::{synthesize_for, Target};

fn model(states: &str, transitions: &str, into: &str) -> String {
    let names: Vec<&str> = transitions
        .lines()
        .filter_map(|line| line.split("name: ").nth(1)?.split(',').next())
        .collect();
    let mut movers = String::new();
    let mut may = String::new();
    for (index, transition) in names.iter().enumerate() {
        let _ = write!(may, ", example.doc.Move{index}");
        let _ = write!(
            movers,
            "  - name: example.doc.Move{index}\n    input:\n      - {{name: doc_id, type: example.doc.DocId}}\n    outcomes:\n      - name: moved\n        moves: example.doc.Doc.{transition}\n        instance: doc_id\n        emits: [example.doc.Moved]\n        payload:\n          example.doc.Moved: {{doc_id: input.doc_id}}\n      - {{name: elsewhere, wrong_state: true, refuses: false}}\n"
        );
    }
    format!(
        "format: ess/15
system: example
version: v1
domain: example.doc
types:
  - {{name: example.doc.DocId, kind: newtype, of: Uuid}}
entities:
  - name: example.doc.Doc
    identity: {{name: doc_id, type: example.doc.DocId}}
    lifecycle:
      initial: Draft
      states: [{states}]
      terminal: [{into}]
      transitions:
{transitions}
events:
  - name: example.doc.Drafted
    fields: [{{name: doc_id, type: example.doc.DocId}}]
  - name: example.doc.Imported
    fields: [{{name: doc_id, type: example.doc.DocId}}]
  - name: example.doc.Moved
    fields: [{{name: doc_id, type: example.doc.DocId}}]
actors:
  - name: example.doc.Author
    may: [example.doc.Draft, example.doc.Import{may}]
commands:
  - name: example.doc.Draft
    input:
      - {{name: doc_id, type: example.doc.DocId}}
    outcomes:
      - name: drafted
        creates: example.doc.Doc
        instance: doc_id
        emits: [example.doc.Drafted]
        payload:
          example.doc.Drafted: {{doc_id: input.doc_id}}
  - name: example.doc.Import
    input:
      - {{name: doc_id, type: example.doc.DocId}}
    outcomes:
      - name: imported
        creates: example.doc.Doc
        instance: doc_id
        into: {into}
        emits: [example.doc.Imported]
        payload:
          example.doc.Imported: {{doc_id: input.doc_id}}
{movers}"
    )
}

fn ir(text: &str) -> EssIr {
    let raw = RawSpecFile::parse(text).unwrap();
    let spec = Specification::assemble([(Source::new("adversary.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("{errors}"));
    compile(&spec, &SourceMap::new()).unwrap()
}

/// Every `pub fn <name>(` the Rust target emits, where synthesis admits the model.
fn methods(ir: &EssIr) -> Option<Vec<String>> {
    let synthesis = synthesize_for(ir, Target::Rust).ok()?;
    let mut names = Vec::new();
    for artifact in synthesis.artifacts.values() {
        for piece in artifact.contents.split("pub fn ").skip(1) {
            if let Some((name, _)) = piece.split_once('(') {
                names.push(name.to_owned());
            }
        }
    }
    Some(names)
}

fn is_ident(token: &str) -> bool {
    let bare = token.strip_prefix("r#").unwrap_or(token);
    let mut chars = bare.chars();
    matches!(chars.next(), Some(c) if c.is_ascii_alphabetic() || c == '_')
        && chars.all(|c| c.is_ascii_alphanumeric() || c == '_')
}

#[test]
fn adversary_a_creation_into_a_state_named_for_a_keyword_emits_a_valid_constructor() {
    // `Final` is a legal state name and a reserved Rust keyword; the design admits `into:` a
    // terminal state (open question 1). Target admission must refuse the model, or the constructor
    // must be a Rust identifier.
    let text = model(
        "Draft, Final",
        "        - {name: finalize, from: [Draft], to: Final}",
        "Final",
    );
    let Some(names) = methods(&ir(&text)) else {
        return;
    };
    let invalid: Vec<&String> = names.iter().filter(|name| !is_ident(name)).collect();
    assert!(
        invalid.is_empty(),
        "the generated Rust declares methods that are not identifiers: {invalid:?}"
    );
}

#[test]
fn adversary_a_constructor_does_not_collide_with_a_transition_method_of_its_state() {
    // `new-archived` out of `Archived` is a method on `Doc<Archived>`, and so is the constructor
    // `into: Archived` generates: two inherent methods of one name on one type do not compile.
    let text = model(
        "Draft, Archived, Gone",
        "        - {name: archive, from: [Draft], to: Archived}\n        - {name: new-archived, from: [Archived], to: Gone}",
        "Archived",
    )
    .replace("terminal: [Archived]", "terminal: [Gone]");
    let Some(names) = methods(&ir(&text)) else {
        return;
    };
    let count = names.iter().filter(|name| *name == "new_archived").count();
    assert!(
        count <= 1,
        "`Doc<Archived>` declares `new_archived` {count} times: a constructor and a transition"
    );
}
