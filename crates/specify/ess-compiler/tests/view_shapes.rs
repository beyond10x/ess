//! Named view shapes survive resolution as references and as checked, normalized fields.

use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;

const SPEC: &str = r"
format: ess/1
system: desk
version: v1
domains: [desk.todo]
domain: desk.todo
types:
  - name: desk.todo.ItemId
    kind: newtype
    of: Uuid
  - name: desk.todo.ItemRow
    kind: struct
    fields:
      - name: item_id
        type: desk.todo.ItemId
      - name: title
        type: String
entities:
  - name: desk.todo.Item
    identity:
      name: item_id
      type: desk.todo.ItemId
    fields:
      - name: title
        type: String
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
views:
  - name: desk.todo.ItemById
    source: desk.todo.Item
    shape: desk.todo.ItemRow
    consistency: read_your_writes
  - name: desk.todo.VisibleItems
    source: desk.todo.Item
    shape: desk.todo.ItemRow
";

#[test]
fn a_shape_is_one_handle_with_checked_fields_in_every_view() {
    let raw = RawSpecFile::parse(SPEC).expect("well formed");
    let specification = Specification::assemble([(Source::new("desk.yaml"), raw)])
        .expect("the named struct agrees with the entity");
    let mut sources = SourceMap::new();
    sources.insert("desk.yaml", SPEC);
    let ir = compile(&specification, &sources).expect("the shape resolves");

    assert_eq!(ir.views().len(), 2);
    for view in ir.views().values() {
        let shape = view
            .shape
            .as_ref()
            .expect("the declaration stays a reference");
        assert_eq!(shape.to_string(), "desk.todo.ItemRow");
        assert_eq!(ir.named_type(shape).name.to_string(), "desk.todo.ItemRow");
        assert_eq!(
            view.fields
                .iter()
                .map(|field| field.name.as_str())
                .collect::<Vec<_>>(),
            ["item_id", "title"],
            "the IR is still normalized for consumers that need concrete fields"
        );
    }

    let persisted = ir.to_canonical_json();
    assert_eq!(
        persisted.matches(r#""shape": "desk.todo.ItemRow""#).count(),
        2,
        "persisted IR must not erase whether reuse was declared: {persisted}"
    );
}
