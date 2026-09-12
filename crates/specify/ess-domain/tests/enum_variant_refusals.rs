//! Which type an undeclared-enum-variant refusal names.
//!
//! `story:enum-variant-in-an-entity-invariant`, acceptance clause 2, quoted whole: "The refusal
//! names the enum, not only the field, so the fix is one lookup away." The enum is the declaration
//! the author has to edit to make the literal legal, so it is the one the refusal owes a name.
//!
//! A field's *declared* type is not always that enum: through a newtype it is the wrapper, and the
//! variants listed beside it belong to the type the wrapper aliases. This file holds both shapes
//! side by side so neither can drift into the other.

use ess_domain::expression::{check_predicate, DomainEnvironment};
use ess_domain::{Field, NamedType, Naming, TypeBody, TypeRef, TypeRegistry};
use ess_primitives::error::ValidationCode;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

fn field(name: &str, kind: &str) -> Field {
    Field::new(name, TypeRef::parse(kind).unwrap())
}

/// An enum, and a newtype over that enum. Both shapes are representable and both are declared here.
fn registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    for (name, body) in [
        (
            "sample.Channel",
            TypeBody::Enum {
                variants: vec!["Email".into(), "Post".into(), "Portal".into()],
            },
        ),
        (
            "sample.WrappedChannel",
            TypeBody::Newtype {
                of: TypeRef::parse("sample.Channel").unwrap(),
                invariants: vec![],
            },
        ),
    ] {
        registry
            .insert(NamedType {
                reading: None,
                name: name.parse().unwrap(),
                body,
                naming: Naming::default(),
            })
            .unwrap();
    }
    registry
}

/// One field of each shape, so the two refusals can be compared side by side.
fn fields() -> Vec<Field> {
    vec![
        field("channel", "sample.Channel"),
        field("wrapped_channel", "sample.WrappedChannel"),
    ]
}

/// `<fact> == Fax`, the story's own reproduction shape.
fn compared(fact: &str) -> Predicate {
    Predicate::Compare {
        left: Operand::Fact(FactPath::new(fact).unwrap()),
        op: CompareOp::Eq,
        right: Operand::Literal(FactValue::text("Fax")),
    }
}

/// The one `UndeclaredReference` message the checker produces for `predicate`.
fn refusal(predicate: &Predicate) -> String {
    let registry = registry();
    let fields = fields();
    let environment = DomainEnvironment::new(&registry, &fields);
    let checked = check_predicate(&environment, predicate, "owner");
    let refused = checked
        .errors
        .iter()
        .find(|error| error.code == ValidationCode::UndeclaredReference)
        .unwrap_or_else(|| panic!("`Fax` was admitted by `{predicate}`"));
    refused.message.clone()
}

/// Acceptance clause 2, applied to a field whose declared type is a newtype over the enum.
///
/// The control is the first assertion: comparing a field declared *as* the enum names the enum, so
/// the clause is satisfiable and this case is not asking for something the checker never does.
///
/// The second assertion is the finding. `sample.WrappedChannel` is a newtype whose `of:` is
/// `sample.Channel`, so the variants the message lists are the enum's — but the only type it names
/// is the wrapper. A reader given `sample.WrappedChannel` has to open that declaration, read its
/// `of:`, and open the enum: two lookups, where the clause asks for one. The enum is the thing the
/// author has to edit to make `Fax` legal, and the refusal never says its name.
///
/// `crates/specify/ess-domain/tests/expression.rs` reaches the same state from the other side —
/// its `enum_literal_cases` includes a case labelled "through a newtype" over
/// `sample.WrappedState` — and asserts the enum's name exactly, not a namespace prefix.
#[test]
fn a_refusal_about_a_newtype_wrapped_enum_field_names_the_enum_as_the_story_requires() {
    let direct = refusal(&compared("channel"));
    assert!(
        direct.contains("`sample.Channel`"),
        "the control: a field declared as the enum names the enum: {direct}"
    );

    let wrapped = refusal(&compared("wrapped_channel"));
    assert!(
        wrapped.contains("`sample.Channel`"),
        "acceptance clause 2 asks that the refusal name the enum, not only the field, `so the fix \
         is one lookup away`; through a newtype it names only the wrapper: {wrapped}"
    );
}
