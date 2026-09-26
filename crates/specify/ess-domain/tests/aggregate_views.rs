//! Aggregate views: `group_by:` and a field-level `aggregate:` (beyond10x/ess#96).
//!
//! `docs/design/aggregate-views.md` is the binding design. Each validation case here is one row of
//! that page's V1–V15 table, asserting the `ValidationCode` and the location it names.
use ess_domain::{
    spec::RawSpecFile,
    system::{FormatVersion, Source},
    view::{AggregateFunction, RawViewSpec, ViewSpec},
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const METRICS: &str =
    include_str!("../../../verify/ess-conformance/tests/fixtures/aggregate-views.yaml");

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("metrics.yaml"), raw)])
}

fn parse_error(text: &str) -> String {
    RawSpecFile::parse(text)
        .err()
        .unwrap_or_else(|| panic!("the reader must refuse:\n{text}"))
        .to_string()
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
}

fn has(errors: &ValidationErrors, code: ValidationCode, site: &str) -> bool {
    errors
        .as_slice()
        .iter()
        .any(|error| error.code == code && error.location == site)
}

fn assert_has(errors: &ValidationErrors, code: ValidationCode, site: &str) {
    assert!(
        has(errors, code, site),
        "expected {code:?} at `{site}`, got:\n{}",
        errors
            .as_slice()
            .iter()
            .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
            .collect::<Vec<_>>()
            .join("\n")
    );
}

/// The fixture with its `views:` section replaced by `views`.
fn with_views(views: &str) -> String {
    let head = METRICS
        .split_once("views:\n")
        .expect("the fixture declares views")
        .0;
    format!("{head}views:\n{views}")
}

/// One view `metrics.session.V` over the session entity, with these extra lines.
fn view(body: &str) -> String {
    with_views(&format!(
        "  - name: metrics.session.V\n    source: metrics.session.Session\n{body}"
    ))
}

fn name(value: &str) -> ess_domain::name::QualifiedName {
    value.parse().expect("a qualified name")
}

#[test]
fn the_example_assembles_with_its_aggregation() {
    let spec = assemble(METRICS).unwrap_or_else(|errors| panic!("{errors}"));
    let by_agent = &spec.views()[&name("metrics.session.TalkTimeByAgent")];
    assert!(by_agent.is_aggregate());
    let aggregation = by_agent.aggregation.as_ref().expect("an aggregation");
    assert_eq!(aggregation.group_by, vec!["agent_id".to_owned()]);
    let functions: Vec<(String, AggregateFunction, Option<String>)> = aggregation
        .functions
        .iter()
        .map(|(field, aggregate)| (field.clone(), aggregate.function, aggregate.input.clone()))
        .collect();
    assert_eq!(
        functions,
        vec![
            (
                "distinct_callers".to_owned(),
                AggregateFunction::CountDistinct,
                Some("caller".to_owned())
            ),
            (
                "longest_wait".to_owned(),
                AggregateFunction::Max,
                Some("wait_seconds".to_owned())
            ),
            (
                "mean_talk".to_owned(),
                AggregateFunction::Avg,
                Some("talk_seconds".to_owned())
            ),
            ("sessions".to_owned(), AggregateFunction::Count, None),
            (
                "talk_seconds".to_owned(),
                AggregateFunction::Sum,
                Some("talk_seconds".to_owned())
            ),
        ]
    );
    // Aggregate fields stay among the fields, at their declared types.
    assert_eq!(by_agent.fields.len(), 6);
    let totals = &spec.views()[&name("metrics.session.QueueTotals")];
    assert!(totals.aggregation.as_ref().unwrap().group_by.is_empty());
}

#[test]
fn a_view_without_an_aggregate_keeps_its_bytes_and_is_not_an_aggregate() {
    let text = view(
        "    fields:\n      - {name: session_id, type: Uuid}\n      - {name: agent_id, type: String}\n",
    );
    let spec = assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
    let plain = &spec.views()[&name("metrics.session.V")];
    assert!(!plain.is_aggregate());
    assert!(plain.aggregation.is_none());
    let written = serde_json::to_string(plain).unwrap();
    assert_eq!(
        written,
        r#"{"name":"metrics.session.V","source":"metrics.session.Session","fields":[{"name":"session_id","type":"Uuid"},{"name":"agent_id","type":"String"}],"consistency":"eventual"}"#
    );
}

#[test]
fn an_empty_group_by_is_read_as_absent_and_not_written_back() {
    let text = view("    group_by: []\n    fields:\n      - {name: session_id, type: Uuid}\n");
    let spec = assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
    let plain = &spec.views()[&name("metrics.session.V")];
    assert!(!plain.is_aggregate());
    assert!(!serde_json::to_string(plain).unwrap().contains("group_by"));
}

