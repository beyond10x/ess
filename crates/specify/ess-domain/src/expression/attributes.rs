//! A predicate's read of an enum attribute, `<fact>.<attribute>`, lowered to membership over the
//! enum's variants (`ess/23`, beyond10x/ess#450).
//!
//! An enum declares `attributes: [{name, type}]` and every variant gives each one a typed literal,
//! so which variants satisfy a comparison with a literal is known when the specification is read.
//! The comparison is rewritten as `any_of` over exactly those variants, and nothing downstream — the
//! coverage proof, the compiler, a suite, a runner, Entity Runtime — ever reads an attribute:
//!
//! | written | lowered |
//! |---|---|
//! | `plan.sso == true`, `plan.max_seats >= 10` (either side) | `plan: {any_of: [the variants it holds for]}` |
//! | `plan.sso`, `defined(plan.max_seats)` | `any_of` over the variants it holds for |
//! | `plan.tier: {any_of: […]}`, `{none_of: […]}` | `any_of` over the variants whose value is (not) listed |
//! | `seats >= plan.max_seats` | one branch per variant that fills the attribute: `all: [plan: {any_of: [V]}, seats >= <V's value>]`, joined by `any:` |
//!
//! A comparison holds for a variant exactly where the predicate evaluator says it holds for the
//! variant's value, so the lowering and every runner agree by construction. A variant that leaves
//! an `Optional` attribute unfilled satisfies no comparison and no `any_of`, and is the one variant
//! `defined(…)` excludes. A comparison with no satisfying variant is `never`.
//!
//! What is not lowered is left as written, and the checker that runs next refuses the attribute
//! read by name ([`super::resolve_path`]): a literal that is not of the attribute's type, two
//! attributes compared with each other, a fact comparison whose expansion exceeds
//! [`MAX_PREDICATE_NODES`], and every other predicate form.

use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate, Truth};

use super::{resolve_path, EnumAttributes, ScalarKind, TypeEnvironment};
use crate::command::finite::MAX_PREDICATE_NODES;

/// `predicate` with every attribute read it can lower rewritten as membership.
pub(crate) fn lower<E: TypeEnvironment>(environment: &E, predicate: Predicate) -> Predicate {
    match predicate {
        Predicate::All(items) => Predicate::All(
            items
                .into_iter()
                .map(|item| lower(environment, item))
                .collect(),
        ),
        Predicate::Any(items) => Predicate::Any(
            items
                .into_iter()
                .map(|item| lower(environment, item))
                .collect(),
        ),
        Predicate::Not(inner) => Predicate::Not(Box::new(lower(environment, *inner))),
        Predicate::Compare {
            left,
            op,
            right,
            kind: CompareKind::Value,
        } => compare(environment, &left, op, &right).unwrap_or(Predicate::Compare {
            left,
            op,
            right,
            kind: CompareKind::Value,
        }),
        Predicate::Truthy(path) => read(environment, &path)
            .map(|read| read.members(|value| value == Some(&FactValue::Bool(true))))
            .unwrap_or(Predicate::Truthy(path)),
        Predicate::Defined(path) => read(environment, &path)
            .map(|read| read.members(|value| value.is_some()))
            .unwrap_or(Predicate::Defined(path)),
        Predicate::AnyOf { path, values } => match read(environment, &path) {
            Some(read) if values.iter().all(|value| read.admits(value)) => {
                read.members(|value| value.is_some_and(|value| values.contains(value)))
            }
            _ => Predicate::AnyOf { path, values },
        },
        Predicate::NoneOf { path, values } => match read(environment, &path) {
            Some(read) if values.iter().all(|value| read.admits(value)) => {
                read.members(|value| value.is_some_and(|value| !values.contains(value)))
            }
            _ => Predicate::NoneOf { path, values },
        },
        other => other,
    }
}

/// One attribute read: the enum fact it is read through, and the attribute.
struct Read {
    fact: FactPath,
    attribute: String,
    scalar: ScalarKind,
    attributes: EnumAttributes,
}

impl Read {
    /// Whether a literal is of the attribute's scalar: a literal of another is a mistake the
    /// checker names, not a comparison that holds for no variant.
    fn admits(&self, literal: &FactValue) -> bool {
        matches!(
            (self.scalar, literal),
            (ScalarKind::Bool, FactValue::Bool(_))
                | (ScalarKind::Number, FactValue::Number(_))
                | (ScalarKind::Text, FactValue::Text(_))
        )
    }

    /// The value `variant` gives the attribute, where it gives one.
    fn value<'a>(
        &'a self,
        values: &'a std::collections::BTreeMap<String, FactValue>,
    ) -> Option<&'a FactValue> {
        values.get(&self.attribute)
    }

    /// Membership of the fact in the variants whose value satisfies `holds`; `never` for none.
    fn members(&self, holds: impl Fn(Option<&FactValue>) -> bool) -> Predicate {
        let values: Vec<FactValue> = self
            .attributes
            .variants
            .iter()
            .filter(|(_, values)| holds(self.value(values)))
            .map(|(variant, _)| FactValue::Text(variant.clone()))
            .collect();
        if values.is_empty() {
            Predicate::Never
        } else {
            Predicate::AnyOf {
                path: self.fact.clone(),
                values,
            }
        }
    }
}

/// `path` as an attribute read: its last segment an attribute of the enum the rest resolves to.
fn read<E: TypeEnvironment>(environment: &E, path: &FactPath) -> Option<Read> {
    let segments = path.segments();
    let (attribute, prefix) = segments.split_last()?;
    if prefix.is_empty() {
        return None;
    }
    let fact = FactPath::from_segments(prefix.iter().cloned());
    let resolution = resolve_path(environment, &fact, "").ok()?;
    let attributes = environment.enum_attributes(&resolution.terminal)?;
    let (_, scalar, _) = attributes.attribute(attribute)?;
    Some(Read {
        fact,
        attribute: attribute.clone(),
        scalar: *scalar,
        attributes,
    })
}

