//! Conditional aggregate measures (beyond10x/ess#363, source format `ess/22`).
//!
//! `docs/design/conditional-aggregate-measures.md`. One sibling `where:` beside an aggregate's
//! function selects, for that measure alone, which rows of the group it reads. These are the
//! source, admission, parameter, paging, resolution and identity halves of the design's named
//! acceptance; execution lives in `ess-conformance` and `ess-synth`.
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use ess_domain::{
    spec::RawSpecFile,
    system::Source,
    view::{Aggregate, AggregateFunction, RawViewSpec, ViewSpec},
    Specification,
};
use ess_primitives::error::{ValidationCode, ValidationErrors};
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate};

const CASES: &str = include_str!(
    "../../../verify/ess-conformance/tests/fixtures/conditional-aggregate-measures.yaml"
);

fn assemble(text: &str) -> Result<Specification, ValidationErrors> {
    let raw = RawSpecFile::parse(text).unwrap_or_else(|error| panic!("{error}\n{text}"));
    Specification::assemble([(Source::new("cases.yaml"), raw)])
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("{}\n{text}", listed(&errors)))
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
        .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
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

/// The fixture with its `views:` replaced by one view `demo.cases.V` with these lines.
fn view(body: &str) -> String {
    let head = CASES.split_once("views:\n").unwrap().0;
    format!("{head}views:\n  - name: demo.cases.V\n    source: demo.cases.Case\n{body}")
}

/// One grouped view whose second field is `{name: m, type: <declared>, aggregate: <aggregate>}`.
fn measured(declared: &str, aggregate: &str) -> String {
    view(&format!(
        "    group_by: [team]\n    fields:\n      - {{name: team, type: String}}\n      - {{name: \
         m, type: \"{declared}\", aggregate: {aggregate}}}\n"
    ))
}

const V: &str = "view.demo.cases.V";

fn name(value: &str) -> ess_domain::name::QualifiedName {
    value.parse().unwrap()
}

fn condition(spec: &Specification, view: &str, field: &str) -> Option<Predicate> {
    spec.views()[&name(view)]
        .aggregation
        .as_ref()
        .and_then(|aggregation| aggregation.function(field))
        .unwrap_or_else(|| panic!("{view}.{field} is an aggregate"))
        .r#where
        .clone()
}

fn fact(path: &str) -> Operand {
    Operand::Fact(FactPath::new(path).unwrap())
}

fn equals(path: &str, value: FactValue) -> Predicate {
    Predicate::Compare {
        left: fact(path),
        op: CompareOp::Eq,
        right: Operand::Literal(value),
        kind: CompareKind::Value,
    }
}

// ---- conditional_measures_admit_only_from_source22 ---------------------------------------------

#[test]
fn conditional_measures_admit_only_from_source22() {
    // All six functions take the sibling, beside `skip_absent` where that is admitted.
    for (declared, aggregate) in [
        ("Integer", "{count: {}, where: state == Completed}"),
        (
            "Integer",
            "{count_distinct: label, where: escalated == true}",
        ),
        ("Integer", "{sum: cents, where: escalated == true}"),
        (
            "Optional<Integer>",
            "{min: cents, where: escalated == true}",
        ),
        (
            "Optional<Integer>",
            "{max: cents, where: escalated == true}",
        ),
        (
            "Optional<Decimal>",
            "{avg: cents, where: escalated == true}",
        ),
        ("Integer", "{where: escalated == true, sum: cents}"),
    ] {
        let spec = admitted(&measured(declared, aggregate));
        assert!(
            condition(&spec, "demo.cases.V", "m").is_some(),
            "{aggregate}: the condition is kept"
        );
    }
    let spec = admitted(CASES);
    assert_eq!(
        condition(&spec, "demo.cases.Escalations", "cost"),
        Some(equals("escalated", FactValue::Bool(true)))
    );
    assert_eq!(condition(&spec, "demo.cases.Escalations", "cases"), None);

    // Below ess/22 the sibling is refused where it is written, naming the format.
    for format in ["ess/21", "ess/15", "ess/10"] {
        let text = measured("Integer", "{sum: cents, where: escalated == true}")
            .replace("format: ess/22", &format!("format: {format}"));
        let errors = refused(&text);
        let error = at(
            &errors,
            ValidationCode::UnsupportedFormatVersion,
            &format!("{V}.fields[1].aggregate.where"),
        );
        assert!(
            error.message.contains("ess/22"),
            "{format}: {}",
            error.message
        );
    }

    // The reader refuses a malformed sibling before any validation.
    for (aggregate, says) in [
        (
            "{sum: cents, where: escalated == true, where: state == Open}",
            "where",
        ),
        ("{sum: cents, where: null}", "where"),
        ("{sum: cents, where: \"\"}", "where"),
        ("{sum: cents, where: []}", "where"),
        ("{sum: cents, where: {}}", "where"),
        (
            "{sum: cents, max: cents, where: escalated == true}",
            "exactly one function",
        ),
        ("{where: escalated == true}", "one function"),
        ("{sum: cents, filter: escalated == true}", "filter"),
        ("{sum: cents, having: escalated == true}", "having"),
        ("{count: {where: escalated == true}}", "where"),
    ] {
        let message = parse_error(&measured("Integer", aggregate));
        assert!(message.contains(says), "{aggregate}: {message}");
    }

    // An unconditioned aggregate reads and writes exactly as before.
    let raw = RawSpecFile::parse(&measured("Integer", "{sum: cents}")).unwrap();
    let written = serde_json::to_value(&raw.views[0]).unwrap();
    assert_eq!(
        written["fields"][1]["aggregate"],
        serde_json::json!({"sum": "cents"})
    );
    let view: ViewSpec = RawViewSpec::try_into(raw.views[0].clone()).unwrap();
    let back = serde_json::to_value(RawViewSpec::from(view)).unwrap();
    assert_eq!(
        back, written,
        "an unconditioned view writes back its own bytes"
    );
}

