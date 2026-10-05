//! The invariant check on each entity's data: `broken_invariant`, and the evaluator it shares.
//!
//! An entity's `invariants:` are predicates the specification fully determines, so the check is
//! generated rather than owed. Each invariant becomes one Rust expression over the data type's own
//! fields, read in the three values the conformance interpreter reads it in
//! (`ess-conformance`'s `interpret/execute.rs`, `at_rest`): true, false, or unknown where a value
//! it reads is absent. Only false breaks an invariant, so an absent `Optional`, a list position
//! past the end, or `state` — which the data type does not hold — decides nothing, exactly as the
//! interpreter decides nothing about a field no outcome has written.
//!
//! The shared evaluator is one fixed module appended to the types crate's `primitives` module,
//! and only in a model where some entity declares an invariant: a model that declares none keeps
//! every byte it had.
//!
//! An invariant this emitter cannot spell as such an expression is refused before anything is
//! rendered ([`preflight`]), naming the invariant, rather than dropped from the check.

use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedEntity, ResolvedTypeRef};
use ess_domain::entity::Invariant;
use ess_domain::Primitive;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{CompareOp, FoldOp, Operand, Predicate, Quantified, TextOp};

use super::{name, Emit};
use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::plan::{CapabilityKind, SynthesisPlan};

/// How deep a path may unwrap before the walk gives up — the compiler's own bound on a type chain.
const MAX_STEPS: usize = 64;

/// Whether any entity of the model declares an invariant, which is what brings the evaluator in.
pub(super) fn used(ir: &EssIr) -> bool {
    ir.entities()
        .values()
        .any(|entity| !entity.invariants.is_empty())
}

/// Refuses the target when an entity it emits declares an invariant this emitter cannot evaluate.
///
/// Every cause names the invariant by its position and its declared text; no artifact accompanies
/// the failure.
pub(super) fn preflight(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &super::layout::Layout,
) -> Result<(), TargetFailure> {
    let mut causes = Vec::new();
    for entity in ir.entities().values() {
        if !plan.is_generated(CapabilityKind::EntityLifecycle, &entity.name.to_string()) {
            continue;
        }
        let emit = Emit {
            ir,
            layout,
            domain: layout.owner(&entity.name),
        };
        for (index, invariant) in entity.invariants.iter().enumerate() {
            if let Err(reason) = Check::new(&emit, entity).invariant(invariant) {
                causes.push(refusal(entity, index, invariant, &reason));
            }
        }
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, crate::Target::Rust, plan, causes))
    }
}

/// The cause naming one invariant the check cannot evaluate.
fn refusal(
    entity: &ResolvedEntity,
    index: usize,
    invariant: &Invariant,
    reason: &str,
) -> TargetFailureCause {
    TargetFailureCause::new(
        TargetFailureCode::MissingRepresentation,
        vec![format!("entities.{}.invariants.{index}", entity.name)],
        format!(
            "the generated invariant check cannot evaluate `{}`: {reason}",
            invariant.statement
        ),
    )
}

/// The doc lines naming each declared invariant and the method that checks it.
pub(super) fn doc(out: &mut String, entity: &ResolvedEntity, type_name: &str) {
    if entity.invariants.is_empty() {
        return;
    }
    out.push_str("///\n");
    for invariant in &entity.invariants {
        let _ = writeln!(
            out,
            "/// Every value satisfies `{}` — checked by [`{type_name}Data::broken_invariant`].",
            invariant.statement
        );
    }
}

/// `impl <Entity>Data { pub fn broken_invariant(&self) -> Option<&'static str> }`, for an entity
/// that declares at least one invariant.
///
/// # Panics
///
/// If an invariant cannot be rendered — [`preflight`] refuses such a model before any rendering
/// starts, so reaching it is a defect in this crate.
pub(super) fn check(out: &mut String, emit: &Emit<'_>, entity: &ResolvedEntity, type_name: &str) {
    if entity.invariants.is_empty() {
        return;
    }
    let _ = writeln!(
        out,
        "\nimpl {type_name}Data {{\n    /// The first declared invariant of `{}` this value breaks, as the specification \
         declares it,\n    /// or `None` when it breaks none.\n    ///\n    /// An invariant is \
         broken only when it is false of this value. One that reads something\n    /// absent \
         — an empty `Optional`, a list position past the end, or `state`, which this\n    /// \
         type does not hold — decides nothing, as the conformance interpreter reads it.\n    \
         pub fn broken_invariant(&self) -> Option<&'static str> {{\n        use \
         crate::primitives::invariant as iv;",
        entity.name
    );
    for invariant in &entity.invariants {
        let expression = Check::new(emit, entity)
            .invariant(invariant)
            .unwrap_or_else(|reason| {
                panic!(
                    "preflight admitted `{}`, which cannot be rendered: {reason}",
                    invariant.statement
                )
            });
        let _ = writeln!(
            out,
            "        if iv::broken({expression}) {{\n            return Some({:?});\n        }}",
            invariant.statement
        );
    }
    out.push_str("        None\n    }\n}\n");
}

