//! The invariant check on each entity's data: `BrokenInvariant`, and the evaluator it shares.
//!
//! The Go rendering of [`crate::rust`]'s `invariant` module, with the same semantics: each
//! invariant is read in the three values the conformance interpreter reads it in — true, false, or
//! unknown where a value it reads is absent — and only false breaks it. An absent `Optional`, a
//! list position past the end, or `state`, which the data type does not hold, decides nothing.
//!
//! Go has no `Option` to chain through, so a read that may find nothing is written as statements:
//! a temporary holding the fact, assigned inside the `if` that found every step present. A
//! quantifier is a loop folding its body with Kleene conjunction or disjunction, which give the
//! same answer in any order — the one order Go does not keep, a map's, does not matter.
//!
//! The shared evaluator is its own leaf package, `types/invariant`, emitted only in a model where
//! some entity declares an invariant, so a model that declares none keeps every byte it had. An
//! invariant this emitter cannot evaluate is refused before anything is rendered ([`preflight`]),
//! naming the invariant, rather than dropped from the check.

use std::collections::BTreeSet;
use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedEntity, ResolvedField, ResolvedTypeRef};
use ess_domain::Primitive;
use ess_gen::{Artifact, Provenance};
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::predicate::{
    CompareOp, FoldOp, Operand, Predicate, Quantified, TextOp, TextOperand,
};

use super::layout::Layout;
use super::selection::go_string;
use super::{items, Emit};
use crate::failure::{TargetFailure, TargetFailureCause, TargetFailureCode};
use crate::plan::{CapabilityKind, SynthesisPlan, REGENERATE};

/// How deep a path may unwrap before the walk gives up — the compiler's own bound on a type chain.
const MAX_STEPS: usize = 64;

/// Whether any entity of the model declares an invariant, which is what brings the evaluator in.
pub(super) fn used(ir: &EssIr) -> bool {
    ir.entities()
        .values()
        .any(|entity| !entity.invariants.is_empty())
}

/// Refuses the target when an entity it emits declares an invariant this emitter cannot evaluate.
pub(super) fn preflight(
    ir: &EssIr,
    plan: &SynthesisPlan,
    layout: &Layout,
) -> Result<(), TargetFailure> {
    let mut causes = Vec::new();
    for entity in ir.entities().values() {
        if !plan.is_generated(CapabilityKind::EntityLifecycle, &entity.name.to_string()) {
            continue;
        }
        let package = layout.package_of(&entity.name);
        let emit = Emit::new(ir, layout, package, Some(layout.owner(&entity.name)));
        for (index, invariant) in entity.invariants.iter().enumerate() {
            let mut check = Check::new(&emit, entity, layout.package_names());
            if let Err(reason) = check.predicate(&invariant.predicate) {
                causes.push(TargetFailureCause::new(
                    TargetFailureCode::MissingRepresentation,
                    vec![format!("entities.{}.invariants.{index}", entity.name)],
                    format!(
                        "the generated invariant check cannot evaluate `{}`: {reason}",
                        invariant.statement
                    ),
                ));
            }
        }
    }
    if causes.is_empty() {
        Ok(())
    } else {
        Err(TargetFailure::new(ir, crate::Target::Go, plan, causes))
    }
}

/// The field names an entity's data struct has taken before its first field: `BrokenInvariant`,
/// the check's method, where the entity declares an invariant. Go gives a struct one namespace for
/// fields and methods, so a field exporting to that name takes the struct's usual repair
/// (`BrokenInvariant_`) rather than the method's.
pub(super) fn data_taken(entity: &ResolvedEntity) -> std::collections::BTreeMap<String, usize> {
    let mut taken = std::collections::BTreeMap::new();
    if !entity.invariants.is_empty() {
        taken.insert("BrokenInvariant".to_owned(), 0);
    }
    taken
}

/// The Go member of an entity's data struct that holds `field` — the identity or a stored field —
/// as the struct declares it.
pub(super) fn data_member(entity: &ResolvedEntity, field: &str) -> String {
    let mut taken = data_taken(entity);
    for stored in std::iter::once(&entity.identity).chain(&entity.fields) {
        let ident = items::field_ident(&mut taken, &stored.name);
        if stored.name == field {
            return ident;
        }
    }
    unreachable!("`{field}` is a field of `{}`", entity.name)
}

