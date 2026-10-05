//! Family F part A2 (beyond10x/ess#233, #244): one constant offset on a right-hand fact.
//!
//! `upper == lower + 5` and `promoted_at <= previous_at - 24h` compare a fact with another fact
//! moved by a constant. The canonical form is the closed mapping
//! `{offset: {fact: <path>, add|subtract: <magnitude>}}`; an Integer magnitude is compared as the
//! exact mathematical sum, never wrapped, clamped, rounded through binary64 or turned `Unknown`, and
//! an elapsed `s`/`m`/`h` magnitude moves a `Timestamp` by UTC seconds.
//! `docs/design/expression-family-source22.md` is the design; the source-format resolver that reads
//! the compact spelling as an offset lives in `ess-domain`.

// Every arithmetic below shares one signature, and two of them answer Unknown, so the ones that
// never do still return `Option`.
#![allow(clippy::unnecessary_wraps)]

use ess_primitives::facts::{FactPath, FactStore, FactValue, Number};
use ess_primitives::node::Node;
use ess_primitives::predicate::{
    reading_source22_operands, CompareKind, CompareOp, OffsetDirection, OffsetMagnitude,
    OffsetOperand, Operand, Predicate, Truth,
};
use ess_primitives::time::{CurrentTime, ElapsedUnit, Rfc3339Instant};

fn path(text: &str) -> FactPath {
    text.parse().expect("a fact path")
}

fn offset(base: &str, direction: OffsetDirection, magnitude: OffsetMagnitude) -> Operand {
    Operand::Offset(OffsetOperand {
        base: path(base),
        direction,
        magnitude,
    })
}

fn integer(value: i64) -> OffsetMagnitude {
    OffsetMagnitude::Integer(Number::from(value))
}

fn compare(left: &str, op: CompareOp, right: Operand) -> Predicate {
    Predicate::Compare {
        left: Operand::Fact(path(left)),
        op,
        right,
        kind: CompareKind::Value,
    }
}

fn read(text: &str) -> Result<Predicate, String> {
    serde_json::from_str::<Predicate>(text).map_err(|error| error.to_string())
}

fn json(predicate: &Predicate) -> String {
    serde_json::to_string(predicate).expect("a predicate serialises")
}

