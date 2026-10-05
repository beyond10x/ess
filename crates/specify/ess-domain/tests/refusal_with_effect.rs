//! A refusal that declares its compensating change (`compensates: true`, ess/22,
//! beyond10x/ess#197, `docs/design/refusal-with-effect.md`).
//!
//! The issue's `JoinOrder` model validates at `ess/22` and is refused below it naming `ess/22`;
//! every unmarked refusal keeps `refusal_mutated_state` with the text it has today; the marker is
//! admitted on an `external:` refusal changing its one addressed row and nowhere else.
use ess_domain::{spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};

const MODEL: &str = include_str!("../../ess-compiler/tests/fixtures/refusal-with-effect.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("order.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{errors}\n{text}"))
}

fn refused(text: &str) -> Vec<ValidationError> {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
        .as_slice()
        .to_vec()
}

fn replaced(text: &str, from: &str, to: &str) -> String {
    let out = text.replace(from, to);
    assert_ne!(out, text, "`{from}` is in the fixture");
    out
}

const MARKER: &str = "        compensates: true\n";

fn only(errors: &[ValidationError]) -> &ValidationError {
    assert_eq!(errors.len(), 1, "{errors:#?}");
    &errors[0]
}

#[test]
fn the_issue_model_validates_at_ess_22() {
    let spec = accepted(MODEL);
    let command = &spec.commands()[&"shop.order.JoinOrder".parse().unwrap()];
    let failed = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "failed")
        .unwrap();
    assert!(failed.compensates);
    assert!(failed.is_refusal());
    assert!(failed.subject.is_some());
    assert!(command
        .outcomes
        .iter()
        .filter(|outcome| outcome.name.as_str() != "failed")
        .all(|outcome| !outcome.compensates));
}

#[test]
fn below_ess_22_the_marker_is_refused_once_naming_ess_22() {
    for format in ["ess/21", "ess/16"] {
        let text = replaced(MODEL, "format: ess/22\n", &format!("format: {format}\n"));
        let errors = refused(&text);
        let error = only(&errors);
        assert_eq!(
            error.code,
            ValidationCode::UnsupportedFormatVersion,
            "{error:#?}"
        );
        assert_eq!(
            error.location,
            "command.shop.order.JoinOrder.outcomes.failed.compensates"
        );
        assert!(error.message.contains("ess/22"), "{error:#?}");
    }
}

#[test]
fn an_unmarked_refusal_with_a_move_is_refused_with_todays_text() {
    let text = replaced(MODEL, MARKER, "");
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(error.code, ValidationCode::RefusalMutatedState);
    assert_eq!(
        error.location,
        "command.shop.order.JoinOrder.outcomes.failed"
    );
    assert_eq!(
        error.message,
        "outcome `failed` reports `shop.order.Refused` and also moves `shop.order.Order`; a refused \
         command changes nothing, so a refusal has no subject"
    );
    assert_eq!(
        error.hint.as_deref(),
        Some("drop the subject from the refusal, and declare it on the branch that succeeds")
    );
    // The same at every format: the rule did not move for an unmarked branch.
    let below = replaced(&text, "format: ess/22\n", "format: ess/21\n");
    let errors = refused(&below);
    assert_eq!(only(&errors), error);
}

#[test]
fn compensates_false_is_the_document_without_the_key() {
    let text = replaced(MODEL, MARKER, "        compensates: false\n");
    let errors = refused(&text);
    assert_eq!(only(&errors).code, ValidationCode::RefusalMutatedState);
}

#[test]
fn the_marker_on_a_branch_that_names_no_error_is_a_conflict() {
    // The accepting branch, marked.
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.join\n",
        "        compensates: true\n        moves: shop.order.Order.join\n",
    );
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::ConflictingDeclaration,
        "{error:#?}"
    );
    assert_eq!(
        error.location,
        "command.shop.order.JoinOrder.outcomes.joined.compensates"
    );
}

#[test]
fn a_marked_refusal_that_changes_nothing_is_missing_its_change() {
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n",
        "",
    );
    // `reset` now has no cause, which is its own refusal; the branch's is the marker's.
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::MissingDeclaration
                && error.location == "command.shop.order.JoinOrder.outcomes.failed.compensates"),
        "{errors:#?}"
    );
    assert!(
        !errors
            .iter()
            .any(|error| error.code == ValidationCode::RefusalMutatedState),
        "{errors:#?}"
    );
}

