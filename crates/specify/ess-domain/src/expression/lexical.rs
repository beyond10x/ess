//! The lexical predicate: a predicate as an authored source wrote it, before the source format and
//! the declarations decide what each bare word on the right of a comparison names
//! (`docs/design/expression-family-source22.md`, "Compatibility and the two-stage representation").
//!
//! `a == b` reads `b` as the text `"b"` in every format before `ess/22`. From `ess/22` the same
//! word names the root `b` of the place it is written in, when there is one and the left side is
//! not an enum declaring `b`. A quoted `"b"` is the text in every format. The reader in
//! `ess-primitives` keeps both as one text literal, so this tree keeps the one bit that tells them
//! apart, [`LexicalOperand::UnquotedText`], until [`crate::expression::resolve_lexical`] decides it.
//!
//! The tree is deliberately not a [`Predicate`]: it implements no serializer, is no IR condition,
//! is never evaluated and feeds no digest. The only ways out of it are [`LexicalPredicate::literal`],
//! the meaning every format before `ess/22` gives it, and the resolver.
//!
//! The two refusals below differ from this, which compiles, in one line each:
//!
//! ```
//! let lexical = ess_domain::expression::lexical::LexicalPredicate::parse_expression("a == b")
//!     .expect("parses");
//! let condition: ess_primitives::predicate::Predicate = lexical.literal();
//! assert_eq!(condition.to_string(), "a == b");
//! ```
//!
//! ```compile_fail
//! // lexical_not_serializable: a lexical predicate cannot be persisted.
//! fn persisted<T: serde::Serialize>(_: &T) {}
//! let lexical = ess_domain::expression::lexical::LexicalPredicate::parse_expression("a == b")
//!     .expect("parses");
//! persisted(&lexical);
//! ```
//!
//! ```compile_fail
//! // lexical_not_serializable: nor stands where a resolved predicate is required.
//! let lexical = ess_domain::expression::lexical::LexicalPredicate::parse_expression("a == b")
//!     .expect("parses");
//! let condition: ess_primitives::predicate::Predicate = lexical;
//! ```

use std::collections::BTreeMap;

use ess_primitives::error::ParseError;
use ess_primitives::facts::{FactPath, FactValue};
use ess_primitives::node::Node;
use ess_primitives::predicate::{CompareKind, CompareOp, Operand, Predicate, Quantified, Spelled};

use crate::command::{CommandSpec, OutcomeCondition, OutcomeName};
use crate::entity::EntitySpec;
use crate::name::QualifiedName;
use crate::system::FormatVersion;
use crate::types::NamedType;

/// The right-hand side of one comparison, as written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalOperand {
    /// Read the way every format reads it: a typed literal, a quoted text, a dotted fact, a binder
    /// in scope, or the explicit `{fact: …}` operand.
    Read(Operand),
    /// An unquoted, undotted word that no binder in scope names, or unquoted text spelled
    /// `<fact> ± <magnitude>`. Text before `ess/22`; from it, possibly a root fact (A1) or one
    /// constant offset of a fact (A2).
    UnquotedText(String),
    /// A dotted fact path with a `-` in it, `window.lower-5`: the fact it reads in every format,
    /// and from `ess/22` the offset it also spells where it names no fact (A2, rule 3a — a field
    /// name holds no `-`, so the dotted reading names nothing there).
    DottedSpelling(FactPath),
}

/// How one undecided right side was written, as the resolver is asked about it.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Spelling<'w> {
    /// An unquoted word or text ([`LexicalOperand::UnquotedText`]).
    Word(&'w str),
    /// A dotted path with a `-` in it ([`LexicalOperand::DottedSpelling`]).
    Dotted(&'w FactPath),
}

/// A predicate whose comparisons keep how their right-hand sides were written.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LexicalPredicate {
    /// One comparison. The left side is a fact path, as the compact grammar requires.
    Compare {
        /// Left-hand side.
        left: Operand,
        /// The operator.
        op: CompareOp,
        /// Right-hand side, as written.
        right: LexicalOperand,
        /// How it compares: as written, or tagged to compare instants (decision 2).
        kind: CompareKind,
    },
    /// Every child.
    All(Vec<LexicalPredicate>),
    /// At least one child.
    Any(Vec<LexicalPredicate>),
    /// Negation.
    Not(Box<LexicalPredicate>),
    /// A universal quantifier.
    Forall(Box<LexicalQuantified>),
    /// An existential quantifier.
    Exists(Box<LexicalQuantified>),
    /// A leaf with no right-hand side to decide: `always`, `never`, a bare path, `defined()`, a
    /// value list or a text operator.
    Leaf(Predicate),
}

