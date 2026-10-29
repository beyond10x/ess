//! Adversarial cases against the invariant lowering of open-issue batch 1, unit U1
//! (beyond10x/ess#685, #686, #687, #688).
//!
//! Each case compares a published schema with the model's own verdict on the same value. The model
//! is the predicate the compiler resolved, evaluated by `ess_primitives` over the facts the value
//! binds; a value the model does not refuse (`True` or `Unknown`, which is how the lowering's own
//! documentation reads an absent key or a `null` element) must be admitted by the schema, and where
//! a comparison is lowered exactly, a value the model refuses must be refused by it. The validator
//! is the one the crate's other schema tests use.

use ess_compiler::ir::ResolvedBody;
use ess_compiler::resolve::compile;
use ess_compiler::source::SourceMap;
use ess_compiler::EssIr;
use ess_domain::spec::{RawSpecFile, Specification};
use ess_domain::system::Source;
use ess_gen::artifact::run;
use ess_gen::schema::JsonSchema;
use ess_primitives::predicate::{Predicate, Truth};
use ess_primitives::{FactStore, FactValue, Number};
use serde_json::{json, Value};

const MODEL: &str = "format: ess/22
system: demo
version: v1
domain: demo.probe
types:
  - name: demo.probe.Maybe
    kind: newtype
    of: Optional<String>
  - name: demo.probe.Held
    kind: struct
    fields:
      - {name: tags, type: List<demo.probe.Maybe>}
    invariants:
      - {distinct: {in: tags, as: tag}}
  - name: demo.probe.Large
    kind: struct
    fields:
      - {name: big, type: Optional<Binary64>}
      - {name: pinned, type: Optional<Binary64>}
      - {name: floor, type: Optional<Binary64>}
    invariants:
      - big <= 9007199254740992
      - pinned == 9007199254740992
      - floor >= -9007199254740992
  - name: demo.probe.Floats
    kind: struct
    fields:
FLOAT_FIELDS
    invariants:
FLOAT_INVARIANTS
      - {any: ['m0 > 5', 'm0 < 0']}
      - not m1 > 3
      - {all: ['m2 > 1', {any: ['m2 < 0', 'm2 > 2']}]}
      - {all: ['m3 >= 1', 'm3 <= 2']}
  - name: demo.probe.Texts
    kind: struct
    fields:
TEXT_FIELDS
    invariants:
      - s0.count > 2
      - s1.count < 2
      - s2.count >= 0
      - s3.count < 0
      - s4.count == 3
      - s5.count >= 5
      - s5.count <= 3
      - s6.count > -1
      - s7.count <= 0
      - s8.count: {gte: 2, lte: 3}
      - {all: ['s9.count > 1', {any: ['s9.count < 1', 's9.count > 3']}]}
      - not s10.count > 2
      - s11.count == -1
  - name: demo.probe.Integers
    kind: struct
    fields:
INTEGER_FIELDS
    invariants:
      - i0 > 0
      - i1 < 0
      - i2 > -1
      - i3 < 9223372036854775807
      - i4 > -9223372036854775808
      - i5 > 9223372036854775807
      - i6 < -9223372036854775808
      - i7 == 5
      - i8 >= 3
      - i8 <= 1
";

/// The `Binary64` literals each operator is tried with: zero of both signs, a fraction with no
/// exact binary64, a subnormal, the largest finite value, an integer past every `i64`.
const LITERALS: [&str; 11] = [
    "0",
    "-0.0",
    "0.1",
    "1",
    "-2.5",
    "100.5",
    "5e-324",
    "1e20",
    "1.7976931348623157e308",
    "0.30000000000000004",
    "-1e-300",
];

const OPERATORS: [&str; 5] = [">", ">=", "<", "<=", "=="];

/// The text fields whose bound is lowered exactly; the rest are checked for soundness only.
const EXACT_TEXTS: [&str; 8] = ["s0", "s1", "s2", "s4", "s5", "s6", "s7", "s8"];

/// The integer fields whose bound is lowered exactly.
const EXACT_INTEGERS: [&str; 7] = ["i0", "i1", "i2", "i3", "i4", "i7", "i8"];

