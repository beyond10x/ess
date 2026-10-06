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
//! | `plan.tier: {any_of: […]}`, `{none_of: […]}` | `any_of` over the variants it holds for |
//! | `not: …` over any of these | `any_of` over the variants it is *false* for |
//! | `seats >= plan.max_seats` | one branch per variant that fills the attribute: `all: [plan: {any_of: [V]}, seats >= <V's value>]`, joined by `any:` |
//!
//! Which variants a read holds for is decided by the predicate evaluator itself, given each
//! variant's value — or no value, where the variant leaves an `Optional` attribute unfilled — so
//! the lowering and every runner agree by construction, truthiness included. The evaluator answers
//! three ways: a read is `Unknown` for a variant that leaves the attribute unfilled, and that
//! variant is in neither the membership of the read nor of its negation. A negation is pushed to
//! the reads by De Morgan's laws, which hold for the three-valued connectives, so `not: a == 2` and
//! `a != 2` lower to the same membership. A read that holds for no variant is `never`.
//!
//! A literal that is no value of the attribute's type — a word naming no variant of an enum-typed
//! attribute, a number compared with a `Boolean` — and an ordering over a `Boolean` are refused
//! here, by name, as the checker refuses them over a plain fact. What is not lowered otherwise is
//! left as written, and the checker that runs next refuses the attribute read by name
//! ([`super::resolve_path`]): two attributes compared with each other, a fact comparison whose
//! expansion exceeds [`MAX_PREDICATE_NODES`], and every other predicate form.

use std::cell::RefCell;

use ess_primitives::facts::{FactPath, FactStore, FactValue};
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate, Truth};

use super::{resolve_path, EnumAttributes, ScalarKind, TypeEnvironment};
use crate::command::finite::MAX_PREDICATE_NODES;

/// `predicate` with every attribute read it can lower rewritten as membership. Each read refused
/// here is added to `refusals` and stands as `never`, so nothing reports it a second time.
pub(crate) fn lower<E: TypeEnvironment>(
    environment: &E,
    refusals: &RefCell<Vec<String>>,
    predicate: Predicate,
) -> Predicate {
    match predicate {
        Predicate::All(items) => Predicate::All(
            items
                .into_iter()
                .map(|item| lower(environment, refusals, item))
                .collect(),
        ),
        Predicate::Any(items) => Predicate::Any(
            items
                .into_iter()
                .map(|item| lower(environment, refusals, item))
                .collect(),
        ),
        Predicate::Not(inner) if reads_attribute(environment, &inner) => {
            negated(environment, *inner, refusals)
        }
        Predicate::Not(inner) => Predicate::Not(Box::new(lower(environment, refusals, *inner))),
        atom => lowered(environment, atom, Truth::True, refusals),
    }
}

/// `not: predicate`, lowered: the negation pushed down to the reads (De Morgan, three-valued).
fn negated<E: TypeEnvironment>(
    environment: &E,
    predicate: Predicate,
    refusals: &RefCell<Vec<String>>,
) -> Predicate {
    if !reads_attribute(environment, &predicate) {
        return Predicate::Not(Box::new(lower(environment, refusals, predicate)));
    }
    match predicate {
        Predicate::All(items) => Predicate::Any(
            items
                .into_iter()
                .map(|item| negated(environment, item, refusals))
                .collect(),
        ),
        Predicate::Any(items) => Predicate::All(
            items
                .into_iter()
                .map(|item| negated(environment, item, refusals))
                .collect(),
        ),
        Predicate::Not(inner) => lower(environment, refusals, *inner),
        atom => lowered(environment, atom, Truth::False, refusals),
    }
}

/// One atom reading an attribute, as membership in the variants for which the evaluator answers
/// `want`; a comparison with another fact expanded per variant; anything else as written.
fn lowered<E: TypeEnvironment>(
    environment: &E,
    atom: Predicate,
    want: Truth,
    refusals: &RefCell<Vec<String>>,
) -> Predicate {
    match atom_read(environment, &atom) {
        Some(Ok(read)) => read.members(&atom, want),
        Some(Err(refusal)) => {
            refusals.borrow_mut().push(refusal);
            Predicate::Never
        }
        None => match &atom {
            Predicate::Compare {
                left,
                op,
                right,
                kind: CompareKind::Value,
            } => compare_facts(environment, left, *op, right, want == Truth::False),
            _ => None,
        }
        .unwrap_or_else(|| match want {
            Truth::False => Predicate::Not(Box::new(atom)),
            _ => atom,
        }),
    }
}

/// Whether `predicate` reads an enum attribute anywhere.
fn reads_attribute<E: TypeEnvironment>(environment: &E, predicate: &Predicate) -> bool {
    let operand = |operand: &Operand| matches!(operand, Operand::Fact(path) if read(environment, path).is_some());
    match predicate {
        Predicate::All(items) | Predicate::Any(items) => {
            items.iter().any(|item| reads_attribute(environment, item))
        }
        Predicate::Not(inner) => reads_attribute(environment, inner),
        Predicate::Compare { left, right, .. } => operand(left) || operand(right),
        Predicate::Truthy(path)
        | Predicate::Defined(path)
        | Predicate::AnyOf { path, .. }
        | Predicate::NoneOf { path, .. } => read(environment, path).is_some(),
        _ => false,
    }
}

/// One attribute read: the path written, the enum fact it is read through, and the attribute.
struct Read {
    path: FactPath,
    fact: FactPath,
    attribute: String,
    scalar: ScalarKind,
    written: String,
    attributes: EnumAttributes,
}

