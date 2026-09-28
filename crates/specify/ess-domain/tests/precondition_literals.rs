//! A precondition's literal input may be structured (beyond10x/ess#205): a list, a map, a struct or
//! an absent optional, held to the input's declared type all the way down to its scalar leaves.
//!
//! Before this, every literal went through the scalar-only `example:` check, so a session-opening
//! command taking a list could not be a precondition at all: the literal was refused as "not a
//! scalar", leaving it out was refused as missing, and a fixture was refused by both explorers.
//! A `Json` or `Binary64` leaf stays refused, as it is for `example:`.

use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

fn assemble(body: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(body)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{body}"));
    Specification::assemble([(ess_domain::system::Source::new("session.yaml"), raw)])
}

fn admitted(input: &str) -> Specification {
    let body = session(input);
    assemble(&body).unwrap_or_else(|errors| panic!("the model is admitted: {errors}\n{body}"))
}

fn refused(input: &str) -> ValidationErrors {
    let body = session(input);
    match assemble(&body) {
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

/// The issue's shape: a session that opens by creating the user with the accounts it may act for,
/// with `input` as the precondition's input map (flow style, without the braces).
fn session(input: &str) -> String {
    format!(
        "format: ess/15
system: shop
version: v1
preconditions:
  - command: shop.session.OpenSession
    as: shop.session.Server
    input: {{{input}}}
domain: shop.session
types:
  - name: shop.session.AccountCode
    kind: newtype
    of: String
    alphabet: abc123
  - name: shop.session.Account
    kind: struct
    fields:
      - {{name: account_id, type: shop.session.AccountCode}}
      - {{name: name, type: String}}
      - {{name: note, type: Optional<String>}}
  - name: shop.session.Window
    kind: struct
    fields:
      - {{name: low, type: Integer}}
      - {{name: high, type: Integer}}
    invariants: [low >= 0]
entities:
  - name: shop.session.User
    identity: {{name: user_id, type: String}}
    fields:
      - {{name: accounts, type: 'List<shop.session.Account>'}}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
actors:
  - {{name: shop.session.Server, may: [shop.session.OpenSession]}}
events:
  - {{name: shop.session.SessionOpened, fields: [{{name: user_id, type: String}}]}}
commands:
  - name: shop.session.OpenSession
    input:
      - {{name: user_id, type: String}}
      - {{name: accounts, type: 'List<shop.session.Account>'}}
      - {{name: limits, type: 'Optional<Map<String, Integer>>'}}
      - {{name: window, type: 'Optional<shop.session.Window>'}}
      - {{name: extra, type: 'Optional<Json>'}}
      - {{name: digest, type: 'Optional<List<Binary64>>'}}
      - {{name: tags, type: 'Optional<List<shop.session.AccountCode>>'}}
    outcomes:
      - name: opened
        creates: shop.session.User
        instance: user_id
        sets: {{accounts: input.accounts}}
        emits: [shop.session.SessionOpened]
        payload: {{shop.session.SessionOpened: {{user_id: input.user_id}}}}
views:
  - name: shop.session.Users
    source: shop.session.User
    consistency: read_your_writes
    fields:
      - {{name: user_id, type: String}}
      - {{name: accounts, type: 'List<shop.session.Account>'}}
"
    )
}

#[test]
fn issue_205_an_empty_list_opens_the_session() {
    let spec = admitted("user_id: u1, accounts: []");
    let input = &spec.system().preconditions[0].input;
    assert!(input.contains_key("accounts"), "{input:?}");
}

#[test]
fn a_list_of_structs_is_held_to_the_element_type() {
    admitted("user_id: u1, accounts: [{account_id: a1, name: First}, {account_id: b2, name: Second, note: kept}]");
    assert_code(
        &refused("user_id: u1, accounts: [{account_id: a1, name: 7}]"),
        ValidationCode::TypeMismatch,
        "accounts",
    );
    assert_code(
        &refused("user_id: u1, accounts: {account_id: a1, name: First}"),
        ValidationCode::TypeMismatch,
        "is not a list",
    );
}

#[test]
fn a_struct_literal_names_its_declared_fields_and_every_required_one() {
    assert_code(
        &refused("user_id: u1, accounts: [{account_id: a1}]"),
        ValidationCode::MissingDeclaration,
        "`name`",
    );
    assert_code(
        &refused("user_id: u1, accounts: [{account_id: a1, name: First, nmae: typo}]"),
        ValidationCode::UndeclaredReference,
        "`nmae`",
    );
    assert_code(
        &refused("user_id: u1, accounts: [{account_id: a1, name: First, note: null, x: 1}]"),
        ValidationCode::UndeclaredReference,
        "`x`",
    );
    admitted("user_id: u1, accounts: [{account_id: a1, name: First, note: null}]");
}

#[test]
fn a_leaf_is_held_to_its_newtype_as_an_example_is() {
    assert_code(
        &refused("user_id: u1, accounts: [{account_id: zz, name: First}]"),
        ValidationCode::ConflictingDeclaration,
        "alphabet",
    );
    admitted("user_id: u1, accounts: [], tags: [abc, c3]");
    assert_code(
        &refused("user_id: u1, accounts: [], tags: [abc, xyz]"),
        ValidationCode::ConflictingDeclaration,
        "alphabet",
    );
}

#[test]
fn a_struct_literal_is_held_to_the_struct_invariants() {
    admitted("user_id: u1, accounts: [], window: {low: 1, high: 2}");
    assert_code(
        &refused("user_id: u1, accounts: [], window: {low: -1, high: 2}"),
        ValidationCode::ConflictingDeclaration,
        "low >= 0",
    );
}

#[test]
fn a_map_literal_is_held_to_its_value_type() {
    admitted("user_id: u1, accounts: [], limits: {daily: 5, weekly: 20}");
    admitted("user_id: u1, accounts: [], limits: {}");
    assert_code(
        &refused("user_id: u1, accounts: [], limits: {daily: many}"),
        ValidationCode::TypeMismatch,
        "limits",
    );
}

#[test]
fn an_optional_input_may_be_written_absent() {
    admitted("user_id: u1, accounts: [], window: null, limits: null");
    assert_code(
        &refused("user_id: null, accounts: []"),
        ValidationCode::TypeMismatch,
        "user_id",
    );
    assert_code(
        &refused("user_id: u1, accounts: null"),
        ValidationCode::TypeMismatch,
        "accounts",
    );
}

#[test]
fn json_and_binary64_leaves_stay_refused() {
    assert_code(
        &refused("user_id: u1, accounts: [], extra: {any: thing}"),
        ValidationCode::TypeMismatch,
        "Json",
    );
    assert_code(
        &refused("user_id: u1, accounts: [], digest: [1.5]"),
        ValidationCode::TypeMismatch,
        "Binary64",
    );
    admitted("user_id: u1, accounts: [], extra: null, digest: []");
}