// ---- conditional_measure_parameters_are_view_inputs --------------------------------------------

#[test]
fn conditional_measure_parameters_are_view_inputs() {
    // A parameter read only by a measure is a parameter the view reads.
    let spec = admitted(&view(
        "    params: [{name: floor, type: Integer}]\n    group_by: [team]\n    fields:\n      - \
         {name: team, type: String}\n      - {name: big, type: Integer, aggregate: {count: {}, \
         where: cents > param.floor}}\n",
    ));
    assert!(condition(&spec, "demo.cases.V", "big").is_some());

    // One no measure and no filter reads is still refused.
    let errors = refused(&view(
        "    params: [{name: floor, type: Integer}]\n    group_by: [team]\n    fields:\n      - \
         {name: team, type: String}\n      - {name: big, type: Integer, aggregate: {count: {}, \
         where: cents > 10}}\n",
    ));
    at(
        &errors,
        ValidationCode::UnobservableFact,
        &format!("{V}.params"),
    );

    // One a measure reads and `params:` does not declare is refused at the condition.
    let errors = refused(&measured(
        "Integer",
        "{count: {}, where: cents > param.floor}",
    ));
    assert!(
        errors
            .as_slice()
            .iter()
            .any(|error| error.location == format!("{V}.fields[1].aggregate.where")),
        "{}",
        listed(&errors)
    );

    // A condition reads the source row, never a result: `cents` here is the row's value, though
    // the view names its sum `cents`.
    let spec = admitted(&view(
        "    group_by: [team]\n    fields:\n      - {name: team, type: String}\n      - {name: \
         cents, type: Integer, aggregate: {sum: cents, where: cents > 10}}\n",
    ));
    assert!(condition(&spec, "demo.cases.V", "cents").is_some());

    // A condition reading something the source does not hold, or comparing it with a value of
    // another type, is refused at the condition; so is `now`.
    for aggregate in [
        "{count: {}, where: total > 1}",
        "{count: {}, where: cents == yes}",
        "{count: {}, where: state == Closed}",
        "{count: {}, where: input.cents > 1}",
    ] {
        let errors = refused(&measured("Integer", aggregate));
        assert!(
            errors
                .as_slice()
                .iter()
                .any(|error| error.location == format!("{V}.fields[1].aggregate.where")),
            "{aggregate}: {}",
            listed(&errors)
        );
    }
}

// ---- conditional_measure_paging_inputs_do_not_select -------------------------------------------