/// Where a path has got to: a value that is certainly there — as a place, or as a reference — or
/// an `Option<&T>` expression once something on the way may be absent.
#[derive(Clone)]
enum Reach {
    /// A place expression that always holds a value, such as `self.total.amount`.
    Place(String),
    /// An expression of type `&T` that always holds a value: a quantifier's element.
    Ref(String),
    /// An expression of type `Option<&T>`.
    Maybe(String),
}

impl Reach {
    /// Takes one step that cannot fail: `.0` of a newtype, or a struct member.
    fn field(self, member: &str) -> Self {
        match self {
            Self::Place(at) | Self::Ref(at) => Self::Place(format!("{at}.{member}")),
            Self::Maybe(expression) => Self::Maybe(format!("{expression}.map(|v| &v.{member})")),
        }
    }

    /// Takes one step that may find nothing, spelled as a method returning `Option<&T>`.
    fn fallible(self, method: &str) -> Self {
        match self {
            Self::Place(at) | Self::Ref(at) => Self::Maybe(format!("{at}.{method}")),
            Self::Maybe(expression) => {
                Self::Maybe(format!("{expression}.and_then(|v| v.{method})"))
            }
        }
    }

    /// The value as an `Option<U>`: `place` spells `U` from a place holding the value, and
    /// `reference` from an expression of type `&T`.
    fn map(&self, place: impl Fn(&str) -> String, reference: impl Fn(&str) -> String) -> String {
        match self {
            Self::Place(at) => format!("Some({})", place(at)),
            Self::Ref(at) => format!("Some({})", reference(at)),
            Self::Maybe(expression) => format!("{expression}.map(|v| {})", reference("v")),
        }
    }

    /// The value as an `Option<U>`, where `reference` spells an `Option<U>` from a `&T`.
    fn and_then(
        &self,
        place: impl Fn(&str) -> String,
        reference: impl Fn(&str) -> String,
    ) -> String {
        match self {
            Self::Place(at) => place(at),
            Self::Ref(at) => reference(at),
            Self::Maybe(expression) => format!("{expression}.and_then(|v| {})", reference("v")),
        }
    }

    /// Whether the value is there, as a truth value.
    fn defined(&self) -> String {
        match self {
            Self::Place(_) | Self::Ref(_) => "Some(true)".to_owned(),
            Self::Maybe(expression) => format!("Some({expression}.is_some())"),
        }
    }

    /// The values of a list or a map, as an `Option<impl Iterator>`.
    fn items(&self, method: &str) -> String {
        match self {
            Self::Place(at) | Self::Ref(at) => format!("Some({at}.{method})"),
            Self::Maybe(expression) => format!("{expression}.map(|v| v.{method})"),
        }
    }
}

/// What a path reads, once walked to its end.
enum Walked {
    /// A value of the terminal type, which is neither a newtype nor an `Optional`.
    Value {
        reach: Reach,
        terminal: ResolvedTypeRef,
    },
    /// A `.count`: the expression is an `Option<iv::Fact>`.
    Count(String),
    /// Something the data type does not hold — `state` — which reads as absent.
    Absent,
}

/// One quantifier's binder in scope: its authored name, its Rust variable, and its element type.
struct Binder {
    name: String,
    variable: String,
    element: ResolvedTypeRef,
}

/// Renders one invariant of one entity.
struct Check<'a> {
    emit: &'a Emit<'a>,
    entity: &'a ResolvedEntity,
    binders: Vec<Binder>,
}

impl<'a> Check<'a> {
    fn new(emit: &'a Emit<'a>, entity: &'a ResolvedEntity) -> Self {
        Self {
            emit,
            entity,
            binders: Vec::new(),
        }
    }

    fn invariant(&mut self, invariant: &Invariant) -> Result<String, String> {
        self.predicate(&invariant.predicate)
    }