/// The doc lines naming each declared invariant and the method that checks it.
pub(super) fn doc(out: &mut String, entity: &ResolvedEntity, data: &str) {
    if entity.invariants.is_empty() {
        return;
    }
    out.push_str("//\n");
    for invariant in &entity.invariants {
        let _ = writeln!(
            out,
            "// Every value satisfies `{}` — checked by [{data}.BrokenInvariant].",
            invariant.statement
        );
    }
}

/// `func (d <Entity>Data) BrokenInvariant() (string, bool)`, for an entity that declares at least
/// one invariant.
///
/// # Panics
///
/// If an invariant cannot be rendered — [`preflight`] refuses such a model before any rendering
/// starts, so reaching it is a defect in this crate.
pub(super) fn check(out: &mut String, emit: &Emit<'_>, entity: &ResolvedEntity) {
    if entity.invariants.is_empty() {
        return;
    }
    let data = emit.layout.data(&entity.name);
    let mut check = Check::new(emit, entity, emit.layout.package_names());
    let receiver = check.receiver.clone();
    let mut body = Lines::new(1);
    for invariant in &entity.invariants {
        let truth = check
            .predicate(&invariant.predicate)
            .unwrap_or_else(|reason| {
                panic!(
                    "preflight admitted `{}`, which cannot be rendered: {reason}",
                    invariant.statement
                )
            });
        body.extend(std::mem::take(&mut check.prelude));
        let broken = emit.qualify(emit.layout.invariant(), "Broken");
        body.push(&format!("if {broken}({truth}) {{"));
        body.push(&format!(
            "\treturn {}, true",
            go_string(&invariant.statement)
        ));
        body.push("}");
    }
    let _ = writeln!(
        out,
        "\n// BrokenInvariant is the first declared invariant of `{}` this value breaks, as the\n// \
         specification declares it, and true; or \"\" and false when it breaks none.\n//\n// An \
         invariant is broken only when it is false of this value. One that reads something\n// \
         absent — an empty `Optional`, a list position past the end, or `state`, which this\n// \
         type does not hold — decides nothing, as the conformance interpreter reads it.\nfunc \
         ({receiver} {data}) BrokenInvariant() (string, bool) {{\n{}\treturn \"\", false\n}}",
        entity.name,
        body.text()
    );
}

/// The `invariant` package: the evaluator, in a model where some entity declares an invariant.
pub(super) fn package(ir: &EssIr, layout: &Layout, provenance: &Provenance) -> Option<Artifact> {
    if !used(ir) {
        return None;
    }
    let mut out = provenance.commented_for("//", REGENERATE);
    out.push('\n');
    out.push_str(RUNTIME);
    Some(Artifact::new(layout.invariant().file(), out))
}

/// Statements, each indented by tabs to its depth.
pub(super) struct Lines {
    depth: usize,
    lines: Vec<String>,
}

impl Lines {
    /// No statements yet, at `depth` tabs.
    pub(super) fn new(depth: usize) -> Self {
        Self {
            depth,
            lines: Vec::new(),
        }
    }

    /// One statement at the current depth; a leading tab in `line` indents it one deeper.
    pub(super) fn push(&mut self, line: &str) {
        self.lines
            .push(format!("{}{line}", "\t".repeat(self.depth)));
    }

    /// Statements already indented.
    pub(super) fn extend(&mut self, lines: Vec<String>) {
        self.lines.extend(lines);
    }

    /// One block deeper.
    pub(super) fn open(&mut self, line: &str) {
        self.push(line);
        self.depth += 1;
    }

    /// One block shallower, closing it with `line`.
    pub(super) fn close(&mut self, line: &str) {
        self.depth -= 1;
        self.push(line);
    }

    /// One level deeper, with no line of its own: the body of a `case` or an `else`.
    pub(super) fn indent(&mut self) {
        self.depth += 1;
    }