#[test]
fn conditional_measure_paging_inputs_do_not_select() {
    // `page` and `size` each refuse where only a measure reads them: a paging parameter slices,
    // it never selects. (An aggregate view is never paged — V14 and the paging block's own order
    // rule refuse that already — and that refusal stands beside this one.)
    for read in ["page", "size"] {
        let errors = refused(&view(&format!(
            "    params: [{{name: page, type: Integer}}, {{name: size, type: Integer}}]\n    \
             paging: {{page: page, size: size}}\n    group_by: [team]\n    fields:\n      - \
             {{name: team, type: String}}\n      - {{name: m, type: Integer, aggregate: {{count: \
             {{}}, where: cents > param.{read}}}}}\n"
        )));
        let error = at(
            &errors,
            ValidationCode::ConflictingDeclaration,
            &format!("{V}.fields[1].aggregate.where"),
        );
        assert!(error.message.contains(read), "{}", error.message);
    }
}

// ---- conditional_measure_predicates_are_resolved_before_persistence ----------------------------

#[test]
fn conditional_measure_predicates_are_resolved_before_persistence() {
    // Explicit `true` and `false` are Always and Never, and both differ from an omitted condition.
    let always = admitted(&measured("Integer", "{count: {}, where: true}"));
    assert_eq!(
        condition(&always, "demo.cases.V", "m"),
        Some(Predicate::Always)
    );
    let never = admitted(&measured("Integer", "{count: {}, where: false}"));
    assert_eq!(
        condition(&never, "demo.cases.V", "m"),
        Some(Predicate::Never)
    );
    let omitted = admitted(&measured("Integer", "{count: {}}"));
    assert_eq!(condition(&omitted, "demo.cases.V", "m"), None);
    assert_ne!(
        always.views()[&name("demo.cases.V")].aggregation,
        omitted.views()[&name("demo.cases.V")].aggregation
    );

    // From ess/22 a bare word on the right of a comparison is resolved before the model holds it:
    // `priority == High` is the enum variant, `label == team` compares two fields of the row.
    let spec = admitted(&measured(
        "Integer",
        "{count: {}, where: [priority == High, label == team]}",
    ));
    let resolved = condition(&spec, "demo.cases.V", "m").unwrap();
    assert_eq!(
        resolved,
        Predicate::All(vec![
            equals("priority", FactValue::text("High")),
            Predicate::Compare {
                left: fact("label"),
                op: CompareOp::Eq,
                right: fact("team"),
                kind: CompareKind::Value,
            },
        ]),
        "{resolved}"
    );
    // What a canonical write carries is the resolved condition, with the fact made explicit.
    let back = serde_json::to_value(RawViewSpec::from(
        spec.views()[&name("demo.cases.V")].clone(),
    ))
    .unwrap();
    assert_eq!(
        back["fields"][1]["aggregate"]["where"],
        serde_json::to_value(&resolved).unwrap()
    );
}

// ---- conditional_aggregate_traits_preserve_structural_identity --------------------------------

fn hash<T: Hash>(value: &T) -> u64 {
    let mut hasher = DefaultHasher::new();
    value.hash(&mut hasher);
    hasher.finish()
}

fn traits<T: Clone + PartialEq + Eq + PartialOrd + Ord + Hash>() {}

fn aggregate(r#where: Option<Predicate>) -> Aggregate {
    Aggregate {
        function: AggregateFunction::Sum,
        input: Some("cents".to_owned()),
        skip_absent: false,
        r#where,
    }
}

fn number(text: &str) -> FactValue {
    FactValue::Number(Number::decimal_literal(text).unwrap_or_else(|| panic!("{text}")))
}

/// One predicate of every resolved variant, with nested composites, both quantifiers and two
/// adjacent integers above 2^53.
fn every_resolved_variant() -> Vec<Predicate> {
    let quantified = |universal: bool| {
        let body = ess_primitives::predicate::Quantified {
            over: FactPath::new("tags").unwrap(),
            bind: "t".to_owned(),
            body: equals("t", FactValue::text("rush")),
        };
        if universal {
            Predicate::Forall(Box::new(body))
        } else {
            Predicate::Exists(Box::new(body))
        }
    };
    vec![
        Predicate::Always,
        Predicate::Never,
        Predicate::All(vec![Predicate::Always, Predicate::Never]),
        Predicate::Any(vec![Predicate::Never, Predicate::Always]),
        Predicate::Not(Box::new(equals("escalated", FactValue::Bool(true)))),
        equals("cents", number("9007199254740992")),
        equals("cents", number("9007199254740993")),
        Predicate::Compare {
            left: fact("cents"),
            op: CompareOp::Gt,
            right: fact("team"),
            kind: CompareKind::Value,
        },
        Predicate::Truthy(FactPath::new("escalated").unwrap()),
        Predicate::Defined(FactPath::new("label").unwrap()),
        Predicate::AnyOf {
            path: FactPath::new("label").unwrap(),
            values: vec![FactValue::text("a"), FactValue::text("b")],
        },
        Predicate::NoneOf {
            path: FactPath::new("label").unwrap(),
            values: vec![FactValue::text("a")],
        },
        Predicate::TextMatch {
            path: FactPath::new("label").unwrap(),
            op: ess_primitives::predicate::TextOp::StartsWith,
            value: FactValue::text("x"),
        },
        Predicate::FoldMatch {
            path: FactPath::new("label").unwrap(),
            op: ess_primitives::predicate::FoldOp::EqualsIgnoreCase,
            values: vec![FactValue::text("x")],
        },
        quantified(true),
        quantified(false),
    ]
}