/// What a resolver answers for one bare word: given the comparison's left side, its operator, the
/// word, and the quantifiers it sits in, outermost first, the operand it is.
pub(crate) type Decide<'a, 'f> =
    dyn FnMut(&Operand, CompareOp, Spelling<'_>, &[&'a LexicalQuantified]) -> Operand + 'f;

/// A quantifier whose body is still lexical.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LexicalQuantified {
    /// The collection.
    pub over: FactPath,
    /// The binder.
    pub bind: String,
    /// The body.
    pub body: LexicalPredicate,
}

impl LexicalPredicate {
    /// Reads either predicate form, keeping which right-hand sides were bare words.
    pub fn from_node(node: &Node) -> Result<Self, ParseError> {
        Predicate::from_node_spelled(node).map(Self::from_spelled)
    }

    /// Reads the compact form, keeping which right-hand sides were bare words.
    pub fn parse_expression(text: &str) -> Result<Self, ParseError> {
        Predicate::parse_expression_spelled(text).map(Self::from_spelled)
    }

    /// The lexical tree of a predicate that was never authored — built in code, or read from a
    /// canonical document — in which no word is left to decide.
    pub fn resolved(predicate: &Predicate) -> Self {
        Self::build(predicate.clone(), &mut std::iter::repeat(false))
    }

    /// A right side is left to decide where it was a bare word (A1) or unquoted text spelled as an
    /// offset (A2, rule 3a): the resolver tells the two apart by their spelling.
    fn from_spelled(spelled: Spelled) -> Self {
        let mut words = spelled
            .words
            .into_iter()
            .zip(spelled.offsets)
            .map(|(word, offset)| word || offset);
        Self::build(spelled.predicate, &mut words)
    }

    /// Pairs each comparison of `predicate`, in pre-order, with the next flag of `words` — the
    /// order the reader recorded them in.
    fn build(predicate: Predicate, words: &mut dyn Iterator<Item = bool>) -> Self {
        let all = |children: Vec<Predicate>, words: &mut dyn Iterator<Item = bool>| {
            children
                .into_iter()
                .map(|child| Self::build(child, words))
                .collect()
        };
        match predicate {
            Predicate::Compare {
                left,
                op,
                right,
                kind,
            } => {
                let word = words.next().unwrap_or(false);
                let right = match right {
                    Operand::Literal(FactValue::Text(text)) if word => {
                        LexicalOperand::UnquotedText(text)
                    }
                    Operand::Fact(path) if word => LexicalOperand::DottedSpelling(path),
                    other => LexicalOperand::Read(other),
                };
                Self::Compare {
                    left,
                    op,
                    right,
                    kind,
                }
            }
            Predicate::All(children) => Self::All(all(children, words)),
            Predicate::Any(children) => Self::Any(all(children, words)),
            Predicate::Not(inner) => Self::Not(Box::new(Self::build(*inner, words))),
            Predicate::Forall(quantified) => {
                Self::Forall(Box::new(Self::quantified(*quantified, words)))
            }
            Predicate::Exists(quantified) => {
                Self::Exists(Box::new(Self::quantified(*quantified, words)))
            }
            leaf => Self::Leaf(leaf),
        }
    }

    fn quantified(
        quantified: Quantified,
        words: &mut dyn Iterator<Item = bool>,
    ) -> LexicalQuantified {
        LexicalQuantified {
            over: quantified.over,
            bind: quantified.bind,
            body: Self::build(quantified.body, words),
        }
    }