    /// One predicate as an `Option<bool>` expression.
    fn predicate(&mut self, predicate: &Predicate) -> Result<String, String> {
        Ok(match predicate {
            Predicate::Always => "Some(true)".to_owned(),
            Predicate::Never => "Some(false)".to_owned(),
            Predicate::All(children) => format!("iv::all([{}])", self.children(children)?),
            Predicate::Any(children) => format!("iv::any([{}])", self.children(children)?),
            Predicate::Not(inner) => format!("iv::not({})", self.predicate(inner)?),
            Predicate::Compare {
                left, op, right, ..
            } => self.compare(left, *op, right)?,
            Predicate::Truthy(path) => format!("iv::truthy({})", self.fact(path)?),
            Predicate::Defined(path) => match self.walk(path)? {
                Walked::Value { reach, .. } => reach.defined(),
                Walked::Count(expression) => format!("Some({expression}.is_some())"),
                Walked::Absent => "Some(false)".to_owned(),
            },
            Predicate::AnyOf { path, values } => {
                format!("iv::any_of({}, &[{}])", self.fact(path)?, literals(values))
            }
            Predicate::NoneOf { path, values } => {
                format!("iv::none_of({}, &[{}])", self.fact(path)?, literals(values))
            }
            Predicate::TextMatch { path, op, value } => {
                let op = match op {
                    TextOp::StartsWith => "StartsWith",
                    TextOp::EndsWith => "EndsWith",
                    TextOp::Contains => "Contains",
                };
                let literal = match value {
                    FactValue::Text(text) => format!("Some({text:?})"),
                    _ => "None".to_owned(),
                };
                format!(
                    "iv::text_match({}, iv::TextOp::{op}, {literal})",
                    self.fact(path)?
                )
            }
            Predicate::FoldMatch { path, op, values } => {
                // Either spelling folds against every text literal it lists; a literal that is not
                // text matches nothing, as the interpreter reads it.
                let (FoldOp::EqualsIgnoreCase | FoldOp::InIgnoreCase) = op;
                let texts: Vec<String> = values
                    .iter()
                    .filter_map(FactValue::as_text)
                    .map(|text| format!("{text:?}"))
                    .collect();
                format!(
                    "iv::fold_match({}, &[{}])",
                    self.fact(path)?,
                    texts.join(", ")
                )
            }
            Predicate::Forall(quantified) => self.quantified("forall", quantified)?,
            Predicate::Exists(quantified) => self.quantified("exists", quantified)?,
            // The shared invariant evaluator compares no keys across a list's elements: refused by
            // name rather than dropped from the check.
            Predicate::Distinct(_) => {
                return Err(format!(
                    "`{predicate}` requires distinct list members, which the generated invariant \
                     check does not evaluate"
                ))
            }
        })
    }

    fn children(&mut self, children: &[Predicate]) -> Result<String, String> {
        Ok(children
            .iter()
            .map(|child| self.predicate(child))
            .collect::<Result<Vec<_>, _>>()?
            .join(", "))
    }

    /// A comparison, with the two facts about its operands the interpreter orders by: whether a
    /// fact operand is a declared `Timestamp`, and whether every fact operand orders its text by
    /// bytes (all but a `Duration`).
    fn compare(
        &mut self,
        left: &Operand,
        op: CompareOp,
        right: &Operand,
    ) -> Result<String, String> {
        let mut instant = false;
        let mut bytes = true;
        let mut operand = |this: &mut Self, operand: &Operand| -> Result<String, String> {
            match operand {
                Operand::Literal(value) => Ok(literal(value)),
                // The shared invariant evaluator moves no value by a constant (A2): refused by
                // name rather than read as the text it is spelled like.
                Operand::Offset(offset) => Err(format!(
                    "`{offset}` moves a fact by a constant, which the generated invariant check \
                     does not evaluate"
                )),
                Operand::Fact(path) => {
                    let walked = this.walk(path)?;
                    if let Walked::Value { terminal, .. } = &walked {
                        let primitive = Self::primitive(terminal);
                        instant |= primitive == Some(Primitive::Timestamp);
                        bytes &= primitive != Some(Primitive::Duration);
                    }
                    this.walked_fact(walked, path)
                }
            }
        };
        let left = operand(self, left)?;
        let right = operand(self, right)?;
        let op = match op {
            CompareOp::Eq => "Eq",
            CompareOp::Ne => "Ne",
            CompareOp::Lt => "Lt",
            CompareOp::Le => "Le",
            CompareOp::Gt => "Gt",
            CompareOp::Ge => "Ge",
        };
        Ok(format!(
            "iv::compare({left}, iv::Op::{op}, {right}, {instant}, {bytes})"
        ))
    }

    /// `forall` or `exists`: the collection's values, and the body under a fresh binder.
    fn quantified(&mut self, which: &str, quantified: &Quantified) -> Result<String, String> {
        let (reach, terminal) = match self.walk(&quantified.over)? {
            Walked::Value { reach, terminal } => (reach, terminal),
            Walked::Count(_) | Walked::Absent => {
                return Err(format!("`{}` is not a collection", quantified.over));
            }
        };
        let (element, method) = match &terminal {
            ResolvedTypeRef::List { of } => ((**of).clone(), "iter()"),
            ResolvedTypeRef::Map { value, .. } => ((**value).clone(), "values()"),
            other => {
                return Err(format!(
                    "`{}` is `{other}`, not a collection",
                    quantified.over
                ))
            }
        };
        let items = reach.items(method);
        let variable = format!("item{}", self.binders.len());
        self.binders.push(Binder {
            name: quantified.bind.clone(),
            variable: variable.clone(),
            element,
        });
        let body = self.predicate(&quantified.body);
        self.binders.pop();
        Ok(format!("iv::{which}({items}, |{variable}| {})", body?))
    }

