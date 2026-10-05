//! Family F part A2 at the source (beyond10x/ess#233, #244): from `ess/22` an unquoted right side
//! `<binder | root | dotted path> ws? (+|-) ws? <magnitude>` whose base resolves is one constant
//! offset of that fact (final review decision 4, rule 3a). An Integer base takes a whole magnitude
//! without a sign, fraction or leading zero; a `Timestamp` base takes a whole `s`, `m` or `h`
//! magnitude under the current-time bound. A base of another type is a `type_mismatch`, an
//! unresolved base stays the text it always was, and below `ess/22` nothing changes.
//! `docs/design/expression-family-source22.md`, "A2: one constant offset", is the design.

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::{FactPath, FactValue, Number};
use ess_primitives::predicate::{
    CompareKind, CompareOp, OffsetDirection, OffsetMagnitude, OffsetOperand, Operand, Predicate,
    Quantified,
};
use ess_primitives::time::ElapsedUnit;

/// One command whose `refused` branch is guarded by `when`, over Integer, Decimal, Timestamp,
/// newtype, Optional, struct and list inputs, and an entity whose invariant is `invariant`.
fn model(format: u32, when: &str, invariant: &str) -> String {
    format!(
        r"format: ess/{format}
system: pool
version: v1
domain: pool.lease
types:
  - {{name: pool.lease.Units, kind: newtype, of: Integer}}
  - {{name: pool.lease.Moment, kind: newtype, of: Timestamp}}
  - name: pool.lease.Window
    kind: struct
    fields:
      - {{name: lower, type: Integer}}
      - {{name: opened_at, type: Timestamp}}
  - name: pool.lease.Slot
    kind: struct
    fields:
      - {{name: low, type: Integer}}
      - {{name: high, type: Integer}}
entities:
  - name: pool.lease.Lease
    identity: {{name: lease_id, type: Uuid}}
    fields:
      - {{name: lower, type: Integer}}
      - {{name: upper, type: Integer}}
      - {{name: issued_at, type: Timestamp}}
      - {{name: expires_at, type: Timestamp}}
    invariants:
      - {invariant}
    lifecycle: {{initial: Open, states: [Open], terminal: [Open]}}
errors:
  - name: pool.lease.Refused
    summary: The lease is refused.
events:
  - name: pool.lease.Opened
    fields:
      - {{name: lease_id, type: Uuid}}
commands:
  - name: pool.lease.Open
    input:
      - {{name: lower, type: Integer}}
      - {{name: upper, type: Integer}}
      - {{name: units, type: pool.lease.Units}}
      - {{name: reserved, type: pool.lease.Units}}
      - {{name: maybe_lower, type: Optional<Integer>}}
      - {{name: amount, type: Decimal}}
      - {{name: limit, type: Decimal}}
      - {{name: label, type: String}}
      - {{name: issued_at, type: Timestamp}}
      - {{name: expires_at, type: Timestamp}}
      - {{name: renewed_at, type: pool.lease.Moment}}
      - {{name: window, type: pool.lease.Window}}
      - {{name: slots, type: List<pool.lease.Slot>}}
    outcomes:
      - name: refused
        when: {when}
        error: pool.lease.Refused
      - name: opened
        creates: pool.lease.Lease
        instance: lease_id
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened: {{lease_id: {{generated: true}}}}
        sets: {{lower: input.lower, upper: input.upper, issued_at: input.issued_at, expires_at: input.expires_at}}
"
    )
}

const PLAIN: &str = "upper >= lower";

/// An invariant every format reads alike.
const OLD: &str = "upper >= 0";

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn admitted(text: &str) -> Specification {
    assemble(text).unwrap_or_else(|errors| panic!("admitted:\n{errors}\n---\n{text}"))
}

fn guard(spec: &Specification) -> Predicate {
    let command = &spec.commands()[&"pool.lease.Open".parse().unwrap()];
    match &command.outcomes[0].condition {
        OutcomeCondition::When(predicate) => predicate.clone(),
        other => panic!("a plain when, not {other:?}"),
    }
}

fn invariant(spec: &Specification) -> Predicate {
    spec.entities()[&"pool.lease.Lease".parse().unwrap()].invariants[0]
        .predicate
        .clone()
}

fn path(text: &str) -> FactPath {
    text.parse().expect("a path")
}

