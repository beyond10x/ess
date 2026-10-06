//! Adversary pass 1 on unit W3-2 (beyond10x/ess#445, #440, #450).
//!
//! Each case holds the `ess/23` enum-attribute lowering to what the unit's own module
//! documentation (`expression/attributes.rs`) and guide section claim: a lowered read holds for a
//! variant exactly where the predicate evaluator says it holds for the variant's value, and what is
//! not lowered is refused by name rather than silently answered.

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationError};
use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::predicate::{Predicate, Truth};

const OPERATOR: &str = "  - name: demo.rules.Family
    kind: enum
    variants: [Numeric, Textual]
  - name: demo.rules.Operator
    kind: enum
    attributes:
      - {name: takes_number, type: Boolean}
      - {name: label, type: String}
      - {name: arity, type: Optional<Integer>}
      - {name: family, type: demo.rules.Family}
    variants:
      - {name: GreaterThan, attributes: {takes_number: true, label: '>', arity: 2, family: Numeric}}
      - {name: LessThan, attributes: {takes_number: true, label: '<', arity: 2, family: Numeric}}
      - {name: Contains, attributes: {takes_number: false, label: contains, family: Textual}}
";

/// One command guarded by `guard` on its first outcome, with an unguarded default after it, so
/// coverage never decides the verdict.
fn model(format: &str, guard: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.rules
types:
{OPERATOR}events:
  - name: demo.rules.RuleAdded
    fields:
      - {{name: operator, type: demo.rules.Operator}}
commands:
  - name: demo.rules.AddRule
    input:
      - {{name: operator, type: demo.rules.Operator}}
      - {{name: family, type: demo.rules.Family}}
      - {{name: flag, type: Boolean}}
    outcomes:
      - name: picked
        when: {guard}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{operator: input.operator}}
      - name: other
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{operator: input.operator}}
"
    )
}

fn assemble(document: &str) -> Result<Specification, Vec<ValidationError>> {
    let raw = RawSpecFile::parse(document).map_err(|error| {
        vec![ValidationError::new(
            ValidationCode::UnsupportedConstruct,
            "parse",
            error.to_string(),
        )]
    })?;
    Specification::assemble([(Source::new("rules.yaml"), raw)])
        .map_err(|errors| errors.as_slice().to_vec())
}

/// The lowered guard of the outcome `name` of `AddRule`.
fn guard_of(spec: &Specification, name: &str) -> Predicate {
    let command = spec
        .commands()
        .values()
        .find(|command| command.name.to_string() == "demo.rules.AddRule")
        .expect("AddRule");
    let outcome = command
        .outcomes
        .iter()
        .find(|outcome| outcome.name.to_string() == name)
        .expect("the outcome");
    match &outcome.condition {
        OutcomeCondition::When(predicate) => predicate.clone(),
        other => panic!("a `when:` guard: {other:?}"),
    }
}

fn lowered(guard: &str) -> Predicate {
    let spec =
        assemble(&model("ess/23", guard)).unwrap_or_else(|errors| panic!("{guard}: {errors:#?}"));
    guard_of(&spec, "picked")
}

/// `predicate` evaluated where the input `operator` is `variant`.
fn at(predicate: &Predicate, variant: &str) -> Truth {
    let mut store = FactStore::new();
    store.set(
        FactPath::from_segments(["operator"]),
        FactValue::Text(variant.to_owned()),
    );
    predicate.evaluate(&store)
}

/// `operator.arity != 2` and `not: operator.arity == 2` are one condition for every variant that
/// fills `arity`, and for an `Optional` field the evaluator answers both `Unknown` where the field
/// is absent, so they agree there too. The lowering must not tell them apart for `Contains`, the
/// variant that leaves `arity` out: one says "not taken", the other "taken".
#[test]
fn adv_w3_2_not_equal_and_negated_equal_agree_on_unfilled_optional_attribute() {
    let not_equal = lowered("operator.arity != 2");
    let negated = lowered("{not: 'operator.arity == 2'}");
    for variant in ["GreaterThan", "LessThan", "Contains"] {
        assert_eq!(
            at(&not_equal, variant) == Truth::True,
            at(&negated, variant) == Truth::True,
            "`operator.arity != 2` lowered to {not_equal:?} and `not: operator.arity == 2` lowered \
             to {negated:?} disagree for `{variant}`"
        );
    }
}