    /// What every format before `ess/22` reads: each bare word is the text it is spelled as.
    ///
    /// Exactly the predicate the ordinary reader returns for the same document, so a source below
    /// `ess/22` keeps its meaning and its canonical bytes.
    pub fn literal(&self) -> Predicate {
        self.lower(&mut |_, _, spelling| match spelling {
            Spelling::Word(word) => Operand::Literal(FactValue::Text(word.to_owned())),
            Spelling::Dotted(path) => Operand::Fact(path.clone()),
        })
    }

    /// Rebuilds the predicate, asking `decide` what each bare word is: given the comparison's left
    /// side, its operator and the word, under the binders enclosing it.
    pub(crate) fn lower(
        &self,
        decide: &mut dyn FnMut(&Operand, CompareOp, Spelling<'_>) -> Operand,
    ) -> Predicate {
        self.lower_scoped(
            &mut |left, op, spelling, _| decide(left, op, spelling),
            &mut Vec::new(),
        )
    }

    /// [`Self::lower`], telling `decide` the quantifiers it sits in, outermost first.
    pub(crate) fn lower_scoped<'a>(
        &'a self,
        decide: &mut Decide<'a, '_>,
        scope: &mut Vec<&'a LexicalQuantified>,
    ) -> Predicate {
        match self {
            Self::Compare {
                left,
                op,
                right,
                kind,
            } => Predicate::Compare {
                kind: *kind,
                left: left.clone(),
                op: *op,
                right: match right {
                    LexicalOperand::Read(operand) => operand.clone(),
                    LexicalOperand::UnquotedText(word) => {
                        decide(left, *op, Spelling::Word(word), scope)
                    }
                    LexicalOperand::DottedSpelling(path) => {
                        decide(left, *op, Spelling::Dotted(path), scope)
                    }
                },
            },
            // Rebuilt as the variants they were, never through the simplifying constructors: the
            // reader already simplified, and resolution must not move a byte it does not decide.
            Self::All(children) => Predicate::All(
                children
                    .iter()
                    .map(|child| child.lower_scoped(decide, scope))
                    .collect(),
            ),
            Self::Any(children) => Predicate::Any(
                children
                    .iter()
                    .map(|child| child.lower_scoped(decide, scope))
                    .collect(),
            ),
            Self::Not(inner) => Predicate::Not(Box::new(inner.lower_scoped(decide, scope))),
            Self::Forall(quantified) => {
                Predicate::Forall(Box::new(Self::lower_quantified(quantified, decide, scope)))
            }
            Self::Exists(quantified) => {
                Predicate::Exists(Box::new(Self::lower_quantified(quantified, decide, scope)))
            }
            Self::Leaf(leaf) => leaf.clone(),
        }
    }

    fn lower_quantified<'a>(
        quantified: &'a LexicalQuantified,
        decide: &mut Decide<'a, '_>,
        scope: &mut Vec<&'a LexicalQuantified>,
    ) -> Quantified {
        scope.push(quantified);
        let body = quantified.body.lower_scoped(decide, scope);
        scope.pop();
        Quantified {
            over: quantified.over.clone(),
            bind: quantified.bind.clone(),
            body,
        }
    }

    /// Whether any comparison still has a bare word to decide.
    pub fn has_words(&self) -> bool {
        match self {
            Self::Compare { right, .. } => matches!(
                right,
                LexicalOperand::UnquotedText(_) | LexicalOperand::DottedSpelling(_)
            ),
            Self::All(children) | Self::Any(children) => children.iter().any(Self::has_words),
            Self::Not(inner) => inner.has_words(),
            Self::Forall(quantified) | Self::Exists(quantified) => quantified.body.has_words(),
            Self::Leaf(_) => false,
        }
    }
}

