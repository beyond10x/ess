//! An outcome that reports an error declares where the error's fields come from
//! (`story:error-payload-sources`, `ess/19`).
//!
//! `payload:` keyed by the error the branch reports takes the sources an event payload takes: the
//! input, a literal, the row the refusal is answered for, the caller and a generated value. An
//! older header refuses it by name; a source whose type is not the error field's, and a field the
//! error does not declare, are refused with the codes an event payload's are.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// Every source kind but the caller's and a generated value, over three refusal positions.
const ORDERS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/error-payload-sources.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("orders.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, fragment: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.to_string().contains(fragment))
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

fn at(text: &str, format: &str) -> String {
    replaced(text, "format: ess/19\n", &format!("format: {format}\n"))
}

#[test]
fn error_payload_sources_validate_under_ess_19() {
    accepted(ORDERS);
}

#[test]
fn an_error_payload_keeps_its_sources_on_the_outcome_and_round_trips() {
    let spec = accepted(ORDERS);
    let place = &spec.commands()[&"demo.order.PlaceOrder".parse().unwrap()];
    let refusal = place
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "too-many")
        .unwrap();
    assert!(refusal.payload.is_empty(), "no event payload on a refusal");
    let sources: Vec<String> = refusal
        .error_payload
        .iter()
        .map(|(field, source)| format!("{field}={source}"))
        .collect();
    assert_eq!(
        sources,
        ["limit=10", "reason=Quantity", "requested=input.quantity"]
    );
    let written = serde_yaml::to_string(refusal).expect("an outcome serialises");
    assert!(
        written.contains("demo.order.TooMany"),
        "the error's block is written back under `payload:`:\n{written}"
    );
}

#[test]
fn an_older_header_refuses_an_error_payload_by_name() {
    let errors = refused(&at(ORDERS, "ess/18"));
    for error in [
        "demo.order.TooMany",
        "demo.order.NoSuchOrder",
        "demo.order.AlreadyClosed",
    ] {
        assert!(
            has(
                &errors,
                ValidationCode::UnsupportedFormatVersion,
                &format!("`payload:` for the error `{error}` requires specification format ess/19")
            ),
            "{error}: {errors}"
        );
    }
}

#[test]
fn a_source_whose_type_is_not_the_error_fields_is_refused() {
    // An input of another type.
    let text = replaced(
        ORDERS,
        "demo.order.NoSuchOrder: {order_id: input.order_id}",
        "demo.order.NoSuchOrder: {order_id: input.order_id}\n      - name: spare\n        when: order_id == order_id\n        error: demo.order.TooMany\n        payload:\n          demo.order.TooMany: {requested: input.order_id}",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::TypeMismatch,
            "`demo.order.TooMany.requested` requires `Integer`"
        ),
        "{errors}"
    );
    // A literal no value of the field's type is.
    let errors = refused(&replaced(ORDERS, "reason: Quantity", "reason: Price"));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch
                && error.to_string().contains("Price")),
        "{errors}"
    );
    // A stored field of another type.
    let errors = refused(&replaced(
        ORDERS,
        "order_id: input.order_id, quantity: {subject: quantity}",
        "order_id: {subject: quantity}, quantity: {subject: quantity}",
    ));
    assert!(
        has(&errors, ValidationCode::TypeMismatch, "order_id"),
        "{errors}"
    );
}

#[test]
fn a_payload_naming_a_field_the_error_does_not_declare_is_refused() {
    let errors = refused(&replaced(
        ORDERS,
        "demo.order.NoSuchOrder: {order_id: input.order_id}",
        "demo.order.NoSuchOrder: {order_id: input.order_id, reference: input.order_id}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::UndeclaredReference,
            "`reference` is not a field `demo.order.NoSuchOrder` carries"
        ),
        "{errors}"
    );
    let repair = errors
        .as_slice()
        .iter()
        .find(|error| error.to_string().contains("`reference`"))
        .and_then(|error| error.hint.clone())
        .unwrap_or_default();
    assert!(
        repair.contains("order_id"),
        "the repair names what it carries: {repair}"
    );
}

#[test]
fn a_payload_for_an_error_the_branch_does_not_report_is_still_refused() {
    let errors = refused(&replaced(
        ORDERS,
        "demo.order.NoSuchOrder: {order_id: input.order_id}",
        "demo.order.AlreadyClosed: {order_id: input.order_id}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::UndeclaredReference,
            "says where `demo.order.AlreadyClosed`'s payload comes from"
        ),
        "{errors}"
    );
}

#[test]
fn a_subject_source_on_a_refusal_that_reads_no_row_is_refused() {
    // The input-guarded refusal answers before any row is read, and there is none: it creates.
    let errors = refused(&replaced(
        ORDERS,
        "{requested: input.quantity, limit: 10, reason: Quantity}",
        "{requested: {subject: quantity}, limit: 10, reason: Quantity}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::UndeclaredReference,
            "`{subject: …}` reads the entity an outcome acts on, and outcome `too-many` acts on \
             no entity"
        ),
        "{errors}"
    );
    // The wrong-state refusal reads the row its siblings name, and a field it does not hold is
    // refused by name.
    let errors = refused(&replaced(
        ORDERS,
        "quantity: {subject: quantity}}",
        "quantity: {subject: amount}}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::UndeclaredReference,
            "`amount` is not a field of `demo.order.Order`"
        ),
        "{errors}"
    );
}

#[test]
fn the_caller_and_a_generated_value_fill_an_error_field() {
    let text = replaced(
        ORDERS,
        "errors:\n",
        "actors:\n  - name: demo.order.Clerk\n    attributes: [{name: clerk_id, type: demo.order.OrderId}]\n    may: [demo.order.PlaceOrder, demo.order.CloseOrder]\n\nerrors:\n  - name: demo.order.Busy\n    fields:\n      - {name: clerk, type: demo.order.OrderId}\n      - {name: ticket, type: demo.order.OrderId}\n",
    );
    let text = replaced(
        &text,
        "      - name: placed\n",
        "      - name: busy\n        when: quantity == 0\n        error: demo.order.Busy\n        payload:\n          demo.order.Busy: {clerk: {caller: clerk_id}, ticket: {generated: true}}\n      - name: placed\n",
    );
    accepted(&text);
    let errors = refused(&replaced(
        &text,
        "{clerk: {caller: clerk_id}",
        "{clerk: {caller: desk_id}",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.to_string().contains("desk_id")),
        "{errors}"
    );
}

#[test]
fn a_response_or_cleared_source_on_an_error_is_refused() {
    let errors = refused(&replaced(
        ORDERS,
        "demo.order.NoSuchOrder: {order_id: input.order_id}",
        "demo.order.NoSuchOrder: {order_id: {response: order_id}}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "`demo.order.NoSuchOrder.order_id` reads the command's response"
        ),
        "{errors}"
    );
    let errors = refused(&replaced(
        ORDERS,
        "demo.order.NoSuchOrder: {order_id: input.order_id}",
        "demo.order.NoSuchOrder: {order_id: {cleared: true}}",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::UndeclaredReference,
            "{cleared: true}"
        ),
        "{errors}"
    );
}
