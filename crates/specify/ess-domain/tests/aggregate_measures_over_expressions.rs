//! An aggregate measure takes an expression of the source row, and `min`/`max` admit every ordered
//! type, an enum declaring `ordered: true` included (`ess/36`,
//! `story:aggregate-measures-over-expressions`, beyond10x/ess#561, beyond10x/ess#566,
//! `docs/design/generic-language-features.md` § G3).
//!
//! The fixture is the conformance suite's incident ledger: `Mttr` averages `ended_at -
//! started_at`, `Downtime` sums `duration_s * level.weight` and `Headline` takes the `max` of an
//! ordered enum. These tests hold validation to the design: what each measure computes, the result
//! type it declares, the refusals that name what to write, the format gate, and the round trip.
use ess_domain::{
    spec::RawSpecFile,
    system::Source,
    types::TypeBody,
    view::{AggregateFunction, RawViewSpec, ViewSpec},
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};

const LEDGER: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/aggregate-measures-over-expressions.yaml"
);

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("ledger.yaml"), raw)])
}

fn accepted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{}\n{text}", listed(&errors)))
}

fn refused(text: &str) -> ValidationErrors {
    assemble(text)
        .err()
        .unwrap_or_else(|| panic!("must refuse:\n{text}"))
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

/// The ledger with its views replaced by one view `demo.ledger.V`, grouped by `incident`, whose
/// second field is `field`.
fn view(field: &str) -> String {
    let head = LEDGER.split_once("views:\n").unwrap().0;
    format!(
        "{head}views:\n  - name: demo.ledger.V\n    source: demo.ledger.Window\n    \
         group_by: [incident]\n    fields:\n      - {{name: incident, type: String}}\n      - \
         {field}\n"
    )
}

fn spec_view<'s>(spec: &'s Specification, name: &str) -> &'s ViewSpec {
    spec.views()
        .get(&name.parse().unwrap())
        .unwrap_or_else(|| panic!("no view {name}"))
}

const V: &str = "view.demo.ledger.V";

/// The ledger validates: each measure's expression is typed against the row, and its declared
/// result type is the one the function gives that type.
#[test]
fn the_ledger_validates_with_expression_measures_and_an_ordered_enum() {
    let spec = accepted(LEDGER);
    let mttr = spec_view(&spec, "demo.ledger.Mttr")
        .aggregation
        .as_ref()
        .and_then(|aggregation| aggregation.function("mttr"))
        .expect("an aggregate");
    assert_eq!(mttr.function, AggregateFunction::Avg);
    assert_eq!(mttr.input, None);
    assert_eq!(mttr.expr.as_deref(), Some("ended_at - started_at"));
    assert_eq!(mttr.to_string(), "avg(ended_at - started_at)");
    let level = spec
        .system()
        .types
        .get(&"demo.ledger.ImpactLevel".parse().unwrap())
        .expect("the enum");
    assert!(
        matches!(&level.body, TypeBody::Enum { ordered: true, .. }),
        "{level:?}"
    );
}

/// The written form survives the model: a view read back writes `{expr: …}` where it was written,
/// and a field input as the bare name it always was.
#[test]
fn an_expression_measure_round_trips() {
    let spec = accepted(LEDGER);
    for name in [
        "demo.ledger.Mttr",
        "demo.ledger.Downtime",
        "demo.ledger.Headline",
    ] {
        let view = spec_view(&spec, name);
        let raw = RawViewSpec::from(view.clone());
        let json = serde_json::to_value(&raw).unwrap();
        let back: RawViewSpec = serde_json::from_value(json.clone()).unwrap();
        assert_eq!(ViewSpec::try_from(back).unwrap(), *view, "{json}");
    }
    let raw = RawViewSpec::from(spec_view(&spec, "demo.ledger.Mttr").clone());
    let json = serde_json::to_value(&raw).unwrap();
    assert_eq!(
        json["fields"][1]["aggregate"],
        serde_json::json!({"avg": {"expr": "ended_at - started_at"}})
    );
    let raw = RawViewSpec::from(spec_view(&spec, "demo.ledger.Headline").clone());
    let json = serde_json::to_value(&raw).unwrap();
    assert_eq!(
        json["fields"][1]["aggregate"],
        serde_json::json!({"max": "level"})
    );
}

/// Below `ess/36` an expression measure and an enum's `ordered:` are refused where they are
/// written, naming the format; the same view over a field keeps validating there.
#[test]
fn below_ess_36_an_expression_measure_and_an_ordered_enum_are_refused() {
    let old = LEDGER.replace("format: ess/36", "format: ess/35");
    let errors = refused(&old);
    let measure = at(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "view.demo.ledger.Mttr.fields[1].aggregate",
    );
    assert!(measure.message.contains("ess/36"), "{}", listed(&errors));
    let ordered = at(
        &errors,
        ValidationCode::UnsupportedFormatVersion,
        "types.demo.ledger.ImpactLevel.ordered",
    );
    assert!(ordered.message.contains("ess/36"), "{}", listed(&errors));

    let plain = old
        .replace("    ordered: true\n", "")
        .split_once("views:\n")
        .unwrap()
        .0
        .to_owned()
        + "views:\n  - name: demo.ledger.V\n    source: demo.ledger.Window\n    group_by: \
           [incident]\n    fields:\n      - {name: incident, type: String}\n      - {name: total, \
           type: Integer, aggregate: {sum: duration_s}}\n";
    accepted(&plain);
}

