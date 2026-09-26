//! An outcome guarded by a predicate over the addressed entity's stored fields (ess#75, rule 1).
//!
//! `docs/design/cross-record-and-stored-field-guards.md` is the binding design. Each test here is
//! one line of that page's "What it refuses" table, its partition section, or its format section.
use ess_domain::{
    command::{OutcomeCondition, TestStrategy},
    spec::RawSpecFile,
    system::Source,
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const PARCELS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/stored-field-guards.yaml");

const GUARD: &str = "        when_subject:\n          predicate:\n            all:\n              - service == Express\n              - weight_kg > 20\n";

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("parcels.yaml"), raw)])
}

fn parse_error(text: &str) -> String {
    RawSpecFile::parse(text)
        .err()
        .unwrap_or_else(|| panic!("the reader must refuse:\n{text}"))
        .to_string()
}

fn refused(text: &str) -> ValidationErrors {
    match RawSpecFile::parse(text) {
        Ok(raw) => Specification::assemble([(Source::new("parcels.yaml"), raw)])
            .err()
            .unwrap_or_else(|| panic!("must refuse:\n{text}")),
        Err(error) => panic!("the reader accepted nothing to validate: {error}\n{text}"),
    }
}

fn has(errors: &ValidationErrors, code: ValidationCode, site: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.location.ends_with(site))
}

fn with_guard(guard: &str) -> String {
    PARCELS.replace(GUARD, guard)
}

#[test]
fn the_parcels_rule_is_a_checked_predicate_over_the_stored_fields() {
    let spec = assemble(PARCELS).unwrap_or_else(|errors| panic!("{errors}"));
    let dispatch = &spec.commands()[&"shipping.parcel.Dispatch".parse().unwrap()];
    let refusal = &dispatch.outcomes[0];
    let OutcomeCondition::SubjectPredicate { predicate, input } = &refusal.condition else {
        panic!("expected a subject predicate, got {:?}", refusal.condition)
    };
    assert_eq!(
        predicate.to_string(),
        "(service == Express and weight_kg > 20)"
    );
    assert!(input.is_none());
    assert_eq!(refusal.test_strategy(), TestStrategy::ObserveSubjectFact);
    assert!(
        refusal.subject.is_none(),
        "the refusal takes its subject from its sibling"
    );
    assert!(!refusal.is_unconditional());
}

#[test]
fn the_predicate_form_round_trips_through_the_document_form() {
    let raw = RawSpecFile::parse(PARCELS).unwrap();
    let spec = Specification::assemble([(Source::new("parcels.yaml"), raw)]).unwrap();
    let dispatch = &spec.commands()[&"shipping.parcel.Dispatch".parse().unwrap()];
    let written = serde_yaml::to_string(dispatch).unwrap();
    assert!(written.contains("when_subject:"), "{written}");
    assert!(written.contains("predicate:"), "{written}");
    assert!(!written.contains("field:"), "{written}");
    let reread: ess_domain::command::RawCommandSpec = serde_yaml::from_str(&written).unwrap();
    let reread = ess_domain::command::CommandSpec::try_from(reread).unwrap();
    assert_eq!(&reread, dispatch);
}

#[test]
fn the_field_equals_form_keeps_its_written_bytes() {
    let raw: ess_domain::command::RawOutcome = serde_yaml::from_str(
        "name: kept\nwhen_subject: {field: channel, equals: Post}\npreserves: billing.invoice.Invoice\ninstance: invoice_id\n",
    )
    .unwrap();
    let written = serde_json::to_string(&raw).unwrap();
    assert!(
        written.contains(r#""when_subject":{"field":"channel","equals":"Post"}"#),
        "{written}"
    );
}

#[test]
fn both_shapes_in_one_when_subject_are_refused_by_the_reader() {
    let error = parse_error(&with_guard(
        "        when_subject:\n          field: service\n          equals: Express\n          predicate: weight_kg > 20\n",
    ));
    assert!(error.contains("never both"), "{error}");
    let error = parse_error(&with_guard(
        "        when_subject:\n          field: service\n",
    ));
    assert!(error.contains("equals"), "{error}");
    let error = parse_error(&with_guard(
        "        when_subject:\n          predicate: weight_kg > 20\n          other: 1\n",
    ));
    assert!(error.contains("unknown field"), "{error}");
}

#[test]
fn a_null_comparison_is_refused_where_the_predicate_is_read() {
    let error = parse_error(&with_guard(
        "        when_subject:\n          predicate: weight_kg == null\n",
    ));
    assert!(error.contains("ESS-SPEC-017"), "{error}");
}

#[test]
fn the_predicate_form_requires_source_format_ess_9_and_the_old_form_does_not() {
    for format in ["ess/6", "ess/7"] {
        let errors = refused(&PARCELS.replace("format: ess/9", &format!("format: {format}")));
        assert!(
            errors.as_slice().iter().any(|error| error.code
                == ValidationCode::UnsupportedFormatVersion
                && error
                    .message
                    .contains("subject predicates require specification format ess/9")),
            "{errors}"
        );
    }
    let old = PARCELS.replace("format: ess/9", "format: ess/6").replace(
        GUARD,
        "        when_subject: {field: service, equals: Express}\n",
    );
    let errors = refused(&old);
    assert!(
        !errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedFormatVersion),
        "the {{field, equals}} form keeps ess/6: {errors}"
    );
}

