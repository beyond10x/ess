//! Adversarial cases against `src/uniqueness.rs` and its class claim.
//!
//! The unit's own contract document is the module header of `src/uniqueness.rs`:
//!
//! > Every validator this crate compiles starts here; nothing else calls `jsonschema::options()`.
//!
//! and the case `every_validator_this_crate_builds_comes_from_the_checked_options` in
//! `tests/schema_unique_items.rs` is what is supposed to hold that claim. These cases drive the
//! implementation from that document: they apply the guard's own predicate to the parts of the
//! crate the guard's filters exclude, and they show what the excluded validators do.

use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use jsonschema::json::cmp;
use schema_contract::bundle::{self, Bundle, Dialect, SCHEMA_DIALECT};
use serde_json::{json, Value};

/// The needle the shipped guard greps for, spelled so it is never a literal match on this file.
fn needle() -> String {
    format!("jsonschema{}options()", "::")
}

/// The guard's own predicate, copied from `schema_unique_items.rs` so this file attacks the rule
/// rather than a paraphrase of it.
fn builds_a_validator(path: &str) -> bool {
    const CONSTRUCTIONS: [&str; 9] = [
        "options",
        "options_for",
        "async_options",
        "async_options_for",
        "validator_for",
        "validator_map_for",
        "is_valid",
        "validate",
        "evaluate",
    ];
    path.split("::")
        .skip(1)
        .any(|segment| CONSTRUCTIONS.contains(&segment))
}

fn crate_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_owned()
}

/// Every file under `root` whose text contains the guard's needle, relative to `root`.
///
/// This is the shipped guard's body with its `extension == "rs"` filter removed; `keep` stands in
/// for the directory it scans.
fn sites(root: &Path, keep: impl Fn(&Path) -> bool) -> Vec<PathBuf> {
    let mut found = Vec::new();
    let mut pending = vec![root.to_owned()];
    while let Some(directory) = pending.pop() {
        for entry in std::fs::read_dir(&directory).unwrap() {
            let path = entry.unwrap().path();
            if path.is_dir() {
                pending.push(path);
            } else if keep(&path) {
                let Ok(text) = std::fs::read_to_string(&path) else {
                    continue;
                };
                if text.contains(&needle()) {
                    found.push(path.strip_prefix(root).unwrap().to_owned());
                }
            }
        }
    }
    found.sort();
    found
}

/// A document-rooted bundle whose root is an array asserting `uniqueItems`.
fn array_bundle() -> Bundle {
    let source = json!({"$schema": SCHEMA_DIALECT, "type": "array", "uniqueItems": true});
    let bytes = format!("{}\n", serde_json::to_string_pretty(&source).unwrap());
    bundle::import_document(&bytes, "Root", &BTreeSet::new(), Dialect::Draft202012).unwrap()
}

/// `items` followed by however many distinct fillers reach `length`.
fn padded(items: &[Value], length: usize) -> Value {
    let mut values = items.to_vec();
    let mut filler = 1_000_i64;
    while values.len() < length {
        values.push(json!(filler));
        filler += 1;
    }
    Value::Array(values)
}

// --- the class claim -------------------------------------------------------------------------

/// The guard only reads files whose extension is exactly `rs`, so the validator this crate emits
/// into a consumer's normalization runtime — `src/realize/normalize/rust_runtime.rs.txt` — is
/// invisible to it, although it sits inside the directory the guard walks.
#[test]
fn every_options_call_under_src_comes_from_the_checked_options_whatever_the_file_is_named() {
    let src = crate_dir().join("src");
    let found = sites(&src, |path| {
        matches!(
            path.extension().and_then(std::ffi::OsStr::to_str),
            Some("rs" | "txt")
        )
    });
    assert_eq!(
        found,
        vec![PathBuf::from("uniqueness.rs")],
        "the module header of src/uniqueness.rs says nothing else calls the dependency's own \
         options(); these files under src/ do, and the shipped guard's `extension == \"rs\"` \
         filter does not read them"
    );
}

/// The guard only walks `src`, so the crate's own tests build validators the fix never reached.
/// `tests/bundle.rs` uses one as the oracle a bundle's verdict is compared against.
#[test]
fn the_crates_own_tests_build_no_validator_outside_the_checked_options() {
    let tests = crate_dir().join("tests");
    let found = sites(&tests, |path| {
        // This file is excluded: it quotes the call deliberately, in the case below.
        path.extension().and_then(std::ffi::OsStr::to_str) == Some("rs")
            && path.file_name().and_then(std::ffi::OsStr::to_str)
                != Some("schema_unique_items_adversary.rs")
    });
    assert_eq!(
        found,
        Vec::<PathBuf>::new(),
        "the shipped guard walks src/ only, so these test validators decide uniqueItems by the \
         dependency's length-dependent algorithm while production no longer does"
    );
}

