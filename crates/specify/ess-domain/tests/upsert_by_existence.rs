//! An outcome selected by whether the addressed record exists (ess/16; beyond10x/ess#164,
//! `docs/design/outcome-shapes.md`).
//!
//! Two forms, one mechanism, both built on the `unknown_instance:` marker of ess/15:
//!
//! * create-or-update: a `creates:` branch marked `unknown_instance: true` is taken when no row
//!   carries the identity the input names, beside the `updates:`/`moves:` branch taken when one
//!   does;
//! * create-or-refuse: a `creates:` branch beside an `existing_instance: true` branch naming the
//!   `error:` reported when a row with that identity already exists.
//!
//! The pair is exhaustive, so neither is refused as undetermined by input (ESS-COMMAND-004). Below
//! `ess/16` each is refused with `unsupported_format_version`.

use ess_domain::command::{OutcomeCondition, RawOutcome, TestStrategy};
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("items.yaml"), raw)])
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

/// The #164 repro, with `outcomes` as the branches of `demo.items.PutItem`.
fn items(format: u32, outcomes: &str) -> String {
    format!(
        "format: ess/{format}
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
      states: [Active, Retired]
      terminal: [Retired]
      transitions:
        - {{name: retire, from: [Active], to: Retired}}
  - name: demo.items.Tag
    identity: {{name: tag_id, type: demo.items.ItemId}}
    fields: []
    lifecycle: {{initial: Open, states: [Open], terminal: [Open], transitions: []}}
events:
  - name: demo.items.ItemStored
    fields:
      - {{name: item_id, type: demo.items.ItemId}}
      - {{name: label, type: demo.items.Label}}
  - name: demo.items.TagStored
    fields:
      - {{name: tag_id, type: demo.items.ItemId}}
errors:
  - name: demo.items.ItemAlreadyExists
    summary: An item with this id is already stored.
    fields: []
actors:
  - {{name: demo.items.Admin, may: [demo.items.PutItem, demo.items.RetireItem]}}
commands:
  - name: demo.items.PutItem
    input:
      - {{name: item_id, type: demo.items.ItemId}}
      - {{name: other_id, type: demo.items.ItemId}}
      - {{name: label, type: demo.items.Label}}
    outcomes:
{outcomes}  - name: demo.items.RetireItem
    input:
      - {{name: item_id, type: demo.items.ItemId}}
    outcomes:
      - name: retired
        moves: demo.items.Item.retire
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {{item_id: input.item_id, label: {{subject: label}}}}
"
    )
}

const UPDATED: &str = "      - name: updated
        updates: demo.items.Item
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}
        sets:
          label: input.label
";

/// The creating branch, with `extra` as further keys on it.
fn created(extra: &str) -> String {
    format!(
        "      - name: created
        creates: demo.items.Item
        instance: item_id
{extra}        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {{item_id: input.item_id, label: input.label}}
        sets:
          label: input.label
"
    )
}

const UNKNOWN: &str = "        unknown_instance: true\n";

const EXISTS: &str =
    "      - {name: already-exists, existing_instance: true, error: demo.items.ItemAlreadyExists}\n";

fn upsert(format: u32) -> String {
    items(format, &format!("{UPDATED}{}", created(UNKNOWN)))
}

fn create_or_refuse(format: u32) -> String {
    items(format, &format!("{}{EXISTS}", created("")))
}

fn put_outcome<'s>(spec: &'s Specification, name: &str) -> &'s ess_domain::command::Outcome {
    spec.commands()
        .get(&"demo.items.PutItem".parse().unwrap())
        .expect("PutItem is declared")
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == name)
        .unwrap_or_else(|| panic!("PutItem/{name} is declared"))
}

// ---- create-or-update ---------------------------------------------------------------------------

#[test]
fn the_issue_repro_without_a_marker_is_still_undetermined_by_input() {
    let errors = refused(&items(16, &format!("{UPDATED}{}", created(""))));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "all unconditional",
    );
}

