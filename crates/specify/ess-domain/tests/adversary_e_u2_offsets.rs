//! Adversary, pass 1, for unit E-U2 (one constant offset, A2,
//! `docs/design/expression-family-source22.md` "A2: one constant offset" and final review decision
//! 4, rule 3a). Each case drives the `ess/22` source reader and checker from the design's own words:
//! `<binder | root | dotted path> ws? (+|-) ws? <magnitude>` whose base resolves is an offset, an
//! Integer magnitude is a whole number without a sign, fraction or leading zero that fits
//! `i64::MAX`, and Decimal, Binary64 and Duration are refused rather than coerced.

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_primitives::facts::{FactPath, Number};
use ess_primitives::predicate::{
    CompareKind, CompareOp, OffsetDirection, OffsetMagnitude, OffsetOperand, Operand, Predicate,
    Quantified,
};
use ess_primitives::time::ElapsedUnit;

/// One command whose `refused` branch is guarded by `when`, over Integer, Decimal, Binary64,
/// Duration, Timestamp, struct and list inputs.
fn model(when: &str) -> String {
    format!(
        r"format: ess/22
system: pool
version: v1
domain: pool.lease
types:
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
errors:
  - name: pool.lease.Refused
    summary: The lease is refused.
events:
  - name: pool.lease.Opened
    fields:
      - {{name: upper, type: Integer}}
commands:
  - name: pool.lease.Open
    input:
      - {{name: lower, type: Integer}}
      - {{name: upper, type: Integer}}
      - {{name: ratio, type: Binary64}}
      - {{name: other_ratio, type: Binary64}}
      - {{name: amount, type: Decimal}}
      - {{name: wait, type: Duration}}
      - {{name: lead, type: Duration}}
      - {{name: renewed_at, type: pool.lease.Moment}}
      - {{name: window, type: pool.lease.Window}}
      - {{name: slots, type: List<pool.lease.Slot>}}
    outcomes:
      - name: refused
        when: {when}
        error: pool.lease.Refused
      - name: opened
        emits: [pool.lease.Opened]
        payload:
          pool.lease.Opened: {{upper: input.upper}}
"
    )
}

fn assemble(text: &str) -> Result<Specification, String> {
    let raw = RawSpecFile::parse(text).map_err(|error| format!("parse: {error}"))?;
    Specification::assemble([(Source::new("model.yaml"), raw)]).map_err(|errors| errors.to_string())
}