#[test]
fn conditional_aggregate_traits_preserve_structural_identity() {
    traits::<Aggregate>();

    // An unconditioned aggregate orders and hashes as the old `(function, input, skip_absent)`.
    let plain = aggregate(None);
    assert_eq!(
        hash(&plain),
        hash(&(AggregateFunction::Sum, Some("cents".to_owned()), false)),
        "the prior hash feed is preserved"
    );
    let mut count = plain.clone();
    count.function = AggregateFunction::Count;
    count.input = None;
    assert!(count < plain, "the old tuple still orders first");
    assert!(
        plain < aggregate(Some(Predicate::Never)),
        "None sorts first"
    );

    // Every resolved variant, nested composites and quantifiers, exact numbers.
    let variants = every_resolved_variant();
    let all: Vec<Aggregate> = std::iter::once(None)
        .chain(variants.into_iter().map(Some))
        .map(aggregate)
        .collect();
    for left in &all {
        for right in &all {
            assert_eq!(
                left.cmp(right) == std::cmp::Ordering::Equal,
                left == right,
                "{left:?} against {right:?}"
            );
            assert_eq!(left.cmp(right), right.cmp(left).reverse());
            if left == right {
                assert_eq!(hash(left), hash(right));
            }
        }
    }
    // Adjacent integers above 2^53 are two values; equivalent decimals and signed zero are one.
    assert_ne!(all[6], all[7]);
    assert_ne!(all[6].cmp(&all[7]), std::cmp::Ordering::Equal);
    for (a, b) in [
        (Number::new(-0.0).unwrap(), Number::from(0_i64)),
        (
            Number::new(1.5).unwrap(),
            Number::decimal_literal("1.5").unwrap(),
        ),
        (Number::new(2.0).unwrap(), Number::from(2_i64)),
    ] {
        let left = aggregate(Some(equals("cents", FactValue::Number(a))));
        let right = aggregate(Some(equals("cents", FactValue::Number(b))));
        assert_eq!(left, right, "{a:?} and {b:?}");
        assert_eq!(left.cmp(&right), std::cmp::Ordering::Equal);
        assert_eq!(hash(&left), hash(&right), "{a:?} and {b:?}");
    }
    // Structural, not logical: reordered children are another value.
    assert_ne!(
        aggregate(Some(Predicate::All(vec![
            equals("a", FactValue::Bool(true)),
            equals("b", FactValue::Bool(true))
        ]))),
        aggregate(Some(Predicate::All(vec![
            equals("b", FactValue::Bool(true)),
            equals("a", FactValue::Bool(true))
        ])))
    );
    // Two spellings that resolve to one predicate are one aggregate.
    let compact = admitted(&measured(
        "Integer",
        "{count: {}, where: escalated == true}",
    ));
    let mapped = admitted(&measured(
        "Integer",
        "{count: {}, where: {all: [escalated == true]}}",
    ));
    let of = |spec: &Specification| {
        spec.views()[&name("demo.cases.V")]
            .aggregation
            .as_ref()
            .unwrap()
            .function("m")
            .unwrap()
            .clone()
    };
    assert_eq!(of(&compact), of(&mapped));
    assert_eq!(hash(&of(&compact)), hash(&of(&mapped)));
}

// ---- rendering --------------------------------------------------------------------------------

#[test]
fn a_condition_is_rendered_only_where_it_is_written() {
    assert_eq!(aggregate(None).to_string(), "sum(cents)");
    assert_eq!(
        aggregate(Some(equals("escalated", FactValue::Bool(true)))).to_string(),
        "sum(cents) where escalated == true"
    );
}
