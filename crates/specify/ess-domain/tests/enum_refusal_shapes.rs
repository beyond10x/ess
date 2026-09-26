//! What the enum refusal's parenthetical says for every shape that reaches the enum.
//!
//! `ValueType` carries `declaring_variants: Option<String>`, filled from the resolution's
//! `terminal`, and the message reads `` `<enum>` (Text, reached through `<declared>`) `` when the
//! two differ and `` `<enum>` (Text) `` when they do not. Nothing else pins that clause:
//! `tests/enum_variant_refusals.rs` asserts only that the enum is named, and `tests/expression.rs`
//! asserts three `contains` calls, every one of which is satisfied without it. Before this file,
//! `let reached = String::new();` was a surviving mutant — the clause could be deleted outright
//! and the whole suite stayed green.
//!
//! Six shapes, one registry: the enum declared directly, one newtype over it, a newtype over that
//! newtype, the enum behind `Optional`, a list element, and a newtype over a non-enum. The direct
//! case is the one that catches an empty or duplicated clause, because it is the only shape where
//! `terminal` and `declared` are the same type.

use ess_domain::expression::{check_predicate, DomainEnvironment};
use ess_domain::{Field, NamedType, Naming, TypeBody, TypeRef, TypeRegistry};
use ess_primitives::error::ValidationCode;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

fn field(name: &str, kind: &str) -> Field {
    Field::new(name, TypeRef::parse(kind).unwrap())
}

fn newtype(of: &str) -> TypeBody {
    TypeBody::Newtype {
        alphabet: None,
        of: TypeRef::parse(of).unwrap(),
        invariants: vec![],
    }
}

/// An enum, a newtype over it, a newtype over that newtype, and a newtype over a non-enum.
fn registry() -> TypeRegistry {
    let mut registry = TypeRegistry::new();
    for (name, body) in [
        (
            "sample.Channel",
            TypeBody::Enum {
                variants: vec!["Email".into(), "Post".into(), "Portal".into()],
            },
        ),
        ("sample.WrappedChannel", newtype("sample.Channel")),
        ("sample.DoubleWrapped", newtype("sample.WrappedChannel")),
        ("sample.Code", newtype("String")),
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

fn fields() -> Vec<Field> {
    vec![
        field("channel", "sample.Channel"),
        field("wrapped_channel", "sample.WrappedChannel"),
        field("double_wrapped", "sample.DoubleWrapped"),
        field("optional_channel", "Optional<sample.Channel>"),
        field("channels", "List<sample.Channel>"),
        field("code", "sample.Code"),
    ]
}

/// Every `UndeclaredReference` message the checker produces for `<fact> == Fax`.
fn refusals(fact: &str) -> Vec<String> {
    let registry = registry();
    let fields = fields();
    let environment = DomainEnvironment::new(&registry, &fields);
    let predicate = Predicate::Compare {
        left: Operand::Fact(FactPath::new(fact).unwrap()),
        op: CompareOp::Eq,
        right: Operand::Literal(FactValue::text("Fax")),
    };
    check_predicate(&environment, &predicate, "owner")
        .errors
        .iter()
        .filter(|error| error.code == ValidationCode::UndeclaredReference)
        .map(|error| error.message.clone())
        .collect()
}

/// The one refusal for `<fact> == Fax`.
fn refusal(fact: &str) -> String {
    let found = refusals(fact);
    assert_eq!(
        found.len(),
        1,
        "`{fact} == Fax` is refused exactly once: {found:?}"
    );
    found[0].clone()
}

/// The parenthetical names the enum once, and the wrapper only when there is one.
///
/// Four shapes, and the question each answers:
///
/// * `channel` — declared *as* the enum. `terminal` and `declared` are the same type, so the
///   parenthetical must not carry an empty or duplicated `reached through` clause.
/// * `wrapped_channel` — one newtype. The enum is named and the wrapper is kept beside it.
/// * `double_wrapped` — a newtype over a newtype. The declared type is the outer wrapper and the
///   terminal is the enum; the middle wrapper is named by neither.
/// * `optional_channel` — the enum behind `Optional`, which `resolve` unwraps transparently.
#[test]
fn the_parenthetical_names_the_enum_once_and_the_wrapper_only_when_there_is_one() {
    let direct = refusal("channel");
    assert!(
        direct.contains("`sample.Channel` (Text)"),
        "declared as the enum: no wrapper to report, so no clause at all: {direct}"
    );
    assert!(
        !direct.contains("reached through"),
        "declared as the enum: `terminal` and `declared` are one type: {direct}"
    );

    let wrapped = refusal("wrapped_channel");
    assert!(
        wrapped.contains("`sample.Channel` (Text, reached through `sample.WrappedChannel`)"),
        "one newtype: the enum, and how the field reached it: {wrapped}"
    );

    let double = refusal("double_wrapped");
    assert!(
        double.contains("`sample.Channel` (Text, reached through `sample.DoubleWrapped`)"),
        "a newtype over a newtype: the enum, and the type the field declares: {double}"
    );

    let optional = refusal("optional_channel");
    assert!(
        optional.contains("`sample.Channel`"),
        "behind `Optional`, which the story's scope names as a shape to preserve: {optional}"
    );
}

/// A newtype over a non-enum carries no variants, so nothing is refused and nothing is named.
#[test]
fn a_newtype_over_a_non_enum_is_not_an_enum_refusal() {
    assert!(
        refusals("code").is_empty(),
        "`sample.Code` is a newtype over `String` and declares no variants: {:?}",
        refusals("code")
    );
}

/// A declared variant is admitted through every wrapper shape, or the refusal above is worthless.
#[test]
fn a_declared_variant_is_admitted_through_every_wrapper() {
    let registry = registry();
    let fields = fields();
    let environment = DomainEnvironment::new(&registry, &fields);
    for fact in [
        "channel",
        "wrapped_channel",
        "double_wrapped",
        "optional_channel",
    ] {
        let predicate = Predicate::Compare {
            left: Operand::Fact(FactPath::new(fact).unwrap()),
            op: CompareOp::Eq,
            right: Operand::Literal(FactValue::text("Email")),
        };
        let checked = check_predicate(&environment, &predicate, "owner");
        assert!(
            checked.errors.is_empty(),
            "`{fact} == Email` names a declared variant: {}",
            checked.validation_errors()
        );
    }
}