/// Whether `left op right` holds where the attribute is `value`, by the predicate evaluator.
fn holds(value: &FactValue, attribute_left: bool, op: CompareOp, literal: &FactValue) -> bool {
    let at = FactPath::from_segments(["attribute"]);
    let mut store = FactStore::new();
    store.set(at.clone(), value.clone());
    let (left, right) = if attribute_left {
        (Operand::Fact(at), Operand::Literal(literal.clone()))
    } else {
        (Operand::Literal(literal.clone()), Operand::Fact(at))
    };
    Predicate::Compare {
        left,
        op,
        right,
        kind: CompareKind::Value,
    }
    .evaluate(&store)
        == Truth::True
}

/// The operator that reads the same with its operands swapped.
fn flipped(op: CompareOp) -> CompareOp {
    match op {
        CompareOp::Eq => CompareOp::Eq,
        CompareOp::Ne => CompareOp::Ne,
        CompareOp::Lt => CompareOp::Gt,
        CompareOp::Le => CompareOp::Ge,
        CompareOp::Gt => CompareOp::Lt,
        CompareOp::Ge => CompareOp::Le,
    }
}

/// A comparison reading an attribute, lowered; `None` to leave it as written.
fn compare<E: TypeEnvironment>(
    environment: &E,
    left: &Operand,
    op: CompareOp,
    right: &Operand,
) -> Option<Predicate> {
    let as_read = |operand: &Operand| match operand {
        Operand::Fact(path) => read(environment, path),
        _ => None,
    };
    match (as_read(left), as_read(right)) {
        (Some(read), None) => match right {
            Operand::Literal(literal) if read.admits(literal) => Some(
                read.members(|value| value.is_some_and(|value| holds(value, true, op, literal))),
            ),
            Operand::Fact(other) => expand(&read, other, flipped(op)),
            _ => None,
        },
        (None, Some(read)) => match left {
            Operand::Literal(literal) if read.admits(literal) => Some(
                read.members(|value| value.is_some_and(|value| holds(value, false, op, literal))),
            ),
            Operand::Fact(other) => expand(&read, other, op),
            _ => None,
        },
        _ => None,
    }
}

/// `other op <attribute>` for each variant that fills the attribute, as one branch per variant:
/// `all: [fact: {any_of: [V]}, other op <V's value>]`, joined by `any:`. `None` past
/// [`MAX_PREDICATE_NODES`] — one node for the join and three per branch.
fn expand(read: &Read, other: &FactPath, op: CompareOp) -> Option<Predicate> {
    let branches: Vec<Predicate> = read
        .attributes
        .variants
        .iter()
        .filter_map(|(variant, values)| {
            let value = read.value(values)?;
            Some(Predicate::All(vec![
                Predicate::AnyOf {
                    path: read.fact.clone(),
                    values: vec![FactValue::Text(variant.clone())],
                },
                Predicate::Compare {
                    left: Operand::Fact(other.clone()),
                    op,
                    right: Operand::Literal(value.clone()),
                    kind: CompareKind::Value,
                },
            ]))
        })
        .collect();
    if 1 + 3 * read.attributes.variants.len() > MAX_PREDICATE_NODES {
        return None;
    }
    Some(match branches.len() {
        0 => Predicate::Never,
        _ => Predicate::Any(branches),
    })
}

/// Whether a guard may read an enum attribute: a fact path of more than one segment whose root
/// is a declared input of a named type. Asked before the types are known, so a command whose
/// guards may read one has its coverage decided once they are and the reads are lowered.
pub(crate) fn may_read(guards: &[&Predicate], input: &[crate::types::Field]) -> bool {
    CONVERTING_ATTRIBUTES.with(std::cell::Cell::get)
        && guards.iter().any(|guard| reads(guard, input))
}

/// Whether `predicate` reads a fact path of more than one segment rooted at a named input.
fn reads(predicate: &Predicate, input: &[crate::types::Field]) -> bool {
    let named = |path: &FactPath| {
        path.segments().len() > 1
            && input.iter().any(|field| {
                field.name == path.namespace()
                    && matches!(field.type_ref.required(), crate::types::TypeRef::Named(_))
            })
    };
    let operand = |operand: &Operand| matches!(operand, Operand::Fact(path) if named(path));
    match predicate {
        Predicate::All(items) | Predicate::Any(items) => {
            items.iter().any(|item| reads(item, input))
        }
        Predicate::Not(inner) => reads(inner, input),
        Predicate::Compare { left, right, .. } => operand(left) || operand(right),
        Predicate::Truthy(path)
        | Predicate::Defined(path)
        | Predicate::AnyOf { path, .. }
        | Predicate::NoneOf { path, .. } => named(path),
        _ => false,
    }
}

std::thread_local! {
    /// Whether the source being converted is `ess/23` or later, where a guard may read an enum
    /// attribute; set by the assembly around the conversion, off everywhere else, so a command
    /// of an older format keeps the coverage decision it always had.
    static CONVERTING_ATTRIBUTES: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

/// Runs `convert` with enum attribute reads admitted in guards when `admitted`.
pub(crate) fn converting<T>(admitted: bool, convert: impl FnOnce() -> T) -> T {
    let before = CONVERTING_ATTRIBUTES.with(|cell| cell.replace(admitted));
    let converted = convert();
    CONVERTING_ATTRIBUTES.with(|cell| cell.set(before));
    converted
}