/// One place an authored source writes a predicate, as the resolver finds it again in the
/// assembled specification.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub(crate) enum Site {
    /// An outcome's `when:`, over the command's input.
    Guard {
        command: QualifiedName,
        outcome: OutcomeName,
    },
    /// An outcome's `when_subject: {predicate}`, over the subject's stored fields and `input.`.
    Subject {
        command: QualifiedName,
        outcome: OutcomeName,
    },
    /// An outcome's `when_related: {predicate}`, over the related row and `input.`.
    Related {
        command: QualifiedName,
        outcome: OutcomeName,
    },
    /// An outcome's `instances: {where}`, over the entity's stored fields and `input.`.
    Instances {
        command: QualifiedName,
        outcome: OutcomeName,
    },
    /// One `affects: [{where}]` entry of an outcome, by position.
    Affect {
        command: QualifiedName,
        outcome: OutcomeName,
        index: usize,
    },
    /// An entity invariant, by position.
    Invariant { entity: QualifiedName, index: usize },
    /// A struct's or newtype's invariant, by position.
    TypeInvariant { name: QualifiedName, index: usize },
    /// A view's `filter:`.
    View { view: QualifiedName },
}

/// The lexical trees of one source's authored predicates, read from the document beside its typed
/// reading and kept until the header's format is known (`docs/design/expression-family-source22.md`,
/// A1).
///
/// The typed reading keeps each predicate as [`LexicalPredicate::literal`], which is what every
/// format before `ess/22` means by it. This keeps how it was written. Nothing here is persisted.
/// An outcome group writes no predicate (its outcomes are external refusals), and a binding's
/// selection reads only its `item`, an aggregate no bare word could compare with, so neither has
/// a site here.
#[derive(Debug, Clone, Default)]
pub struct Written {
    pub(crate) sites: BTreeMap<Site, LexicalPredicate>,
}

impl Written {
    /// Reads every predicate site of one source document, as the typed reader will find them.
    /// Whatever does not read here is left out: the typed reader refuses it with its own
    /// diagnostic, and a predicate left out keeps the meaning it had before `ess/22`.
    pub(crate) fn from_document(document: &serde_yaml::Value) -> Self {
        use serde_yaml::Value;
        let mut written = Self::default();
        let items = |value: &Value, key: &str| -> Vec<Value> {
            value
                .get(key)
                .and_then(Value::as_sequence)
                .cloned()
                .unwrap_or_default()
        };
        let named =
            |item: &Value| -> Option<QualifiedName> { item.get("name")?.as_str()?.parse().ok() };
        let lexical = |value: Option<&Value>| -> Option<LexicalPredicate> {
            let node: Node = serde_yaml::from_value(value?.clone()).ok()?;
            LexicalPredicate::from_node(&node).ok()
        };
        let mut put = |site: Site, value: Option<&Value>| {
            if let Some(tree) = lexical(value) {
                written.sites.entry(site).or_insert(tree);
            }
        };
        for command in items(document, "commands") {
            let Some(name) = named(&command) else {
                continue;
            };
            for outcome in items(&command, "outcomes") {
                let Some(outcome_name) = outcome
                    .get("name")
                    .and_then(Value::as_str)
                    .and_then(|name| name.parse::<OutcomeName>().ok())
                else {
                    continue;
                };
                let at = |make: fn(QualifiedName, OutcomeName) -> Site| {
                    make(name.clone(), outcome_name.clone())
                };
                put(
                    at(|command, outcome| Site::Guard { command, outcome }),
                    outcome.get("when"),
                );
                put(
                    at(|command, outcome| Site::Subject { command, outcome }),
                    outcome
                        .get("when_subject")
                        .and_then(|it| it.get("predicate")),
                );
                put(
                    at(|command, outcome| Site::Related { command, outcome }),
                    outcome
                        .get("when_related")
                        .and_then(|it| it.get("predicate")),
                );
                put(
                    at(|command, outcome| Site::Instances { command, outcome }),
                    outcome.get("instances").and_then(|it| it.get("where")),
                );
                for (index, affect) in items(&outcome, "affects").iter().enumerate() {
                    put(
                        Site::Affect {
                            command: name.clone(),
                            outcome: outcome_name.clone(),
                            index,
                        },
                        affect.get("where"),
                    );
                }
            }
        }
        for (key, invariant_site) in [
            (
                "entities",
                (|entity, index| Site::Invariant { entity, index })
                    as fn(QualifiedName, usize) -> Site,
            ),
            ("types", |name, index| Site::TypeInvariant { name, index }),
        ] {
            for declared in items(document, key) {
                let Some(name) = named(&declared) else {
                    continue;
                };
                for (index, invariant) in items(&declared, "invariants").iter().enumerate() {
                    put(invariant_site(name.clone(), index), Some(invariant));
                }
            }
        }
        for view in items(document, "views") {
            if let Some(name) = named(&view) {
                put(Site::View { view: name }, view.get("filter"));
            }
        }
        written
    }

