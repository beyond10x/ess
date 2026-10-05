//! Typed per-variant enum attributes (`ess/23`, beyond10x/ess#450): an enum declares
//! `attributes: [{name, type}]`, each variant fills them with typed literals, and a predicate reads
//! `<fact>.<attribute>`, lowered to membership over the variants that satisfy it.

use std::fmt::Write as _;
use std::path::PathBuf;

use ess_domain::command::OutcomeCondition;
use ess_domain::spec::RawSpecFile;
use ess_domain::system::Source;
use ess_domain::Specification;
use ess_primitives::error::{ValidationCode, ValidationError};
use ess_primitives::facts::FactValue;
use ess_primitives::predicate::Predicate;

const OPERATOR: &str = "  - name: demo.rules.Operator
    kind: enum
    attributes:
      - {name: takes_number, type: Boolean}
      - {name: label, type: String}
      - {name: arity, type: Optional<Integer>}
    variants:
      - {name: GreaterThan, attributes: {takes_number: true, label: '>', arity: 2}}
      - {name: LessThan, attributes: {takes_number: true, label: '<', arity: 2}}
      - {name: Contains, attributes: {takes_number: false, label: contains}}
";

/// A rules domain with the `Operator` enum (`types`) and one command guarded by `numeric` and
/// `textual`.
fn model(format: &str, types: &str, numeric: &str, textual: &str) -> String {
    format!(
        "format: {format}
system: demo
version: v1
domain: demo.rules
types:
{types}events:
  - name: demo.rules.RuleAdded
    fields:
      - {{name: operator, type: demo.rules.Operator}}
commands:
  - name: demo.rules.AddRule
    input:
      - {{name: operator, type: demo.rules.Operator}}
    outcomes:
      - name: numeric
        when: {numeric}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{operator: input.operator}}
      - name: textual
        when: {textual}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{operator: input.operator}}
"
    )
}