fn source() -> String {
    let optional = |prefix: &str, count: usize, of: &str| {
        (0..count)
            .map(|index| format!("      - {{name: {prefix}{index}, type: Optional<{of}>}}"))
            .collect::<Vec<_>>()
            .join("\n")
    };
    let mut float_fields = optional("b", LITERALS.len() * OPERATORS.len(), "Binary64");
    float_fields.push('\n');
    float_fields.push_str(&optional("m", 4, "Binary64"));
    let float_invariants = float_cases()
        .iter()
        .map(|(field, op, literal)| format!("      - {field} {op} {literal}"))
        .collect::<Vec<_>>()
        .join("\n");
    MODEL
        .replace("FLOAT_FIELDS", &float_fields)
        .replace("FLOAT_INVARIANTS", &float_invariants)
        .replace("TEXT_FIELDS", &optional("s", 12, "String"))
        .replace("INTEGER_FIELDS", &optional("i", 9, "Integer"))
}

/// `(field, operator, literal)` for every simple `Binary64` comparison.
fn float_cases() -> Vec<(String, &'static str, &'static str)> {
    let mut cases = Vec::new();
    for (at, literal) in LITERALS.iter().enumerate() {
        for (by, op) in OPERATORS.iter().enumerate() {
            cases.push((format!("b{}", at * OPERATORS.len() + by), *op, *literal));
        }
    }
    cases
}

fn compiled() -> EssIr {
    let text = source();
    let raw = RawSpecFile::parse(&text).unwrap_or_else(|error| panic!("well formed: {error}"));
    let spec = Specification::assemble([(Source::new("probe.yaml"), raw)])
        .unwrap_or_else(|errors| panic!("validates: {errors}"));
    let mut sources = SourceMap::new();
    sources.insert("probe.yaml", &text);
    compile(&spec, &sources).unwrap_or_else(|errors| panic!("compiles: {errors}"))
}

fn schema_of(ir: &EssIr, name: &str) -> Value {
    let artifacts = run(&JsonSchema, ir).expect("generates");
    let path = format!("schema/types/{name}.schema.json");
    let artifact = artifacts
        .get(&path)
        .unwrap_or_else(|| panic!("no `{path}`"));
    serde_json::from_str(&artifact.contents).expect("JSON")
}

/// The invariants the compiler resolved for the struct `name`.
fn invariants_of(ir: &EssIr, name: &str) -> Vec<Predicate> {
    let declared = ir
        .types()
        .values()
        .find(|declared| declared.name.to_string() == name)
        .unwrap_or_else(|| panic!("no type `{name}`"));
    match &declared.body {
        ResolvedBody::Struct { invariants, .. } => invariants
            .iter()
            .map(|invariant| invariant.predicate.clone())
            .collect(),
        other => panic!("`{name}` is not a struct: {other:?}"),
    }
}

/// Whether the model refuses the value `facts` bind: some invariant is `False`.
fn model_refuses(invariants: &[Predicate], facts: &FactStore) -> bool {
    invariants
        .iter()
        .any(|predicate| predicate.evaluate(facts) == Truth::False)
}

fn one(path: &str, value: impl Into<FactValue>) -> FactStore {
    let mut facts = FactStore::new();
    facts.set_path(path, value);
    facts
}

/// The next binary64 towards positive infinity, `f64::next_up` before Rust 1.86.
fn up(value: f64) -> f64 {
    if value == 0.0 {
        f64::from_bits(1)
    } else if value > 0.0 {
        f64::from_bits(value.to_bits() + 1)
    } else {
        f64::from_bits(value.to_bits() - 1)
    }
}

/// The next binary64 towards negative infinity.
fn down(value: f64) -> f64 {
    -up(-value)
}