    /// Takes `other`'s trees for every site this does not hold yet; the first source that declares
    /// a name keeps it, as the assembly keeps its declaration.
    pub(crate) fn absorb(&mut self, other: Self) {
        for (site, tree) in other.sites {
            self.sites.entry(site).or_insert(tree);
        }
    }
}

/// The input guard an outcome's `when:` became, wherever its condition keeps it: the predicate
/// its source format resolved the written one to.
pub fn input_guard(condition: &OutcomeCondition) -> Option<&Predicate> {
    match condition {
        OutcomeCondition::When(predicate) | OutcomeCondition::ExternalWhen { predicate, .. } => {
            Some(predicate)
        }
        OutcomeCondition::SubjectField { predicate, .. }
        | OutcomeCondition::SubjectState { predicate, .. }
        | OutcomeCondition::StateChange { predicate, .. } => predicate.as_ref(),
        OutcomeCondition::SubjectPredicate { input, .. }
        | OutcomeCondition::Related { input, .. } => input.as_ref(),
        _ => None,
    }
}

/// [`input_guard`], to write.
fn input_guard_mut(condition: &mut OutcomeCondition) -> Option<&mut Predicate> {
    match condition {
        OutcomeCondition::When(predicate) | OutcomeCondition::ExternalWhen { predicate, .. } => {
            Some(predicate)
        }
        OutcomeCondition::SubjectField { predicate, .. }
        | OutcomeCondition::SubjectState { predicate, .. }
        | OutcomeCondition::StateChange { predicate, .. } => predicate.as_mut(),
        OutcomeCondition::SubjectPredicate { input, .. }
        | OutcomeCondition::Related { input, .. } => input.as_mut(),
        _ => None,
    }
}

/// The predicate an assembled specification holds at `site`, to write. `None` where the site no
/// longer exists in that shape, which leaves it as it is.
pub(crate) fn slot<'a>(
    site: &Site,
    commands: &'a mut BTreeMap<QualifiedName, CommandSpec>,
    entities: &'a mut BTreeMap<QualifiedName, EntitySpec>,
    views: &'a mut BTreeMap<QualifiedName, crate::view::ViewSpec>,
    types: &'a mut crate::types::TypeRegistry,
) -> Option<&'a mut Predicate> {
    let outcome = |commands: &'a mut BTreeMap<QualifiedName, CommandSpec>,
                   command: &QualifiedName,
                   outcome: &OutcomeName| {
        commands
            .get_mut(command)?
            .outcomes
            .iter_mut()
            .find(|candidate| &candidate.name == outcome)
    };
    match site {
        Site::Guard {
            command,
            outcome: name,
        } => input_guard_mut(&mut outcome(commands, command, name)?.condition),
        Site::Subject {
            command,
            outcome: name,
        } => match &mut outcome(commands, command, name)?.condition {
            OutcomeCondition::SubjectPredicate { predicate, .. } => Some(predicate),
            _ => None,
        },
        Site::Related {
            command,
            outcome: name,
        } => match &mut outcome(commands, command, name)?.condition {
            OutcomeCondition::Related {
                test: crate::command::related_guard::RelatedTest::Holds(predicate),
                ..
            } => Some(predicate),
            _ => None,
        },
        Site::Instances {
            command,
            outcome: name,
        } => outcome(commands, command, name)?
            .set_effects
            .instances
            .as_mut()
            .map(|instances| &mut instances.filter),
        Site::Affect {
            command,
            outcome: name,
            index,
        } => outcome(commands, command, name)?
            .set_effects
            .affects
            .get_mut(*index)
            .map(|affect| &mut affect.filter),
        Site::Invariant { entity, index } => entities
            .get_mut(entity)?
            .invariants
            .get_mut(*index)
            .map(|invariant| &mut invariant.predicate),
        Site::TypeInvariant { name, index } => match &mut types.get_mut(name)?.body {
            crate::types::TypeBody::Struct { invariants, .. }
            | crate::types::TypeBody::Newtype { invariants, .. } => invariants
                .get_mut(*index)
                .map(|invariant| &mut invariant.predicate),
            _ => None,
        },
        Site::View { view } => views.get_mut(view)?.filter.as_mut(),
    }
}

