//! Adversary cases for selection by existence (ess/16, beyond10x/ess#164,
//! `docs/design/outcome-shapes.md`), domain half.
//!
//! The design note's refusal table is driven as the specification it claims to be.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("items.yaml"), raw)])
}

fn items(outcomes: &str) -> String {
    format!(
        "format: ess/16
system: demo
version: v1
domain: demo.items
types:
  - {{name: demo.items.ItemId, kind: newtype, of: String}}
  - {{name: demo.items.Label, kind: newtype, of: String}}
entities:
  - name: demo.items.Item
    identity: {{name: item_id, type: demo.items.ItemId}}
    fields:
      - {{name: label, type: demo.items.Label}}
    lifecycle:
      initial: Active
      states: [Active]
      terminal: [Active]
      transitions: []
events:
  - name: demo.items.ItemStored
    fields:
      - {{name: item_id, type: demo.items.ItemId}}
      - {{name: label, type: demo.items.Label}}
  - name: demo.items.ItemDropped
    fields:
      - {{name: item_id, type: demo.items.ItemId}}
errors:
  - name: demo.items.ItemAlreadyExists
    summary: An item with this id is already stored.
    fields: []
actors:
  - {{name: demo.items.Admin, may: [demo.items.PutItem]}}
commands:
  - name: demo.items.PutItem
    input:
      - {{name: item_id, type: demo.items.ItemId}}
      - {{name: label, type: demo.items.Label}}
    outcomes:
{outcomes}"
    )
}

const CREATED_UNKNOWN: &str = "      - name: created
        unknown_instance: true
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
";

/// `docs/design/outcome-shapes.md`, refusal table, `conflicting_declaration`: a creating
/// `unknown_instance:` "with no sibling `moves:`/`updates:` on the same entity reading `instance:`
/// from that input" is refused, and the refusal's own hint says "declare the `updates:` or `moves:`
/// branch". A `deletes:` sibling is neither, yet it satisfies the pairing check
/// (`names_existing` counts `Effect::Deletes`), so a create-or-delete toggle is admitted as a
/// create-or-update. The conformance side's `paired` (synthesize/existence.rs) counts only
/// `Moves`/`Updates`, so the second call of such a command gets no one-row witness either.
#[test]
fn adversary_a_creation_paired_only_with_a_deletion_is_refused_as_the_design_note_says() {
    let body = items(&format!(
        "      - name: dropped
        deletes: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemDropped]
        payload:
          demo.items.ItemDropped: {{item_id: input.item_id}}
{CREATED_UNKNOWN}"
    ));
    match assemble(&body) {
        Ok(_) => panic!(
            "a creating `unknown_instance:` whose only sibling on the record is a `deletes:` is \
             admitted, though the design note refuses one with no `moves:`/`updates:` sibling"
        ),
        Err(errors) => assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::ConflictingDeclaration
                    && error.to_string().contains("created")),
            "expected conflicting_declaration at `created`, got:\n{errors}"
        ),
    }
}

const UPDATED_WHEN_X: &str = "      - name: updated
        when: label == \"x\"
        updates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
";

/// Baseline: a command whose only branch is guarded by input answers nothing for other input.
#[test]
fn adversary_baseline_an_update_guarded_by_input_alone_is_refused() {
    assert!(
        assemble(&items(UPDATED_WHEN_X)).is_err(),
        "an input-guarded branch with no default is admitted"
    );
}

/// The design note: "In each form the other branch is unconditional, so the pair counts as
/// exhaustive". Here the other branch is not unconditional: an existing identity sent with any
/// label but `x` selects no branch, and the creating marker must not paper over that.
#[test]
fn adversary_a_creation_beside_an_input_guarded_update_only_is_not_exhaustive() {
    assert!(
        assemble(&items(&format!("{UPDATED_WHEN_X}{CREATED_UNKNOWN}"))).is_err(),
        "an existing identity sent with a label other than `x` selects no branch, yet the \
         command is admitted"
    );
}