    /// The value a leaf reads, as an `Option<iv::Fact>` expression.
    fn fact(&mut self, path: &FactPath) -> Result<String, String> {
        let walked = self.walk(path)?;
        self.walked_fact(walked, path)
    }

    fn walked_fact(&self, walked: Walked, path: &FactPath) -> Result<String, String> {
        let (reach, terminal) = match walked {
            Walked::Value { reach, terminal } => (reach, terminal),
            Walked::Count(expression) => return Ok(expression),
            Walked::Absent => return Ok("None".to_owned()),
        };
        Ok(match &terminal {
            ResolvedTypeRef::Primitive { name } => match name {
                Primitive::Boolean => reach.map(
                    |at| format!("iv::Fact::Bool({at})"),
                    |at| format!("iv::Fact::Bool(*{at})"),
                ),
                Primitive::Integer => reach.map(
                    |at| format!("iv::Fact::integer({at})"),
                    |at| format!("iv::Fact::integer(*{at})"),
                ),
                Primitive::Decimal => {
                    let number = |at: &str| format!("iv::Fact::number(&{at}.0)");
                    reach.and_then(number, number)
                }
                Primitive::String => reach.map(
                    |at| format!("iv::Fact::text(&{at})"),
                    |at| format!("iv::Fact::text({at})"),
                ),
                Primitive::Timestamp | Primitive::Duration | Primitive::Uuid => {
                    let text = |at: &str| format!("iv::Fact::text(&{at}.0)");
                    reach.map(text, text)
                }
                Primitive::Bytes => reach.map(
                    |at| format!("iv::Fact::bytes(&{at})"),
                    |at| format!("iv::Fact::bytes({at})"),
                ),
                Primitive::Binary64 | Primitive::Json => {
                    return Err(format!(
                        "`{path}` is a `{name}`, which the check does not read"
                    ));
                }
            },
            ResolvedTypeRef::Declared { name: handle } => {
                match &self.emit.ir.named_type(handle).body {
                    ResolvedBody::Enum { variants } => {
                        let enumeration = self.emit.reference(handle);
                        let arms: Vec<String> = variants
                            .iter()
                            .map(|variant| {
                                format!(
                                    "{enumeration}::{} => {:?}",
                                    name::pascal(variant.name()),
                                    variant.name()
                                )
                            })
                            .collect();
                        let names = |at: &str| {
                            format!("iv::Fact::text(match {at} {{ {} }})", arms.join(", "))
                        };
                        reach.map(names, names)
                    }
                    _ => {
                        return Err(format!(
                            "`{path}` reaches `{}`, which holds no single value",
                            handle.name()
                        ))
                    }
                }
            }
            other => {
                return Err(format!(
                    "`{path}` reaches `{other}`, which holds no single value"
                ))
            }
        })
    }

    /// The primitive a terminal type is, if it is one.
    fn primitive(terminal: &ResolvedTypeRef) -> Option<Primitive> {
        match terminal {
            ResolvedTypeRef::Primitive { name } => Some(*name),
            _ => None,
        }
    }