/// A future `src/` site spelled through the draft module builds a default validator and is not a
/// substring match for the guard's needle, so the guard stays green.
#[test]
fn the_guards_predicate_sees_an_options_call_spelled_through_the_draft_module() {
    // Rewritten by the wave-23 coordinator. As written this case could not pass for any state of
    // the crate: it asked whether one constant in this file is a substring of another, and it is
    // not. That was an argument written as an assertion, and the argument was right — the guard did
    // match one literal spelling, and four other entry points escaped it.
    //
    // The guard now splits a `jsonschema::` path and asks whether any segment names a construction,
    // so what this case can honestly hold is the property that replaced the literal: every spelling
    // the argument named is recognised, and a path that builds nothing is not.
    let recognised = [
        "jsonschema::draft202012::options",
        "jsonschema::validator_for",
        "jsonschema::is_valid",
        "jsonschema::options_for",
        "jsonschema::async_options",
        "jsonschema::validator_map_for",
        "jsonschema::evaluate",
    ];
    let missed: Vec<&str> = recognised
        .into_iter()
        .filter(|path| !builds_a_validator(path))
        .collect();
    assert!(
        missed.is_empty(),
        "each of these builds a validator carrying the dependency's own length-dependent \
         uniqueItems, and the guard has to recognise every one of them, not one spelling: {missed:?}"
    );
    // The other direction, so the predicate is not simply true of everything.
    for benign in [
        "jsonschema::Draft",
        "jsonschema::ValidationError",
        "jsonschema::paths",
    ] {
        assert!(
            !builds_a_validator(benign),
            "`{benign}` compiles no validator and the guard must not claim it does"
        );
    }
}

// --- what the excluded validators do ----------------------------------------------------------

/// The reference and the runtime this crate emits must agree about the same schema and instance.
///
/// The validator is built exactly as `src/realize/normalize/rust_runtime.rs.txt:72-75` builds it,
/// for a schema `realize.rs:571` admits with the obligation "validate this constraint against the
/// source schema at runtime".
#[test]
fn the_dependencys_own_construction_is_why_this_crate_ships_a_keyword() {
    // Rewritten by the wave-23 coordinator. As written, this case built a validator with the
    // dependency's own `jsonschema::options()` and asserted it refuses — a claim about the
    // dependency, which nothing in this crate can satisfy. Its premise was that the emitted Rust
    // runtime builds its validator that way, and that is no longer true: the runtime now carries
    // this crate's own keyword, which `the_emitted_rust_runtime_carries_this_crates_own_keyword_verbatim`
    // holds byte for byte.
    //
    // What is worth pinning instead is the reason the keyword exists at all, and the condition that
    // would end it. Measured on the pinned dependency: `=0.52.1` accepts this instance, `0.53.0`
    // refuses it. So this case goes red the day the pin moves, and the message says what to do then.
    let schema = json!({"type": "array", "uniqueItems": true});
    let instance = padded(&[json!(0), json!(-0.0)], 16);

    let reference = array_bundle();
    assert!(
        !reference.validate("Root", &instance).unwrap().is_empty(),
        "the reference refuses {instance}"
    );

    let dependency = jsonschema::options()
        .with_draft(jsonschema::Draft::Draft202012)
        .should_validate_formats(false)
        .build(&schema)
        .unwrap();
    assert!(
        dependency.validate(&instance).is_ok(),
        "the pinned dependency now refuses {instance} on its own, so the keyword in \
         `src/realize/normalize/unique_items.rs` is no longer the thing standing between this \
         crate and the length-dependent algorithm; check whether it can be retired and whether the \
         emitted runtimes should go back to the dependency's construction"
    );
}

// --- the bucketing key -------------------------------------------------------------------------

/// Every numeric spelling this corpus can reach, parsed the way a document reaches the crate.
const NUMBERS: [&str; 36] = [
    "0",
    "-0",
    "0.0",
    "-0.0",
    "0e0",
    "-0e-0",
    "1",
    "1.0",
    "-1",
    "-1.0",
    "9007199254740992",
    "9007199254740993",
    "9007199254740992.0",
    "9007199254740994",
    "18446744073709551615",
    "18446744073709551616",
    "1e19",
    "10000000000000000000",
    "-9223372036854775808",
    "-9223372036854775808.0",
    "-9223372036854775809",
    "1e308",
    "1e-308",
    "0.1",
    "0.30000000000000004",
    "1.7976931348623157e308",
    "170141183460469231731687303715884105728",
    "1.7014118346046923e38",
    "-170141183460469231731687303715884105728",
    "340282366920938463463374607431768211456",
    "5e-324",
    "-5e-324",
    "2.5",
    "-2.5",
    "1e15",
    "1000000000000000",
];

/// The keyword's stated contract: the key only groups candidates, so the verdict for any pair is
/// exactly the dependency's own `cmp::equal`, at every array length.
///
/// A pair that is equal but keys apart is never compared and the array passes — a false pass, and
/// the one-way danger the key carries.
#[test]
fn the_bucketing_key_never_decides_a_pair_the_dependency_calls_equal() {
    let bundle = array_bundle();
    let values: Vec<Value> = NUMBERS
        .iter()
        .map(|text| serde_json::from_str(text).unwrap())
        .collect();
    for (i, left) in values.iter().enumerate() {
        for right in values.iter().skip(i + 1) {
            let expected = cmp::equal(left, right);
            for length in [2_usize, 15, 16, 17, 64] {
                let instance = padded(&[left.clone(), right.clone()], length);
                let refused = !bundle.validate("Root", &instance).unwrap().is_empty();
                assert_eq!(
                    refused,
                    expected,
                    "{} vs {} at length {length}: cmp::equal says {expected}",
                    NUMBERS[i],
                    serde_json::to_string(right).unwrap()
                );
            }
        }
    }
}