/// A direct enum fact compared with a word that names no variant is refused by the checker. The
/// same word compared with an enum-typed attribute must not be silently lowered to `never`.
#[test]
fn adv_w3_2_enum_attribute_literal_naming_no_variant_is_refused() {
    let control = assemble(&model("ess/23", "family == Numerik"))
        .expect_err("a direct enum fact compared with a non-variant is refused");
    assert!(
        control
            .iter()
            .any(|error| error.message.contains("Numerik")),
        "control names the word: {control:#?}"
    );

    match assemble(&model("ess/23", "operator.family == Numerik")) {
        Ok(spec) => panic!(
            "`operator.family == Numerik` names no variant of `demo.rules.Family` and was \
             admitted, lowered to {:?}",
            guard_of(&spec, "picked")
        ),
        Err(errors) => assert!(
            errors.iter().any(|error| error.message.contains("Numerik")),
            "the refusal names the word: {errors:#?}"
        ),
    }
}

/// Truthiness over a text fact is admitted by the checker, and the evaluator holds it for any
/// non-empty text other than `false`. The lowering of `operator.label` must hold where that does,
/// or be refused — never lowered to a membership that holds for no variant.
#[test]
fn adv_w3_2_truthiness_of_text_attribute_agrees_with_the_evaluator() {
    let Ok(spec) = assemble(&model("ess/23", "operator.label")) else {
        return;
    };
    let lowered = guard_of(&spec, "picked");
    let mut store = FactStore::new();
    store.set(
        FactPath::from_segments(["label"]),
        FactValue::Text(">".to_owned()),
    );
    assert_eq!(
        Predicate::Truthy(FactPath::from_segments(["label"])).evaluate(&store),
        Truth::True,
        "the evaluator holds `>` truthy"
    );
    assert_eq!(
        at(&lowered, "GreaterThan"),
        Truth::True,
        "`operator.label` was admitted and lowered to {lowered:?}, which does not hold for \
         `GreaterThan`, whose label `>` the evaluator holds truthy"
    );
}

/// The `ess/23` conversion flag is thread-local. An `ess/22` document assembled after an `ess/23`
/// one in the same thread must get exactly the verdict it gets alone.
#[test]
fn adv_w3_2_ess22_verdict_unchanged_after_ess23_in_one_thread() {
    // Guards over an Integer input and a struct member, no default: whether coverage is deferred
    // at conversion is what the flag decides.
    let ess22 = "format: ess/22
system: demo
version: v1
domain: demo.rules
types:
  - name: demo.rules.Address
    kind: struct
    fields:
      - {name: country, type: String}
events:
  - name: demo.rules.Noted
    fields: []
commands:
  - name: demo.rules.Note
    input:
      - {name: count, type: Integer}
      - {name: address, type: demo.rules.Address}
    outcomes:
      - name: many
        when: count > 3
        emits: [demo.rules.Noted]
      - name: german
        when: address.country == \"DE\"
        emits: [demo.rules.Noted]
";
    let render = |result: Result<Specification, Vec<ValidationError>>| match result {
        Ok(_) => "valid".to_owned(),
        Err(errors) => errors
            .iter()
            .map(|error| format!("{:?} {} {}", error.code, error.location, error.message))
            .collect::<Vec<_>>()
            .join("\n"),
    };
    let alone = render(assemble(ess22));
    let _ = assemble(&model("ess/23", "operator.takes_number == true"));
    let after = render(assemble(ess22));
    assert_eq!(
        alone, after,
        "the ess/22 verdict moved after an ess/23 assembly"
    );
}

/// An ordering over a `Boolean` fact is refused by the checker (`flag < true`). The same ordering
/// over a `Boolean` attribute must not be lowered into a membership the checker never sees.
#[test]
fn adv_w3_2_ordering_a_boolean_attribute_is_refused_as_for_a_boolean_fact() {
    let control = assemble(&model("ess/23", "flag < true"))
        .expect_err("an ordering over a Boolean fact is refused");
    assert!(
        control
            .iter()
            .any(|error| error.code == ValidationCode::TypeMismatch),
        "control: {control:#?}"
    );
    match assemble(&model("ess/23", "operator.takes_number < true")) {
        Ok(spec) => panic!(
            "`operator.takes_number < true` orders a Boolean and was admitted, lowered to {:?}",
            guard_of(&spec, "picked")
        ),
        Err(errors) => assert!(
            errors
                .iter()
                .any(|error| error.message.contains("takes_number")),
            "{errors:#?}"
        ),
    }
}