/// `min`/`max` over an enum that declares no order is refused, naming the flag that gives it one.
#[test]
fn max_over_an_unordered_enum_names_the_flag() {
    let text =
        view("{name: worst, type: Optional<demo.ledger.ImpactLevel>, aggregate: {max: level}}")
            .replace("    ordered: true\n", "");
    let errors = refused(&text);
    let refusal = at(
        &errors,
        ValidationCode::TypeMismatch,
        &format!("{V}.fields[1].aggregate"),
    );
    assert!(
        refusal
            .hint
            .as_deref()
            .unwrap_or_default()
            .contains("ordered: true"),
        "{}",
        listed(&errors)
    );
}

/// The result type follows grid O: `sum` of a Duration is a Duration, `min` of one an optional
/// Duration, `avg` of one an optional Decimal of seconds; a declared type that is not it is refused
/// naming the one to declare.
#[test]
fn the_result_type_follows_the_expression() {
    accepted(&view(
        "{name: down, type: Duration, aggregate: {sum: {expr: \"ended_at - started_at\"}}}",
    ));
    accepted(&view(
        "{name: shortest, type: Optional<Duration>, aggregate: {min: {expr: \"ended_at - started_at\"}}}",
    ));
    accepted(&view(
        "{name: latest, type: Optional<Timestamp>, aggregate: {max: {expr: \"ended_at\"}}}",
    ));
    let errors = refused(&view(
        "{name: mttr, type: Optional<Duration>, aggregate: {avg: {expr: \"ended_at - started_at\"}}}",
    ));
    let refusal = at(
        &errors,
        ValidationCode::TypeMismatch,
        &format!("{V}.fields[1].type"),
    );
    assert!(
        refusal
            .hint
            .as_deref()
            .unwrap_or_default()
            .contains("Optional<Decimal>"),
        "{}",
        listed(&errors)
    );
    // `sum` admits no Timestamp: an instant has no total.
    let errors = refused(&view(
        "{name: total, type: Integer, aggregate: {sum: {expr: \"ended_at\"}}}",
    ));
    at(
        &errors,
        ValidationCode::TypeMismatch,
        &format!("{V}.fields[1].aggregate"),
    );
}

/// An expression reads the row's fields and the attributes of an enum field; anything else is
/// refused by name: a field the source does not declare, an attribute its enum does not declare,
/// `now`, and an expression that does not parse.
#[test]
fn an_expression_reads_the_row_and_nothing_else() {
    let errors = refused(&view(
        "{name: total, type: Integer, aggregate: {sum: {expr: \"duration_s * stretch\"}}}",
    ));
    at(
        &errors,
        ValidationCode::UndeclaredReference,
        &format!("{V}.fields[1].aggregate"),
    );
    let errors = refused(&view(
        "{name: total, type: Integer, aggregate: {sum: {expr: \"duration_s * level.cost\"}}}",
    ));
    at(
        &errors,
        ValidationCode::UndeclaredReference,
        &format!("{V}.fields[1].aggregate"),
    );
    let errors = refused(&view(
        "{name: age, type: Duration, aggregate: {sum: {expr: \"now - started_at\"}}}",
    ));
    at(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &format!("{V}.fields[1].aggregate"),
    );
    let errors = refused(&view(
        "{name: total, type: Integer, aggregate: {sum: {expr: \"duration_s *\"}}}",
    ));
    at(
        &errors,
        ValidationCode::UnparsablePredicate,
        &format!("{V}.fields[1].aggregate"),
    );
}

/// A quotient may be absent, so a measure over one says what an absent value means, as a measure
/// over an `Optional` field does.
#[test]
fn an_expression_that_may_be_absent_needs_skip_absent() {
    let errors = refused(&view(
        "{name: rate, type: Optional<Decimal>, aggregate: {avg: {expr: \"duration_s / duration_s\"}}}",
    ));
    at(
        &errors,
        ValidationCode::UnsupportedConstruct,
        &format!("{V}.fields[1].aggregate"),
    );
    accepted(&view(
        "{name: rate, type: Optional<Decimal>, aggregate: {avg: {expr: \"duration_s / duration_s\"}, skip_absent: true}}",
    ));
    let errors = refused(&view(
        "{name: total, type: Integer, aggregate: {sum: {expr: \"duration_s * 2\"}, skip_absent: true}}",
    ));
    at(
        &errors,
        ValidationCode::ConflictingDeclaration,
        &format!("{V}.fields[1].aggregate"),
    );
}

/// `count` takes no argument, so it takes no expression either: the reader refuses it.
#[test]
fn count_takes_no_expression() {
    let text = view("{name: n, type: Integer, aggregate: {count: {expr: \"duration_s\"}}}");
    assert!(RawSpecFile::parse(&text).is_err(), "{text}");
}
