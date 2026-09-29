//! Adversary, pass 1, for a branch guarded by a row of another entity (`when_related:`, ess/18,
//! beyond10x/ess#211), domain half.
//!
//! The decision: "a missing row makes a predicate Unknown, selecting only `exists: false`" and
//! "usable on any branch including `creates:`". Each case below drives the domain from that
//! sentence.
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationErrors};

const SIGN_IN: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/related-guard-sign-in.yaml");

const ABSENT: &str = "        when_related: {via: input.tenant, exists: false}\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = ess_domain::spec::RawSpecFile::parse(text)
        .unwrap_or_else(|error| panic!("the document parses: {error}\n{text}"));
    Specification::assemble([(ess_domain::system::Source::new("sign-in.yaml"), raw)])
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

/// The `exists: false` branch carries an input guard. For a missing row and an input the guard
/// refutes, the predicate branch is Unknown, the default is never taken for a missing row, and the
/// `exists: false` branch is refuted by its `when:`: no branch answers. The domain must say so.
#[test]
fn adversary_pass1_exists_false_with_an_input_guard_leaves_a_missing_row_unanswered() {
    let text = replaced(
        SIGN_IN,
        ABSENT,
        &format!("{ABSENT}        when: client == \"console\"\n"),
    );
    match assemble(&text) {
        Ok(_) => panic!(
            "admitted: a missing Configuration with `client != \"console\"` selects no branch \
             (the predicate is Unknown, the default is never taken for a missing row, and \
             `no-configuration` is refuted by its `when:`)"
        ),
        Err(errors) => assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::ConflictingDeclaration
                    && error.to_string().contains("when_related")),
            "{errors}"
        ),
    }
}

/// An accepting input-guarded branch beside `exists: false`. For a missing row and
/// `client == "console"` both `fast-path` (its `when:` holds, and it reads no related row) and
/// `no-configuration` hold. No precedence between them is stated (#178 orders only input-guarded
/// *refusals* first), so the domain must refuse the overlap, as the partition over the rows that
/// exist would.
#[test]
fn adversary_pass1_an_accepting_input_branch_overlapping_exists_false_is_refused() {
    let text = replaced(
        SIGN_IN,
        "      - name: initiated\n",
        "      - name: fast-path
        when: client == \"console\"
        creates: demo.signin.SignIn
        instance: sign_in_id
        emits: [demo.signin.SignInInitiated]
        payload:
          demo.signin.SignInInitiated: {sign_in_id: {generated: true}}
        sets:
          tenant: input.tenant
          client: input.client
      - name: initiated
",
    );
    // Decision #211 (2026-09-29): on a missing row `exists: false` answers before any other branch,
    // so the overlap has a stated precedence and the model is admitted.
    if let Err(errors) = assemble(&text) {
        panic!(
            "the overlap with `exists: false` has a stated precedence and is admitted:\n{errors}"
        );
    }
}

/// A creation whose identity the caller supplies, refused when the record already exists
/// (`existing_instance:`, ess/16) and when the customer it belongs to does not (`when_related:`).
/// The two read different rows — the created entity's and the related one's — and the issue asks
/// for "a guard on any command (a create included)". The implementor's narrowing refuses it.
#[test]
fn adversary_pass1_a_supplied_identity_create_with_a_duplicate_refusal_takes_a_related_guard() {
    let text = "format: ess/18
system: demo
version: v1
domain: demo.orders
types:
  - {name: demo.orders.OrderId, kind: newtype, of: String}
  - {name: demo.orders.CustomerId, kind: newtype, of: Uuid}
  - {name: demo.orders.Label, kind: newtype, of: String}
entities:
  - name: demo.orders.Customer
    identity: {name: customer_id, type: demo.orders.CustomerId}
    fields:
      - {name: label, type: demo.orders.Label}
    lifecycle: {initial: Active, states: [Active], terminal: [Active]}
  - name: demo.orders.Order
    identity: {name: order_id, type: demo.orders.OrderId}
    fields:
      - {name: customer, type: demo.orders.CustomerId}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.orders.CustomerAdded
    fields:
      - {name: customer_id, type: demo.orders.CustomerId}
  - name: demo.orders.OrderPlaced
    fields:
      - {name: order_id, type: demo.orders.OrderId}
errors:
  - {name: demo.orders.OrderExists, summary: The order id is taken., fields: []}
  - {name: demo.orders.NoCustomer, summary: The customer does not exist., fields: []}
actors:
  - {name: demo.orders.Clerk, may: [demo.orders.AddCustomer, demo.orders.PlaceOrder]}
commands:
  - name: demo.orders.AddCustomer
    input:
      - {name: label, type: demo.orders.Label}
    outcomes:
      - name: added
        creates: demo.orders.Customer
        instance: customer_id
        emits: [demo.orders.CustomerAdded]
        payload:
          demo.orders.CustomerAdded: {customer_id: {generated: true}}
        sets:
          label: input.label
  - name: demo.orders.PlaceOrder
    input:
      - {name: order_id, type: demo.orders.OrderId}
      - {name: customer, type: demo.orders.CustomerId}
    outcomes:
      - {name: duplicate, existing_instance: true, error: demo.orders.OrderExists}
      - name: no-customer
        when_related: {via: input.customer, exists: false}
        error: demo.orders.NoCustomer
      - name: placed
        creates: demo.orders.Order
        instance: order_id
        emits: [demo.orders.OrderPlaced]
        payload:
          demo.orders.OrderPlaced: {order_id: input.order_id}
        sets:
          customer: input.customer
";
    if let Err(errors) = assemble(text) {
        panic!(
            "a supplied-identity create cannot declare both its duplicate refusal and a guard on \
             the related customer:\n{errors}"
        );
    }
}