    /// Walks a path from its root — a binder, the identity or a field — through every newtype and
    /// `Optional`, member, list position and `.count`, the way the compiler resolves it.
    fn walk(&self, path: &FactPath) -> Result<Walked, String> {
        let segments = path.segments();
        let root = path.namespace();
        let (mut reach, mut current) =
            if let Some(binder) = self.binders.iter().rev().find(|binder| binder.name == root) {
                (Reach::Ref(binder.variable.clone()), binder.element.clone())
            } else if let Some(field) = std::iter::once(&self.entity.identity)
                .chain(&self.entity.fields)
                .find(|field| field.name == root)
            {
                (
                    Reach::Place(format!("self.{}", name::value_ident(&field.name))),
                    field.type_ref.clone(),
                )
            } else if root == ess_domain::entity::EntitySpec::STATE {
                return Ok(Walked::Absent);
            } else {
                return Err(format!(
                    "`{path}` reads `{root}`, which this entity does not hold"
                ));
            };
        let mut position = 1;
        for _ in 0..MAX_STEPS {
            match current {
                ResolvedTypeRef::Optional { of } => {
                    reach = reach.fallible("as_ref()");
                    current = *of;
                    continue;
                }
                ResolvedTypeRef::Declared { ref name } => {
                    if let ResolvedBody::Newtype { of, .. } = &self.emit.ir.named_type(name).body {
                        reach = reach.field("0");
                        current = of.clone();
                        continue;
                    }
                }
                _ => {}
            }
            let Some(segment) = segments.get(position) else {
                return Ok(Walked::Value {
                    reach,
                    terminal: current,
                });
            };
            let last = position + 1 == segments.len();
            match &current {
                ResolvedTypeRef::Declared { name } => {
                    let ResolvedBody::Struct { fields, .. } = &self.emit.ir.named_type(name).body
                    else {
                        return Err(format!(
                            "`{path}` selects `{segment}` from `{}`",
                            name.name()
                        ));
                    };
                    let field = fields
                        .iter()
                        .find(|field| field.name == *segment)
                        .ok_or_else(|| format!("`{}` has no field `{segment}`", name.name()))?;
                    reach = reach.field(&name::value_ident(&field.name));
                    current = field.type_ref.clone();
                }
                ResolvedTypeRef::Primitive {
                    name: Primitive::String,
                } if segment == "count" && last => {
                    let length = |at: &str| format!("iv::Fact::count({at}.chars().count())");
                    return Ok(Walked::Count(reach.map(length, length)));
                }
                ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. }
                    if segment == "count" && last =>
                {
                    let length = |at: &str| format!("iv::Fact::count({at}.len())");
                    return Ok(Walked::Count(reach.map(length, length)));
                }
                ResolvedTypeRef::List { of } if ordinal(segment) => {
                    reach = reach.fallible(&format!("get({segment})"));
                    current = (**of).clone();
                }
                other => return Err(format!("`{path}` cannot select `{segment}` from `{other}`")),
            }
            position += 1;
        }
        Err(format!("`{path}` unwraps more than {MAX_STEPS} types"))
    }
}

/// A list position as the compiler admits one: `0`, or digits without a leading zero.
fn ordinal(segment: &str) -> bool {
    segment == "0"
        || (!segment.starts_with('0')
            && !segment.is_empty()
            && segment.bytes().all(|byte| byte.is_ascii_digit()))
}

/// A literal operand, as an `Option<iv::Fact>` expression.
fn literal(value: &FactValue) -> String {
    match value {
        FactValue::Bool(flag) => format!("Some(iv::Fact::Bool({flag}))"),
        FactValue::Number(number) => format!("iv::Fact::number({:?})", number.to_string()),
        FactValue::Text(text) => format!("Some(iv::Fact::text({text:?}))"),
    }
}

/// Literal operands, comma separated.
fn literals(values: &[FactValue]) -> String {
    values.iter().map(literal).collect::<Vec<_>>().join(", ")
}

/// The evaluator every generated check calls, appended to the `primitives` module of a model in
/// which some entity declares an invariant.
pub(super) fn runtime(ir: &EssIr) -> &'static str {
    if used(ir) {
        RUNTIME
    } else {
        ""
    }
}

const RUNTIME: &str = r#"
/// The three-valued reading every generated `broken_invariant` shares.
///
/// `Some(true)` holds, `Some(false)` is broken and `None` is unknown: a value the invariant reads
/// is absent, so it decides nothing. Numbers compare by their exact decimal value, a `Timestamp` by
/// the RFC 3339 instant it names, other text by its UTF-8 bytes, and `Bytes` as padded base64 —
/// the reading the conformance interpreter gives the same values.
pub mod invariant {
    use std::borrow::Cow;
    use std::cmp::Ordering;