#[test]
fn beside_another_selection_authority_on_the_same_branch_is_refused() {
    for extra in [
        "        when_subject_state: Created\n",
        "        when_state_changes: true\n",
        "        external: the scale is offline\n",
        "        wrong_state: true\n",
    ] {
        let text = with_guard(&format!("{GUARD}{extra}"));
        let errors = match RawSpecFile::parse(&text) {
            Ok(raw) => Specification::assemble([(Source::new("p.yaml"), raw)]).unwrap_err(),
            Err(error) => panic!("reader accepted nothing: {error}"),
        };
        assert!(
            has(
                &errors,
                ValidationCode::ConflictingDeclaration,
                "outcomes.refused-overweight.when_subject"
            ),
            "{extra}: {errors}"
        );
    }
}

#[test]
fn on_a_creating_branch_it_is_refused_at_when_subject() {
    let text = PARCELS.replace(
        "      - name: created\n        creates:",
        "      - name: created\n        when_subject: {predicate: weight_kg > 20}\n        creates:",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "outcomes.created.when_subject"
        ),
        "{errors}"
    );
    assert!(
        !errors
            .as_slice()
            .iter()
            .any(|error| error.location.ends_with("when_state_changes")),
        "never sited at a key the author did not write: {errors}"
    );
}

#[test]
fn in_a_command_with_no_subject_bearing_sibling_it_is_refused_at_when_subject() {
    let text = PARCELS.replace(
        "      - name: dispatched\n        moves: shipping.parcel.Parcel.dispatch\n        instance: parcel_id\n",
        "      - name: dispatched\n",
    );
    let errors = refused(&text);
    assert!(
        has(
            &errors,
            ValidationCode::ConflictingDeclaration,
            "outcomes.refused-overweight.when_subject"
        ),
        "{errors}"
    );
}

#[test]
fn it_reads_the_entity_fields_and_nothing_else() {
    for guard in [
        "colour == Red",
        "state == Created",
        "parcel_id == x",
        "input.parcel_id == x",
    ] {
        let errors = refused(&with_guard(&format!(
            "        when_subject:\n          predicate: {guard}\n"
        )));
        assert!(
            has(
                &errors,
                ValidationCode::UnobservableFact,
                "outcomes.refused-overweight.when_subject"
            ),
            "{guard}: {errors}"
        );
    }
}

#[test]
fn the_expression_checker_refuses_literals_of_the_wrong_type_and_bare_field_names() {
    let errors = refused(&with_guard(
        "        when_subject:\n          predicate: service == Overnight\n",
    ));
    assert!(
        errors.contains(ValidationCode::TypeMismatch)
            || errors.contains(ValidationCode::UndeclaredReference),
        "{errors}"
    );
    let errors = refused(&with_guard(
        "        when_subject:\n          predicate: weight_kg > heavy\n",
    ));
    assert!(errors.contains(ValidationCode::TypeMismatch), "{errors}");
    let errors = refused(&with_guard(
        "        when_subject:\n          predicate: service == weight_kg\n",
    ));
    assert!(
        errors.contains(ValidationCode::UndeclaredReference),
        "{errors}"
    );
    let timestamped = with_guard("        when_subject:\n          predicate: weighed_at < yesterday\n").replace(
        "      - {name: weight_kg, type: Integer}\n    lifecycle:",
        "      - {name: weight_kg, type: Integer}\n      - {name: weighed_at, type: Timestamp}\n    lifecycle:",
    );
    let errors = refused(&timestamped);
    assert!(errors.contains(ValidationCode::TypeMismatch), "{errors}");
    assert!(
        errors
            .as_slice()
            .iter()
            .all(|error| error.location.contains("when_subject")),
        "every checker diagnostic is sited at when_subject: {errors}"
    );
}

