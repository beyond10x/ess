//! Aggregates over, and group keys of, an `Optional` field (beyond10x/ess#148, source format
//! `ess/15`).
//!
//! `docs/design/aggregate-views.md`, "Absent values (ess/15)". An aggregate over an optional field
//! states that absent values are skipped with `skip_absent: true` inside the aggregate map; a group
//! key that may be absent makes the absent value its own group. Below `ess/15` both are refused as
//! a format the document does not declare.
use ess_domain::{
    spec::RawSpecFile,
    system::Source,
    view::{AggregateFunction, RawViewSpec, ViewSpec},
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const ORDERS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-optional-fields.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("orders.yaml"), raw)])
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn parse_error(text: &str) -> String {
    RawSpecFile::parse(text)
        .err()
        .unwrap_or_else(|| panic!("the reader must refuse:\n{text}"))
        .to_string()
}

fn listed(errors: &ValidationErrors) -> String {
    errors
        .as_slice()
        .iter()
        .map(|error| {
            format!(
                "{:?} {} {} (hint: {})",
                error.code,
                error.location,
                error.message,
                error.hint.clone().unwrap_or_default()
            )
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn at<'e>(
    errors: &'e ValidationErrors,
    code: ValidationCode,
    site: &str,
) -> &'e ess_primitives::error::ValidationError {
    errors
        .as_slice()
        .iter()
        .find(|error| error.code == code && error.location == site)
        .unwrap_or_else(|| panic!("expected {code:?} at `{site}`, got:\n{}", listed(errors)))
}

/// The fixture with its `views:` section replaced by one view `demo.orders.V` with these lines.
fn view(body: &str) -> String {
    let head = ORDERS.split_once("views:\n").unwrap().0;
    format!("{head}views:\n  - name: demo.orders.V\n    source: demo.orders.Order\n{body}")
}

fn at_format(text: &str, format: &str) -> String {
    text.replace("format: ess/15", &format!("format: {format}"))
}

const V: &str = "view.demo.orders.V";

fn name(value: &str) -> ess_domain::name::QualifiedName {
    value.parse().unwrap()
}

#[test]
fn both_views_of_the_issue_validate_under_ess_15() {
    let spec = assemble(ORDERS).unwrap_or_else(|errors| panic!("{}", listed(&errors)));
    let total = &spec.views()[&name("demo.orders.DurationTotal")];
    let aggregate = total
        .aggregation
        .as_ref()
        .unwrap()
        .function("total")
        .unwrap();
    assert_eq!(aggregate.function, AggregateFunction::Sum);
    assert_eq!(aggregate.input.as_deref(), Some("duration"));
    assert!(aggregate.skip_absent);
    let per_group = &spec.views()[&name("demo.orders.PerGroup")];
    let aggregation = per_group.aggregation.as_ref().unwrap();
    assert_eq!(aggregation.group_by, ["group"]);
    assert!(!aggregation.function("orders").unwrap().skip_absent);
}

#[test]
fn skip_absent_is_admitted_for_every_function_that_reads_a_value() {
    for (declared, aggregate) in [
        ("Optional<Integer>", "{sum: duration, skip_absent: true}"),
        ("Optional<Decimal>", "{avg: duration, skip_absent: true}"),
        ("Integer", "{count_distinct: duration, skip_absent: true}"),
        ("Optional<Integer>", "{min: duration, skip_absent: true}"),
        ("Optional<Integer>", "{max: duration, skip_absent: true}"),
        ("Optional<Integer>", "{skip_absent: true, max: duration}"),
    ] {
        let text = view(&format!(
            "    fields:\n      - {{name: n, type: \"{declared}\", aggregate: {aggregate}}}\n"
        ));
        assemble(&text).unwrap_or_else(|errors| panic!("{aggregate}: {}", listed(&errors)));
    }
}

#[test]
fn the_result_type_of_a_skipping_aggregate_is_the_one_the_page_names() {
    for (declared, aggregate, expected) in [
        (
            "Integer",
            "{sum: duration, skip_absent: true}",
            "Optional<Integer>",
        ),
        (
            "Integer",
            "{max: duration, skip_absent: true}",
            "Optional<Integer>",
        ),
        (
            "Optional<Integer>",
            "{count_distinct: duration, skip_absent: true}",
            "Integer",
        ),
        (
            "Decimal",
            "{avg: duration, skip_absent: true}",
            "Optional<Decimal>",
        ),
    ] {
        let errors = refused(&view(&format!(
            "    fields:\n      - {{name: n, type: \"{declared}\", aggregate: {aggregate}}}\n"
        )));
        let error = at(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("{V}.fields[0].type"),
        );
        assert!(
            error.message.contains(expected),
            "{aggregate}: {}",
            listed(&errors)
        );
    }
}

#[test]
fn an_extreme_over_an_optional_newtype_keeps_the_newtype() {
    let text = view(
        "    group_by: [customer]\n    fields:\n      - {name: customer, type: String}\n      - {name: g, type: Optional<demo.orders.Group>, aggregate: {max: group, skip_absent: true}}\n      - {name: n, type: Integer, aggregate: {count_distinct: group, skip_absent: true}}\n",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{}", listed(&errors)));
}

#[test]
fn without_skip_absent_the_optional_refusal_stands_and_its_hint_names_the_key() {
    for format in ["ess/14", "ess/15"] {
        let errors = refused(&at_format(
            &view("    fields:\n      - {name: n, type: Optional<Integer>, aggregate: {sum: duration}}\n"),
            format,
        ));
        let error = at(
            &errors,
            ValidationCode::UnsupportedConstruct,
            &format!("{V}.fields[0].aggregate"),
        );
        let hint = error.hint.clone().unwrap_or_default();
        assert!(hint.contains("skip_absent: true"), "{format}: {hint}");
    }
}

#[test]
fn count_over_rows_is_unaffected_and_takes_no_skip_absent() {
    let text = view("    fields:\n      - {name: n, type: Integer, aggregate: {count: {}}}\n");
    assemble(&text).unwrap_or_else(|errors| panic!("{}", listed(&errors)));
    let error = parse_error(&view(
        "    fields:\n      - {name: n, type: Integer, aggregate: {count: {}, skip_absent: true}}\n",
    ));
    assert!(error.contains("skip_absent"), "{error}");
    assert!(error.contains("count"), "{error}");
}

#[test]
fn skip_absent_is_written_true_or_not_at_all() {
    let error = parse_error(&view(
        "    fields:\n      - {name: n, type: Optional<Integer>, aggregate: {sum: duration, skip_absent: false}}\n",
    ));
    assert!(error.contains("skip_absent"), "{error}");
    let error = parse_error(&view(
        "    fields:\n      - {name: n, type: Optional<Integer>, aggregate: {skip_absent: true}}\n",
    ));
    assert!(error.contains("sum"), "{error}");
    let error = parse_error(&view(
        "    fields:\n      - {name: n, type: Optional<Integer>, aggregate: {sum: duration, skip_absent: true, skip_absent: true}}\n",
    ));
    assert_ne!(
        error.len(),
        0,
        "the duplicate key is refused with a message"
    );
}

#[test]
fn skip_absent_over_a_field_every_row_holds_is_refused() {
    let errors = refused(&view(
        "    fields:\n      - {name: n, type: Optional<Integer>, aggregate: {sum: customer_count, skip_absent: true}}\n",
    ));
    // `customer_count` does not exist: V6 first, and no second report about `skip_absent`.
    at(
        &errors,
        ValidationCode::UndeclaredReference,
        &format!("{V}.fields[0].aggregate"),
    );
    let errors = refused(&view(
        "    fields:\n      - {name: n, type: Integer, aggregate: {count_distinct: customer, skip_absent: true}}\n",
    ));
    let error = at(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &format!("{V}.fields[0].aggregate"),
    );
    assert!(error.message.contains("customer"), "{}", listed(&errors));
}

#[test]
fn an_optional_group_key_is_admitted_under_ess_15() {
    for (key, declared) in [
        ("group", "Optional<demo.orders.Group>"),
        ("channel", "Optional<demo.orders.Channel>"),
        ("duration", "Optional<Integer>"),
    ] {
        let text = view(&format!(
            "    group_by: [{key}]\n    fields:\n      - {{name: {key}, type: \"{declared}\"}}\n      - {{name: n, type: Integer, aggregate: {{count: {{}}}}}}\n"
        ));
        assemble(&text).unwrap_or_else(|errors| panic!("{key}: {}", listed(&errors)));
    }
}

#[test]
fn an_optional_timestamp_key_is_still_time_bucketing() {
    let text = ORDERS
        .replace(
            "      - {name: channel, type: Optional<demo.orders.Channel>}\n    lifecycle:",
            "      - {name: channel, type: Optional<demo.orders.Channel>}\n      - {name: at, type: Optional<Timestamp>}\n    lifecycle:",
        );
    let head = text.split_once("views:\n").unwrap().0;
    let errors = refused(&format!(
        "{head}views:\n  - name: demo.orders.V\n    source: demo.orders.Order\n    group_by: [at]\n    fields:\n      - {{name: at, type: Optional<Timestamp>}}\n      - {{name: n, type: Integer, aggregate: {{count: {{}}}}}}\n"
    ));
    let error = at(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &format!("{V}.group_by[0]"),
    );
    assert!(
        error.message.contains("time bucketing"),
        "{}",
        listed(&errors)
    );
}

#[test]
fn below_ess_15_skip_absent_is_refused_as_a_format_the_document_does_not_declare() {
    let errors = refused(&at_format(ORDERS, "ess/14"));
    let error = at(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.demo.orders.DurationTotal.fields[0].aggregate",
    );
    assert!(error.message.contains("ess/15"), "{}", listed(&errors));
    at(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.demo.orders.DurationByCustomer.fields[6].aggregate",
    );
}

#[test]
fn below_ess_15_an_optional_group_key_is_refused_as_a_format_the_document_does_not_declare() {
    let errors = refused(&at_format(ORDERS, "ess/14"));
    let error = at(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.demo.orders.PerGroup.group_by[0]",
    );
    assert!(error.message.contains("ess/15"), "{}", listed(&errors));
    at(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.demo.orders.PerCustomerChannel.group_by[1]",
    );
    assert!(
        !errors
            .as_slice()
            .iter()
            .any(|error| error.location == "view.demo.orders.PerCustomerChannel.group_by[0]"),
        "a required key is not refused: {}",
        listed(&errors)
    );
}

#[test]
fn skip_absent_round_trips_and_is_not_written_where_absent() {
    let spec = assemble(ORDERS).unwrap();
    for view in spec.views().values() {
        let written = serde_yaml::to_string(view).unwrap();
        let reread =
            ViewSpec::try_from(serde_yaml::from_str::<RawViewSpec>(&written).unwrap()).unwrap();
        assert_eq!(&reread, view, "{written}");
    }
    let total = serde_json::to_string(&spec.views()[&name("demo.orders.DurationTotal")]).unwrap();
    assert!(
        total.contains(r#""aggregate":{"sum":"duration","skip_absent":true}"#),
        "{total}"
    );
    let per_group = serde_json::to_string(&spec.views()[&name("demo.orders.PerGroup")]).unwrap();
    assert!(!per_group.contains("skip_absent"), "{per_group}");
}