/// What every authored predicate of a specification of `format` resolves to (A1): the site and
/// the resolved predicate, for each site whose predicate is still exactly what the document wrote.
///
/// Below `ess/22` nothing: each predicate already holds [`LexicalPredicate::literal`]. From
/// `ess/22` each bare word is decided against the declarations of the place it is written in —
/// the environment the checker checks that place in — a comparison of two `Timestamp` facts is
/// tagged to compare instants, and a command's input guard reads `input.<path>` as the input it
/// names (decision 6). The checker that runs next refuses what the decision cannot make sense of;
/// this pass refuses nothing.
// One arm per predicate site, each building the environment that site is checked in.
#[allow(clippy::too_many_lines)]
pub(crate) fn resolutions(
    spec: &crate::spec::Specification,
    registry: &crate::types::TypeRegistry,
    written: &Written,
) -> Vec<(Site, Predicate)> {
    use crate::command::subject_fact::INPUT_NAMESPACE;
    use crate::expression::{
        read_input_namespace, resolve_lexical, resolve_lexical_reading, DomainEnvironment,
    };
    if spec.system().format.major() < FormatVersion::V22.major() {
        return Vec::new();
    }
    let declares =
        |fields: &[crate::types::Field], name: &str| fields.iter().any(|field| field.name == name);
    let outcome = |command: &QualifiedName, name: &OutcomeName| {
        let command = spec.commands().get(command)?;
        let outcome = command
            .outcomes
            .iter()
            .find(|candidate| &candidate.name == name)?;
        Some((command, outcome))
    };
    // The environment each stored-row site is checked in: the row's fields and, unless the entity
    // declares a field named `input`, the command's input under `input.`.
    let over_row = |fields: &[crate::types::Field],
                    entity: &EntitySpec,
                    command: &CommandSpec,
                    lexical: &LexicalPredicate| {
        let environment = DomainEnvironment::new(registry, fields);
        if declares(&entity.fields, INPUT_NAMESPACE) {
            resolve_lexical(&environment, lexical)
        } else {
            resolve_lexical(&environment.with_input(&command.input), lexical)
        }
    };
    let mut resolved = Vec::new();
    for (site, lexical) in &written.sites {
        let found = match site {
            Site::Guard {
                command,
                outcome: name,
            } => outcome(command, name).and_then(|(command, outcome)| {
                let current = input_guard(&outcome.condition)?;
                let environment =
                    DomainEnvironment::new(registry, &command.input).with_current_time();
                let namespace = !declares(&command.input, INPUT_NAMESPACE);
                let mut predicate = resolve_lexical_reading(&environment, lexical, namespace);
                if namespace {
                    predicate = read_input_namespace(&predicate);
                }
                Some((current.clone(), predicate))
            }),
            Site::Subject {
                command,
                outcome: name,
            } => outcome(command, name).and_then(|(command, outcome)| {
                let OutcomeCondition::SubjectPredicate { predicate, .. } = &outcome.condition
                else {
                    return None;
                };
                let entity = command
                    .selection_subject(outcome)
                    .and_then(|subject| spec.entities().get(&subject.entity))
                    .or_else(|| crate::command::related_guard::addressed_subject(spec, command))?;
                let fields = crate::command::subject_fact::readable_fields(entity, true);
                Some((
                    predicate.clone(),
                    over_row(&fields, entity, command, lexical),
                ))
            }),
            Site::Related {
                command,
                outcome: name,
            } => outcome(command, name).and_then(|(command, outcome)| {
                let OutcomeCondition::Related {
                    via,
                    test: crate::command::related_guard::RelatedTest::Holds(predicate),
                    ..
                } = &outcome.condition
                else {
                    return None;
                };
                let crate::command::related_value::Referenced::Entity(entity) =
                    crate::command::related_guard::related_entity_via(spec, command, via)
                else {
                    return None;
                };
                let fields = crate::command::subject_fact::readable_fields(entity, true);
                Some((
                    predicate.clone(),
                    over_row(&fields, entity, command, lexical),
                ))
            }),
            Site::Instances {
                command,
                outcome: name,
            } => outcome(command, name).and_then(|(command, outcome)| {
                let instances = outcome.set_effects.instances.as_ref()?;
                let entity = spec.entities().get(&instances.entity)?;
                Some((
                    instances.filter.clone(),
                    over_row(&entity.fields, entity, command, lexical),
                ))
            }),
            Site::Affect {
                command,
                outcome: name,
                index,
            } => outcome(command, name).and_then(|(command, outcome)| {
                let affect = outcome.set_effects.affects.get(*index)?;
                let entity = spec.entities().get(&affect.entity)?;
                Some((
                    affect.filter.clone(),
                    over_row(&entity.fields, entity, command, lexical),
                ))
            }),
            Site::Invariant { entity, index } => spec.entities().get(entity).and_then(|entity| {
                let invariant = entity.invariants.get(*index)?;
                let fields = entity.observable_fields();
                let environment = DomainEnvironment::new(registry, &fields);
                Some((
                    invariant.predicate.clone(),
                    resolve_lexical(&environment, lexical),
                ))
            }),
            Site::TypeInvariant { name, index } => registry.get(name).and_then(|declared| {
                let value;
                let (fields, invariants) = match &declared.body {
                    crate::types::TypeBody::Struct { fields, invariants } => {
                        (fields.as_slice(), invariants)
                    }
                    crate::types::TypeBody::Newtype { of, invariants, .. } => {
                        value = [crate::types::Field::new(NamedType::VALUE, of.clone())];
                        (value.as_slice(), invariants)
                    }
                    _ => return None,
                };
                let invariant = invariants.get(*index)?;
                let environment = DomainEnvironment::new(registry, fields);
                Some((
                    invariant.predicate.clone(),
                    resolve_lexical(&environment, lexical),
                ))
            }),
            Site::View { view } => spec.views().get(view).and_then(|view| {
                let filter = view.filter.as_ref()?;
                let entity = spec.entities().get(&view.source)?;
                let fields = entity.observable_fields();
                let environment =
                    DomainEnvironment::new(registry, &fields).with_params(&view.params);
                Some((filter.clone(), resolve_lexical(&environment, lexical)))
            }),
        };
        // Only the predicate this source wrote: anything else put there is not this tree's.
        if let Some((current, predicate)) = found {
            if current == lexical.literal() && current != predicate {
                resolved.push((site.clone(), predicate));
            }
        }
    }
    resolved
}