#[test]
fn a_marked_update_that_sets_nothing_is_missing_its_change() {
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n",
        "        updates: shop.order.Order\n        instance: order_id\n",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::MissingDeclaration
                && error.location == "command.shop.order.JoinOrder.outcomes.failed.compensates"),
        "{errors:#?}"
    );
}

#[test]
fn a_marked_update_that_sets_a_field_is_admitted() {
    // `reset` keeps a cause through a second command.
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n",
        "        updates: shop.order.Order\n        instance: order_id\n        sets: {failure: input.reason}\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "  - name: shop.order.CloseOrder\n    input:\n      - {name: order_id, type: shop.order.OrderId}\n    outcomes:\n      - name: reset\n        moves: shop.order.Order.reset\n        instance: order_id\n        emits: [shop.order.OrderJoined]\n        payload:\n          shop.order.OrderJoined: {order_id: input.order_id}\nevents:\n",
    );
    accepted(&text);
}

#[test]
fn the_marker_is_admitted_only_on_an_external_refusal() {
    // An input-guarded refusal, marked.
    let text = replaced(
        MODEL,
        "        external: the upstream refuses the join\n",
        "        when: reason == \"refused\"\n",
    );
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::RefusalMutatedState,
        "{error:#?}"
    );
    assert_eq!(
        error.location,
        "command.shop.order.JoinOrder.outcomes.failed"
    );
    assert!(error.message.contains("external:"), "{error:#?}");
}

#[test]
fn an_external_refusal_with_an_input_eligibility_may_compensate() {
    let text = replaced(
        MODEL,
        "        external: the upstream refuses the join\n",
        "        external: the upstream refuses the join\n        when: reason != \"\"\n",
    );
    accepted(&text);
}

#[test]
fn a_marked_refusal_still_emits_nothing() {
    let text = replaced(
        MODEL,
        "        sets: {failure: input.reason}\n      - name: closed\n",
        "        sets: {failure: input.reason}\n        emits: [shop.order.OrderJoined]\n        payload:\n          shop.order.OrderJoined: {order_id: input.order_id}\n      - name: closed\n",
    );
    let errors = refused(&text);
    let error = only(&errors);
    assert_eq!(
        error.code,
        ValidationCode::RefusalMutatedState,
        "{error:#?}"
    );
    assert!(error.message.contains("also emits"), "{error:#?}");
}

#[test]
fn a_marked_refusal_changes_no_other_row() {
    let text = replaced(
        MODEL,
        "        sets: {failure: input.reason}\n      - name: closed\n",
        "        sets: {failure: input.reason}\n        affects:\n          - entity: shop.order.Order\n            where: failure == input.reason\n            sets: {failure: input.reason}\n      - name: closed\n",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::RefusalMutatedState
                && error.location == "command.shop.order.JoinOrder.outcomes.failed"
                && error.message.contains("affects:")),
        "{errors:#?}"
    );
}

#[test]
fn a_marked_refusal_creates_nothing() {
    let text = replaced(
        MODEL,
        "        moves: shop.order.Order.reset\n        instance: order_id\n        sets: {failure: input.reason}\n",
        "        deletes: shop.order.Order\n        instance: order_id\n",
    );
    let errors = refused(&text);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::RefusalMutatedState
                && error.location == "command.shop.order.JoinOrder.outcomes.failed"),
        "{errors:#?}"
    );
}

#[test]
fn a_transition_only_a_compensating_refusal_takes_has_a_cause() {
    // `reset` is taken by `failed` alone in the fixture; the model is admitted, so no
    // `missing_causation` was raised for it.
    let spec = accepted(MODEL);
    assert!(spec
        .commands()
        .values()
        .flat_map(|command| &command.outcomes)
        .filter(
            |outcome| outcome.subject.as_ref().is_some_and(|subject| subject
                .effect
                .transition()
                .is_some_and(|transition| transition == "reset"))
        )
        .all(|outcome| outcome.compensates));
}
