//! Adversary pass 2, story:a-rust-runner-compares-a-decimal-by-exact-value.

use ess_primitives::facts::Number;
use ess_primitives::node::Node;

/// `from_json_token` says "a zero is zero whatever its exponent, and is read at once" and that
/// the exponent parse saturates so no token stalls. A zero with two or more fraction zeroes and an
/// exponent below `-i64::MAX` makes `Dec::of_parts` subtract past `i64::MIN`: a panic in a debug
/// build (overflow checks), where Go and TypeScript refuse the token.
#[test]
fn adv_t3p2_a_zero_with_a_saturated_negative_exponent_reads_as_zero() {
    let mut wrong = Vec::new();
    for token in [
        "0.00e-99999999999999999999",
        "-0.00e-9223372036854775807",
        "0.000000e-9223372036854775808",
    ] {
        match std::panic::catch_unwind(|| Number::from_json_token(token)) {
            Ok(read) if read == Some(Number::from(0_i64)) => {}
            Ok(read) => wrong.push(format!("{token}: {read:?}")),
            Err(_) => wrong.push(format!("{token}: from_json_token panicked")),
        }
        if std::panic::catch_unwind(|| Node::from_json_exact(token)).is_err() {
            wrong.push(format!("{token}: Node::from_json_exact panicked"));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// An integer token spelt with a fraction or an exponent, between 2^53 and 2^63, is `Observed`;
/// it is equal to the `Exact` integer, yet `as_i64` / `is_integral` answer `None` / `false` for it
/// only. `as_i64` is documented as "the exact integer, when this value is one that fits an i64",
/// and `Primitive::Integer` admits by `is_integral`, so a retained `Integer` field reported as
/// `9007199254740993.0` is refused under Rust while Go (`big.Rat` `IsInt && IsInt64`) and
/// TypeScript (`exactInteger`) admit it.
#[test]
fn adv_t3p2_an_observed_integer_answers_as_its_equal_exact_integer() {
    let mut wrong = Vec::new();
    for (token, integer) in [
        ("9007199254740993.0", 9_007_199_254_740_993_i64),
        ("9.007199254740993e15", 9_007_199_254_740_993),
        ("1.152921504606847e18", 1_152_921_504_606_847_000),
    ] {
        let read = Number::from_json_token(token).unwrap();
        let exact = Number::from(integer);
        if read != exact {
            wrong.push(format!("{token}: not equal to {integer}"));
        }
        if read.as_i64() != Some(integer) || !read.is_integral() {
            wrong.push(format!(
                "{token}: equal to {integer}, but as_i64 = {:?}, is_integral = {}",
                read.as_i64(),
                read.is_integral()
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Pin: the order is total and value-consistent across exact, observed and binary64-only numbers
/// at the new sizes: the binary64 magnitude bound, `2^127`, `2^53` and the subnormals.
#[test]
fn adv_t3p2_order_holds_at_the_binary64_edges() {
    let ascending = [
        "-1.79769313486231580793e308",
        "-1.7976931348623157e308",
        "-1e300",
        "-170141183460469231731687303715884105728",
        "-1.7014118346046923e38",
        "-9007199254740993",
        "-9007199254740992.5",
        "-9007199254740992",
        "-5e-324",
        "-4.9406564584124654e-324",
        "-1e-400",
        "0",
        "1e-400",
        "4.9406564584124654e-324",
        "5e-324",
        "2.2250738585072014e-308",
        "1e-256",
        "1e-255",
        "9007199254740992",
        "9007199254740992.5",
        "9007199254740993",
        "9.007199254740994e15",
        "1.7014118346046923e38",
        "170141183460469231731687303715884105727",
        "170141183460469231731687303715884105728",
        "1.7014118346046927e38",
        "1e300",
        "1.7976931348623157e308",
        "1.79769313486231580793e308",
    ]
    .map(|token| (token, Number::from_json_token(token).expect(token)));
    let mut wrong = Vec::new();
    for (i, (left_token, left)) in ascending.iter().enumerate() {
        for (j, (right_token, right)) in ascending.iter().enumerate() {
            if left.cmp(right) != i.cmp(&j) {
                wrong.push(format!(
                    "{left_token} against {right_token}: {:?}",
                    left.cmp(right)
                ));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
    assert!(Number::from_json_token("1.79769313486231581e308").is_none());
}

/// Pass 3. `as_i64` / `is_integral` at both ends of `i64`, one past each, negative and zero, for
/// tokens a target may spell with a fraction or an exponent: the answers Go (`big.Rat` `IsInt &&
/// IsInt64`) and TypeScript (`exactInteger`) give, and `==` with the `Exact` integer.
#[test]
fn adv_t3p3_observed_integers_at_the_i64_edges_answer_as_go_and_typescript() {
    let cases: [(&str, Option<i64>); 20] = [
        ("9223372036854775807.0", Some(i64::MAX)),
        ("9223372036854775807e0", Some(i64::MAX)),
        ("922337203685477580.7e1", Some(i64::MAX)),
        ("0.9223372036854775807e19", Some(i64::MAX)),
        ("9223372036854775808.0", None),
        ("9223372036854775806.5", None),
        ("-9223372036854775808.0", Some(i64::MIN)),
        ("-9223372036854775808e0", Some(i64::MIN)),
        ("-9223372036854775807.0", Some(i64::MIN + 1)),
        ("-9223372036854775809.0", None),
        ("-9007199254740993.0", Some(-9_007_199_254_740_993)),
        ("-9.007199254740993e15", Some(-9_007_199_254_740_993)),
        ("-0.0", Some(0)),
        ("-0e5", Some(0)),
        ("1e18", Some(1_000_000_000_000_000_000)),
        ("1000000000000000000.5", None),
        ("1.0000000000000000001e18", None),
        ("10000000000000000000.0", None),
        ("1e-400", None),
        ("-1e-400", None),
    ];
    let mut wrong = Vec::new();
    for (token, expected) in cases {
        let Some(read) = Number::from_json_token(token) else {
            wrong.push(format!("{token}: refused"));
            continue;
        };
        if read.as_i64() != expected || read.is_integral() != expected.is_some() {
            wrong.push(format!(
                "{token}: as_i64 = {:?}, is_integral = {}, expected {expected:?}",
                read.as_i64(),
                read.is_integral()
            ));
        }
        if let Some(integer) = expected {
            if read != Number::from(integer) {
                wrong.push(format!("{token}: not equal to {integer}"));
            }
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Pass 3. The admission half of the round-trip law (`docs/design/review-primitive-semantics.md`,
/// "the one with teeth"): `is_integral(read(write(n))) == is_integral(n)`. Since pass 2 an
/// `Observed` `9223372036854775807.0` is `is_integral` (admitted as a `Primitive::Integer`), and
/// it still writes its binary64, `2^63`, which reads back as `2^63`: not an `i64`, refused. The
/// failure the `Deserialize` doc names ("admitted, written, and then refused").
#[test]
fn adv_t3p3_an_admitted_observed_integer_is_still_admitted_after_its_write() {
    let mut wrong = Vec::new();
    for token in [
        "9223372036854775807.0",
        "9223372036854775807e0",
        "0.9223372036854775807e19",
    ] {
        let read = Number::from_json_token(token).unwrap();
        let written = serde_json::to_string(&read).unwrap();
        let back: Number = serde_json::from_str(&written).unwrap();
        if back.is_integral() != read.is_integral() {
            wrong.push(format!(
                "{token}: is_integral {} (as_i64 {:?}), written {written}, read back is_integral {}",
                read.is_integral(),
                read.as_i64(),
                back.is_integral()
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}

/// Pass 3. The value half for the integer row of the round-trip law: "an integer binary64 does
/// not carry (`i64::MAX`, `2^53 + 1`) | the integer token | the same integer". An `Observed`
/// integer now answers `as_i64` as that integer, and writes a different one.
#[test]
fn adv_t3p3_an_observed_integer_writes_the_integer_it_answers() {
    let mut wrong = Vec::new();
    for token in [
        "9007199254740993.0",
        "-9007199254740993.0",
        "9223372036854775807.0",
    ] {
        let read = Number::from_json_token(token).unwrap();
        let written = serde_json::to_string(&read).unwrap();
        let back: Number = serde_json::from_str(&written).unwrap();
        if back != read || back.as_i64() != read.as_i64() {
            wrong.push(format!(
                "{token}: as_i64 {:?}, written {written}, read back as_i64 {:?}",
                read.as_i64(),
                back.as_i64()
            ));
        }
    }
    assert!(wrong.is_empty(), "{wrong:#?}");
}