fn guarded(types: &str) -> String {
    model(
        "ess/23",
        types,
        "operator.takes_number == true",
        "operator.takes_number == false",
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

fn refused(document: &str) -> Vec<ValidationError> {
    assemble(document).expect_err("the document is refused")
}

fn one(errors: &[ValidationError], code: ValidationCode) -> &ValidationError {
    let found: Vec<&ValidationError> = errors.iter().filter(|error| error.code == code).collect();
    assert_eq!(found.len(), 1, "one {code:?}: {errors:#?}");
    found[0]
}

/// The input guard of `AddRule`'s outcome `name`.
fn guard(spec: &Specification, name: &str) -> Predicate {
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

#[test]
fn enum_attributes_declared_and_filled_validate() {
    if let Err(errors) = assemble(&guarded(OPERATOR)) {
        panic!("an enum whose variants fill each attribute validates: {errors:#?}");
    }

    // A required attribute a variant leaves out.
    let missing = OPERATOR.replace(
        "{takes_number: false, label: contains}",
        "{label: contains}",
    );
    let errors = refused(&guarded(&missing));
    let error = one(&errors, ValidationCode::MissingDeclaration);
    assert!(
        error.message.contains("Contains") && error.message.contains("takes_number"),
        "names the variant and the attribute: {error:#?}"
    );

    // An attribute the enum does not declare.
    let extra = OPERATOR.replace(
        "{takes_number: false, label: contains}",
        "{takes_number: false, label: contains, weight: 3}",
    );
    let errors = refused(&guarded(&extra));
    let error = one(&errors, ValidationCode::UndeclaredReference);
    assert!(
        error.message.contains("weight") && error.message.contains("Contains"),
        "{error:#?}"
    );

    // A value that is not one of the attribute's type.
    let mistyped = OPERATOR.replace(
        "{takes_number: false, label: contains}",
        "{takes_number: 3, label: contains}",
    );
    let errors = refused(&guarded(&mistyped));
    let error = one(&errors, ValidationCode::TypeMismatch);
    assert!(
        error.message.contains("takes_number") && error.message.contains("Contains"),
        "{error:#?}"
    );
    assert!(
        error.location.contains("demo.rules.Operator"),
        "located at the type: {error:#?}"
    );

    // A `List` attribute is refused by name in this cut.
    let listed = OPERATOR.replace(
        "      - {name: arity, type: Optional<Integer>}\n",
        "      - {name: arity, type: Optional<Integer>}\n      - {name: aliases, type: List<String>}\n",
    );
    let errors = refused(&guarded(&listed));
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedConstruct
                && error.message.contains("aliases")
                && error.message.contains("List")),
        "{errors:#?}"
    );
}

#[test]
fn enum_attribute_guard_lowers_to_membership_in_the_model() {
    let spec = assemble(&guarded(OPERATOR)).unwrap_or_else(|errors| panic!("{errors:#?}"));
    let text = |names: &[&str]| -> Vec<FactValue> {
        names
            .iter()
            .map(|name| FactValue::Text((*name).to_owned()))
            .collect()
    };
    match guard(&spec, "numeric") {
        Predicate::AnyOf { path, values } => {
            assert_eq!(path.to_string(), "operator");
            assert_eq!(values, text(&["GreaterThan", "LessThan"]));
        }
        other => panic!("lowered to membership: {other:?}"),
    }
    match guard(&spec, "textual") {
        Predicate::AnyOf { path, values } => {
            assert_eq!(path.to_string(), "operator");
            assert_eq!(values, text(&["Contains"]));
        }
        other => panic!("lowered to membership: {other:?}"),
    }
}

#[test]
fn enum_attribute_fact_comparison_expands_within_bounds() {
    let plans = "  - name: demo.rules.Operator
    kind: enum
    attributes:
      - {name: max_seats, type: Integer}
    variants:
      - {name: Basic, attributes: {max_seats: 5}}
      - {name: Team, attributes: {max_seats: 50}}
";
    let document = model(
        "ess/23",
        plans,
        "{all: [operator.max_seats >= 10, operator.max_seats < 100]}",
        "operator.max_seats < 10",
    );
    let spec = assemble(&document).unwrap_or_else(|errors| panic!("{errors:#?}"));
    // Each comparison against a literal is the variants that satisfy it.
    assert!(
        matches!(guard(&spec, "textual"), Predicate::AnyOf { ref values, .. }
            if values == &[FactValue::Text("Basic".into())]),
        "{:?}",
        guard(&spec, "textual")
    );

    // A comparison with another fact expands per variant.
    let fact = model(
        "ess/23",
        plans,
        "count >= operator.max_seats",
        "count < operator.max_seats",
    )
    .replace(
        "      - {name: operator, type: demo.rules.Operator}\n    outcomes:",
        "      - {name: operator, type: demo.rules.Operator}\n      - {name: count, type: Integer}\n    outcomes:",
    )
    .replacen(DEFAULTLESS, DEFAULTED, 1);
    let spec = assemble(&fact).unwrap_or_else(|errors| panic!("{errors:#?}"));
    let expanded = guard(&spec, "numeric");
    let rendered = expanded.to_string();
    for part in ["Basic", "Team", "5", "50", "count"] {
        assert!(rendered.contains(part), "{part} in {rendered}");
    }
    assert!(
        matches!(expanded, Predicate::Any(ref arms) if arms.len() == 2),
        "{expanded:?}"
    );

    // Past 128 nodes the expansion is refused by name.
    let mut many = String::from(
        "  - name: demo.rules.Operator\n    kind: enum\n    attributes:\n      - {name: max_seats, type: Integer}\n    variants:\n",
    );
    for index in 0..43 {
        let _ = writeln!(
            many,
            "      - {{name: Plan{index}, attributes: {{max_seats: {index}}}}}"
        );
    }
    let errors = refused(
        &model("ess/23", &many, "count >= operator.max_seats", "count < operator.max_seats")
            .replace(
                "      - {name: operator, type: demo.rules.Operator}\n    outcomes:",
                "      - {name: operator, type: demo.rules.Operator}\n      - {name: count, type: Integer}\n    outcomes:",
            )
            .replacen(DEFAULTLESS, DEFAULTED, 1),
    );
    assert!(
        errors
            .iter()
            .any(|error| error.message.contains("operator.max_seats")
                && error.message.contains("128")),
        "{errors:#?}"
    );
}

#[test]
fn enum_attribute_view_filter_and_invariant() {
    let document = format!(
        "format: ess/23
system: demo
version: v1
domain: demo.rules
types:
{OPERATOR}  - {{name: demo.rules.RuleId, kind: newtype, of: Uuid}}
entities:
  - name: demo.rules.Rule
    identity: {{name: rule_id, type: demo.rules.RuleId}}
    fields:
      - {{name: operator, type: demo.rules.Operator}}
    invariants:
      - any: [operator.takes_number == true, operator.label == contains]
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
events:
  - name: demo.rules.RuleAdded
    fields:
      - {{name: rule_id, type: demo.rules.RuleId}}
commands:
  - name: demo.rules.AddRule
    input:
      - {{name: operator, type: demo.rules.Operator}}
    outcomes:
      - name: added
        creates: demo.rules.Rule
        instance: rule_id
        sets: {{operator: input.operator}}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{rule_id: {{generated: true}}}}
views:
  - name: demo.rules.NumericRules
    source: demo.rules.Rule
    consistency: read_your_writes
    filter: operator.takes_number == true
    fields:
      - {{name: rule_id, type: demo.rules.RuleId}}
      - {{name: operator, type: demo.rules.Operator}}
"
    );
    let spec = assemble(&document).unwrap_or_else(|errors| panic!("{errors:#?}"));
    let view = spec.views().values().next().expect("the view");
    assert!(
        matches!(&view.filter, Some(Predicate::AnyOf { values, .. }) if values.len() == 2),
        "the filter is lowered: {:?}",
        view.filter
    );
    let entity = spec.entities().values().next().expect("the entity");
    let invariant = entity.invariants[0].predicate.to_string();
    assert!(
        !invariant.contains("takes_number") && !invariant.contains("label"),
        "the invariant is lowered: {invariant}"
    );
}