    /// The current depth.
    pub(super) fn depth(&self) -> usize {
        self.depth
    }

    /// Every statement, one per line.
    pub(super) fn text(&self) -> String {
        self.lines.iter().fold(String::new(), |mut out, line| {
            let _ = writeln!(out, "{line}");
            out
        })
    }
}

/// One step a read takes only where it finds something: the condition, and the variable bound to
/// what it found.
struct Guard {
    condition: String,
    bind: String,
    found: String,
}

/// What a path reads, once walked to its end.
enum Walked {
    /// A value of the terminal type, which is neither a newtype nor an `Optional`, reached through
    /// `guards`.
    Value {
        guards: Vec<Guard>,
        expression: String,
        terminal: ResolvedTypeRef,
    },
    /// A `.count`, as an `invariant.Fact` expression reached through `guards`.
    Count {
        guards: Vec<Guard>,
        expression: String,
    },
    /// Something the data type does not hold — `state` — which reads as absent.
    Absent,
}

/// One quantifier's binder in scope: its authored name, its Go variable, and its element type.
struct Binder {
    name: String,
    variable: String,
    element: ResolvedTypeRef,
}

/// The fact a leaf reads: an expression, or one arm per enum variant.
enum Leaf {
    Expression(String),
    Variants(Vec<(String, String)>),
}

/// Renders the invariants of one entity into statements and `invariant.Truth` expressions.
struct Check<'a> {
    emit: &'a Emit<'a>,
    entity: &'a ResolvedEntity,
    binders: Vec<Binder>,
    /// Statements the next truth expression reads, in order.
    prelude: Vec<String>,
    /// The depth the prelude is written at.
    depth: usize,
    /// Names no local may take: the packages the file imports.
    reserved: BTreeSet<String>,
    /// The next temporary's number.
    next: usize,
    /// The receiver's name.
    receiver: String,
}

impl<'a> Check<'a> {
    fn new(emit: &'a Emit<'a>, entity: &'a ResolvedEntity, reserved: BTreeSet<String>) -> Self {
        let receiver = fresh(&reserved, "d");
        Self {
            emit,
            entity,
            binders: Vec::new(),
            prelude: Vec::new(),
            depth: 1,
            reserved,
            next: 0,
            receiver,
        }
    }

    /// A fresh local named from `base`.
    fn local(&mut self, base: &str) -> String {
        let candidate = format!("{base}{}", self.next);
        self.next += 1;
        fresh(&self.reserved, &candidate)
    }

    /// `invariant.<name>`, imported.
    fn iv(&self, item: &str) -> String {
        self.emit.qualify(self.emit.layout.invariant(), item)
    }

    /// One statement of the prelude at the current depth.
    fn line(&mut self, line: &str) {
        self.prelude
            .push(format!("{}{line}", "\t".repeat(self.depth)));
    }