/// Refuses an ordering between two identity tokens in an `ess/22` command guard
/// (`docs/design/expression-family-source22.md`, final review decision 9): a fact whose type is
/// some entity's identity compares with another only by `==` and `!=`. An identity is opaque;
/// that one is before another names nothing a caller supplied. Every guard a command decides by is
/// read — its `when:`, a `when_subject:` predicate over the subject's row and a `when_related:`
/// predicate over the related row, each with `input.` — and so is every quantifier body inside
/// one, with its binders. Below `ess/22` nothing is refused here: no source could compare two
/// inputs there.
pub(crate) fn identity_orderings(
    spec: &crate::spec::Specification,
    registry: &crate::types::TypeRegistry,
) -> ess_primitives::error::ValidationErrors {
    use crate::command::subject_fact::INPUT_NAMESPACE;
    use crate::expression::DomainEnvironment;
    use ess_primitives::error::{ValidationCode, ValidationError, ValidationErrors};
    let mut errors = ValidationErrors::new();
    if spec.system().format.major() < FormatVersion::V22.major() {
        return errors;
    }
    let identities: std::collections::BTreeSet<String> = spec
        .entities()
        .values()
        .map(|entity| entity.identity.type_ref.to_string())
        .collect();
    let over_row = |entity: &EntitySpec, command: &CommandSpec, predicate: &Predicate| {
        let fields = crate::command::subject_fact::readable_fields(entity, true);
        let environment = DomainEnvironment::new(registry, &fields);
        if entity
            .fields
            .iter()
            .any(|field| field.name == INPUT_NAMESPACE)
        {
            identity_orderings_in(&environment, predicate, &identities)
        } else {
            identity_orderings_in(
                &environment.with_input(&command.input),
                predicate,
                &identities,
            )
        }
    };
    for command in spec.commands().values() {
        let environment = DomainEnvironment::new(registry, &command.input);
        for outcome in &command.outcomes {
            let site = command.site().key("outcomes").named(outcome.name.as_str());
            let mut found = Vec::new();
            if let Some(guard) = input_guard(&outcome.condition) {
                found.extend(
                    identity_orderings_in(&environment, guard, &identities)
                        .into_iter()
                        .map(|ordering| ("when", ordering)),
                );
            }
            match &outcome.condition {
                OutcomeCondition::SubjectPredicate { predicate, .. } => {
                    let entity = command
                        .selection_subject(outcome)
                        .and_then(|subject| spec.entities().get(&subject.entity))
                        .or_else(|| {
                            crate::command::related_guard::addressed_subject(spec, command)
                        });
                    if let Some(entity) = entity {
                        found.extend(
                            over_row(entity, command, predicate)
                                .into_iter()
                                .map(|ordering| ("when_subject", ordering)),
                        );
                    }
                }
                OutcomeCondition::Related {
                    via,
                    test: crate::command::related_guard::RelatedTest::Holds(predicate),
                    ..
                } => {
                    if let crate::command::related_value::Referenced::Entity(entity) =
                        crate::command::related_guard::related_entity_via(spec, command, via)
                    {
                        found.extend(
                            over_row(entity, command, predicate)
                                .into_iter()
                                .map(|ordering| ("when_related", ordering)),
                        );
                    }
                }
                _ => {}
            }
            for (key, ordering) in found {
                errors.push(ValidationError::at(
                    site.clone().key(key),
                    ValidationCode::TypeMismatch,
                    format!(
                        "`{ordering}` orders an identity: an identity token compares only by `==` \
                         and `!=`, because which of two identities comes first names nothing a \
                         caller supplied"
                    ),
                ));
            }
        }
    }
    errors
}