#[test]
fn a_create_marked_unknown_instance_beside_an_update_is_admitted() {
    let spec = admitted(&upsert(16));
    let created = put_outcome(&spec, "created");
    assert_eq!(created.condition, OutcomeCondition::UnknownInstance);
    assert_eq!(created.test_strategy(), TestStrategy::SendUnknownIdentity);
    assert!(created.error.is_none());
    let raw = RawOutcome::from(created.clone());
    assert!(raw.unknown_instance, "the marker is written back");
    assert_eq!(raw.refuses, None, "a creating marker writes no `refuses:`");
    assert!(raw.creates.is_some(), "the creation is written back");
    let updated = put_outcome(&spec, "updated");
    assert!(
        updated.is_unconditional(),
        "the update is the default branch"
    );
}

#[test]
fn a_create_marked_unknown_instance_beside_a_move_is_admitted() {
    admitted(&items(
        16,
        &format!(
            "      - name: retired
        moves: demo.items.Item.retire
        instance: item_id
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {{item_id: input.item_id, label: input.label}}
{}",
            created(UNKNOWN)
        ),
    ));
}

#[test]
fn a_creating_unknown_instance_is_refused_below_ess_16() {
    let errors = refused(&upsert(15));
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "unknown_instance",
    );
    assert_code(&errors, ValidationCode::UnsupportedFormatVersion, "ess/16");
}

