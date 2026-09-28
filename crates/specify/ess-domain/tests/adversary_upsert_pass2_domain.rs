//! Adversary, pass 2, for selection by existence (ess/16, beyond10x/ess#164 and its follow-up
//! comment, `docs/design/outcome-shapes.md`), domain half.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("items.yaml"), raw)])
}

/// One `demo.items.CreateItem` with `input` as its fields and `outcomes` as its branches.
fn items(input: &str, outcomes: &str) -> String {
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
errors:
  - name: demo.items.ItemAlreadyExists
    summary: An item with this id is already stored.
    fields: []
actors:
  - {{name: demo.items.Admin, may: [demo.items.CreateItem]}}
commands:
  - name: demo.items.CreateItem
    input:
{input}    outcomes:
{outcomes}"
    )
}

const REQUIRED_ID: &str = "      - {name: item_id, type: demo.items.ItemId}
      - {name: label, type: demo.items.Label}
";

const EXISTS: &str =
    "      - {name: already-exists, existing_instance: true, error: demo.items.ItemAlreadyExists}\n";

fn created(identity: &str) -> String {
    format!(
        "      - name: created
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {{item_id: {identity}, label: input.label}}
        sets:
          label: input.label
"
    )
}

/// Baseline: the follow-up with a required id is admitted.
#[test]
fn adversary_pass2_baseline_create_or_refuse_with_a_required_id_is_admitted() {
    let body = items(
        REQUIRED_ID,
        &format!("{}{EXISTS}", created("input.item_id")),
    );
    if let Err(errors) = assemble(&body) {
        panic!("the baseline is refused:\n{errors}");
    }
}

/// The #164 follow-up comment, verbatim: "`demo.items.CreateItem` with an **optional**
/// `item_id`. When no row with that id is stored it creates one; when one exists it answers with
/// an error". An optional caller id is written `{input: item_id, else: {generated: true}}` (ess/14,
/// #137): the caller's id when sent, a minted one when not. A caller that sends an id can send it
/// twice, so the `existing_instance:` branch is reachable; it is refused as
/// `unreachable_branch` ("no call can name an existing one") because `created_identity` only
/// recognises a bare `input.` source.
#[test]
fn adversary_pass2_the_follow_up_repro_with_an_optional_id_is_admitted() {
    let input = "      - {name: item_id, type: Optional<demo.items.ItemId>}
      - {name: label, type: demo.items.Label}
";
    let body = items(
        input,
        &format!(
            "{}{EXISTS}",
            created("{input: item_id, else: {generated: true}}")
        ),
    );
    if let Err(errors) = assemble(&body) {
        panic!(
            "the #164 follow-up (an optional caller-supplied id, refused when taken) cannot be \
             written:\n{errors}"
        );
    }
}

/// `existing_instance:` is "refused and changes nothing" (`validate_existing`); `refuses: false`
/// says the opposite — the request is accepted. `wrong_state:` and `unknown_instance:` are the
/// only markers `refuses:` belongs to, and the round trip `From<Outcome> for RawOutcome` writes
/// `refuses:` back only for those two, so an admitted `refuses: false` here is also lost on
/// re-serialisation.
#[test]
fn adversary_pass2_existing_instance_with_refuses_false_is_refused() {
    let body = items(
        REQUIRED_ID,
        &format!(
            "{}      - {{name: already-exists, existing_instance: true, refuses: false, error: demo.items.ItemAlreadyExists}}\n",
            created("input.item_id")
        ),
    );
    match assemble(&body) {
        Ok(_) => panic!("`existing_instance: true` beside `refuses: false` is admitted"),
        Err(errors) => assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.to_string().contains("already-exists")),
            "expected a refusal at `already-exists`, got:\n{errors}"
        ),
    }
}

/// The design note: "In each form the other branch is unconditional, so the pair counts as
/// exhaustive". A creating `unknown_instance:` guarded by `when:` leaves an unknown identity sent
/// with any other label answered by nothing, and the command must not be admitted as exhaustive.
#[test]
fn adversary_pass2_a_creating_unknown_instance_guarded_by_input_is_refused() {
    let body = items(
        REQUIRED_ID,
        "      - name: updated
        updates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
      - name: created
        unknown_instance: true
        when: label == \"x\"
        creates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
",
    );
    match assemble(&body) {
        Ok(_) => panic!(
            "a creating `unknown_instance:` guarded by `when:` is admitted; an unknown identity \
             sent with a label other than `x` selects no branch"
        ),
        Err(errors) => assert!(
            errors.as_slice().iter().any(|error| matches!(
                error.code,
                ValidationCode::ConflictingDeclaration | ValidationCode::NonExhaustiveBranches
            )),
            "expected a refusal of the guarded creation, got:\n{errors}"
        ),
    }
}