/// Every ordering in `predicate` — inside quantifier bodies too, with their binders — one of
/// whose facts is typed by an identity in `identities`, rendered as it reads.
fn identity_orderings_in<E: crate::expression::TypeEnvironment>(
    environment: &E,
    predicate: &Predicate,
    identities: &std::collections::BTreeSet<String>,
) -> Vec<String> {
    fn walk<E: crate::expression::TypeEnvironment>(
        environment: &E,
        predicate: &Predicate,
        identities: &std::collections::BTreeSet<String>,
        scope: &mut Vec<(FactPath, String)>,
        found: &mut Vec<String>,
    ) {
        match predicate {
            Predicate::Compare {
                left: Operand::Fact(left),
                op,
                right: Operand::Fact(right),
                ..
            } if op.needs_ordering() => {
                let pairs: Vec<(&FactPath, &str)> = scope
                    .iter()
                    .map(|(over, bind)| (over, bind.as_str()))
                    .collect();
                let bindings = super::bindings_of(environment, &pairs);
                let identity = |path: &FactPath| {
                    super::resolve(environment, path, "", &bindings).is_ok_and(|resolved| {
                        let declared = resolved.declared.as_str();
                        let unwrapped = declared
                            .strip_prefix("Optional<")
                            .and_then(|rest| rest.strip_suffix('>'))
                            .unwrap_or(declared);
                        identities.contains(unwrapped)
                    })
                };
                if identity(left) || identity(right) {
                    found.push(predicate.to_string());
                }
            }
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    walk(environment, child, identities, scope, found);
                }
            }
            Predicate::Not(inner) => walk(environment, inner, identities, scope, found),
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                scope.push((quantified.over.clone(), quantified.bind.clone()));
                walk(environment, &quantified.body, identities, scope, found);
                scope.pop();
            }
            _ => {}
        }
    }
    let mut found = Vec::new();
    walk(
        environment,
        predicate,
        identities,
        &mut Vec::new(),
        &mut found,
    );
    found
}