    /// A number by its exact decimal value: `digits × 10^exponent`, with no leading or trailing
    /// zero in `digits`, and zero as no digits at all — so equal values are equal here.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub struct Number {
        negative: bool,
        digits: String,
        exponent: i128,
    }

    impl Number {
        /// Reads a decimal spelling such as `-10.50` or `15e-1`, or `None` when it is not one.
        pub fn parse(text: &str) -> Option<Self> {
            let (negative, rest) = match text.as_bytes().first() {
                Some(b'-') => (true, &text[1..]),
                Some(b'+') => (false, &text[1..]),
                _ => (false, text),
            };
            let (mantissa, exponent) = match rest.find(['e', 'E']) {
                Some(at) => (&rest[..at], rest[at + 1..].parse::<i128>().ok()?),
                None => (rest, 0),
            };
            let (whole, fraction) = mantissa.split_once('.').unwrap_or((mantissa, ""));
            let digit = |part: &str| part.bytes().all(|byte| byte.is_ascii_digit());
            if (whole.is_empty() && fraction.is_empty()) || !digit(whole) || !digit(fraction) {
                return None;
            }
            let all = format!("{whole}{fraction}");
            let leading = all.trim_start_matches('0');
            let significant = leading.trim_end_matches('0');
            if significant.is_empty() {
                return Some(Self { negative: false, digits: String::new(), exponent: 0 });
            }
            let dropped = i128::try_from(leading.len() - significant.len()).ok()?;
            let scale = i128::try_from(fraction.len()).ok()?;
            Some(Self {
                negative,
                digits: significant.to_owned(),
                exponent: exponent.checked_sub(scale)?.checked_add(dropped)?,
            })
        }

        fn is_zero(&self) -> bool {
            self.digits.is_empty()
        }

        fn sign(&self) -> i8 {
            if self.is_zero() {
                0
            } else if self.negative {
                -1
            } else {
                1
            }
        }

        /// Where the most significant digit sits.
        fn order(&self) -> i128 {
            i128::try_from(self.digits.len()).unwrap_or(i128::MAX).saturating_add(self.exponent)
        }
    }

    impl PartialOrd for Number {
        fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
            Some(self.cmp(other))
        }
    }

    impl Ord for Number {
        fn cmp(&self, other: &Self) -> Ordering {
            let sign = self.sign().cmp(&other.sign());
            if sign != Ordering::Equal || self.is_zero() {
                return sign;
            }
            let magnitude = self
                .order()
                .cmp(&other.order())
                .then_with(|| self.digits.as_str().cmp(other.digits.as_str()));
            if self.negative {
                magnitude.reverse()
            } else {
                magnitude
            }
        }
    }

    /// One value an invariant reads.
    #[derive(Debug, Clone, PartialEq, Eq)]
    pub enum Fact<'a> {
        /// A Boolean.
        Bool(bool),
        /// A number.
        Number(Number),
        /// Text: a `String`, an enum variant's name, or the rendering of a `Timestamp`,
        /// `Duration`, `Uuid` or `Bytes`.
        Text(Cow<'a, str>),
    }

    impl<'a> Fact<'a> {
        /// An `Integer`.
        pub fn integer(value: i64) -> Self {
            Self::Number(Number::parse(&value.to_string()).unwrap_or(Number {
                negative: false,
                digits: String::new(),
                exponent: 0,
            }))
        }

        /// The number of elements, or of Unicode scalar values in a text.
        pub fn count(value: usize) -> Self {
            Self::integer(i64::try_from(value).unwrap_or(i64::MAX))
        }

        /// A `Decimal` or a number literal by its spelling; `None` when it spells no number.
        pub fn number(text: &str) -> Option<Self> {
            Number::parse(text).map(Self::Number)
        }

        /// Text.
        pub fn text(text: &'a str) -> Self {
            Self::Text(Cow::Borrowed(text))
        }

        /// `Bytes`, as the padded base64 the wire carries them in.
        pub fn bytes(bytes: &[u8]) -> Self {
            const ALPHABET: &[u8; 64] =
                b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
            let mut out = String::with_capacity(bytes.len().div_ceil(3) * 4);
            for chunk in bytes.chunks(3) {
                let group = [chunk[0], *chunk.get(1).unwrap_or(&0), *chunk.get(2).unwrap_or(&0)];
                let indices = [
                    group[0] >> 2,
                    ((group[0] & 0b11) << 4) | (group[1] >> 4),
                    ((group[1] & 0b1111) << 2) | (group[2] >> 6),
                    group[2] & 0b11_1111,
                ];
                for (position, index) in indices.iter().enumerate() {
                    if position <= chunk.len() {
                        out.push(char::from(ALPHABET[usize::from(*index)]));
                    } else {
                        out.push('=');
                    }
                }
            }
            Self::Text(Cow::Owned(out))
        }
    }

    /// A comparison operator.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum Op {
        /// `==`
        Eq,
        /// `!=`
        Ne,
        /// `<`
        Lt,
        /// `<=`
        Le,
        /// `>`
        Gt,
        /// `>=`
        Ge,
    }

    impl Op {
        fn orders(self) -> bool {
            !matches!(self, Self::Eq | Self::Ne)
        }

        fn accepts(self, ordering: Ordering) -> bool {
            match self {
                Self::Eq => ordering == Ordering::Equal,
                Self::Ne => ordering != Ordering::Equal,
                Self::Lt => ordering == Ordering::Less,
                Self::Le => ordering != Ordering::Greater,
                Self::Gt => ordering == Ordering::Greater,
                Self::Ge => ordering != Ordering::Less,
            }
        }
    }

    /// A string operator.
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub enum TextOp {
        /// `starts_with`
        StartsWith,
        /// `ends_with`
        EndsWith,
        /// `contains`
        Contains,
    }

    /// Whether a value breaks the invariant: false, and not merely unknown.
    pub fn broken(truth: Option<bool>) -> bool {
        truth == Some(false)
    }

    /// Kleene conjunction: false dominates, then unknown.
    pub fn all<I: IntoIterator<Item = Option<bool>>>(truths: I) -> Option<bool> {
        let mut result = Some(true);
        for truth in truths {
            match truth {
                Some(false) => return Some(false),
                None => result = None,
                Some(true) => {}
            }
        }
        result
    }

    /// Kleene disjunction: true dominates, then unknown.
    pub fn any<I: IntoIterator<Item = Option<bool>>>(truths: I) -> Option<bool> {
        let mut result = Some(false);
        for truth in truths {
            match truth {
                Some(true) => return Some(true),
                None => result = None,
                Some(false) => {}
            }
        }
        result
    }

    /// Kleene negation: unknown stays unknown.
    pub fn not(truth: Option<bool>) -> Option<bool> {
        truth.map(|holds| !holds)
    }

    /// Every element satisfies `body`; an empty collection holds, an absent one is unknown.
    pub fn forall<I: IntoIterator>(
        items: Option<I>,
        body: impl FnMut(I::Item) -> Option<bool>,
    ) -> Option<bool> {
        all(items?.into_iter().map(body))
    }

    /// Some element satisfies `body`; an empty collection does not, an absent one is unknown.
    pub fn exists<I: IntoIterator>(
        items: Option<I>,
        body: impl FnMut(I::Item) -> Option<bool>,
    ) -> Option<bool> {
        any(items?.into_iter().map(body))
    }

    /// The fact is observed and truthy: `true`, a number other than zero, or text that is neither
    /// empty nor `false`.
    pub fn truthy(fact: Option<Fact<'_>>) -> Option<bool> {
        Some(match fact? {
            Fact::Bool(flag) => flag,
            Fact::Number(number) => !number.is_zero(),
            Fact::Text(text) => !text.is_empty() && text != "false",
        })
    }

    /// The observed fact equals one of `values`.
    pub fn any_of(fact: Option<Fact<'_>>, values: &[Option<Fact<'_>>]) -> Option<bool> {
        let fact = fact?;
        Some(values.iter().any(|value| value.as_ref() == Some(&fact)))
    }

    /// The observed fact equals none of `values`.
    pub fn none_of(fact: Option<Fact<'_>>, values: &[Option<Fact<'_>>]) -> Option<bool> {
        not(any_of(fact, values))
    }

    /// The observed text begins with, ends with or contains `literal`, byte for byte; any other
    /// observed value, or a literal that is not text, does not match.
    pub fn text_match(fact: Option<Fact<'_>>, op: TextOp, literal: Option<&str>) -> Option<bool> {
        let fact = fact?;
        Some(match (fact, literal) {
            (Fact::Text(text), Some(literal)) => match op {
                TextOp::StartsWith => text.as_bytes().starts_with(literal.as_bytes()),
                TextOp::EndsWith => text.as_bytes().ends_with(literal.as_bytes()),
                TextOp::Contains => text.contains(literal),
            },
            _ => false,
        })
    }

    /// The observed text equals one of `literals` under ASCII case folding.
    pub fn fold_match(fact: Option<Fact<'_>>, literals: &[&str]) -> Option<bool> {
        Some(match fact? {
            Fact::Text(text) => literals.iter().any(|literal| text.eq_ignore_ascii_case(literal)),
            _ => false,
        })
    }

    /// Compares two operands. `instant` says a fact operand is a declared `Timestamp`, ordered by
    /// the instant it names; `bytes` says every fact operand orders its text by UTF-8 bytes.
    pub fn compare(
        left: Option<Fact<'_>>,
        op: Op,
        right: Option<Fact<'_>>,
        instant: bool,
        bytes: bool,
    ) -> Option<bool> {
        let (left, right) = (left?, right?);
        if let (true, Fact::Text(left), Fact::Text(right)) = (instant, &left, &right) {
            if let (Some(left), Some(right)) = (rfc3339(left), rfc3339(right)) {
                return Some(op.accepts(left.cmp(&right)));
            }
        }
        match (&left, &right) {
            (Fact::Number(left), Fact::Number(right)) => Some(op.accepts(left.cmp(right))),
            (Fact::Text(left), Fact::Text(right)) if op.orders() => {
                (!instant && bytes).then(|| op.accepts(left.as_bytes().cmp(right.as_bytes())))
            }
            _ if op.orders() => None,
            _ => Some((left == right) == (op == Op::Eq)),
        }
    }

    /// The instant an RFC 3339 `date-time` names, as seconds from the epoch and nanoseconds.
    fn rfc3339(text: &str) -> Option<(i64, u32)> {
        let bytes = text.as_bytes();
        let digits = |from: usize, to: usize| -> Option<u32> {
            let slice = bytes.get(from..to)?;
            if slice.is_empty() || !slice.iter().all(u8::is_ascii_digit) {
                return None;
            }
            slice
                .iter()
                .try_fold(0u32, |total, digit| Some(total * 10 + u32::from(digit - b'0')))
        };
        let at = |index: usize, expected: &[u8]| bytes.get(index).is_some_and(|b| expected.contains(b));
        if !(at(4, b"-") && at(7, b"-") && at(10, b"Tt") && at(13, b":") && at(16, b":")) {
            return None;
        }
        let (year, month, day) = (i64::from(digits(0, 4)?), digits(5, 7)?, digits(8, 10)?);
        let leap = (year % 4 == 0 && year % 100 != 0) || year % 400 == 0;
        let length = match month {
            1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
            4 | 6 | 9 | 11 => 30,
            2 if leap => 29,
            2 => 28,
            _ => return None,
        };
        if day < 1 || day > length {
            return None;
        }
        let (hour, minute, second) = (digits(11, 13)?, digits(14, 16)?, digits(17, 19)?);
        if hour > 23 || minute > 59 || second > 59 {
            return None;
        }
        let mut position = 19;
        let mut nanos = 0u32;
        if at(position, b".") {
            let start = position + 1;
            let mut end = start;
            while bytes.get(end).is_some_and(u8::is_ascii_digit) {
                end += 1;
            }
            let width = end - start;
            if width == 0 || width > 9 {
                return None;
            }
            nanos = digits(start, end)? * 10u32.pow(u32::try_from(9 - width).ok()?);
            position = end;
        }
        let offset = match bytes.get(position..)? {
            b"Z" | b"z" => 0i64,
            [sign @ (b'+' | b'-'), _, _, b':', _, _] => {
                let (hours, minutes) = (digits(position + 1, position + 3)?, digits(position + 4, position + 6)?);
                if hours > 23 || minutes > 59 {
                    return None;
                }
                let magnitude = i64::from(hours * 3600 + minutes * 60);
                if *sign == b'-' {
                    -magnitude
                } else {
                    magnitude
                }
            }
            _ => return None,
        };
        let shifted = year - i64::from(month <= 2);
        let era = if shifted >= 0 { shifted } else { shifted - 399 } / 400;
        let year_of_era = shifted - era * 400;
        let month = i64::from(month);
        let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + i64::from(day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        let days = era * 146_097 + day_of_era - 719_468;
        Some((
            days * 86_400 + i64::from(hour * 3600 + minute * 60 + second) - offset,
            nanos,
        ))
    }
}
"#;