fn integer(base: &str, direction: OffsetDirection, magnitude: i64) -> Operand {
    Operand::Offset(OffsetOperand {
        base: path(base),
        direction,
        magnitude: OffsetMagnitude::Integer(Number::from(magnitude)),
    })
}

fn elapsed(base: &str, direction: OffsetDirection, seconds: i64, unit: ElapsedUnit) -> Operand {
    Operand::Offset(OffsetOperand {
        base: path(base),
        direction,
        magnitude: OffsetMagnitude::ElapsedSeconds {
            seconds,
            written_unit: unit,
        },
    })
}

fn compare(left: &str, op: CompareOp, right: Operand) -> Predicate {
    Predicate::Compare {
        kind: CompareKind::Value,
        left: Operand::Fact(path(left)),
        op,
        right,
    }
}

fn json(predicate: &Predicate) -> String {
    serde_json::to_string(predicate).expect("serialises")
}

#[test]
fn a2_integer_offset_equality_and_inequality_resolve_to_the_typed_operand() {
    for (written, op) in [
        ("upper == lower + 5", CompareOp::Eq),
        ("upper != lower + 5", CompareOp::Ne),
        ("upper == lower+5", CompareOp::Eq),
    ] {
        let spec = admitted(&model(22, written, PLAIN));
        assert_eq!(
            guard(&spec),
            compare("upper", op, integer("lower", OffsetDirection::Add, 5)),
            "{written}"
        );
    }
    let spec = admitted(&model(22, "upper == lower + 5", PLAIN));
    assert_eq!(
        json(&guard(&spec)),
        r#"{"upper":{"eq":{"offset":{"add":5.0,"fact":"lower"}}}}"#
    );
}

#[test]
fn a2_every_ordering_and_subtraction_below_zero_resolve() {
    for (written, op, direction) in [
        ("upper < lower + 5", CompareOp::Lt, OffsetDirection::Add),
        ("upper <= lower + 5", CompareOp::Le, OffsetDirection::Add),
        (
            "upper > lower - 5",
            CompareOp::Gt,
            OffsetDirection::Subtract,
        ),
        (
            "upper >= lower - 5",
            CompareOp::Ge,
            OffsetDirection::Subtract,
        ),
        (
            "{upper: {lte: lower - 5}}",
            CompareOp::Le,
            OffsetDirection::Subtract,
        ),
    ] {
        let spec = admitted(&model(22, written, PLAIN));
        assert_eq!(
            guard(&spec),
            compare("upper", op, integer("lower", direction, 5)),
            "{written}"
        );
    }
    let spec = admitted(&model(22, "upper == lower + 0", PLAIN));
    assert_eq!(
        guard(&spec),
        compare(
            "upper",
            CompareOp::Eq,
            integer("lower", OffsetDirection::Add, 0)
        ),
        "zero is legal"
    );
    let spec = admitted(&model(22, "upper == lower + 9223372036854775807", PLAIN));
    assert_eq!(
        guard(&spec),
        compare(
            "upper",
            CompareOp::Eq,
            integer("lower", OffsetDirection::Add, i64::MAX)
        ),
        "the largest magnitude"
    );
}

#[test]
fn a2_timestamp_offsets_in_each_unit_admit_all_six_operators() {
    for (written, op, seconds, unit) in [
        (
            "expires_at <= issued_at - 24h",
            CompareOp::Le,
            86_400,
            ElapsedUnit::Hours,
        ),
        (
            "expires_at == issued_at - 5m",
            CompareOp::Eq,
            300,
            ElapsedUnit::Minutes,
        ),
        (
            "expires_at != issued_at - 30s",
            CompareOp::Ne,
            30,
            ElapsedUnit::Seconds,
        ),
    ] {
        let spec = admitted(&model(22, written, PLAIN));
        assert_eq!(
            guard(&spec),
            compare(
                "expires_at",
                op,
                elapsed("issued_at", OffsetDirection::Subtract, seconds, unit)
            ),
            "{written}"
        );
    }
    let spec = admitted(&model(22, "expires_at > issued_at + 1h", PLAIN));
    assert_eq!(
        json(&guard(&spec)),
        r#"{"expires_at":{"gt":{"offset":{"add":"1h","fact":"issued_at"}}}}"#,
        "an offset is never tagged: its unit already says instant"
    );
}

