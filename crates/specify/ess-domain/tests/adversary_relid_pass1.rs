//! Adversary pass 1 for beyond10x/ess#230 (a relation carried by the entity's own identity).
//!
//! Each case names what it asserts and why it is expected to hold.
#![allow(clippy::too_many_lines)]

use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::ValidationErrors;

const SIGN_IN: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("adversary.yaml"), raw)])
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replacen(from, to, 1);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The sign-in fixture, with a second entity keyed by the same `TenantId` (so the type alone names
/// two entities), a `references` relation on `SignIn.tenant` that decides which one the guard reads,
/// and — before the creating branch — a branch that updates the second entity by `input.tenant`.
///
/// No relation is carried by an identity here, and nothing in it uses the construct #230 adds.
fn guard_decided_by_a_later_branch() -> String {
    let mut text = SIGN_IN.to_owned();
    text = replaced(
        &text,
        "  - name: demo.signin.SignIn\n",
        "  - name: demo.signin.Quota\n    identity: {name: tenant, type: demo.signin.TenantId}\n    fields:\n      - {name: used, type: Integer}\n    lifecycle: {initial: Active, states: [Active], terminal: [Active]}\n  - name: demo.signin.SignIn\n",
    );
    text = replaced(
        &text,
        "      - {name: client, type: demo.signin.ClientId}\n    lifecycle: {initial: Initiated",
        "      - {name: client, type: demo.signin.ClientId}\n    relations:\n      - {name: configuration, kind: references, target: demo.signin.Configuration, cardinality: one, via: tenant}\n    lifecycle: {initial: Initiated",
    );
    text = replaced(
        &text,
        "  - name: demo.signin.SignInInitiated\n",
        "  - name: demo.signin.QuotaCounted\n    fields:\n      - {name: tenant, type: demo.signin.TenantId}\n  - name: demo.signin.SignInInitiated\n",
    );
    text = replaced(
        &text,
        "      - name: initiated\n",
        "      - name: counted\n        when: client == \"console\"\n        updates: demo.signin.Quota\n        instance: tenant\n        emits: [demo.signin.QuotaCounted]\n        payload:\n          demo.signin.QuotaCounted: {tenant: input.tenant}\n        sets:\n          used: 1\n      - name: initiated\n",
    );
    text
}

/// The same model with the `counted` branch placed after the creating branch instead of before it.
/// Outcome order is not what a related guard's entity depends on, so both orders must agree.
fn guard_decided_by_an_earlier_branch() -> String {
    let text = guard_decided_by_a_later_branch();
    let counted = "      - name: counted\n        when: client == \"console\"\n        updates: demo.signin.Quota\n        instance: tenant\n        emits: [demo.signin.QuotaCounted]\n        payload:\n          demo.signin.QuotaCounted: {tenant: input.tenant}\n        sets:\n          used: 1\n";
    let without = replaced(&text, counted, "");
    replaced(
        &without,
        "          tenant: input.tenant\n          client: input.client\nviews:",
        &format!("          tenant: input.tenant\n          client: input.client\n{counted}views:"),
    )
}

#[test]
fn a_guard_decided_by_the_creating_branchs_relation_is_not_undone_by_an_earlier_identity_input() {
    // The control: the relation on `SignIn.tenant` decides the guard's entity when the branch that
    // names a `Quota` by `input.tenant` comes after the creating branch.
    assemble(&guard_decided_by_an_earlier_branch()).unwrap_or_else(|errors| {
        panic!("control: the relation on SignIn.tenant decides the guard\n{errors}")
    });
    // The same model with the branches the other way round. `input_carrier` now answers the
    // `counted` branch with the Quota's identity (`identity_from_input`), which carries no relation,
    // and `related_guard::related_entity` takes the first outcome that answers — so the relation
    // that decided the guard before this change is never consulted and the guard is refused as
    // ambiguous between `Configuration` and `Quota`.
    assemble(&guard_decided_by_a_later_branch()).unwrap_or_else(|errors| {
        panic!("the relation on SignIn.tenant still decides the guard\n{errors}")
    });
}

/// A user created under an id its caller supplies. `UserId` is the identity of `User` alone.
fn self_created(email: &str, relations: &str) -> String {
    format!(
        "format: ess/18
system: demo
version: v1
domain: demo.accounts
types:
  - {{name: demo.accounts.UserId, kind: newtype, of: Uuid}}
  - {{name: demo.accounts.Email, kind: newtype, of: String}}
entities:
  - name: demo.accounts.User
    identity: {{name: user_id, type: demo.accounts.UserId}}
    fields:
      - {{name: email, type: demo.accounts.Email}}
{relations}    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
events:
  - name: demo.accounts.UserRegistered
    fields:
      - {{name: user_id, type: demo.accounts.UserId}}
      - {{name: email, type: demo.accounts.Email}}
actors:
  - name: demo.accounts.Operator
    may: [demo.accounts.RegisterUser]
commands:
  - name: demo.accounts.RegisterUser
    input:
      - {{name: user_id, type: demo.accounts.UserId}}
      - {{name: email, type: demo.accounts.Email}}
    outcomes:
      - name: registered
        creates: demo.accounts.User
        instance: user_id
        emits: [demo.accounts.UserRegistered]
        payload:
          demo.accounts.UserRegistered: {{user_id: input.user_id, email: {email}}}
        sets:
          email: input.email
"
    )
}

#[test]
fn a_creating_branch_cannot_read_the_row_it_is_creating_through_its_own_identity() {
    // Control: the model with a plain input validates.
    assemble(&self_created("input.email", "")).unwrap_or_else(|errors| panic!("{errors}"));
    // `{related: {via: user_id}}` on `creates:` reads the identity the branch fills from
    // `input.user_id`; `UserId` names `User` alone, so the related row is the row being created,
    // which does not exist before the outcome. Before this change a subject `via` on `creates:` was
    // admitted only where a `sets:` field held the input, and this was refused.
    let text = self_created("{related: {via: user_id, field: email}}", "");
    assert!(
        assemble(&text).is_err(),
        "a creating branch reads a field of the row it creates, before it exists:\n{text}"
    );
}

#[test]
fn a_relation_from_an_entity_to_itself_through_its_identity_is_refused() {
    // Every row references itself: the relation states nothing, and a related read through it on
    // `creates:` reads the row being created (the case above, now blessed by a declaration).
    let relation = "    relations:\n      - {name: me, kind: references, target: demo.accounts.User, cardinality: one, via: user_id}\n";
    let text = self_created("input.email", relation);
    assert!(
        assemble(&text).is_err(),
        "a `references` from an entity to itself through its own identity is refused:\n{text}"
    );
}
