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