#[cfg(test)]
mod tests {
    use super::*;

    const MODEL: &str = "format: ess/16
system: demo
version: v1
domain: demo.ledger
entities:
  - name: demo.ledger.Account
    identity: {name: id, type: String}
    fields:
      - {name: count, type: Integer}
    invariants:
      - count >= 0
    lifecycle:
      initial: Open
      states: [Open]
      terminal: [Open]
";

    fn ir() -> EssIr {
        let spec = ess_domain::Specification::assemble([(
            ess_domain::system::Source::new("spec.yaml"),
            ess_domain::spec::RawSpecFile::parse(MODEL).expect("well formed"),
        )])
        .expect("validates");
        let mut sources = ess_compiler::source::SourceMap::new();
        sources.insert("spec.yaml", MODEL);
        ess_compiler::resolve::compile(&spec, &sources).expect("compiles")
    }

    /// A read the check cannot spell is a refusal naming the invariant, never a dropped check.
    ///
    /// Validation keeps every such read out of a compiled model, so the invariant here is put
    /// beside the admitted one by hand: a read of a field the entity does not hold.
    #[test]
    fn an_invariant_the_check_cannot_evaluate_is_refused_by_name() {
        let ir = ir();
        let plan = SynthesisPlan::of(&ir);
        let layout = super::super::layout::Layout::of(&ir);
        let mut entity = ir.entities().values().next().expect("one entity").clone();
        let emit = Emit {
            ir: &ir,
            layout: &layout,
            domain: layout.owner(&entity.name),
        };
        assert_eq!(
            Check::new(&emit, &entity).invariant(&entity.invariants[0]),
            Ok(
                "iv::compare(Some(iv::Fact::integer(self.count)), iv::Op::Ge, \
                iv::Fact::number(\"0\"), false, true)"
                    .to_owned()
            )
        );
        let unreadable = Invariant::parse("balance.amount >= 0").expect("parses");
        entity.invariants.push(unreadable.clone());
        let reason = Check::new(&emit, &entity)
            .invariant(&unreadable)
            .expect_err("the entity holds no `balance`");
        let cause = refusal(&entity, 1, &unreadable, &reason);
        assert_eq!(cause.code(), TargetFailureCode::MissingRepresentation);
        assert_eq!(
            cause.sources(),
            ["entities.demo.ledger.Account.invariants.1"]
        );
        assert_eq!(
            cause.detail(),
            "the generated invariant check cannot evaluate `balance.amount >= 0`: \
             `balance.amount` reads `balance`, which this entity does not hold"
        );
        assert!(preflight(&ir, &plan, &layout).is_ok());
    }
}