#[test]
fn a2_newtypes_optionals_dotted_bases_and_binders_resolve() {
    for (written, left, base) in [
        ("units == reserved + 1", "units", "reserved"),
        ("upper == maybe_lower + 1", "upper", "maybe_lower"),
        ("maybe_lower == lower + 1", "maybe_lower", "lower"),
        ("upper == window.lower + 1", "upper", "window.lower"),
        ("upper == input.lower + 1", "upper", "lower"),
    ] {
        let spec = admitted(&model(22, written, PLAIN));
        assert_eq!(
            guard(&spec),
            compare(left, CompareOp::Eq, integer(base, OffsetDirection::Add, 1)),
            "{written}"
        );
    }
    let spec = admitted(&model(22, "renewed_at >= window.opened_at + 1h", PLAIN));
    assert_eq!(
        guard(&spec),
        compare(
            "renewed_at",
            CompareOp::Ge,
            elapsed(
                "window.opened_at",
                OffsetDirection::Add,
                3_600,
                ElapsedUnit::Hours
            )
        )
    );
    let spec = admitted(&model(
        22,
        "{forall: {in: slots, as: slot, that: slot.high == slot.low + 1}}",
        PLAIN,
    ));
    assert_eq!(
        guard(&spec),
        Predicate::Forall(Box::new(Quantified {
            over: path("slots"),
            bind: "slot".to_owned(),
            body: compare(
                "slot.high",
                CompareOp::Eq,
                integer("slot.low", OffsetDirection::Add, 1)
            ),
        }))
    );
}

#[test]
fn a2_an_entity_invariant_reads_an_offset_of_a_sibling() {
    let spec = admitted(&model(22, "label == x", "expires_at <= issued_at + 24h"));
    assert_eq!(
        invariant(&spec),
        compare(
            "expires_at",
            CompareOp::Le,
            elapsed(
                "issued_at",
                OffsetDirection::Add,
                86_400,
                ElapsedUnit::Hours
            )
        )
    );
    let spec = admitted(&model(22, "label == x", "upper == lower + 5"));
    assert_eq!(
        invariant(&spec),
        compare(
            "upper",
            CompareOp::Eq,
            integer("lower", OffsetDirection::Add, 5)
        )
    );
}

#[test]
fn a2_the_canonical_form_reads_back_as_the_compact_spelling_resolves() {
    let compact = admitted(&model(22, "upper < lower + 5", PLAIN));
    let canonical = admitted(&model(
        22,
        "{upper: {lt: {offset: {fact: lower, add: 5}}}}",
        PLAIN,
    ));
    assert_eq!(guard(&compact), guard(&canonical));
    let canonical = admitted(&model(
        22,
        "{expires_at: {lte: {offset: {fact: issued_at, subtract: 24h}}}}",
        PLAIN,
    ));
    assert_eq!(
        guard(&canonical),
        compare(
            "expires_at",
            CompareOp::Le,
            elapsed(
                "issued_at",
                OffsetDirection::Subtract,
                86_400,
                ElapsedUnit::Hours
            )
        )
    );
}

#[test]
fn a2_decimal_and_mismatched_offsets_are_type_mismatches() {
    for written in [
        "amount == limit + 1",
        "upper == amount + 1",
        "amount == lower + 1",
        "expires_at == issued_at + 5",
        "upper == lower + 5m",
        "label == lower + 5",
        "upper == issued_at + 1h",
        "upper == window + 1",
        "{upper: {eq: {offset: {fact: amount, add: 1}}}}",
    ] {
        let refused = assemble(&model(22, written, PLAIN)).expect_err(written);
        assert!(refused.contains("type_mismatch"), "{written}: {refused}");
        assert!(refused.contains("offset"), "{written}: {refused}");
    }
    let refused = assemble(&model(22, "amount == limit + 1", PLAIN)).expect_err("a Decimal");
    assert!(refused.contains("Decimal"), "{refused}");
}

#[test]
fn a2_malformed_and_double_offsets_are_refused() {
    for written in [
        "upper == lower + 05",
        "upper == lower + 5 + 3",
        "upper == lower + -5",
        "upper == lower + 1.5",
        "upper == lower +",
        "expires_at <= issued_at - 1d",
        "expires_at <= issued_at - 05h",
        "expires_at <= issued_at - 3155760001s",
    ] {
        let refused = assemble(&model(22, written, PLAIN)).expect_err(written);
        assert!(refused.contains("type_mismatch"), "{written}: {refused}");
        assert!(refused.contains("offset"), "{written}: {refused}");
    }
}