#[test]
fn a2_the_canonical_offset_reads_and_writes_back_byte_for_byte() {
    // An integral magnitude binary64 carries is written as every such number is, `5.0`.
    let integer_text = r#"{"upper":{"eq":{"offset":{"add":5.0,"fact":"lower"}}}}"#;
    let parsed = read(integer_text).expect("reads");
    assert_eq!(
        parsed,
        compare(
            "upper",
            CompareOp::Eq,
            offset("lower", OffsetDirection::Add, integer(5))
        )
    );
    assert_eq!(json(&parsed), integer_text);

    let elapsed_text =
        r#"{"promoted_at":{"lte":{"offset":{"fact":"previous_at","subtract":"24h"}}}}"#;
    let parsed = read(elapsed_text).expect("reads");
    assert_eq!(
        parsed,
        compare(
            "promoted_at",
            CompareOp::Le,
            offset(
                "previous_at",
                OffsetDirection::Subtract,
                OffsetMagnitude::ElapsedSeconds {
                    seconds: 86_400,
                    written_unit: ElapsedUnit::Hours,
                }
            )
        )
    );
    assert_eq!(json(&parsed), elapsed_text, "the written unit is kept");
    for (written, seconds, unit) in [
        ("30s", 30, ElapsedUnit::Seconds),
        ("5m", 300, ElapsedUnit::Minutes),
        ("1h", 3_600, ElapsedUnit::Hours),
    ] {
        let text = format!(r#"{{"at":{{"gt":{{"offset":{{"add":"{written}","fact":"from"}}}}}}}}"#);
        let parsed = read(&text).expect("reads");
        assert_eq!(
            parsed,
            compare(
                "at",
                CompareOp::Gt,
                offset(
                    "from",
                    OffsetDirection::Add,
                    OffsetMagnitude::ElapsedSeconds {
                        seconds,
                        written_unit: unit
                    }
                )
            ),
            "{written}"
        );
        assert_eq!(json(&parsed), text);
    }
    let maximal = r#"{"a":{"gte":{"offset":{"add":9223372036854775807,"fact":"b"}}}}"#;
    assert_eq!(
        json(&read(maximal).expect("the largest magnitude reads")),
        maximal,
        "i64::MAX is written exactly"
    );
    assert_eq!(
        read(maximal).expect("reads"),
        compare(
            "a",
            CompareOp::Ge,
            offset("b", OffsetDirection::Add, integer(i64::MAX))
        )
    );
}

#[test]
fn a2_below_source22_an_offset_mapping_is_no_operand() {
    let node: Node =
        serde_json::from_str(r#"{"upper": {"eq": {"offset": {"fact": "lower", "add": 5}}}}"#)
            .expect("json");
    let refused = reading_source22_operands(false, || Predicate::from_node(&node))
        .expect_err("refused below ess/22");
    assert!(
        refused.to_string().contains("must be a scalar"),
        "{refused}"
    );
    assert!(
        reading_source22_operands(true, || Predicate::from_node(&node)).is_ok(),
        "read from ess/22"
    );
}

#[test]
fn a2_the_rendering_tells_an_offset_from_the_text_it_is_spelled_like() {
    let offset = compare(
        "upper",
        CompareOp::Eq,
        offset("lower", OffsetDirection::Add, integer(5)),
    );
    let text = compare(
        "upper",
        CompareOp::Eq,
        Operand::Literal(FactValue::text("lower + 5")),
    );
    assert_ne!(offset.to_string(), text.to_string());
    assert_ne!(json(&offset), json(&text));
    assert!(offset.reads_offset());
    assert!(!text.reads_offset());
    assert_eq!(
        offset
            .fact_paths()
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>(),
        ["upper", "lower"],
        "the base is a read"
    );
    let nested: Predicate = serde_json::from_str(
        r#"{"forall": {"in": "slots", "as": "slot", "that": {"slot.high": {"eq": {"offset": {"fact": "slot.low", "add": 1}}}}}}"#,
    )
    .expect("reads");
    assert!(nested.reads_offset(), "inside a quantifier body too");
    assert_eq!(
        Predicate::from_node(&nested.to_node()).expect("reads back"),
        nested
    );
}

#[test]
fn a2_the_spelled_parse_marks_unquoted_offset_spellings() {
    let cases: &[(&str, &[bool])] = &[
        (r#""upper == lower + 5""#, &[true]),
        (r#""upper == lower+5""#, &[true]),
        (r#""upper == window.lower - 24h""#, &[true]),
        (r#""upper == \"lower + 5\"""#, &[false]),
        (r#"{"upper": {"eq": "lower + 5"}}"#, &[true]),
        (r#"{"upper": "lower + 5"}"#, &[false]),
        (r#""upper == lower""#, &[false]),
        (r#""upper == 5""#, &[false]),
        (r#""upper == -5""#, &[false]),
        (r#""at < now - 60s""#, &[true]),
    ];
    for (text, want) in cases {
        let node: Node = serde_json::from_str(text).expect("json");
        let spelled = Predicate::from_node_spelled(&node).expect("parses");
        assert_eq!(spelled.offsets, *want, "{text}");
        assert_eq!(
            Ok(spelled.predicate),
            Predicate::from_node(&node).map_err(|error| error.to_string()),
            "the spelled parse reads the predicate the ordinary parse reads: {text}"
        );
    }
}

#[test]
fn a2_offset_spellings_split_at_every_sign_and_magnitudes_follow_the_digit_rules() {
    let split = |text: &str| {
        OffsetOperand::spellings(text)
            .into_iter()
            .map(|(base, direction, magnitude)| (base.to_string(), direction, magnitude.to_owned()))
            .collect::<Vec<_>>()
    };
    assert_eq!(
        split("lower + 5"),
        [("lower".to_owned(), OffsetDirection::Add, "5".to_owned())]
    );
    assert_eq!(
        split("my-field-5"),
        [
            (
                "my".to_owned(),
                OffsetDirection::Subtract,
                "field-5".to_owned()
            ),
            (
                "my-field".to_owned(),
                OffsetDirection::Subtract,
                "5".to_owned()
            ),
        ],
        "a hyphen may be part of the base"
    );
    assert!(split("+5").is_empty(), "no base");
    assert!(split("\"a\" + 5").is_empty(), "a quoted base is no path");
    assert_eq!(
        OffsetMagnitude::parse("5"),
        Some(OffsetMagnitude::Integer(Number::from(5_i64)))
    );
    assert_eq!(
        OffsetMagnitude::parse("0"),
        Some(OffsetMagnitude::Integer(Number::from(0_i64)))
    );
    assert_eq!(
        OffsetMagnitude::parse("9223372036854775807"),
        Some(OffsetMagnitude::Integer(Number::from(i64::MAX)))
    );
    for refused in [
        "05",
        "-5",
        "+5",
        "5.0",
        "1e3",
        "9223372036854775808",
        "5 + 3",
        "",
        "1d",
        "05m",
        "3155760001s",
        "5 m",
    ] {
        assert_eq!(OffsetMagnitude::parse(refused), None, "{refused}");
    }
    assert_eq!(
        OffsetMagnitude::parse("3155760000s"),
        Some(OffsetMagnitude::ElapsedSeconds {
            seconds: CurrentTime::MAX_OFFSET_SECONDS,
            written_unit: ElapsedUnit::Seconds,
        }),
        "the current-time bound"
    );
}

#[test]
fn a2_current_time_keeps_its_representation_and_shares_the_arithmetic() {
    let reference = Rfc3339Instant::parse_rfc3339("2020-01-01T00:00:00Z").expect("instant");
    let current = CurrentTime::parse("now - 24h").expect("parses");
    assert_eq!(
        current.at(reference),
        reference.plus_elapsed(-86_400),
        "one checked helper"
    );
    let last = Rfc3339Instant::parse_rfc3339("9999-12-31T23:59:59Z").expect("instant");
    assert_eq!(last.plus_elapsed(1), None, "past 9999 is no instant");
    assert_eq!(
        CurrentTime::parse("now + 1s").expect("parses").at(last),
        None
    );
    let predicate = Predicate::parse_expression("at < now - 60s").expect("parses");
    assert_eq!(
        predicate,
        compare(
            "at",
            CompareOp::Lt,
            Operand::Literal(FactValue::text("now - 60s"))
        ),
        "the current-time literal keeps its representation"
    );
    assert!(!predicate.reads_offset());
}

// ---- the shared vectors -----------------------------------------------------------------------

/// The shared vectors the Go and TypeScript readers answer too.
const VECTORS: &str = include_str!("vectors/offset-operand.json");

fn vectors() -> serde_json::Value {
    serde_json::from_str(VECTORS).expect("json")
}

/// A JSON scalar as the fact it binds: an integer token exactly, never through binary64.
fn bind(store: &mut FactStore, path: &str, value: &serde_json::Value) {
    match value {
        serde_json::Value::Object(fields) => {
            for (key, nested) in fields {
                let at = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{path}.{key}")
                };
                bind(store, &at, nested);
            }
        }
        serde_json::Value::Array(items) => {
            store.set_path(&format!("{path}.count"), FactValue::count(items.len()));
            for (index, item) in items.iter().enumerate() {
                bind(store, &format!("{path}.{index}"), item);
            }
        }
        serde_json::Value::String(text) => store.set_path(path, FactValue::text(text.clone())),
        serde_json::Value::Number(_) => store.set_path(
            path,
            FactValue::Number(serde_json::from_value(value.clone()).expect("a number")),
        ),
        serde_json::Value::Bool(flag) => store.set_path(path, FactValue::Bool(*flag)),
        serde_json::Value::Null => {}
    }
}

/// One `integer` or `timestamp` row as the predicate and the row it describes.
fn row_case(vector: &serde_json::Value) -> (Predicate, FactStore) {
    let predicate = serde_json::json!({
        "left": {vector["op"].as_str().expect("an op"): {"offset": {
            "fact": "base",
            vector["direction"].as_str().expect("a direction"): vector["magnitude"].clone(),
        }}}
    });
    let predicate: Predicate =
        serde_json::from_value(predicate).expect("the row's predicate reads");
    let mut store = FactStore::new();
    bind(&mut store, "left", &vector["left"]);
    bind(&mut store, "base", &vector["base"]);
    (predicate, store)
}

#[test]
fn a2_the_rust_evaluator_answers_the_shared_vectors() {
    let vectors = vectors();
    let mut answered = 0;
    for section in ["integer", "timestamp"] {
        for vector in vectors[section].as_array().expect("a list") {
            let name = vector["name"].as_str().expect("a name");
            let (predicate, store) = row_case(vector);
            assert_eq!(
                predicate.evaluate(&store).as_str(),
                vector["truth"].as_str().expect("a truth"),
                "{section}: {name}"
            );
            answered += 1;
        }
    }
    for vector in vectors["evaluate"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        let predicate: Predicate = serde_json::from_value(vector["predicate"].clone())
            .unwrap_or_else(|error| panic!("{name}: {error}"));
        let mut store = FactStore::new();
        bind(&mut store, "", &vector["row"]);
        assert_eq!(
            predicate.evaluate(&store).as_str(),
            vector["truth"].as_str().expect("a truth"),
            "{name}"
        );
        answered += 1;
    }
    for vector in vectors["refused"].as_array().expect("a list") {
        let name = vector["name"].as_str().expect("a name");
        assert!(
            serde_json::from_value::<Predicate>(vector["predicate"].clone()).is_err(),
            "{name} is refused"
        );
        answered += 1;
    }
    assert!(answered >= 60, "{answered} vectors answered");
}

#[test]
fn a2_an_unrepresentable_instant_is_unknown_with_a_note() {
    let predicate = compare(
        "left",
        CompareOp::Lt,
        offset(
            "base",
            OffsetDirection::Add,
            OffsetMagnitude::ElapsedSeconds {
                seconds: 1,
                written_unit: ElapsedUnit::Seconds,
            },
        ),
    );
    let mut store = FactStore::new();
    store.set_path("left", FactValue::text("9999-12-31T23:59:59Z"));
    store.set_path("base", FactValue::text("9999-12-31T23:59:59Z"));
    let outcome = predicate.outcome(&store);
    assert_eq!(outcome.truth, Truth::Unknown);
    let note = outcome.causes[0].note.clone().expect("a note");
    assert!(note.contains("instant"), "{note}");
    assert_eq!(Predicate::not(predicate).evaluate(&store), Truth::Unknown);
}

// ---- paired faults ----------------------------------------------------------------------------

/// What an Integer row's comparison answers under one way of computing `base ± magnitude`: the
/// target as an ordering of `left`, or `None` for Unknown.
type Arithmetic = fn(i64, bool, i64, i64) -> Option<std::cmp::Ordering>;

fn exact(base: i64, add: bool, magnitude: i64, left: i64) -> Option<std::cmp::Ordering> {
    let target = if add {
        i128::from(base) + i128::from(magnitude)
    } else {
        i128::from(base) - i128::from(magnitude)
    };
    Some(i128::from(left).cmp(&target))
}

fn wraps(base: i64, add: bool, magnitude: i64, left: i64) -> Option<std::cmp::Ordering> {
    let target = if add {
        base.wrapping_add(magnitude)
    } else {
        base.wrapping_sub(magnitude)
    };
    Some(left.cmp(&target))
}

fn clamps(base: i64, add: bool, magnitude: i64, left: i64) -> Option<std::cmp::Ordering> {
    let target = if add {
        base.saturating_add(magnitude)
    } else {
        base.saturating_sub(magnitude)
    };
    Some(left.cmp(&target))
}

#[allow(clippy::cast_precision_loss)]
fn rounds(base: i64, add: bool, magnitude: i64, left: i64) -> Option<std::cmp::Ordering> {
    let target = if add {
        base as f64 + magnitude as f64
    } else {
        base as f64 - magnitude as f64
    };
    (left as f64).partial_cmp(&target)
}

fn unknown_on_overflow(
    base: i64,
    add: bool,
    magnitude: i64,
    left: i64,
) -> Option<std::cmp::Ordering> {
    let target = if add {
        base.checked_add(magnitude)
    } else {
        base.checked_sub(magnitude)
    }?;
    Some(left.cmp(&target))
}

fn ignores_sign(base: i64, _: bool, magnitude: i64, left: i64) -> Option<std::cmp::Ordering> {
    exact(base, true, magnitude, left)
}

fn ignores_base(_: i64, _: bool, magnitude: i64, left: i64) -> Option<std::cmp::Ordering> {
    Some(left.cmp(&magnitude))
}

/// `left - base` against `±magnitude`, in `i64`.
fn field_minus_field(
    base: i64,
    add: bool,
    magnitude: i64,
    left: i64,
) -> Option<std::cmp::Ordering> {
    let bound = if add {
        magnitude
    } else {
        magnitude.wrapping_neg()
    };
    Some(left.wrapping_sub(base).cmp(&bound))
}

fn answer(arithmetic: Arithmetic, vector: &serde_json::Value) -> &'static str {
    let number = |key: &str| vector[key].as_i64().expect("an i64");
    let op = match vector["op"].as_str().expect("an op") {
        "eq" => CompareOp::Eq,
        "ne" => CompareOp::Ne,
        "lt" => CompareOp::Lt,
        "lte" => CompareOp::Le,
        "gt" => CompareOp::Gt,
        _ => CompareOp::Ge,
    };
    let add = vector["direction"] == "add";
    match arithmetic(number("base"), add, number("magnitude"), number("left")) {
        None => "unknown",
        Some(ordering) => {
            let holds = match op {
                CompareOp::Eq => ordering.is_eq(),
                CompareOp::Ne => ordering.is_ne(),
                CompareOp::Lt => ordering.is_lt(),
                CompareOp::Le => ordering.is_le(),
                CompareOp::Gt => ordering.is_gt(),
                CompareOp::Ge => ordering.is_ge(),
            };
            if holds {
                "true"
            } else {
                "false"
            }
        }
    }
}

#[test]
fn a2_every_paired_fault_disagrees_with_a_vector() {
    let vectors = vectors();
    let rows = vectors["integer"].as_array().expect("a list");
    for vector in rows {
        assert_eq!(
            answer(exact, vector),
            vector["truth"].as_str().expect("a truth"),
            "the exact reference answers {}",
            vector["name"]
        );
    }
    for (fault, arithmetic) in [
        ("wrap", wraps as Arithmetic),
        ("clamp", clamps),
        ("binary64", rounds),
        ("unknown on overflow", unknown_on_overflow),
        ("ignored sign", ignores_sign),
        ("ignored base", ignores_base),
        ("field minus field", field_minus_field),
    ] {
        let killed: Vec<&str> = rows
            .iter()
            .filter(|vector| {
                answer(arithmetic, vector) != vector["truth"].as_str().expect("a truth")
            })
            .map(|vector| vector["name"].as_str().expect("a name"))
            .collect();
        assert!(
            !killed.is_empty(),
            "no vector tells the {fault} fault apart"
        );
    }
    // The unit: a magnitude read as seconds whatever it was written in.
    let timestamps = vectors["timestamp"].as_array().expect("a list");
    let unit_blind = timestamps.iter().any(|vector| {
        let magnitude = vector["magnitude"].as_str().expect("text");
        let digits: i64 = magnitude[..magnitude.len() - 1].parse().expect("digits");
        let (left, base) = (
            Rfc3339Instant::parse_rfc3339(vector["left"].as_str().expect("text")),
            Rfc3339Instant::parse_rfc3339(vector["base"].as_str().expect("text")),
        );
        let (Some(left), Some(base)) = (left, base) else {
            return false;
        };
        let sign = if vector["direction"] == "add" { 1 } else { -1 };
        let blind = base
            .plus_elapsed(sign * digits)
            .map(|bound| left.cmp(&bound));
        let truth = vector["truth"].as_str().expect("a truth");
        match (blind, vector["op"].as_str().expect("an op")) {
            (Some(ordering), "eq") => ordering.is_eq() != (truth == "true"),
            _ => false,
        }
    });
    assert!(unit_blind, "no vector tells a unit-blind evaluator apart");
}