#[test]
fn an_aggregate_view_round_trips_through_the_document_form() {
    let spec = assemble(METRICS).unwrap();
    for view in spec.views().values() {
        let written = serde_yaml::to_string(view).unwrap();
        let reread: RawViewSpec = serde_yaml::from_str(&written).unwrap();
        let reread = ViewSpec::try_from(reread).unwrap();
        assert_eq!(&reread, view, "{written}");
    }
    let by_agent =
        serde_json::to_string(&spec.views()[&name("metrics.session.TalkTimeByAgent")]).unwrap();
    assert!(
        by_agent.contains(r#""group_by":["agent_id"]"#),
        "{by_agent}"
    );
    assert!(
        by_agent.contains(r#"{"name":"sessions","type":"Integer","aggregate":{"count":{}}}"#),
        "{by_agent}"
    );
    assert!(
        by_agent.contains(r#""aggregate":{"sum":"talk_seconds"}"#),
        "{by_agent}"
    );
}

#[test]
fn the_shape_of_an_aggregate_map_is_the_readers_to_refuse() {
    for (aggregate, expected) in [
        ("{median: talk_seconds}", "unknown variant `median`"),
        ("{sum: talk_seconds, max: talk_seconds}", ""),
        ("{count: talk_seconds}", ""),
        ("{count: {x: 1}}", "unknown field `x`"),
    ] {
        let text = view(&format!(
            "    fields:\n      - {{name: n, type: Integer, aggregate: {aggregate}}}\n"
        ));
        let error = parse_error(&text);
        assert!(error.contains(expected), "{aggregate}: {error}");
    }
    let error = parse_error(&view(
        "    fields:\n      - {name: n, type: Integer, aggregate: {median: talk_seconds}}\n",
    ));
    for function in ["count", "count_distinct", "sum", "min", "max", "avg"] {
        assert!(error.contains(function), "{error}");
    }
}

#[test]
fn aggregate_is_still_an_unknown_field_everywhere_but_a_view_field() {
    let text = METRICS.replace(
        "      - {name: wait_seconds, type: Integer}\n    lifecycle:",
        "      - {name: wait_seconds, type: Integer, aggregate: {count: {}}}\n    lifecycle:",
    );
    assert!(parse_error(&text).contains("unknown field `aggregate`"));
}

const V: &str = "view.metrics.session.V";

#[test]
fn v1_a_group_by_entry_that_is_not_a_field_of_the_view() {
    let errors = refused(&view(
        "    group_by: [queue_id]\n    fields:\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::UndeclaredReference,
        &format!("{V}.group_by[0]"),
    );
}

#[test]
fn v2_a_group_by_entry_naming_an_aggregate_field() {
    let errors = refused(&view(
        "    group_by: [n]\n    fields:\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &format!("{V}.group_by[0]"),
    );
}

#[test]
fn v3_v5_and_v13_accumulate_with_the_shape_errors_of_the_written_view() {
    // V3 beside a duplicated field, which is `validate_shape`'s.
    let errors = refused(&view(
        "    group_by: [agent_id, agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::DuplicateDeclaration,
        &format!("{V}.group_by[1]"),
    );
    assert_has(
        &errors,
        ValidationCode::DuplicateDeclaration,
        &format!("{V}.fields[1]"),
    );
    // V5 beside the same shape error: `group_by` with no aggregate.
    let errors = refused(&view(
        "    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: agent_id, type: String}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::MissingDeclaration,
        &format!("{V}.group_by"),
    );
    assert_has(
        &errors,
        ValidationCode::DuplicateDeclaration,
        &format!("{V}.fields[1]"),
    );
    // V13 beside `shape` and `fields` together.
    let errors = refused(&view(
        "    shape: metrics.session.Channel\n    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &format!("{V}.group_by"),
    );
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &format!("{V}.shape"),
    );
}

#[test]
fn v3_v5_and_v13_are_raised_by_the_conversion_itself() {
    let raw: RawViewSpec = serde_yaml::from_str(
        "name: metrics.session.V\nsource: metrics.session.Session\nshape: metrics.session.Row\ngroup_by: [a, a]\n",
    )
    .unwrap();
    let errors = ViewSpec::try_from(raw).expect_err("refused");
    assert_has(
        &errors,
        ValidationCode::DuplicateDeclaration,
        &format!("{V}.group_by[1]"),
    );
    assert_has(
        &errors,
        ValidationCode::MissingDeclaration,
        &format!("{V}.group_by"),
    );
    assert_has(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &format!("{V}.group_by"),
    );
}

#[test]
fn v4_a_field_that_is_neither_an_aggregate_nor_a_group_key() {
    let errors = refused(&view(
        "    fields:\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::MissingDeclaration,
        &format!("{V}.fields[0]"),
    );
    assert!(errors.as_slice().iter().any(|error| error
        .message
        .contains("is neither an aggregate nor a group key")));
}

#[test]
fn v6_an_aggregate_argument_that_is_not_an_observable_field() {
    let errors = refused(&view(
        "    fields:\n      - {name: n, type: Integer, aggregate: {sum: minutes}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::UndeclaredReference,
        &format!("{V}.fields[0].aggregate"),
    );
    let hint = errors
        .as_slice()
        .iter()
        .find(|error| error.location == format!("{V}.fields[0].aggregate"))
        .and_then(|error| error.hint.clone())
        .unwrap_or_default();
    assert!(hint.contains("`talk_seconds`"), "{hint}");
}

#[test]
fn identity_and_state_are_observable_arguments() {
    let text = view(
        "    fields:\n      - {name: ids, type: Integer, aggregate: {count_distinct: session_id}}\n      - {name: states, type: Integer, aggregate: {count_distinct: state}}\n",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn v7_an_aggregate_argument_with_a_dot() {
    let errors = refused(&view(
        "    fields:\n      - {name: n, type: Integer, aggregate: {sum: total.amount}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &format!("{V}.fields[0].aggregate"),
    );
}

/// A document with extra entity fields and types, for the type-table cases.
fn typed(fields: &str, types: &str, views: &str) -> String {
    let text = METRICS
        .replace(
            "      - {name: wait_seconds, type: Integer}\n    lifecycle:",
            &format!("      - {{name: wait_seconds, type: Integer}}\n{fields}    lifecycle:"),
        )
        .replace("types:\n", &format!("types:\n{types}"));
    let head = text.split_once("views:\n").unwrap().0.to_owned();
    format!(
        "{head}views:\n  - name: metrics.session.V\n    source: metrics.session.Session\n{views}"
    )
}

const EXTRA_FIELDS: &str = "      - {name: note, type: Optional<Integer>}\n      - {name: wrapped, type: metrics.session.MaybeSeconds}\n      - {name: seconds, type: metrics.session.Seconds}\n      - {name: nested, type: metrics.session.Nested}\n      - {name: started, type: Timestamp}\n      - {name: ratio, type: Binary64}\n      - {name: tags, type: List<String>}\n      - {name: spent, type: Decimal}\n      - {name: flag, type: Boolean}\n      - {name: took, type: Duration}\n";
const EXTRA_TYPES: &str = "  - name: metrics.session.MaybeSeconds\n    kind: newtype\n    of: Optional<Integer>\n  - name: metrics.session.Seconds\n    kind: newtype\n    of: Integer\n  - name: metrics.session.Nested\n    kind: newtype\n    of: metrics.session.MaybeSeconds\n";

fn typed_view(fields: &str) -> String {
    typed(EXTRA_FIELDS, EXTRA_TYPES, fields)
}

#[test]
fn every_admitted_argument_and_result_type_assembles() {
    let text = typed_view(
        "    fields:\n      - {name: a, type: Integer, aggregate: {sum: seconds}}\n      - {name: b, type: Optional<metrics.session.Seconds>, aggregate: {max: seconds}}\n      - {name: c, type: Decimal, aggregate: {sum: spent}}\n      - {name: d, type: Optional<Decimal>, aggregate: {avg: spent}}\n      - {name: e, type: Optional<Timestamp>, aggregate: {min: started}}\n      - {name: f, type: Optional<String>, aggregate: {max: caller}}\n      - {name: g, type: Integer, aggregate: {count_distinct: flag}}\n      - {name: h, type: Integer, aggregate: {count_distinct: started}}\n      - {name: i, type: Integer, aggregate: {count_distinct: channel}}\n",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn v8_an_argument_whose_unwrapped_type_the_function_does_not_admit() {
    for (function, field) in [
        ("sum", "caller"),
        ("sum", "started"),
        ("avg", "flag"),
        ("max", "flag"),
        ("min", "channel"),
        ("count_distinct", "tags"),
        ("count_distinct", "ratio"),
        ("sum", "ratio"),
        ("max", "took"),
    ] {
        let text = typed_view(&format!(
            "    fields:\n      - {{name: n, type: Integer, aggregate: {{{function}: {field}}}}}\n"
        ));
        let errors = refused(&text);
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("{V}.fields[0].aggregate"),
        );
    }
}

#[test]
fn v9_an_argument_with_an_optional_anywhere_in_its_newtype_chain() {
    for field in ["note", "wrapped", "nested"] {
        let text = typed_view(&format!(
            "    fields:\n      - {{name: n, type: Integer, aggregate: {{sum: {field}}}}}\n"
        ));
        let errors = refused(&text);
        assert_has(
            &errors,
            ValidationCode::UnsupportedConstruct,
            &format!("{V}.fields[0].aggregate"),
        );
        assert!(
            !has(
                &errors,
                ValidationCode::TypeMismatch,
                &format!("{V}.fields[0].aggregate")
            ),
            "V9 is checked before V8: {errors}"
        );
    }
}

#[test]
fn v10_a_declared_type_other_than_the_result_type() {
    for (declared, aggregate, expected) in [
        ("Decimal", "{count: {}}", "Integer"),
        ("Integer", "{sum: spent}", "Decimal"),
        ("metrics.session.Seconds", "{sum: seconds}", "Integer"),
        (
            "Optional<Integer>",
            "{max: seconds}",
            "Optional<metrics.session.Seconds>",
        ),
        ("Integer", "{max: wait_seconds}", "Optional<Integer>"),
        ("Decimal", "{avg: talk_seconds}", "Optional<Decimal>"),
        ("Optional<Integer>", "{count_distinct: caller}", "Integer"),
    ] {
        let text = typed_view(&format!(
            "    fields:\n      - {{name: n, type: \"{declared}\", aggregate: {aggregate}}}\n"
        ));
        let errors = refused(&text);
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("{V}.fields[0].type"),
        );
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.message.contains(expected)),
            "{declared} {aggregate}: {errors}"
        );
    }
}

#[test]
fn v11_a_group_key_that_is_optional_or_a_timestamp() {
    for key in ["note", "wrapped", "started"] {
        let text = typed_view(&format!(
            "    group_by: [{key}]\n    fields:\n      - {{name: {key}, type: \"{}\"}}\n      - {{name: n, type: Integer, aggregate: {{count: {{}}}}}}\n",
            match key {
                "note" => "Optional<Integer>",
                "wrapped" => "metrics.session.MaybeSeconds",
                _ => "Timestamp",
            }
        ));
        let errors = refused(&text);
        assert_has(
            &errors,
            ValidationCode::UnsupportedConstruct,
            &format!("{V}.group_by[0]"),
        );
        assert!(
            !has(
                &errors,
                ValidationCode::TypeMismatch,
                &format!("{V}.group_by[0]")
            ),
            "{errors}"
        );
    }
}

#[test]
fn v12_a_group_key_that_is_not_an_equality_type() {
    for (key, declared) in [
        ("tags", "List<String>"),
        ("ratio", "Binary64"),
        ("took", "Duration"),
    ] {
        let text = typed_view(&format!(
            "    group_by: [{key}]\n    fields:\n      - {{name: {key}, type: \"{declared}\"}}\n      - {{name: n, type: Integer, aggregate: {{count: {{}}}}}}\n"
        ));
        let errors = refused(&text);
        assert_has(
            &errors,
            ValidationCode::TypeMismatch,
            &format!("{V}.group_by[0]"),
        );
    }
}

#[test]
fn equality_group_keys_assemble() {
    let text = typed_view(
        "    group_by: [agent_id, channel, flag, seconds, spent, session_id, state]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: channel, type: metrics.session.Channel}\n      - {name: flag, type: Boolean}\n      - {name: seconds, type: metrics.session.Seconds}\n      - {name: spent, type: Decimal}\n      - {name: session_id, type: Uuid}\n      - {name: state, type: metrics.session.Session.State}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    );
    assemble(&text).unwrap_or_else(|errors| panic!("{errors}"));
}

#[test]
fn v14_order_by_on_an_aggregate_view() {
    let errors = refused(&view(
        "    group_by: [agent_id]\n    order_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n      - {name: n, type: Integer, aggregate: {count: {}}}\n",
    ));
    assert_has(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &format!("{V}.order_by"),
    );
}

#[test]
fn v15_an_aggregate_view_below_ess_10() {
    let errors = refused(&METRICS.replace("format: ess/10", "format: ess/9"));
    assert_has(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.metrics.session.TalkTimeByAgent.group_by",
    );
    assert_has(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.metrics.session.QueueTotals.fields",
    );
    assert!(errors
        .as_slice()
        .iter()
        .any(|error| error.message == "aggregate views require specification format ess/10"));
    assert_eq!(FormatVersion::V10.major(), 10);
    assert!(FormatVersion::V10.is_supported());
}

#[test]
fn v5_a_group_by_without_an_aggregate_is_refused_at_every_version() {
    for format in ["ess/9", "ess/10"] {
        let text =
            view("    group_by: [agent_id]\n    fields:\n      - {name: agent_id, type: String}\n")
                .replace("format: ess/10", &format!("format: {format}"));
        let errors = refused(&text);
        assert_has(
            &errors,
            ValidationCode::MissingDeclaration,
            &format!("{V}.group_by"),
        );
        assert!(
            !errors
                .as_slice()
                .iter()
                .any(|error| error.code == ValidationCode::UnsupportedFormatVersion),
            "{errors}"
        );
    }
}