    /// One predicate as an `invariant.Truth` expression, its reads in the prelude.
    fn predicate(&mut self, predicate: &Predicate) -> Result<String, String> {
        Ok(match predicate {
            Predicate::Always => self.iv("True"),
            Predicate::Never => self.iv("False"),
            Predicate::All(children) => format!("{}({})", self.iv("All"), self.children(children)?),
            Predicate::Any(children) => format!("{}({})", self.iv("Any"), self.children(children)?),
            Predicate::Not(inner) => format!("{}({})", self.iv("Not"), self.predicate(inner)?),
            Predicate::Compare {
                left, op, right, ..
            } => self.compare(left, *op, right)?,
            Predicate::Truthy(path) => format!("{}({})", self.iv("Truthy"), self.fact(path)?),
            Predicate::Defined(path) => self.defined(path)?,
            Predicate::AnyOf { path, values } => {
                let fact = self.fact(path)?;
                format!("{}({fact}{})", self.iv("AnyOf"), self.literals(values))
            }
            Predicate::NoneOf { path, values } => {
                let fact = self.fact(path)?;
                format!("{}({fact}{})", self.iv("NoneOf"), self.literals(values))
            }
            Predicate::TextMatch { path, op, value } => {
                let op = match op {
                    TextOp::StartsWith => "StartsWith",
                    TextOp::EndsWith => "EndsWith",
                    TextOp::Contains => "Contains",
                };
                let fact = self.fact(path)?;
                let literal = match value {
                    TextOperand::Literal(FactValue::Text(text)) => {
                        format!("{}({})", self.iv("Text"), go_string(text))
                    }
                    TextOperand::Literal(_) => self.iv("Absent"),
                    // No invariant reads a parameter or an input (beyond10x/ess#200).
                    TextOperand::Fact { .. } => {
                        return Err(format!(
                            "`{predicate}` compares with a parameter or an input, which no \
                             invariant reads"
                        ))
                    }
                };
                format!(
                    "{}({fact}, {}, {literal})",
                    self.iv("TextMatch"),
                    self.iv(op)
                )
            }
            Predicate::FoldMatch { path, op, values } => {
                // Either spelling folds against every text literal it lists; a literal that is not
                // text matches nothing, as the interpreter reads it.
                let (FoldOp::EqualsIgnoreCase | FoldOp::InIgnoreCase) = op;
                let fact = self.fact(path)?;
                let texts = values.iter().filter_map(FactValue::as_text).fold(
                    String::new(),
                    |mut texts, text| {
                        let _ = write!(texts, ", {}", go_string(text));
                        texts
                    },
                );
                format!("{}({fact}{texts})", self.iv("FoldMatch"))
            }
            Predicate::Forall(quantified) => self.quantified(true, quantified)?,
            Predicate::Exists(quantified) => self.quantified(false, quantified)?,
            // The shared invariant evaluator compares no keys across a list's elements: refused by
            // name rather than dropped from the check.
            Predicate::Distinct(_) => {
                return Err(format!(
                    "`{predicate}` requires distinct list members, which the generated invariant \
                     check does not evaluate"
                ))
            }
            // Validation admits a calendar window only in a command guard, and the shared invariant
            // evaluator reads no window: refused by name rather than rendered as something else.
            Predicate::Window(window) => {
                return Err(format!(
                    "`{window}` is a calendar window, which the generated invariant check does not \
                     evaluate"
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

    /// `, <literal>` for each literal operand.
    fn literals(&self, values: &[FactValue]) -> String {
        values.iter().fold(String::new(), |mut out, value| {
            let _ = write!(out, ", {}", self.literal(value));
            out
        })
    }

    /// A literal operand, as an `invariant.Fact` expression.
    fn literal(&self, value: &FactValue) -> String {
        match value {
            FactValue::Bool(flag) => format!("{}({flag})", self.iv("Bool")),
            FactValue::Number(number) => {
                format!(
                    "{}({})",
                    self.iv("NumberOf"),
                    go_string(&number.to_string())
                )
            }
            FactValue::Text(text) => format!("{}({})", self.iv("Text"), go_string(text)),
        }
    }

    /// A comparison, with the two facts about its operands the interpreter orders by.
    fn compare(
        &mut self,
        left: &Operand,
        op: CompareOp,
        right: &Operand,
    ) -> Result<String, String> {
        let mut instant = false;
        let mut bytes = true;
        let mut operands = Vec::new();
        for operand in [left, right] {
            operands.push(match operand {
                Operand::Literal(value) => self.literal(value),
                // The shared invariant evaluator moves no value by a constant (A2): refused by
                // name rather than read as the text it is spelled like.
                Operand::Offset(offset) => {
                    return Err(format!(
                        "`{offset}` moves a fact by a constant, which the generated invariant \
                         check does not evaluate"
                    ))
                }
                // The UTF-8 byte length of a `String` (decision 11): `len` of a string that is
                // UTF-8; one that is not — the bytes a lone surrogate leaves — decides nothing.
                Operand::Derived(derived) => {
                    let parent = derived.parent();
                    match self.walk(parent)? {
                        Walked::Value {
                            mut guards,
                            expression,
                            terminal:
                                ResolvedTypeRef::Primitive {
                                    name: Primitive::String,
                                },
                        } => {
                            self.emit.import("unicode/utf8");
                            let bind = self.local("s");
                            guards.push(Guard {
                                condition: format!("utf8.ValidString({expression})"),
                                found: expression,
                                bind: bind.clone(),
                            });
                            let expression = format!("{}(len({bind}))", self.iv("Count"));
                            self.walked_fact(Walked::Count { guards, expression }, parent)?
                        }
                        Walked::Absent => self.iv("Absent"),
                        _ => {
                            return Err(format!(
                                "`{derived}` measures `{parent}`, which is no String"
                            ))
                        }
                    }
                }
                Operand::Fact(path) => {
                    let walked = self.walk(path)?;
                    if let Walked::Value { terminal, .. } = &walked {
                        let primitive = primitive(terminal);
                        instant |= primitive == Some(Primitive::Timestamp);
                        bytes &= primitive != Some(Primitive::Duration);
                    }
                    self.walked_fact(walked, path)?
                }
            });
        }
        let op = match op {
            CompareOp::Eq => "Eq",
            CompareOp::Ne => "Ne",
            CompareOp::Lt => "Lt",
            CompareOp::Le => "Le",
            CompareOp::Gt => "Gt",
            CompareOp::Ge => "Ge",
        };
        Ok(format!(
            "{}({}, {}, {}, {instant}, {bytes})",
            self.iv("Compare"),
            operands[0],
            self.iv(op),
            operands[1]
        ))
    }

    /// `defined(path)`: whether every step of the path found something.
    fn defined(&mut self, path: &FactPath) -> Result<String, String> {
        let guards = match self.walk(path)? {
            Walked::Value { guards, .. } | Walked::Count { guards, .. } => guards,
            Walked::Absent => return Ok(self.iv("False")),
        };
        if guards.is_empty() {
            return Ok(self.iv("True"));
        }
        let variable = self.local("t");
        let (false_, true_) = (self.iv("False"), self.iv("True"));
        self.line(&format!("{variable} := {false_}"));
        let depth = self.open_guards(&guards);
        if let Some(last) = guards.last() {
            self.line(&format!("_ = {}", last.bind));
        }
        self.line(&format!("{variable} = {true_}"));
        self.close_guards(depth);
        Ok(variable)
    }

    /// `forall` (`all`) or `exists`: the body folded over the collection's values under a fresh
    /// binder; unknown where the collection is absent.
    fn quantified(&mut self, all: bool, quantified: &Quantified) -> Result<String, String> {
        let (guards, expression, terminal) = match self.walk(&quantified.over)? {
            Walked::Value {
                guards,
                expression,
                terminal,
            } => (guards, expression, terminal),
            Walked::Count { .. } | Walked::Absent => {
                return Err(format!("`{}` is not a collection", quantified.over));
            }
        };
        let element = match &terminal {
            ResolvedTypeRef::List { of } => (**of).clone(),
            ResolvedTypeRef::Map { value, .. } => (**value).clone(),
            other => {
                return Err(format!(
                    "`{}` is `{other}`, not a collection",
                    quantified.over
                ))
            }
        };
        let (fold, empty) = if all {
            (self.iv("All"), self.iv("True"))
        } else {
            (self.iv("Any"), self.iv("False"))
        };
        let variable = self.local("t");
        let item = fresh(&self.reserved, &format!("item{}", self.binders.len()));
        if guards.is_empty() {
            self.line(&format!("{variable} := {empty}"));
        } else {
            let unknown = self.iv("Unknown");
            self.line(&format!("{variable} := {unknown}"));
        }
        let depth = self.open_guards(&guards);
        if !guards.is_empty() {
            self.line(&format!("{variable} = {empty}"));
        }
        self.line(&format!("for _, {item} := range {expression} {{"));
        self.depth += 1;
        self.line(&format!("_ = {item}"));
        self.binders.push(Binder {
            name: quantified.bind.clone(),
            variable: item,
            element,
        });
        let body = self.predicate(&quantified.body);
        self.binders.pop();
        let body = body?;
        self.line(&format!("{variable} = {fold}({variable}, {body})"));
        self.depth -= 1;
        self.line("}");
        self.close_guards(depth);
        Ok(variable)
    }

    /// Opens one `if` per guard, binding what each found; returns the depth to close back to.
    fn open_guards(&mut self, guards: &[Guard]) -> usize {
        let depth = self.depth;
        for guard in guards {
            self.line(&format!("if {} {{", guard.condition));
            self.depth += 1;
            self.line(&format!("{} := {}", guard.bind, guard.found));
        }
        depth
    }

    /// Closes the guards [`Self::open_guards`] opened.
    fn close_guards(&mut self, depth: usize) {
        while self.depth > depth {
            self.depth -= 1;
            self.line("}");
        }
    }

    /// The value a leaf reads, as an `invariant.Fact` expression.
    fn fact(&mut self, path: &FactPath) -> Result<String, String> {
        let walked = self.walk(path)?;
        self.walked_fact(walked, path)
    }

    fn walked_fact(&mut self, walked: Walked, path: &FactPath) -> Result<String, String> {
        let (guards, expression, leaf) = match walked {
            Walked::Value {
                guards,
                expression,
                terminal,
            } => {
                let leaf = self.leaf(&terminal, &expression, path)?;
                (guards, expression, leaf)
            }
            Walked::Count { guards, expression } => {
                (guards, expression.clone(), Leaf::Expression(expression))
            }
            Walked::Absent => return Ok(self.iv("Absent")),
        };
        if guards.is_empty() {
            if let Leaf::Expression(fact) = &leaf {
                return Ok(fact.clone());
            }
        }
        let variable = self.local("f");
        let fact_type = self.iv("Fact");
        self.line(&format!("var {variable} {fact_type}"));
        let depth = self.open_guards(&guards);
        match leaf {
            Leaf::Expression(fact) => self.line(&format!("{variable} = {fact}")),
            Leaf::Variants(arms) => {
                self.line(&format!("switch {expression}.(type) {{"));
                for (variant, spelled) in arms {
                    let text = self.iv("Text");
                    self.line(&format!("case {variant}:"));
                    self.line(&format!("\t{variable} = {text}({})", go_string(&spelled)));
                }
                self.line("}");
            }
        }
        self.close_guards(depth);
        Ok(variable)
    }

    /// The fact one present value of `terminal` reads, held in `at`.
    fn leaf(&self, terminal: &ResolvedTypeRef, at: &str, path: &FactPath) -> Result<Leaf, String> {
        let wrap = |function: &str, argument: String| {
            Leaf::Expression(format!("{}({argument})", self.iv(function)))
        };
        Ok(match terminal {
            ResolvedTypeRef::Primitive { name } => match name {
                Primitive::Boolean => wrap("Bool", at.to_owned()),
                Primitive::Integer => wrap("Integer", at.to_owned()),
                Primitive::Decimal => wrap("NumberOf", format!("{at}.Value()")),
                Primitive::String => wrap("Text", at.to_owned()),
                Primitive::Timestamp | Primitive::Duration | Primitive::Uuid => {
                    wrap("Text", format!("{at}.Value()"))
                }
                Primitive::Bytes => wrap("Bytes", at.to_owned()),
                Primitive::Binary64 | Primitive::Json => {
                    return Err(format!(
                        "`{path}` is a `{name}`, which the check does not read"
                    ));
                }
            },
            ResolvedTypeRef::Declared { name: handle } => {
                match &self.emit.ir.named_type(handle).body {
                    ResolvedBody::Enum { variants } => Leaf::Variants(
                        variants
                            .iter()
                            .map(|variant| {
                                (
                                    self.emit.reference_variant(handle.name(), variant.name()),
                                    variant.name().to_owned(),
                                )
                            })
                            .collect(),
                    ),
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

    /// Where a path starts — a binder, the identity or a field — as the expression holding it and
    /// its type; `None` for `state`, which the data type does not hold.
    fn root(&self, path: &FactPath) -> Result<Option<(String, ResolvedTypeRef)>, String> {
        let root = path.namespace();
        if let Some(binder) = self.binders.iter().rev().find(|binder| binder.name == root) {
            return Ok(Some((binder.variable.clone(), binder.element.clone())));
        }
        let stored: Vec<ResolvedField> = std::iter::once(&self.entity.identity)
            .chain(&self.entity.fields)
            .cloned()
            .collect();
        if let Some(field) = stored.iter().find(|field| field.name == root) {
            return Ok(Some((
                format!(
                    "{}.{}",
                    self.receiver,
                    data_member(self.entity, &field.name)
                ),
                field.type_ref.clone(),
            )));
        }
        if root == ess_domain::entity::EntitySpec::STATE {
            return Ok(None);
        }
        Err(format!(
            "`{path}` reads `{root}`, which this entity does not hold"
        ))
    }

    /// Walks a path from its root through every newtype and `Optional`, member, list position and
    /// `.count`, the way the compiler resolves it.
    fn walk(&mut self, path: &FactPath) -> Result<Walked, String> {
        let segments = path.segments();
        let Some((mut expression, mut current)) = self.root(path)? else {
            return Ok(Walked::Absent);
        };
        let mut guards = Vec::new();
        let mut position = 1;
        for _ in 0..MAX_STEPS {
            match current {
                ResolvedTypeRef::Optional { of } => {
                    let bind = self.local("v");
                    guards.push(Guard {
                        condition: format!("{expression} != nil"),
                        found: format!("*{expression}"),
                        bind: bind.clone(),
                    });
                    expression = bind;
                    current = *of;
                    continue;
                }
                ResolvedTypeRef::Declared { ref name } => {
                    if let ResolvedBody::Newtype { of, .. } = &self.emit.ir.named_type(name).body {
                        expression = format!("{expression}.Value()");
                        current = of.clone();
                        continue;
                    }
                }
                _ => {}
            }
            let Some(segment) = segments.get(position) else {
                return Ok(Walked::Value {
                    guards,
                    expression,
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
                    expression =
                        format!("{expression}.{}", items::member_ident(fields, &field.name));
                    current = field.type_ref.clone();
                }
                ResolvedTypeRef::Primitive {
                    name: Primitive::String,
                } if segment == "count" && last => {
                    self.emit.import("unicode/utf8");
                    let runes = "utf8.RuneCountInString";
                    return Ok(Walked::Count {
                        guards,
                        expression: format!("{}({runes}({expression}))", self.iv("Count")),
                    });
                }
                ResolvedTypeRef::List { .. } | ResolvedTypeRef::Map { .. }
                    if segment == "count" && last =>
                {
                    return Ok(Walked::Count {
                        guards,
                        expression: format!("{}(len({expression}))", self.iv("Count")),
                    });
                }
                ResolvedTypeRef::List { of } if ordinal(segment) => {
                    let bind = self.local("v");
                    guards.push(Guard {
                        condition: format!("len({expression}) > {segment}"),
                        found: format!("{expression}[{segment}]"),
                        bind: bind.clone(),
                    });
                    expression = bind;
                    current = (**of).clone();
                }
                other => return Err(format!("`{path}` cannot select `{segment}` from `{other}`")),
            }
            position += 1;
        }
        Err(format!("`{path}` unwraps more than {MAX_STEPS} types"))
    }
}

/// `base`, moved out of the way of every name in `reserved`.
pub(super) fn fresh(reserved: &BTreeSet<String>, base: &str) -> String {
    let mut candidate = base.to_owned();
    while reserved.contains(&candidate) {
        candidate.push('_');
    }
    candidate
}

/// The primitive a terminal type is, if it is one.
fn primitive(terminal: &ResolvedTypeRef) -> Option<Primitive> {
    match terminal {
        ResolvedTypeRef::Primitive { name } => Some(*name),
        _ => None,
    }
}

/// A list position as the compiler admits one: `0`, or digits without a leading zero.
fn ordinal(segment: &str) -> bool {
    segment == "0"
        || (!segment.starts_with('0')
            && !segment.is_empty()
            && segment.bytes().all(|byte| byte.is_ascii_digit()))
}

/// The evaluator, as its own package.
const RUNTIME: &str = include_str!("runtime/invariant.go");