/// A list `distinct:` over a newtype of `Optional<String>`: a `null` element is unknown to the
/// model, exactly as an `Optional` element is, and the lowering's own rule leaves an `Optional`
/// element alone for that reason. Through a newtype it publishes `uniqueItems`, which refuses two
/// `null`s the model does not refuse.
#[test]
fn a_distinct_list_of_newtype_wrapped_optionals_admits_two_nulls() {
    let ir = compiled();
    let schema = schema_of(&ir, "demo.probe.Held");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let invariants = invariants_of(&ir, "demo.probe.Held");

    // Controls: one `null` is a value of the type, and a known duplicate is refused by both.
    assert!(
        validator.is_valid(&json!({"tags": [null]})),
        "the element type admits null: {schema:#}"
    );
    let mut duplicate = one("tags.count", FactValue::count(2));
    duplicate.set_path("tags.0", "a");
    duplicate.set_path("tags.1", "a");
    assert!(model_refuses(&invariants, &duplicate), "two known `a`s");
    assert!(!validator.is_valid(&json!({"tags": ["a", "a"]})));

    let nulls = one("tags.count", FactValue::count(2));
    assert_eq!(
        invariants[0].evaluate(&nulls),
        Truth::Unknown,
        "the model reads two null elements as unknown keys"
    );
    assert!(
        validator.is_valid(&json!({"tags": [null, null]})),
        "the schema refuses `[null, null]`, which the model does not refuse: {}",
        schema["$defs"]["demo.probe.Held"]
    );
}

/// A `Binary64` bound at or past `2^53` is published as an integer token, and a validator compares
/// an integer token in the payload exactly. The model decodes the payload to the nearest binary64
/// first (`docs/design/model-binary64.md`, wire form), so `9007199254740993` is `2^53` to it and
/// meets `<= 2^53`; the schema refuses it.
#[test]
fn a_binary64_bound_past_two_to_the_53_refuses_no_payload_the_model_admits() {
    let ir = compiled();
    let schema = schema_of(&ir, "demo.probe.Large");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let invariants = invariants_of(&ir, "demo.probe.Large");

    let mut admitted_but_refused = Vec::new();
    for (field, token) in [
        ("big", "9007199254740993"),
        ("pinned", "9007199254740993"),
        ("floor", "-9007199254740993"),
    ] {
        let decoded: f64 = token.parse().expect("a number token");
        let facts = one(field, Number::new(decoded).expect("finite"));
        let instance: Value =
            serde_json::from_str(&format!("{{\"{field}\": {token}}}")).expect("JSON");
        if !model_refuses(&invariants, &facts) && !validator.is_valid(&instance) {
            admitted_but_refused.push(format!("{instance} (decoded {decoded})"));
        }
    }
    assert!(
        admitted_but_refused.is_empty(),
        "the model admits and the schema refuses: {admitted_but_refused:?}; {}",
        schema["$defs"]["demo.probe.Large"]["properties"]
    );
}

/// Every lowered `Binary64` comparison agrees with the model on the literal and its two binary64
/// neighbours, on both zeros and on a few ordinary values; a disjunction, a negation or an `all`
/// holding one is never refused where the model admits.
#[test]
fn every_binary64_bound_agrees_with_the_model_at_its_neighbours() {
    let ir = compiled();
    let schema = schema_of(&ir, "demo.probe.Floats");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let invariants = invariants_of(&ir, "demo.probe.Floats");

    let mut disagreements = Vec::new();
    let mut refusals = 0_usize;
    let mut check = |field: &str, value: f64, exact: bool| {
        if !value.is_finite() {
            return;
        }
        let refused = model_refuses(
            &invariants,
            &one(field, Number::new(value).expect("finite")),
        );
        let admitted = validator.is_valid(&json!({ field: value }));
        refusals += usize::from(refused && !admitted);
        if (!refused && !admitted) || (exact && refused && admitted) {
            disagreements.push(format!(
                "{field} = {value:e}: model refuses {refused}, schema admits {admitted}"
            ));
        }
    };
    for (field, _, literal) in float_cases() {
        let at: f64 = literal.parse().expect("a literal");
        for value in [at, up(at), down(at), 0.0, -0.0, 1.0, -1.0, 2.0, 1e300] {
            check(&field, value, true);
        }
    }
    for field in ["m0", "m1", "m2", "m3"] {
        for value in [-1.0, -0.0, 0.0, 0.5, 1.0, 1.5, 2.0, 2.5, 3.0, 3.5, 5.0, 6.0] {
            check(field, value, false);
        }
    }
    assert!(
        refusals > 0,
        "nothing was refused by both: the facts bind nothing"
    );
    assert!(
        disagreements.is_empty(),
        "{disagreements:#?}\n{}",
        schema["$defs"]["demo.probe.Floats"]["properties"]
    );
}

