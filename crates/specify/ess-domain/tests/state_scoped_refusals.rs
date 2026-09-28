//! A refusal scoped to some held states (beyond10x/ess#201) and a stored-field guard that also
//! reads the held state (beyond10x/ess#204), both `ess/18`.
//!
//! #201: `when_subject_state:` is admitted on a refusal that names no subject of its own — it
//! reads the subject its siblings name — and may list several states. It does not extend to
//! `when_state_changes:`. The branch that answers the states the partition leaves to it must still
//! take a move that starts there.
//!
//! #204: `state` is readable inside a `when_subject` predicate, as the held lifecycle state of the
//! row the command addresses. The one-selection-authority rule stands: `when_subject_state:` beside
//! `when_subject:` is still refused.
use ess_domain::{command::OutcomeCondition, spec::RawSpecFile, system::Source, Specification};
use ess_primitives::error::{ValidationCode, ValidationErrors};

/// The shape of #201, with the two refused states named on one branch.
pub const SHIP: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/state-scoped-refusals.yaml");

/// The listed refusal of [`SHIP`].
pub const LISTED: &str =
    "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n";

/// The issue's own spelling: one refusal per state, each naming its state as a scalar.
pub const SPLIT: &str = "      - name: gone-delivered\n        when_subject_state: Delivered\n        error: demo.ship.Gone\n      - name: gone-cancelled\n        when_subject_state: Cancelled\n        error: demo.ship.Gone\n";

/// The shape of #204.
pub const REPORT: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/state-in-subject-predicate.yaml");

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
    replaced(text, "format: ess/18\n", &format!("format: {format}\n"))
}

// ---- #201 ------------------------------------------------------------------------------------

#[test]
fn a_listed_state_refusal_without_a_subject_validates_under_ess_18() {
    let spec = accepted(SHIP);
    let ship = &spec.commands()[&"demo.ship.ShipOrder".parse().unwrap()];
    let gone = ship
        .outcomes
        .iter()
        .find(|outcome| outcome.name.as_str() == "gone")
        .unwrap();
    let OutcomeCondition::SubjectState { state, predicate } = &gone.condition else {
        panic!("expected a held-state guard, got {:?}", gone.condition)
    };
    assert!(predicate.is_none());
    assert_eq!(
        state.to_string(),
        "Cancelled or Delivered",
        "kept in name order"
    );
    assert!(gone.subject.is_none(), "the refusal names no subject");
}

#[test]
fn the_issue_spelling_with_one_scalar_refusal_per_state_validates_under_ess_18() {
    accepted(&replaced(SHIP, LISTED, SPLIT));
}

#[test]
fn a_subjectless_state_refusal_is_refused_below_ess_18() {
    let split = replaced(SHIP, LISTED, SPLIT);
    for format in ["ess/17", "ess/3"] {
        let errors = refused(&at(&split, format));
        assert!(
            has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/18"),
            "{format}: {errors}"
        );
    }
}

#[test]
fn a_listed_state_is_refused_below_ess_18_even_with_one_state() {
    let one = replaced(
        SHIP,
        LISTED,
        "      - name: gone\n        when_subject_state: [Delivered]\n        error: demo.ship.Gone\n      - name: gone-cancelled\n        when_subject_state: Cancelled\n        error: demo.ship.Gone\n",
    );
    accepted(&one);
    let errors = refused(&at(&one, "ess/17"));
    assert!(
        has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/18"),
        "{errors}"
    );
}

#[test]
fn an_empty_or_repeated_state_list_is_refused() {
    let empty = replaced(
        SHIP,
        "when_subject_state: [Delivered, Cancelled]",
        "when_subject_state: []",
    );
    let errors = refused(&empty);
    assert!(!errors.is_empty(), "{errors}");
    let repeated = replaced(
        SHIP,
        "when_subject_state: [Delivered, Cancelled]",
        "when_subject_state: [Delivered, Delivered, Cancelled]",
    );
    let errors = refused(&repeated);
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "Delivered"),
        "{errors}"
    );
}

#[test]
fn an_undeclared_listed_state_is_refused() {
    let errors = refused(&replaced(
        SHIP,
        "when_subject_state: [Delivered, Cancelled]",
        "when_subject_state: [Delivered, Cancelled, Lost]",
    ));
    assert!(
        has(&errors, ValidationCode::UnknownState, "Lost"),
        "{errors}"
    );
}

#[test]
fn a_state_the_partition_leaves_to_a_default_that_cannot_move_there_is_refused() {
    // Only Delivered is refused: Cancelled falls to `shipped`, whose move starts only in Placed.
    let errors = refused(&replaced(
        SHIP,
        "when_subject_state: [Delivered, Cancelled]",
        "when_subject_state: [Delivered]",
    ));
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "Cancelled"),
        "{errors}"
    );
}

#[test]
fn two_refusals_claiming_one_state_are_refused() {
    let errors = refused(&replaced(
        SHIP,
        LISTED,
        "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n        error: demo.ship.Gone\n      - name: lost\n        when_subject_state: Cancelled\n        error: demo.ship.Gone\n",
    ));
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "Cancelled"),
        "{errors}"
    );
}

#[test]
fn a_subjectless_state_change_refusal_stays_refused() {
    let errors = refused(&replaced(
        SHIP,
        LISTED,
        "      - name: gone\n        when_state_changes: false\n        error: demo.ship.Gone\n",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "when_state_changes"
        ),
        "{errors}"
    );
}