#[test]
fn every_form_the_grammar_admits_is_admitted_over_stored_fields() {
    let widened = PARCELS.replace(
        "      - {name: weight_kg, type: Integer}\n    lifecycle:",
        "      - {name: weight_kg, type: Integer}\n      - {name: note, type: Optional<String>}\n      - {name: labels, type: List<String>}\n      - {name: weighed_at, type: Timestamp}\n    lifecycle:",
    );
    for guard in [
        "'not defined(note)'",
        "'defined(note)'",
        "'labels.count > 1'",
        "'labels.0 == fragile'",
        "'note < m'",
        "'weighed_at < \"2026-01-01T00:00:00Z\"'",
        "{service: {in: [Express]}}",
        "{service: {not_in: [Standard]}}",
        "{note: {exists: false}}",
        "{exists: {in: labels, as: label, that: label == fragile}}",
        "{forall: {in: labels, as: label, that: label != \"\"}}",
    ] {
        let text = widened.replace(
            GUARD,
            &format!("        when_subject:\n          predicate: {guard}\n"),
        );
        assemble(&text).unwrap_or_else(|errors| panic!("{guard}: {errors}"));
    }
}

#[test]
fn sibling_predicates_over_different_entities_are_refused() {
    let text = PARCELS
        .replace(
            "  - name: shipping.parcel.Parcel\n",
            "  - name: shipping.parcel.Label\n    identity: {name: label_id, type: Uuid}\n    fields:\n      - {name: service, type: shipping.parcel.Service}\n    lifecycle:\n      initial: Printed\n      states: [Printed]\n      terminal: [Printed]\n  - name: shipping.parcel.Parcel\n",
        )
        .replace(
            "      - name: dispatched\n",
            "      - name: relabelled\n        when_subject: {predicate: service == Standard}\n        updates: shipping.parcel.Label\n        instance: parcel_id\n        emits: [shipping.parcel.Dispatched]\n      - name: dispatched\n",
        );
    let errors = refused(&text);
    assert!(
        errors.contains(ValidationCode::ConflictingDeclaration),
        "{errors}"
    );
}

#[test]
fn beside_a_lifecycle_guard_in_one_command_is_refused() {
    let text = PARCELS.replace(
        "      - name: dispatched\n",
        "      - name: restated\n        when_subject_state: Dispatched\n        updates: shipping.parcel.Parcel\n        instance: parcel_id\n        emits: [shipping.parcel.Dispatched]\n      - name: dispatched\n",
    );
    let errors = refused(&text);
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::ConflictingDeclaration
                && error.message.contains("cannot be combined in one command")),
        "{errors}"
    );
}

/// Two predicate branches that split an enum between them need no default: the stored fact now
/// enters the partition.
fn split(express: &str, standard: &str) -> String {
    PARCELS.replace(
        &format!("{GUARD}        error: shipping.parcel.ExpressOverweight\n      - name: dispatched\n"),
        &format!(
            "        when_subject: {{predicate: '{express}'}}\n        error: shipping.parcel.ExpressOverweight\n      - name: dispatched\n        when_subject: {{predicate: '{standard}'}}\n"
        ),
    )
}

#[test]
fn a_closed_stored_field_partition_is_complete_without_a_default() {
    assemble(&split("service == Express", "service == Standard"))
        .unwrap_or_else(|errors| panic!("{errors}"));
    assemble(&split("service != Standard", "service != Express"))
        .unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn an_uncovered_or_overlapping_stored_field_partition_is_refused() {
    let errors = refused(&split("service == Express", "service == Express"));
    assert!(
        has(&errors, ValidationCode::ConflictingDeclaration, "outcomes")
            && errors
                .as_slice()
                .iter()
                .any(|error| error.message.contains("service = Express")),
        "{errors}"
    );
    let errors = refused(&split(
        "service == Express",
        "service == Express and service == Standard",
    ));
    assert!(
        has(&errors, ValidationCode::NonExhaustiveBranches, "outcomes"),
        "{errors}"
    );
}

#[test]
fn an_open_stored_field_guard_needs_a_genuine_default() {
    let errors = refused(&split("weight_kg > 20", "weight_kg <= 20"));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::NonExhaustiveBranches
                && error.message.contains("declare a genuine default")),
        "{errors}"
    );
}

#[test]
fn a_guard_crossed_with_an_input_guard_is_partitioned_jointly() {
    let joint = PARCELS
        .replace(
            "      - {name: parcel_id, type: Uuid}\n    outcomes:",
            "      - {name: parcel_id, type: Uuid}\n      - {name: priority, type: shipping.parcel.Service}\n    outcomes:",
        )
        .replace(
            &format!("{GUARD}        error: shipping.parcel.ExpressOverweight\n      - name: dispatched\n"),
            "        when_subject: {predicate: service == Express}\n        when: priority == Standard\n        error: shipping.parcel.ExpressOverweight\n      - name: dispatched\n        when_subject: {predicate: service == Standard}\n",
        );
    let errors = refused(&joint);
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.code == ValidationCode::NonExhaustiveBranches
                && error.message.contains("priority = Express")
                && error.message.contains("service = Express")),
        "{errors}"
    );
}