/// The same question one level down: a key that equal values share must survive nesting.
#[test]
fn nesting_does_not_separate_values_the_dependency_calls_equal() {
    let bundle = array_bundle();
    for (left, right) in [
        (json!([[[0]]]), json!([[[-0.0]]])),
        (
            json!({"a": {"b": [1e19]}}),
            json!({"a": {"b": [10_000_000_000_000_000_000_u64]}}),
        ),
        (json!({"a": 1, "b": 2.0}), json!({"b": 2, "a": 1.0})),
        (json!([{"k": -0.0}, 1]), json!([{"k": 0}, 1.0])),
        (json!({"-0.0": 0}), json!({"-0.0": -0.0})),
    ] {
        assert!(cmp::equal(&left, &right), "{left} {right}");
        for length in [2_usize, 16, 17] {
            let instance = padded(&[left.clone(), right.clone()], length);
            assert!(
                !bundle.validate("Root", &instance).unwrap().is_empty(),
                "{instance}"
            );
        }
    }
}

/// Object keys are strings; `-0.0` and `0` spell different members and stay distinct.
#[test]
fn signed_zero_as_an_object_key_is_a_string_and_stays_distinct() {
    let bundle = array_bundle();
    for length in [2_usize, 16] {
        let instance = padded(&[json!({"0": 1}), json!({"-0.0": 1})], length);
        assert!(
            bundle.validate("Root", &instance).unwrap().is_empty(),
            "{instance}"
        );
    }
}

/// The failing array's pointer is the array's, not the document root's, however deep it sits.
#[test]
fn the_refusal_points_at_the_nested_array_that_failed() {
    let source = json!({"$schema": SCHEMA_DIALECT, "type": "array",
        "items": {"type": "array", "uniqueItems": true}});
    let bytes = format!("{}\n", serde_json::to_string_pretty(&source).unwrap());
    let bundle =
        bundle::import_document(&bytes, "Root", &BTreeSet::new(), Dialect::Draft202012).unwrap();
    for length in [2_usize, 16] {
        let inner = padded(&[json!(0), json!(-0.0)], length);
        let found = bundle.validate("Root", &json!([[], inner])).unwrap();
        assert_eq!(found.len(), 1);
        assert_eq!(found[0].pointer, "/1");
    }
}

/// The answer to "what does the keyword do with two NaNs": `serde_json` cannot build the question.
#[test]
fn nan_and_infinity_are_not_reachable_json_values() {
    assert!(json!(f64::NAN).is_null());
    assert!(json!(f64::INFINITY).is_null());
    assert!(serde_json::from_str::<Value>("[NaN,NaN]").is_err());
    // A number past binary64 cannot reach the keyword either, because this crate's document reader
    // refuses it. serde_json alone does not decide that: with its `arbitrary_precision` feature
    // unified into the build (`entity-core` enables it), `serde_json::Value` holds `1e400`.
    let document = |items: &str| {
        format!(
            r#"{{"$schema":"{SCHEMA_DIALECT}","type":"array","uniqueItems":true,"examples":[{items}]}}"#
        )
    };
    bundle::import_document(
        &document("[1e300, 1e300]"),
        "Root",
        &BTreeSet::new(),
        Dialect::Draft202012,
    )
    .expect("a finite example imports");
    for items in ["[1e400]", "[1e400, 1e400]", "[-1e400]"] {
        let refused = bundle::import_document(
            &document(items),
            "Root",
            &BTreeSet::new(),
            Dialect::Draft202012,
        )
        .expect_err(items);
        assert!(
            format!("{refused:?}").contains("out of range"),
            "{items}: {refused:?}"
        );
    }
}

// --- the artifact this crate actually ships -----------------------------------------------------

#[path = "fixtures/normalization_v1.rs"]
#[allow(dead_code)]
mod fixture_v1;

/// The emitted Rust normalization target is a validator this crate builds, in a consumer's process.
///
/// `realize.rs:571` admits `uniqueItems` with the obligation "validate this constraint against the
/// source schema at runtime", and this is the runtime that discharges it.
#[test]
fn the_shipped_rust_normalization_target_builds_no_unchecked_validator() {
    let realization = fixture_v1::plan().rust("adversary-probe").unwrap();
    let carrying: Vec<&String> = realization
        .files
        .iter()
        .filter(|(_, text)| text.contains(&needle()))
        .map(|(path, _)| path)
        .collect();
    assert!(
        carrying.is_empty(),
        "the generated crate this target ships decides uniqueItems by the dependency's \
         length-dependent algorithm, which src/uniqueness.rs exists to replace: {carrying:?}"
    );
}