#[test]
fn a_creating_unknown_instance_reports_no_error_and_no_refusal() {
    let errors = refused(&items(
        16,
        &format!(
            "{UPDATED}{}",
            created(
                "        unknown_instance: true\n        error: demo.items.ItemAlreadyExists\n"
            )
        ),
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "unknown_instance",
    );
    let errors = refused(&items(
        16,
        &format!(
            "{UPDATED}{}",
            created("        unknown_instance: true\n        refuses: false\n")
        ),
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "unknown_instance",
    );
}

#[test]
fn a_creating_unknown_instance_needs_a_branch_acting_on_that_identity() {
    // No sibling acts on an existing instance at all.
    let errors = refused(&items(16, &created(UNKNOWN)));
    assert_code(&errors, ValidationCode::UnreachableBranch, "created");

    // A sibling acts on another entity.
    let errors = refused(&items(
        16,
        &format!(
            "      - name: tagged
        updates: demo.items.Tag
        instance: item_id
        emits: [demo.items.TagStored]
        payload:
          demo.items.TagStored: {{tag_id: input.item_id}}
{}",
            created(UNKNOWN)
        ),
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "created");

    // A sibling acts on the same entity through another input.
    let errors = refused(&items(
        16,
        &format!(
            "{}{}",
            UPDATED.replace("instance: item_id", "instance: other_id"),
            created(UNKNOWN)
        ),
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "created");
}

#[test]
fn a_creating_unknown_instance_needs_a_caller_supplied_identity() {
    // Replace only the creating branch's identity source with a generated one.
    let body = upsert(16).replacen(
        "        unknown_instance: true
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: input.item_id, label: input.label}",
        "        unknown_instance: true
        emits: [demo.items.ItemStored]
        payload:
          demo.items.ItemStored: {item_id: {generated: true}, label: input.label}",
        1,
    );
    assert!(
        body.contains("{generated: true}"),
        "the fixture was rewritten"
    );
    let errors = refused(&body);
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "created");
}

#[test]
fn a_creating_unknown_instance_is_one_answer_per_command() {
    let errors = refused(&items(
        16,
        &format!(
            "{UPDATED}{}{}",
            created(UNKNOWN),
            created(UNKNOWN).replace("name: created", "name: created-again")
        ),
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "unknown_instance",
    );
}

// ---- create-or-refuse ---------------------------------------------------------------------------

#[test]
fn a_create_beside_an_existing_instance_refusal_is_admitted() {
    let spec = admitted(&create_or_refuse(16));
    let exists = put_outcome(&spec, "already-exists");
    assert_eq!(exists.condition, OutcomeCondition::ExistingInstance);
    assert_eq!(exists.test_strategy(), TestStrategy::SendExistingIdentity);
    assert_eq!(
        TestStrategy::SendExistingIdentity.as_str(),
        "send_existing_identity"
    );
    assert!(exists.refuses);
    assert!(!exists.is_unconditional());
    assert_eq!(
        exists.error.as_ref().map(ToString::to_string).as_deref(),
        Some("demo.items.ItemAlreadyExists")
    );
    let raw = RawOutcome::from(exists.clone());
    assert!(raw.existing_instance, "the marker is written back");
    assert_eq!(raw.refuses, None);
    assert!(put_outcome(&spec, "created").is_unconditional());
}

#[test]
fn existing_instance_is_refused_below_ess_16() {
    let errors = refused(&create_or_refuse(15));
    assert_code(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "existing_instance",
    );
    assert_code(&errors, ValidationCode::UnsupportedFormatVersion, "ess/16");
}

#[test]
fn existing_instance_names_its_error_and_is_declared_once() {
    let errors = refused(&items(
        16,
        &format!(
            "{}      - {{name: already-exists, existing_instance: true}}\n",
            created("")
        ),
    ));
    assert_code(
        &errors,
        ValidationCode::MissingDeclaration,
        "already-exists",
    );

    let errors = refused(&items(
        16,
        &format!(
            "{}{EXISTS}{}",
            created(""),
            EXISTS.replace("already-exists", "taken")
        ),
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "existing_instance",
    );

    let errors = refused(&items(
        16,
        &format!(
            "{}      - {{name: already-exists, existing_instance: true, refuses: false}}\n",
            created("")
        ),
    ));
    assert_code(&errors, ValidationCode::ConflictingDeclaration, "refuses");
}

#[test]
fn existing_instance_is_one_condition_and_changes_nothing() {
    for marker in [
        "{name: already-exists, existing_instance: true, when: label == \"x\", error: demo.items.ItemAlreadyExists}",
        "{name: already-exists, existing_instance: true, wrong_state: true, error: demo.items.ItemAlreadyExists}",
        "{name: already-exists, existing_instance: true, unknown_instance: true, error: demo.items.ItemAlreadyExists}",
        "{name: already-exists, existing_instance: true, input_absent: true, error: demo.items.ItemAlreadyExists}",
        "{name: already-exists, existing_instance: true, external: the store refuses, error: demo.items.ItemAlreadyExists}",
    ] {
        let errors = refused(&items(16, &format!("{}      - {marker}\n", created(""))));
        assert!(
            errors.as_slice().iter().any(|error| error.code
                == ValidationCode::ConflictingDeclaration
                && (error.to_string().contains("existing_instance")
                    || error.to_string().contains("input_absent"))),
            "`{marker}` is refused as a conflict:\n{errors}"
        );
    }
    let errors = refused(&items(
        16,
        &format!(
            "{}      - {{name: already-exists, existing_instance: true, error: demo.items.ItemAlreadyExists, emits: [demo.items.ItemStored]}}\n",
            created("")
        ),
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "existing_instance",
    );
}

#[test]
fn existing_instance_needs_a_create_with_a_caller_supplied_identity() {
    // No create at all: the command acts on existing rows only.
    let errors = refused(&items(16, &format!("{UPDATED}{EXISTS}")));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("already-exists")),
        "the marker is refused:\n{errors}"
    );

    // A create whose identity the implementation generates.
    let generated = created("").replace(
        "{item_id: input.item_id, label: input.label}",
        "{item_id: {generated: true}, label: input.label}",
    );
    let errors = refused(&items(16, &format!("{generated}{EXISTS}")));
    assert_code(&errors, ValidationCode::UnreachableBranch, "already-exists");
}

#[test]
fn existing_instance_does_not_sit_beside_a_branch_acting_on_the_existing_row() {
    let errors = refused(&items(
        16,
        &format!("{UPDATED}{}{EXISTS}", created(UNKNOWN)),
    ));
    assert_code(
        &errors,
        ValidationCode::ConflictingDeclaration,
        "already-exists",
    );
}

// ---- preconditions ------------------------------------------------------------------------------

const PRECONDITION: &str = "version: v1
preconditions:
  - command: demo.items.PutItem
    as: demo.items.Admin
    input: {item_id: item-1, other_id: item-2, label: first}
";

#[test]
fn a_precondition_cannot_invoke_a_command_selected_by_existence() {
    // Which branch it takes depends on a record a precondition cannot observe before it runs: the
    // default a literal input selects is the update, which a fresh world answers with the creation.
    for body in [upsert(16), create_or_refuse(16)] {
        let errors = refused(&body.replacen("version: v1\n", PRECONDITION, 1));
        assert_code(
            &errors,
            ValidationCode::UnobservableFact,
            "whether a record carries",
        );
    }
}