fn guard(text: &str) -> Result<Predicate, String> {
    let spec = assemble(text)?;
    let command = &spec.commands()[&"pool.lease.Open".parse().unwrap()];
    match &command.outcomes[0].condition {
        OutcomeCondition::When(predicate) => Ok(predicate.clone()),
        other => Err(format!("a plain when, not {other:?}")),
    }
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

fn hours(base: &str, direction: OffsetDirection, count: i64) -> Operand {
    Operand::Offset(OffsetOperand {
        base: path(base),
        direction,
        magnitude: OffsetMagnitude::ElapsedSeconds {
            seconds: count * 3_600,
            written_unit: ElapsedUnit::Hours,
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

/// Rule 3a reads `ws?` on both sides of the sign for every kind of base. The `+` spellings and the
/// spaced `-` spellings are the controls; the unspaced `-` after a dotted path, an `input.` path or
/// a binder's member is the attack: `FactPath` admits `-` inside a segment, so `window.lower-5`
/// is read as the fact path `window.lower-5` before rule 3a ever sees it.
#[test]
fn adv_e_u2_rule3a_reads_an_unspaced_minus_after_every_kind_of_base() {
    use OffsetDirection::{Add, Subtract};
    let mut wrong = Vec::new();
    for (written, left, base, direction, magnitude) in [
        // controls
        ("upper == lower-5", "upper", "lower", Subtract, 5),
        ("upper == lower+5", "upper", "lower", Add, 5),
        ("upper == window.lower+5", "upper", "window.lower", Add, 5),
        (
            "upper == window.lower - 5",
            "upper",
            "window.lower",
            Subtract,
            5,
        ),
        ("upper == input.lower+5", "upper", "lower", Add, 5),
        // the attack
        (
            "upper == window.lower-5",
            "upper",
            "window.lower",
            Subtract,
            5,
        ),
        ("upper == input.lower-5", "upper", "lower", Subtract, 5),
    ] {
        let want = compare(left, CompareOp::Eq, integer(base, direction, magnitude));
        match guard(&model(written)) {
            Ok(got) if got == want => {}
            Ok(got) => wrong.push(format!("{written}: resolved to {got}")),
            Err(error) => wrong.push(format!("{written}: refused: {error}")),
        }
    }
    let written = "renewed_at >= window.opened_at-1h";
    let want = compare(
        "renewed_at",
        CompareOp::Ge,
        hours("window.opened_at", Subtract, 1),
    );
    match guard(&model(written)) {
        Ok(got) if got == want => {}
        Ok(got) => wrong.push(format!("{written}: resolved to {got}")),
        Err(error) => wrong.push(format!("{written}: refused: {error}")),
    }
    let written = "{forall: {in: slots, as: slot, that: slot.high == slot.low-1}}";
    let want = Predicate::Forall(Box::new(Quantified {
        over: path("slots"),
        bind: "slot".to_owned(),
        body: compare("slot.high", CompareOp::Eq, integer("slot.low", Subtract, 1)),
    }));
    match guard(&model(written)) {
        Ok(got) if got == want => {}
        Ok(got) => wrong.push(format!("{written}: resolved to {got}")),
        Err(error) => wrong.push(format!("{written}: refused: {error}")),
    }
    assert_eq!(wrong, Vec::<String>::new());
}

/// Final review decision 4: an Integer magnitude follows the current-time digit rules — no sign,
/// no fraction, no leading zero, no exponent — and fits `i64::MAX`. Each malformed spelling is
/// refused against an Integer, never read as text or as some nearby whole number.
#[test]
fn adv_e_u2_malformed_integer_magnitudes_are_refused() {
    let mut admitted = Vec::new();
    for written in [
        "upper == lower + 5.5",
        "upper == lower+5.5",
        "upper == lower-5.5",
        "upper == lower + 05",
        "upper == lower + +5",
        "upper == lower + 1e3",
        "upper == lower + 9223372036854775808",
        "upper == lower - 9223372036854775808",
        "upper == lower + 5 - 1",
        "{upper: {eq: {offset: {fact: lower, add: 5.5}}}}",
        "{upper: {eq: {offset: {fact: lower, add: '5'}}}}",
        "{upper: {eq: {offset: {fact: lower, add: -1}}}}",
        "{upper: {eq: {offset: {fact: lower, add: 9223372036854775808}}}}",
        "{upper: {eq: {offset: {fact: lower, add: 5, subtract: 5}}}}",
        "{upper: {eq: {offset: {fact: lower}}}}",
        "{upper: {eq: {offset: {fact: lower, add: 5, extra: 1}}}}",
    ] {
        if let Ok(predicate) = guard(&model(written)) {
            admitted.push(format!("{written}: admitted as {predicate}"));
        }
    }
    assert_eq!(admitted, Vec::<String>::new());
    for (written, want) in [
        (
            "upper == lower + 9223372036854775807",
            integer("lower", OffsetDirection::Add, i64::MAX),
        ),
        (
            "upper == lower - 9223372036854775807",
            integer("lower", OffsetDirection::Subtract, i64::MAX),
        ),
        (
            "upper == lower - 0",
            integer("lower", OffsetDirection::Subtract, 0),
        ),
    ] {
        assert_eq!(
            guard(&model(written)).unwrap_or_else(|error| panic!("{written}: {error}")),
            compare("upper", CompareOp::Eq, want),
            "{written}"
        );
    }
}

/// Decimal and Binary64 are refused for this slice, never coerced, and a Duration is no instant.
#[test]
fn adv_e_u2_binary64_and_duration_offsets_are_refused() {
    let mut admitted = Vec::new();
    for written in [
        "ratio == other_ratio + 1",
        "upper == ratio + 1",
        "ratio == lower + 1",
        "ratio < other_ratio - 1s",
        "amount == lower + 1",
        "wait == lead + 5s",
        "wait <= lead + 1",
        "renewed_at == wait + 1h",
    ] {
        match guard(&model(written)) {
            Ok(predicate) => admitted.push(format!("{written}: admitted as {predicate}")),
            Err(error) if !error.contains("type_mismatch") => {
                admitted.push(format!(
                    "{written}: refused, but not as a type mismatch: {error}"
                ));
            }
            Err(_) => {}
        }
    }
    assert_eq!(admitted, Vec::<String>::new());
}

/// A Timestamp magnitude follows the current-time bound: at `CurrentTime::MAX_OFFSET_SECONDS`
/// (3155760000 seconds) in each unit it is admitted, one unit past it refused.
#[test]
fn adv_e_u2_timestamp_magnitude_at_the_current_time_bound() {
    for (written, seconds, unit) in [
        (
            "renewed_at <= window.opened_at + 3155760000s",
            3_155_760_000,
            ElapsedUnit::Seconds,
        ),
        (
            "renewed_at <= window.opened_at + 52596000m",
            3_155_760_000,
            ElapsedUnit::Minutes,
        ),
        (
            "renewed_at <= window.opened_at + 876600h",
            3_155_760_000,
            ElapsedUnit::Hours,
        ),
    ] {
        let want = compare(
            "renewed_at",
            CompareOp::Le,
            Operand::Offset(OffsetOperand {
                base: path("window.opened_at"),
                direction: OffsetDirection::Add,
                magnitude: OffsetMagnitude::ElapsedSeconds {
                    seconds,
                    written_unit: unit,
                },
            }),
        );
        assert_eq!(
            guard(&model(written)).unwrap_or_else(|error| panic!("{written}: {error}")),
            want,
            "{written}"
        );
    }
    for written in [
        "renewed_at <= window.opened_at + 3155760001s",
        "renewed_at <= window.opened_at + 52596001m",
        "renewed_at <= window.opened_at + 876601h",
        "renewed_at <= window.opened_at + 03h",
        "renewed_at <= window.opened_at + 1d",
    ] {
        assert!(guard(&model(written)).is_err(), "{written} is refused");
    }
}

/// Rule 3a's order puts an enum variant before an offset: a variant spelled like an offset of a
/// declared Integer stays the variant. Variant names carry no pattern, so `lower-1` and `lower + 1`
/// are admissible variants.
#[test]
fn adv_e_u2_enum_variants_spelled_like_offsets_stay_variants() {
    let text = |when: &str| {
        model(when)
            .replace(
                "types:\n",
                "types:\n  - {name: pool.lease.Mark, kind: enum, variants: [lower-1, 'lower + 1', Open]}\n",
            )
            .replace(
                "      - {name: lower, type: Integer}\n",
                "      - {name: lower, type: Integer}\n      - {name: mark, type: pool.lease.Mark}\n",
            )
    };
    for written in ["mark == lower-1", "mark == lower + 1"] {
        let got = guard(&text(written)).unwrap_or_else(|error| panic!("{written}: {error}"));
        let Predicate::Compare { right, .. } = &got else {
            panic!("{written}: {got}");
        };
        assert!(
            matches!(right, Operand::Literal(_)),
            "{written} stays the variant: {got}"
        );
    }
}