#[test]
fn a2_an_unresolved_base_stays_the_text_it_always_was() {
    let spec = admitted(&model(22, "label == nothing + 5", PLAIN));
    assert_eq!(
        guard(&spec),
        compare(
            "label",
            CompareOp::Eq,
            Operand::Literal(FactValue::text("nothing + 5"))
        )
    );
    // Text compared with a `String` stays text where no magnitude reads, and quoted always: rule
    // 3a reads `<fact> ± <magnitude>` only, so `lower-only` beside a field named `lower` is the
    // text it is spelled like.
    for (written, text) in [
        ("label == lower-only", "lower-only"),
        ("label == lower + 5x", "lower + 5x"),
        ("label == \"lower + 5\"", "lower + 5"),
    ] {
        let spec = admitted(&model(22, written, PLAIN));
        assert_eq!(
            guard(&spec),
            compare(
                "label",
                CompareOp::Eq,
                Operand::Literal(FactValue::text(text))
            ),
            "{written}"
        );
    }
    // `now` names no root here, so the current-time literal keeps its representation and bytes.
    let spec = admitted(&model(22, "expires_at > now - 60s", PLAIN));
    assert_eq!(json(&guard(&spec)), r#""expires_at > now - 60s""#);
}

#[test]
fn a2_source21_bytes_unchanged() {
    // Below ess/22 the spelling is the text it always was: refused against an Integer, admitted
    // against a String, with the bytes it always had.
    let refused = assemble(&model(21, "upper == lower + 5", OLD)).expect_err("text vs Integer");
    assert!(!refused.contains("offset"), "{refused}");
    let spec = admitted(&model(21, "label == lower + 5", OLD));
    assert_eq!(json(&guard(&spec)), r#""label == lower + 5""#);
    let spec = admitted(&model(21, "expires_at > now - 60s", OLD));
    assert_eq!(json(&guard(&spec)), r#""expires_at > now - 60s""#);
    let refused = assemble(&model(
        21,
        "{upper: {eq: {offset: {fact: lower, add: 5}}}}",
        OLD,
    ))
    .expect_err("ess/21 refuses the offset mapping");
    assert_eq!(
        refused,
        "parse: cannot parse predicate \"upper: {eq: {offset: {add: 5, fact: lower}}}\": a \
         comparison operand must be a scalar"
    );
}

#[test]
fn a2_an_offset_in_an_older_source_assembled_directly_is_refused_by_format() {
    // A predicate built in code reaches the checker without a source spelling: the format still
    // decides, as it does for `{fact: …}`.
    let raw = RawSpecFile::parse(&model(21, "label == x", OLD)).expect("parses");
    let spec = Specification::assemble([(Source::new("model.yaml"), raw)]).expect("assembles");
    let registry = spec
        .system()
        .types
        .clone()
        .with_format(spec.system().format);
    let command = &spec.commands()[&"pool.lease.Open".parse().unwrap()];
    let environment = ess_domain::expression::DomainEnvironment::new(&registry, &command.input);
    let predicate = compare(
        "upper",
        CompareOp::Eq,
        integer("lower", OffsetDirection::Add, 5),
    );
    let checked = ess_domain::expression::check_predicate(&environment, &predicate, "test");
    let errors: Vec<String> = checked
        .errors
        .iter()
        .map(|error| error.validation_error().to_string())
        .collect();
    assert!(
        errors.iter().any(|error| error.contains("ess/22")),
        "{errors:?}"
    );
}

#[test]
fn a2_a_root_named_now_is_the_base_of_an_offset() {
    let text = model(22, "expires_at > now - 60s", PLAIN).replace(
        "      - {name: label, type: String}\n",
        "      - {name: label, type: String}\n      - {name: now, type: Timestamp}\n",
    );
    let spec = admitted(&text);
    assert_eq!(
        guard(&spec),
        compare(
            "expires_at",
            CompareOp::Gt,
            elapsed("now", OffsetDirection::Subtract, 60, ElapsedUnit::Seconds)
        )
    );
}