impl Read {
    /// Why `literal` is no value of the attribute, where it is not: of another scalar, or a word
    /// naming no variant of an enum-typed attribute.
    fn refuses(&self, literal: &FactValue) -> Option<String> {
        let scalar = matches!(
            (self.scalar, literal),
            (ScalarKind::Bool, FactValue::Bool(_))
                | (ScalarKind::Number, FactValue::Number(_))
                | (ScalarKind::Text, FactValue::Text(_))
        );
        let named = match (self.attributes.words.get(&self.attribute), literal) {
            (Some(words), FactValue::Text(word)) => words.contains(word),
            _ => true,
        };
        (!scalar || !named).then(|| {
            let variants = self
                .attributes
                .words
                .get(&self.attribute)
                .map(|words| format!("; variants: {}", words.join(", ")))
                .unwrap_or_default();
            format!(
                "`{}` is the attribute `{}` of `{}`, which is `{}`, and `{literal}` is no value \
                 of it{variants}",
                self.path, self.attribute, self.attributes.name, self.written
            )
        })
    }

    /// Membership of the fact in the variants for which the evaluator answers `want` to `atom`,
    /// given each variant's value of the attribute; `never` for none.
    fn members(&self, atom: &Predicate, want: Truth) -> Predicate {
        let values: Vec<FactValue> = self
            .attributes
            .variants
            .iter()
            .filter(|(_, values)| {
                let mut store = FactStore::new();
                if let Some(value) = values.get(&self.attribute) {
                    store.set(self.path.clone(), value.clone());
                }
                atom.evaluate(&store) == want
            })
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
    let (_, scalar, written) = attributes.attribute(attribute)?;
    Some(Read {
        path: path.clone(),
        fact,
        attribute: attribute.clone(),
        scalar: *scalar,
        written: written.clone(),
        attributes,
    })
}

/// `atom` as a read of one attribute against literals only: `Some(Ok)` to lower, `Some(Err)` with
/// the refusal of a literal that is no value of it or an ordering over a `Boolean`, `None` for
/// anything else.
fn atom_read<E: TypeEnvironment>(
    environment: &E,
    atom: &Predicate,
) -> Option<Result<Read, String>> {
    let checked = |read: Read, literals: &[&FactValue]| {
        literals
            .iter()
            .find_map(|literal| read.refuses(literal))
            .map_or(Ok(read), Err)
    };
    match atom {
        Predicate::Compare {
            left,
            op,
            right,
            kind: CompareKind::Value,
        } => {
            let ((Operand::Fact(path), Operand::Literal(literal))
            | (Operand::Literal(literal), Operand::Fact(path))) = (left, right)
            else {
                return None;
            };
            let read = read(environment, path)?;
            if read.scalar == ScalarKind::Bool && !matches!(op, CompareOp::Eq | CompareOp::Ne) {
                return Some(Err(format!(
                    "`{}` is the attribute `{}` of `{}`, which is `{}`, and a Boolean is compared \
                     with `==` or `!=`, never ordered with `{}`",
                    read.path,
                    read.attribute,
                    read.attributes.name,
                    read.written,
                    op.as_str()
                )));
            }
            Some(checked(read, &[literal]))
        }
        Predicate::Truthy(path) | Predicate::Defined(path) => read(environment, path).map(Ok),
        Predicate::AnyOf { path, values } | Predicate::NoneOf { path, values } => {
            let read = read(environment, path)?;
            Some(checked(read, &values.iter().collect::<Vec<_>>()))
        }
        _ => None,
    }
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

/// A comparison of an attribute with another fact, expanded per variant (negated when `negate`);
/// `None` to leave it as written.
fn compare_facts<E: TypeEnvironment>(
    environment: &E,
    left: &Operand,
    op: CompareOp,
    right: &Operand,
    negate: bool,
) -> Option<Predicate> {
    let as_read = |operand: &Operand| match operand {
        Operand::Fact(path) => read(environment, path),
        _ => None,
    };
    match (as_read(left), as_read(right), left, right) {
        (Some(read), None, _, Operand::Fact(other)) => expand(&read, other, flipped(op), negate),
        (None, Some(read), Operand::Fact(other), _) => expand(&read, other, op, negate),
        _ => None,
    }
}

/// `other op <attribute>` for each variant that fills the attribute, as one branch per variant:
/// `all: [fact: {any_of: [V]}, other op <V's value>]` — the comparison negated when `negate` —
/// joined by `any:`. A variant that leaves the attribute unfilled is in no branch, either way: the
/// evaluator answers `Unknown` there. `None` past [`MAX_PREDICATE_NODES`] — one node for the join
/// and three per branch, four when negated.
fn expand(read: &Read, other: &FactPath, op: CompareOp, negate: bool) -> Option<Predicate> {
    let per_branch = if negate { 4 } else { 3 };
    if 1 + per_branch * read.attributes.variants.len() > MAX_PREDICATE_NODES {
        return None;
    }
    let branches: Vec<Predicate> = read
        .attributes
        .variants
        .iter()
        .filter_map(|(variant, values)| {
            let value = values.get(&read.attribute)?;
            let comparison = Predicate::Compare {
                left: Operand::Fact(other.clone()),
                op,
                right: Operand::Literal(value.clone()),
                kind: CompareKind::Value,
            };
            Some(Predicate::All(vec![
                Predicate::AnyOf {
                    path: read.fact.clone(),
                    values: vec![FactValue::Text(variant.clone())],
                },
                if negate {
                    Predicate::Not(Box::new(comparison))
                } else {
                    comparison
                },
            ]))
        })
        .collect();
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