#[test]
fn enum_attribute_value_position_refused() {
    let document = format!(
        "format: ess/23
system: demo
version: v1
domain: demo.rules
types:
{OPERATOR}  - {{name: demo.rules.RuleId, kind: newtype, of: Uuid}}
entities:
  - name: demo.rules.Rule
    identity: {{name: rule_id, type: demo.rules.RuleId}}
    fields:
      - {{name: number, type: Boolean}}
    lifecycle: {{initial: Active, states: [Active], terminal: [Active]}}
events:
  - name: demo.rules.RuleAdded
    fields:
      - {{name: rule_id, type: demo.rules.RuleId}}
commands:
  - name: demo.rules.AddRule
    input:
      - {{name: operator, type: demo.rules.Operator}}
    outcomes:
      - name: added
        creates: demo.rules.Rule
        instance: rule_id
        sets: {{number: input.operator.takes_number}}
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {{rule_id: {{generated: true}}}}
"
    );
    let errors = refused(&document);
    assert!(
        errors
            .iter()
            .any(|error| error.code == ValidationCode::UnsupportedConstruct
                && error.message.contains("takes_number")
                && error.message.contains("attribute")
                && error.message.contains("predicate")),
        "the value-position read is refused naming the cut: {errors:#?}"
    );
}

#[test]
fn enum_attributes_below_ess_23_refused() {
    let errors = refused(&model(
        "ess/22",
        OPERATOR,
        "operator == GreaterThan",
        "operator != GreaterThan",
    ));
    assert!(
        errors.iter().any(
            |error| error.code == ValidationCode::UnsupportedFormatVersion
                && error.message.contains("ess/23")
        ),
        "{errors:#?}"
    );
}

const GUIDE: &str = "website/docs/guides/specify/fields-and-invariants.md";
const HEADING: &str = "### Give enum variants typed attributes";

#[test]
fn enum_attribute_guide_section_states_the_construct() {
    let path = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .join(GUIDE);
    let page = std::fs::read_to_string(&path).unwrap_or_else(|error| panic!("{GUIDE}: {error}"));
    let start = page
        .find(&format!("\n{HEADING}\n"))
        .unwrap_or_else(|| panic!("{GUIDE} has no heading `{HEADING}`"));
    let cover = page
        .find("\n### Cover every declared enum value\n")
        .expect("the guide keeps its coverage section");
    assert!(
        start > cover,
        "`{HEADING}` comes after `### Cover every declared enum value`"
    );
    let body = &page[start + 1..];
    let end = body[HEADING.len()..]
        .find("\n## ")
        .into_iter()
        .chain(body[HEADING.len()..].find("\n### "))
        .min()
        .map_or(body.len(), |at| at + HEADING.len());
    let section = &body[..end];
    for phrase in [
        "`attributes:`",
        "`ess/23`",
        "lowered to membership",
        "`x-ess-attributes`",
        "refused",
    ] {
        assert!(
            section.contains(phrase),
            "the section `{HEADING}` does not contain {phrase:?}:\n{section}"
        );
    }
    let open = section
        .find("```yaml\n")
        .unwrap_or_else(|| panic!("the section `{HEADING}` has no fenced yaml model"))
        + "```yaml\n".len();
    let close = section[open..].find("\n```").expect("a closed fence");
    let model = &section[open..=(open + close)];
    if let Err(errors) = assemble(model) {
        panic!("the section's model does not validate: {errors:#?}\n{model}");
    }
}

/// The end of `model`'s command, and the same with a default outcome: a guard comparing two open
/// facts is outside the finite coverage proof, so its command keeps a default.
const DEFAULTLESS: &str = "        payload:
          demo.rules.RuleAdded: {operator: input.operator}
";
const DEFAULTED: &str = "        payload:
          demo.rules.RuleAdded: {operator: input.operator}
      - name: other
        emits: [demo.rules.RuleAdded]
        payload:
          demo.rules.RuleAdded: {operator: input.operator}
";