/// Text lengths count Unicode scalar values on both sides, astral characters, combining marks and
/// joined emoji included; strict bounds are one step in, a bound every or no text meets publishes
/// nothing wrong, a contradictory pair refuses everything, and a disjunction or a negation is never
/// refused where the model admits.
#[test]
fn every_text_length_bound_agrees_with_the_model() {
    let ir = compiled();
    let schema = schema_of(&ir, "demo.probe.Texts");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let invariants = invariants_of(&ir, "demo.probe.Texts");
    let texts = [
        "",
        "a",
        "ab",
        "abc",
        "abcd",
        "abcde",
        "e\u{301}",
        "e\u{301}e\u{301}",
        "\u{1F600}",
        "\u{1F600}\u{1F600}\u{1F600}",
        "\u{1F468}\u{200D}\u{1F469}\u{200D}\u{1F467}",
        "\u{10FFFF}\u{FEFF}",
    ];

    let mut disagreements = Vec::new();
    let mut refusals = 0_usize;
    for index in 0..12 {
        let field = format!("s{index}");
        let exact = EXACT_TEXTS.contains(&field.as_str());
        for text in texts {
            let refused = model_refuses(&invariants, &one(&field, text));
            let admitted = validator.is_valid(&json!({ field.as_str(): text }));
            refusals += usize::from(refused && !admitted);
            if (!refused && !admitted) || (exact && refused && admitted) {
                disagreements.push(format!(
                    "{field} = {text:?} ({} scalars): model refuses {refused}, schema admits \
                     {admitted}",
                    text.chars().count()
                ));
            }
        }
    }
    assert!(
        refusals > 0,
        "nothing was refused by both: the facts bind nothing"
    );
    assert!(
        disagreements.is_empty(),
        "{disagreements:#?}\n{}",
        schema["$defs"]["demo.probe.Texts"]["properties"]
    );
}

/// `Integer` strict bounds one step in, at zero and at both ends of `i64`; a bound past `i64`
/// publishes nothing that refuses a value; `==` on an `Optional` field admits the absent value.
#[test]
fn every_integer_bound_agrees_with_the_model_at_the_ends_of_i64() {
    let ir = compiled();
    let schema = schema_of(&ir, "demo.probe.Integers");
    let validator = jsonschema::validator_for(&schema).expect("a valid schema");
    let invariants = invariants_of(&ir, "demo.probe.Integers");
    let values = [
        i64::MIN,
        i64::MIN + 1,
        -2,
        -1,
        0,
        1,
        2,
        3,
        5,
        6,
        i64::MAX - 1,
        i64::MAX,
    ];

    assert!(validator.is_valid(&json!({})), "every field absent");
    let mut disagreements = Vec::new();
    let mut refusals = 0_usize;
    for index in 0..9 {
        let field = format!("i{index}");
        let exact = EXACT_INTEGERS.contains(&field.as_str());
        for value in values {
            let refused = model_refuses(&invariants, &one(&field, Number::from(value)));
            let admitted = validator.is_valid(&json!({ field.as_str(): value }));
            refusals += usize::from(refused && !admitted);
            if (!refused && !admitted) || (exact && refused && admitted) {
                disagreements.push(format!(
                    "{field} = {value}: model refuses {refused}, schema admits {admitted}"
                ));
            }
        }
    }
    assert!(
        refusals > 0,
        "nothing was refused by both: the facts bind nothing"
    );
    assert!(
        disagreements.is_empty(),
        "{disagreements:#?}\n{}",
        schema["$defs"]["demo.probe.Integers"]["properties"]
    );
}