#[test]
fn a_subjectless_accepting_state_branch_stays_refused() {
    let errors = refused(&replaced(
        SHIP,
        LISTED,
        "      - name: gone\n        when_subject_state: [Delivered, Cancelled]\n",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "a subject-state guard requires"
        ),
        "{errors}"
    );
}

#[test]
fn a_listed_state_on_a_branch_with_a_subject_needs_its_move_to_start_in_each() {
    let text = replaced(
        SHIP,
        "        when_subject_state: Shipped\n        preserves: demo.ship.Order\n",
        "        when_subject_state: [Shipped]\n        preserves: demo.ship.Order\n",
    );
    accepted(&text);
    let moving = replaced(
        SHIP,
        "      - name: already-shipped\n        when_subject_state: Shipped\n        preserves: demo.ship.Order\n        instance: order_id\n",
        "      - name: already-shipped\n        when_subject_state: [Shipped, Delivered]\n        moves: demo.ship.Order.cancel\n        instance: order_id\n        emits: [demo.ship.OrderCancelled]\n        payload: {demo.ship.OrderCancelled: {order_id: input.order_id}}\n",
    );
    let moving = replaced(
        &moving,
        "when_subject_state: [Delivered, Cancelled]",
        "when_subject_state: [Cancelled]",
    );
    let errors = refused(&moving);
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "Delivered"),
        "{errors}"
    );
}

#[test]
fn a_listed_state_round_trips_through_the_document_form() {
    let spec = accepted(SHIP);
    let ship = &spec.commands()[&"demo.ship.ShipOrder".parse().unwrap()];
    let written = serde_yaml::to_string(ship).unwrap();
    assert!(written.contains("when_subject_state:\n"), "{written}");
    assert!(written.contains("when_subject_state: Shipped"), "{written}");
    let reread: ess_domain::command::RawCommandSpec = serde_yaml::from_str(&written).unwrap();
    let again = serde_yaml::to_string(&reread).unwrap();
    assert_eq!(
        serde_yaml::from_str::<serde_yaml::Value>(&again).unwrap()["outcomes"],
        serde_yaml::from_str::<serde_yaml::Value>(&written).unwrap()["outcomes"]
    );
}

// ---- #204 ------------------------------------------------------------------------------------

#[test]
fn state_is_readable_in_a_when_subject_predicate_under_ess_18() {
    let spec = accepted(REPORT);
    let report = &spec.commands()[&"demo.shop.ReportPending".parse().unwrap()];
    let OutcomeCondition::SubjectPredicate { predicate, .. } = &report.outcomes[0].condition else {
        panic!("expected a subject predicate")
    };
    assert_eq!(
        predicate.to_string(),
        r#"(state == Ready and hold_note != "")"#
    );
}

#[test]
fn state_in_a_when_subject_predicate_is_refused_below_ess_18() {
    let errors = refused(&at(REPORT, "ess/17"));
    assert!(
        has(&errors, ValidationCode::UnsupportedFormatVersion, "ess/18"),
        "{errors}"
    );
    assert!(
        !has(&errors, ValidationCode::UnobservableFact, "state"),
        "the header is wrong, not the document: {errors}"
    );
}

#[test]
fn state_compared_with_an_undeclared_state_is_refused() {
    let errors = refused(&replaced(REPORT, "state == Ready", "state == Shipped"));
    assert!(!errors.is_empty(), "{errors}");
}

#[test]
fn state_and_when_subject_state_on_one_branch_is_still_one_authority_too_many() {
    let errors = refused(&replaced(
        REPORT,
        "      - name: kept-ready\n        when_subject:\n",
        "      - name: kept-ready\n        when_subject_state: Ready\n        when_subject:\n",
    ));
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "one selection authority"
        ),
        "{errors}"
    );
}

#[test]
fn a_state_reading_branch_whose_move_does_not_start_in_that_state_is_refused() {
    let text = replaced(
        REPORT,
        "{name: ready, from: [Draft, Pending, Ready], to: Ready}",
        "{name: ready, from: [Draft, Pending], to: Ready}",
    );
    let text = replaced(
        &text,
        "        updates: demo.shop.Order\n        instance: order_id\n        emits: [demo.shop.OrderChanged]\n        payload: {demo.shop.OrderChanged: {order_id: input.order_id}}\n      - name: pending\n",
        "        moves: demo.shop.Order.ready\n        instance: order_id\n        emits: [demo.shop.OrderChanged]\n        payload: {demo.shop.OrderChanged: {order_id: input.order_id}}\n      - name: pending\n",
    );
    let errors = refused(&text);
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "Ready"),
        "{errors}"
    );
}

#[test]
fn a_state_reading_refusal_without_a_subject_validates() {
    let text = replaced(
        REPORT,
        "        updates: demo.shop.Order\n        instance: order_id\n        emits: [demo.shop.OrderChanged]\n        payload: {demo.shop.OrderChanged: {order_id: input.order_id}}\n      - name: pending\n",
        "        error: demo.shop.Held\n      - name: pending\n",
    );
    let text = replaced(
        &text,
        "events:\n",
        "errors:\n  - {name: demo.shop.Held, summary: The order is held., fields: []}\nevents:\n",
    );
    accepted(&text);
}
