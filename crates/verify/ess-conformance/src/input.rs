//! The flattener: a candidate command input, projected into a [`FactSource`].
//!
//! # The bridge that was missing
//!
//! `ess-domain` refuses a `when` whose **first** path segment is not an input field name, and says
//! in the same breath that a deeper path such as `amount.amount` "walks into a named struct, and
//! resolving that belongs with the IR". `ess-compiler` records the other half: a predicate travels
//! parsed but not resolved, and the deep-path rule is one `ess-domain` "deliberately leaves open".
//! Between the two, a guard could be written, validated, compiled and projected without anything
//! ever asking what it reads.
//!
//! [`flatten`] answers that. It walks the command's declared input types beside the candidate's
//! value tree and binds one fact per scalar leaf, so `amount.amount` on a `Money`-typed input field
//! is a bound [`FactValue`] rather than a hope.
//!
//! # The deep-path rule this implements
//!
//! One rule, applied to the resolved type at each step. A newtype is transparent because it wraps a
//! representation rather than naming a member — there is no segment to spell for the inside of one —
//! and the same walk therefore reaches `price.amount` through `Priced = newtype of Money`.
//!
//! | at | with a segment left | with none left |
//! |---|---|---|
//! | `Optional<T>` | walk into `T`; the segment is not consumed | walk into `T` |
//! | a newtype | walk into what it wraps; the segment is not consumed | walk into what it wraps |
//! | a struct | consume it as a field name | not a scalar: `a struct` |
//! | a primitive | nothing to consume: the segment is undeclared | a scalar |
//! | an enum | nothing to consume: the segment is undeclared | a scalar, as text |
//! | a union | not a scalar: `a union` | not a scalar: `a union` |
//! | `List<T>` | `count`, or an element index into `T` | not a scalar: `a list` |
//! | `Map<K, V>` | `count`; any other segment: not a scalar: `a map` | not a scalar: `a map` |
//!
//! A list publishes its size as `<path>.count` and element `n` under `<path>.<n>`, which is the
//! convention [`FactSource::cardinality`] and the quantifiers read for an observed collection
//! (ess#94). So `tags.count > 0`, `lines.0.quantity` and `forall`/`exists` over an input list are
//! decided rather than refused. A map publishes `<path>.count` the same way (ess#196), and its
//! values under `<path>.<index>` in key order (ess#240): a quantifier over a map binds its values,
//! as the compiler types the binder and as Entity Runtime folds it.
//!
//! **Its limits, named rather than discovered later.** A union is not projected *at all*, not even
//! its tag — which is a `String` a fact could hold, and which a later wave may decide to bind as
//! `payee.kind`. A map publishes no key: no fact path spells one, and the ordinal a value is
//! published under is a position a quantifier walks, never a path an author can write. The
//! projection walk is bounded at [`MAX_TYPE_DEPTH`]; semantic path validation has no such depth
//! limit.
//!
//! # A candidate that is not a value of the input's type is refused here
//!
//! Before any predicate is evaluated. The alternative is that a misshapen candidate binds no fact,
//! the guard evaluates to `Unknown`, and the refusal blames the specification for a defect in the
//! caller — which is the same misattribution the crate exists to prevent, pointing the other way.
//! [`ShapeErrors`] accumulates, as every other validation in this workspace does.

use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedCommand, ResolvedField, ResolvedTypeRef};
use ess_domain::types::{Primitive, MAX_TYPE_DEPTH};
use ess_primitives::facts::{FactPath, FactSource, FactStore, FactValue, Scales};
use ess_primitives::node::Node;
use ess_primitives::predicate::{Operand, Predicate, Truth};

use crate::decision::{Decision, Reason, Unevaluable, UnknownCause};

/// Projects a candidate command input into the facts a guard reads.
///
/// The candidate is a map from input field name to value: `Node` rather than a value type of this
/// crate's own, because the workspace already has one format-neutral dynamic value and a second
/// would be a second place for `Map<String, Money>` to mean something slightly different.
///
/// Refuses a candidate that is not a value of the command's declared input type, accumulating every
/// mismatch rather than stopping at the first.
pub fn flatten<'ir>(
    ir: &'ir EssIr,
    command: &'ir ResolvedCommand,
    candidate: &BTreeMap<String, Node>,
) -> Result<InputFacts<'ir>, ShapeErrors> {
    bind(ir, &command.input, candidate, Completeness::Total).map(|facts| InputFacts {
        ir,
        command,
        facts,
        scales: Scales::default(),
    })
}

/// Project only actual literal arguments; instance identities remain unavailable to predicates.
pub(crate) fn replay_facts<'ir>(
    ir: &'ir EssIr,
    command: &'ir ResolvedCommand,
    input: &BTreeMap<String, crate::ScenarioValue>,
) -> Result<InputFacts<'ir>, ShapeErrors> {
    let values = input
        .iter()
        .filter_map(|(name, value)| {
            value
                .as_literal()
                .map(|value| (name.clone(), value.clone()))
        })
        .collect();
    bind(ir, &command.input, &values, Completeness::Partial).map(|facts| InputFacts {
        ir,
        command,
        facts,
        scales: Scales::default(),
    })
}

/// Facts read beside the fields that declare them, the way every ESS evaluator reads them.
///
/// The declared types are what an ordering needs (ess#94): a `Timestamp` is ordered by the instant
/// it names, never by its spelling — `2020-01-01T00:30:00+01:00` is before `2020-01-01T00:00:00Z`
/// although its bytes sort after — a `Duration` is not ordered at all, and any other text no scale
/// orders is ordered by its UTF-8 bytes, as the generated Go and TypeScript runtimes order it. One
/// wrapper for every store this crate decides a predicate over with the declaring fields in hand:
/// an entity's setup, a struct's or a newtype's invariants, a view's filter. [`InputFacts`] answers
/// the same two questions for a command's input through [`declared_as`].
pub(crate) struct TypedFacts<'a> {
    ir: &'a EssIr,
    fields: &'a [ResolvedField],
    facts: FactStore,
}

impl<'a> TypedFacts<'a> {
    /// `facts`, read against `fields`.
    pub(crate) fn new(ir: &'a EssIr, fields: &'a [ResolvedField], facts: FactStore) -> Self {
        Self { ir, fields, facts }
    }

    /// Binds one more fact.
    pub(crate) fn set(&mut self, path: FactPath, value: FactValue) {
        self.facts.set(path, value);
    }

    /// Absorbs every fact and presence mark of `other`, overwriting on conflict.
    pub(crate) fn extend(&mut self, other: FactStore) {
        self.facts.extend(other);
    }
}

impl FactSource for TypedFacts<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.facts.fact(path)
    }

    fn present(&self, path: &FactPath) -> bool {
        self.facts.present(path)
    }

    fn scales(&self) -> &Scales {
        self.facts.scales()
    }

    fn orders_as_instant(&self, path: &FactPath) -> bool {
        declared_as(self.ir, self.fields, path, Primitive::Timestamp)
    }

    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        !declared_as(self.ir, self.fields, path, Primitive::Duration)
    }
}

/// Whether `path` resolves, through any newtype or `Optional`, to the primitive `wanted`.
///
/// A path that does not resolve — `state`, a filter's `param.…` — answers `false`.
pub(crate) fn declared_as(
    ir: &EssIr,
    fields: &[ResolvedField],
    path: &FactPath,
    wanted: Primitive,
) -> bool {
    declared_primitive(ir, fields, path, 0) == Some(wanted)
}

/// The primitive `path` resolves to, through any newtype or `Optional` — and through a map's value
/// at an ordinal, which is where [`project`] publishes it and where a quantifier over the map reads
/// it (beyond10x/ess#240), though no authored path can name one.
fn declared_primitive(
    ir: &EssIr,
    fields: &[ResolvedField],
    path: &FactPath,
    depth: usize,
) -> Option<Primitive> {
    match ess_compiler::expression::resolve_path(ir, fields, path, "conformance facts") {
        Ok(resolved) => match resolved.terminal {
            ResolvedTypeRef::Primitive { name } => Some(name),
            _ => None,
        },
        Err(_) if depth < MAX_TYPE_DEPTH => {
            let segments = path.segments();
            (1..segments.len()).find_map(|at| {
                let ordinal = &segments[at];
                if ordinal.parse::<usize>().ok()?.to_string() != *ordinal {
                    return None;
                }
                let prefix = FactPath::from_segments(&segments[..at]);
                let resolved = ess_compiler::expression::resolve_path(
                    ir,
                    fields,
                    &prefix,
                    "conformance facts",
                )
                .ok()?;
                let value = map_value(ir, &resolved.terminal, 0)?;
                let element = [ResolvedField {
                    name: "value".to_owned(),
                    type_ref: value.clone(),
                    naming: ess_domain::Naming::default(),
                }];
                let mut rest = vec!["value".to_owned()];
                rest.extend(segments[at + 1..].iter().cloned());
                declared_primitive(ir, &element, &FactPath::from_segments(rest), depth + 1)
            })
        }
        Err(_) => None,
    }
}

/// The value type of a map, through any newtype or `Optional` over one.
fn map_value<'a>(
    ir: &'a EssIr,
    type_ref: &'a ResolvedTypeRef,
    depth: usize,
) -> Option<&'a ResolvedTypeRef> {
    if depth > MAX_TYPE_DEPTH {
        return None;
    }
    match type_ref {
        ResolvedTypeRef::Map { value, .. } => Some(value),
        ResolvedTypeRef::Optional { of } => map_value(ir, of, depth + 1),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => map_value(ir, of, depth + 1),
            _ => None,
        },
        _ => None,
    }
}

/// Projects a map of values into facts, guided by the fields some construct declares.
///
/// The half of [`flatten`] that is not about a command. A command's input, an event's payload, a
/// declared error's fields and a view's row are four surfaces the model describes the same way — a
/// flat list of named, typed members — and the question asked of a supplied value is identical at
/// each: *is this a value of the type declared there?* One walk answers all four, because a second
/// one written beside it would be a second opinion about whether `1.5` is an `Integer` and about
/// which variants `Channel` has.
///
/// Refuses everything wrong with the map rather than the first thing, so an author correcting a
/// scenario sees the whole list.
pub fn bind(
    ir: &EssIr,
    fields: &[ResolvedField],
    values: &BTreeMap<String, Node>,
    completeness: Completeness,
) -> Result<FactStore, ShapeErrors> {
    let mut facts = FactStore::new();
    let mut errors = Vec::new();
    let candidate = values;

    for field in fields {
        let Ok(path) = FactPath::new(&field.name) else {
            errors.push(ShapeError::UnnameableField {
                field: field.name.clone(),
            });
            continue;
        };
        match candidate.get(&field.name) {
            Some(value) => project(
                ir,
                &field.type_ref,
                value,
                &path,
                0,
                &mut facts,
                &mut errors,
            ),
            None if admits_absence(ir, &field.type_ref, 0) => {}
            None if completeness == Completeness::Partial => {}
            None => errors.push(ShapeError::MissingField {
                at: String::new(),
                field: field.name.clone(),
            }),
        }
    }
    for supplied in candidate.keys() {
        if !fields.iter().any(|field| &field.name == supplied) {
            errors.push(ShapeError::UndeclaredField {
                at: String::new(),
                field: supplied.clone(),
            });
        }
    }

    if errors.is_empty() {
        Ok(facts)
    } else {
        Err(ShapeErrors(errors))
    }
}

/// Project one actually observed value into an existing fact store at its declared path.
/// Private execution contexts use this for known leaves without passing an incomplete abstract
/// object through concrete input validation.
pub(crate) fn project_observed(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    value: &Node,
    path: &FactPath,
    facts: &mut FactStore,
) -> Result<(), ShapeErrors> {
    let mut errors = Vec::new();
    project(ir, kind, value, path, 0, facts, &mut errors);
    if errors.is_empty() {
        Ok(())
    } else {
        Err(ShapeErrors(errors))
    }
}

/// Whether a member of type `type_ref` may be left out: an `Optional`, through any newtype over one
/// (beyond10x/ess#205). The rule `ess-domain` admits a precondition literal by, so a member it lets
/// a literal omit is one this reader accepts omitted.
fn admits_absence(ir: &EssIr, type_ref: &ResolvedTypeRef, depth: usize) -> bool {
    if depth > MAX_TYPE_DEPTH {
        return false;
    }
    match type_ref {
        ResolvedTypeRef::Optional { .. } => true,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => admits_absence(ir, of, depth + 1),
            _ => false,
        },
        _ => false,
    }
}

/// Validate fixture state against the declared model before establishing backend data.
///
/// Standalone suite admission cannot prove model constraints without an IR. Adapters may call this
/// helper or perform equivalent validation against their own declared contract before writing.
pub fn validate_entity_setup(
    ir: &EssIr,
    entity: &ess_compiler::refs::EntityRef,
    identity: &Node,
    values: &BTreeMap<String, Node>,
    state: &ess_domain::entity::StateName,
) -> Result<(), String> {
    if matches!(identity, Node::Null) {
        return Err("entity setup identity cannot be null".into());
    }
    let declared = ir
        .entities()
        .get(entity.name())
        .ok_or_else(|| format!("undeclared entity `{entity}`"))?;
    if !declared.lifecycle.states.contains(state) {
        return Err(format!("undeclared state `{state}` of `{entity}`"));
    }
    if values.contains_key(&declared.identity.name) {
        return Err("identity must appear only in the identity slot".into());
    }
    let mut fields = declared.fields.clone();
    fields.push(declared.identity.clone());
    let mut supplied = values.clone();
    supplied.insert(declared.identity.name.clone(), identity.clone());
    let budget = ProofBudget::native();
    let (fields_facts, mut unresolved) =
        setup_fields(ir, &fields, &supplied, 0, &budget).map_err(|error| error.to_string())?;
    let mut facts = TypedFacts::new(ir, &fields, fields_facts);
    facts.set(
        FactPath::new("state").expect("static path"),
        FactValue::Text(state.to_string()),
    );
    retain_validation(
        &mut unresolved,
        setup_invariants(&declared.invariants, &facts, &budget, 1),
    )
    .map_err(|error| error.to_string())?;
    unresolved.map_or(Ok(()), |error| Err(error.to_string()))
}

/// A failed complete validation is distinct from a validation the evaluator could not finish.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ValidationFailure {
    Invalid(String),
    Unresolved(String),
}

/// Shared, ephemeral proof-search allowance. Native validation has no new work limit.
pub(crate) struct ProofBudget {
    remaining: std::cell::Cell<Option<usize>>,
    witness_candidates: std::cell::Cell<usize>,
}

impl ProofBudget {
    pub(crate) fn new(work: usize) -> Self {
        Self {
            remaining: std::cell::Cell::new(Some(work)),
            witness_candidates: std::cell::Cell::new(64),
        }
    }
    fn native() -> Self {
        Self {
            remaining: std::cell::Cell::new(None),
            witness_candidates: std::cell::Cell::new(64),
        }
    }
    pub(crate) fn witness_candidate(&self) -> Option<()> {
        let remaining = self.witness_candidates.get().checked_sub(1)?;
        self.witness_candidates.set(remaining);
        self.charge(1).ok()
    }
    pub(crate) fn charge(&self, work: usize) -> Result<(), ValidationFailure> {
        if let Some(remaining) = self.remaining.get() {
            let Some(next) = remaining.checked_sub(work) else {
                self.remaining.set(Some(0));
                return Err(ValidationFailure::Unresolved(
                    "generated-domain proof work exhausted".into(),
                ));
            };
            self.remaining.set(Some(next));
        }
        Ok(())
    }
    pub(crate) fn exhausted(&self) -> bool {
        self.remaining.get() == Some(0)
    }
    pub(crate) fn type_ref(&self, kind: &ResolvedTypeRef) -> Result<(), ValidationFailure> {
        if self.remaining.get().is_none() {
            return Ok(());
        }
        self.charge(16)?;
        match kind {
            ResolvedTypeRef::Declared { name } => {
                for segment in name.name().segments() {
                    self.charge(1 + segment.len())?;
                }
            }
            ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => self.type_ref(of)?,
            ResolvedTypeRef::Map { value, .. } => self.type_ref(value)?,
            ResolvedTypeRef::Primitive { .. } => {}
        }
        Ok(())
    }
    // Preflight the existing concrete projector before it allocates facts/errors. Charge the
    // actual representation walk, declaration widths and possible path/name copies; native
    // projection keeps its existing behavior and does not consume a proof allowance.
    fn projection(
        &self,
        ir: &EssIr,
        kind: &ResolvedTypeRef,
        value: &Node,
        depth: usize,
        path_bytes: usize,
    ) -> Result<(), ValidationFailure> {
        if self.remaining.get().is_none() {
            return Ok(());
        }
        self.charge(1 + path_bytes)?;
        if depth > MAX_TYPE_DEPTH {
            return Ok(());
        }
        self.type_ref(kind)?;
        match kind {
            ResolvedTypeRef::Optional { of } if !matches!(value, Node::Null) => {
                self.projection(ir, of, value, depth + 1, path_bytes)?;
            }
            ResolvedTypeRef::List { of } => {
                if let Node::Seq(values) = value {
                    for child in values {
                        self.charge(32)?;
                        self.projection(ir, of, child, depth + 1, path_bytes.saturating_add(21))?;
                    }
                }
            }
            ResolvedTypeRef::Map { value: of, .. } => {
                if let Node::Map(values) = value {
                    for child in values.values() {
                        self.charge(32)?;
                        self.projection(ir, of, child, depth + 1, path_bytes.saturating_add(21))?;
                    }
                }
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    self.projection(ir, of, value, depth + 1, path_bytes)?;
                }
                ResolvedBody::Enum { variants } => {
                    for variant in variants {
                        self.charge(1 + variant.name().len())?;
                    }
                }
                ResolvedBody::Struct { fields, .. } => {
                    if let Node::Map(values) = value {
                        self.charge(fields.len().saturating_mul(values.len()))?;
                        for field in fields {
                            let child_bytes = path_bytes.saturating_add(1 + field.name.len());
                            self.charge(child_bytes)?;
                            if let Some(child) = values.get(&field.name) {
                                self.projection(
                                    ir,
                                    &field.type_ref,
                                    child,
                                    depth + 1,
                                    child_bytes,
                                )?;
                            } else {
                                self.charge(MAX_TYPE_DEPTH + 1)?;
                            }
                        }
                    }
                }
                ResolvedBody::Union { .. } => {}
            },
            ResolvedTypeRef::Primitive { .. } => {
                self.value(value)?;
            }
            ResolvedTypeRef::Optional { .. } => {}
        }
        Ok(())
    }
    #[allow(
        clippy::items_after_statements,
        reason = "the private recursive walker is scoped to this budgeted value preflight"
    )]
    pub(crate) fn value(&self, value: &Node) -> Result<usize, ValidationFailure> {
        if self.remaining.get().is_none() {
            return Ok(1);
        }
        fn walk(
            budget: &ProofBudget,
            value: &Node,
            depth: usize,
        ) -> Result<usize, ValidationFailure> {
            budget.charge(1)?;
            if depth > MAX_TYPE_DEPTH + 1 {
                return Err(ValidationFailure::Unresolved(
                    "proof value depth exhausted".into(),
                ));
            }
            let mut size = 1_usize;
            match value {
                Node::Text(text) => {
                    budget.charge(text.len())?;
                    size += text.len();
                }
                Node::Map(values) => {
                    for (key, value) in values {
                        budget.charge(key.len())?;
                        size = size
                            .checked_add(key.len())
                            .and_then(|n| n.checked_add(walk(budget, value, depth + 1).ok()?))
                            .ok_or_else(|| {
                                ValidationFailure::Unresolved("proof value work exhausted".into())
                            })?;
                    }
                }
                Node::Seq(values) => {
                    for value in values {
                        size = size
                            .checked_add(walk(budget, value, depth + 1)?)
                            .ok_or_else(|| {
                                ValidationFailure::Unresolved("proof value work exhausted".into())
                            })?;
                    }
                }
                _ => {}
            }
            Ok(size)
        }
        walk(self, value, 0)
    }
    pub(crate) fn predicate(
        &self,
        predicate: &ess_primitives::predicate::Predicate,
        width: usize,
    ) -> Result<(), ValidationFailure> {
        use ess_primitives::predicate::{Operand, Predicate};
        if self.remaining.get().is_none() {
            return Ok(());
        }
        let literal = |value: &FactValue| self.charge(value.as_text().map_or(1, str::len));
        let path = |value: &FactPath| -> Result<(), ValidationFailure> {
            for segment in value.segments() {
                self.charge(segment.len())?;
            }
            Ok(())
        };
        self.charge(1)?;
        match predicate {
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.predicate(child, width)?;
                }
            }
            Predicate::Not(child) => self.predicate(child, width)?,
            Predicate::Compare { left, right, .. } => {
                for operand in [left, right] {
                    match operand {
                        Operand::Fact(value) => path(value)?,
                        Operand::Offset(offset) => {
                            path(&offset.base)?;
                            self.charge(offset.magnitude.to_string().len())?;
                        }
                        Operand::Derived(derived) => {
                            path(derived.parent())?;
                            self.charge(ess_primitives::predicate::Derived::UTF8_BYTES.len())?;
                        }
                        Operand::Literal(value) => literal(value)?,
                    }
                }
            }
            Predicate::Truthy(value) | Predicate::Defined(value) => path(value)?,
            Predicate::AnyOf { path: read, values }
            | Predicate::NoneOf { path: read, values }
            | Predicate::FoldMatch {
                path: read, values, ..
            } => {
                path(read)?;
                for value in values {
                    literal(value)?;
                }
            }
            Predicate::TextMatch {
                path: read, value, ..
            } => {
                path(read)?;
                match value {
                    ess_primitives::predicate::TextOperand::Literal(value) => literal(value)?,
                    ess_primitives::predicate::TextOperand::Fact { path: fact, .. } => {
                        path(fact)?;
                    }
                }
            }
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                path(&quantified.over)?;
                self.charge(quantified.bind.len())?;
                for _ in 0..width {
                    self.predicate(&quantified.body, width)?;
                }
            }
            // One key read per element, each compared with the keys before it.
            Predicate::Distinct(distinct) => {
                path(&distinct.over)?;
                self.charge(distinct.bind.len())?;
                if let Some(key) = &distinct.key {
                    path(key)?;
                }
                for _ in 0..width {
                    self.charge(width)?;
                }
            }
            Predicate::Window(window) => {
                if let Some(read) = window.at.fact_path() {
                    path(read)?;
                }
                self.charge(window.to_string().len())?;
            }
            Predicate::Always | Predicate::Never => {}
        }
        self.charge(width)
    }
}

impl std::fmt::Display for ValidationFailure {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Invalid(detail) | Self::Unresolved(detail) => formatter.write_str(detail),
        }
    }
}

impl From<String> for ValidationFailure {
    fn from(detail: String) -> Self {
        Self::Invalid(detail)
    }
}

impl From<&str> for ValidationFailure {
    fn from(detail: &str) -> Self {
        Self::Invalid(detail.into())
    }
}

impl ValidationFailure {
    fn at(self, name: &str) -> Self {
        match self {
            Self::Invalid(detail) => Self::Invalid(format!("{name}: {detail}")),
            Self::Unresolved(detail) => Self::Unresolved(format!("{name}: {detail}")),
        }
    }

    fn shape(errors: &ShapeErrors) -> Self {
        if errors
            .0
            .iter()
            .any(|error| !matches!(error, ShapeError::TooDeep { .. }))
        {
            Self::Invalid(errors.to_string())
        } else {
            Self::Unresolved(errors.to_string())
        }
    }
}

fn retain_validation(
    unresolved: &mut Option<ValidationFailure>,
    result: Result<(), ValidationFailure>,
) -> Result<(), ValidationFailure> {
    match result {
        Err(error @ ValidationFailure::Invalid(_)) => Err(error),
        Err(error) => {
            unresolved.get_or_insert(error);
            Ok(())
        }
        Ok(()) => Ok(()),
    }
}

fn setup_fields(
    ir: &EssIr,
    fields: &[ResolvedField],
    values: &BTreeMap<String, Node>,
    depth: usize,
    budget: &ProofBudget,
) -> Result<(FactStore, Option<ValidationFailure>), ValidationFailure> {
    for field in fields {
        budget.charge(1 + field.name.len())?;
    }
    for (key, value) in values {
        budget.charge(key.len())?;
        budget.value(value)?;
    }
    budget.charge(fields.len().saturating_mul(values.len()))?;
    for field in fields {
        if let Some(value) = values.get(&field.name) {
            budget.projection(ir, &field.type_ref, value, 0, field.name.len())?;
        } else {
            budget.charge(MAX_TYPE_DEPTH + 1)?;
        }
    }
    let facts = bind(ir, fields, values, Completeness::Total)
        .map_err(|errors| ValidationFailure::shape(&errors))?;
    let mut unresolved = None;
    for field in fields {
        if let Some(value) = values.get(&field.name) {
            match setup_value(ir, &field.type_ref, value, depth + 1, budget) {
                Ok(()) => {}
                Err(error @ ValidationFailure::Invalid(_)) => return Err(error.at(&field.name)),
                Err(error) => {
                    unresolved.get_or_insert_with(|| error.at(&field.name));
                }
            }
            if budget.exhausted() {
                return Err(unresolved.unwrap_or_else(|| {
                    ValidationFailure::Unresolved("proof work exhausted".into())
                }));
            }
        }
    }
    Ok((facts, unresolved))
}

fn setup_invariants(
    invariants: &[ess_domain::entity::Invariant],
    facts: &dyn FactSource,
    budget: &ProofBudget,
    width: usize,
) -> Result<(), ValidationFailure> {
    let mut unresolved = None;
    for invariant in invariants {
        budget.predicate(&invariant.predicate, width)?;
        let truth = invariant.predicate.evaluate(facts);
        let failure = || {
            format!(
                "invariant `{}` is {truth:?}; setup requires True",
                invariant.statement
            )
        };
        match truth {
            Truth::False => return Err(ValidationFailure::Invalid(failure())),
            Truth::Unknown => {
                unresolved.get_or_insert_with(|| ValidationFailure::Unresolved(failure()));
            }
            Truth::True => {}
        }
    }
    unresolved.map_or(Ok(()), Err)
}

/// Check one concrete value against its declared type and nested invariants.
///
/// Setup and synthesized command inputs share this bounded validator; an invariant must be True.
pub(crate) fn validate_typed_value(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    value: &Node,
) -> Result<(), String> {
    setup_value(ir, kind, value, 0, &ProofBudget::native()).map_err(|error| error.to_string())
}

#[cfg(test)]
pub(crate) fn validate_typed_value_proof(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    value: &Node,
) -> Result<(), ValidationFailure> {
    validate_typed_value_bounded(ir, kind, value, &ProofBudget::new(16_384))
}

pub(crate) fn validate_typed_value_bounded(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    value: &Node,
    budget: &ProofBudget,
) -> Result<(), ValidationFailure> {
    setup_value(ir, kind, value, 0, budget)
}

fn setup_value(
    ir: &EssIr,
    kind: &ResolvedTypeRef,
    value: &Node,
    depth: usize,
    budget: &ProofBudget,
) -> Result<(), ValidationFailure> {
    budget.charge(1)?;
    if depth > MAX_TYPE_DEPTH {
        return Err(ValidationFailure::Unresolved(format!(
            "setup type expansion exceeds {MAX_TYPE_DEPTH}"
        )));
    }
    match kind {
        ResolvedTypeRef::Optional { of } => {
            if matches!(value, Node::Null) {
                Ok(())
            } else {
                setup_value(ir, of, value, depth + 1, budget)
            }
        }
        ResolvedTypeRef::Primitive { name } if *name == Primitive::Json => {
            budget.value(value).map(|_| ())
        }
        ResolvedTypeRef::Primitive { name } => {
            budget.value(value)?;
            primitive_value(*name, value)
                .map(|_| ())
                .ok_or_else(|| ValidationFailure::Invalid(format!("value does not hold {name}")))
        }
        ResolvedTypeRef::List { of } => {
            budget.value(value)?;
            let Node::Seq(values) = value else {
                return Err("expected a list".into());
            };
            let mut unresolved = None;
            for (index, child) in values.iter().enumerate() {
                retain_validation(
                    &mut unresolved,
                    setup_value(ir, of, child, depth + 1, budget),
                )?;
                if budget.exhausted() && index + 1 < values.len() {
                    return Err(ValidationFailure::Unresolved(
                        "proof work exhausted before all list elements were validated".into(),
                    ));
                }
            }
            unresolved.map_or(Ok(()), Err)
        }
        ResolvedTypeRef::Map { key, value: of } => {
            budget.value(value)?;
            let Node::Map(values) = value else {
                return Err("expected a map".into());
            };
            let mut unresolved = None;
            for (index, (spelling, child)) in values.iter().enumerate() {
                setup_map_key(*key, spelling)?;
                retain_validation(
                    &mut unresolved,
                    setup_value(ir, of, child, depth + 1, budget),
                )?;
                if budget.exhausted() && index + 1 < values.len() {
                    return Err(ValidationFailure::Unresolved(
                        "proof work exhausted before all map entries were validated".into(),
                    ));
                }
            }
            unresolved.map_or(Ok(()), Err)
        }
        ResolvedTypeRef::Declared { name } => {
            let declared = ir.named_type(name);
            for segment in declared.name.segments() {
                budget.charge(1 + segment.len())?;
            }
            setup_body(ir, &declared.name, &declared.body, value, depth + 1, budget)
        }
    }
}

#[allow(
    clippy::too_many_lines,
    reason = "one match keeps every resolved type body under the same bounded validator"
)]
fn setup_body(
    ir: &EssIr,
    name: &ess_domain::QualifiedName,
    body: &ResolvedBody,
    value: &Node,
    depth: usize,
    budget: &ProofBudget,
) -> Result<(), ValidationFailure> {
    let width = budget.value(value)?;
    match body {
        ResolvedBody::Newtype {
            of,
            alphabet,
            prefix,
            invariants,
        } => {
            let mut unresolved = None;
            retain_validation(&mut unresolved, setup_value(ir, of, value, depth, budget))?;
            if let (Some(prefix), Some(text)) = (prefix, value.as_text()) {
                budget.charge(prefix.len())?;
                if !text.starts_with(prefix.as_str()) {
                    return Err(format!(
                        "{text:?} does not start with {prefix:?}, the prefix of {name}"
                    )
                    .into());
                }
            }
            // Before the invariants: every character of a text is one of the declared alphabet's
            // (`docs/design/string-alphabet-and-length.md`, section 1).
            if let (Some(alphabet), Some(text)) = (alphabet, value.as_text()) {
                budget.charge(text.len().saturating_mul(alphabet.len()))?;
                if let Some(outside) = text
                    .chars()
                    .find(|character| !alphabet.contains(*character))
                {
                    return Err(format!(
                        "{outside:?} in {text:?} is not in the alphabet of {name}"
                    )
                    .into());
                }
            }
            let mut facts = FactStore::new();
            let mut errors = Vec::new();
            budget.projection(ir, of, value, 0, "value".len())?;
            project(
                ir,
                of,
                value,
                &FactPath::new("value").expect("static path"),
                0,
                &mut facts,
                &mut errors,
            );
            if !errors.is_empty() {
                return Err(
                    ValidationFailure::shape(&ShapeErrors(errors)).at("newtype facts unavailable")
                );
            }
            let wrapped = [ResolvedField {
                name: "value".to_owned(),
                type_ref: {
                    budget.type_ref(of)?;
                    of.clone()
                },
                naming: ess_domain::name::Naming::default(),
            }];
            retain_validation(
                &mut unresolved,
                setup_invariants(
                    invariants,
                    &TypedFacts::new(ir, &wrapped, facts),
                    budget,
                    width,
                ),
            )?;
            unresolved.map_or(Ok(()), Err)
        }
        ResolvedBody::Struct { fields, invariants } => {
            let Node::Map(values) = value else {
                return Err("expected a struct mapping".into());
            };
            let (facts, mut unresolved) = setup_fields(ir, fields, values, depth, budget)?;
            retain_validation(
                &mut unresolved,
                setup_invariants(
                    invariants,
                    &TypedFacts::new(ir, fields, facts),
                    budget,
                    width,
                ),
            )?;
            unresolved.map_or(Ok(()), Err)
        }
        ResolvedBody::Enum { variants } => {
            for variant in variants {
                budget.charge(1 + variant.name().len())?;
            }
            if value
                .as_text()
                .is_some_and(|value| variants.iter().any(|variant| variant == value))
            {
                Ok(())
            } else {
                Err("undeclared enum variant".into())
            }
        }
        ResolvedBody::Union { tag, variants } => {
            let Node::Map(values) = value else {
                return Err("expected a tagged union mapping".into());
            };
            let content = if tag == "value" { "content" } else { "value" };
            let selected = values
                .get(tag)
                .and_then(Node::as_text)
                .and_then(|label| variants.get(label))
                .ok_or("unknown or missing union tag")?;
            // A unit variant (ess/22) is the tag alone.
            let Some(selected) = selected else {
                return if values.len() == 1 {
                    Ok(())
                } else {
                    Err("a unit variant holds its tag alone".into())
                };
            };
            if values.len() != 2 {
                return Err("a union holds exactly its tag and payload".into());
            }
            let payload = values.get(content).ok_or("missing union payload")?;
            setup_value(ir, selected, payload, depth, budget)
        }
    }
}

pub(crate) fn setup_map_key(kind: Primitive, spelling: &str) -> Result<(), String> {
    // A `sets:` literal gained a `Decimal` spelling (#135); a map key did not.
    if !matches!(kind, Primitive::Decimal) && primitive_literal(kind, spelling).is_some() {
        return Ok(());
    }
    Err(match kind {
        Primitive::Boolean => "invalid Boolean map key".to_owned(),
        Primitive::Integer if spelling.parse::<i64>().is_err() => {
            "invalid Integer map key".to_owned()
        }
        Primitive::Integer => "Integer map keys require canonical decimal spelling".to_owned(),
        Primitive::Decimal | Primitive::Binary64 => {
            "numeric decimal map keys have no admitted setup spelling".to_owned()
        }
        _ => format!("invalid {kind} map key"),
    })
}

/// The value a literal's TEXT is when read as `kind`, or `None` where it is not one.
///
/// One reader for the two places a spelling has to become a value: a `Map<K, V>` setup key, and a
/// `sets:` or `payload:` literal. `ess-domain::primitive_literal` refuses at authoring time exactly
/// what this returns `None` for, and its own documentation names this reader as the reason the two
/// spellings are what they are — so the pair has to stay one rule. Two readers is how a literal
/// gets admitted by the validator and then dropped by the generator, which is precisely the defect
/// the `sets:` literal work of 2026-09-16 was: `paused: "false"` over a `Boolean` validated and
/// then vanished.
///
/// `true`/`false`, and a decimal that round-trips through `i64` — so `007`, `+7` and ` 7` answer
/// `None` rather than being normalised, because normalising puts a spelling into the suite that
/// nothing else writes. `Decimal` reads by `Number::decimal_literal`, the function `ess-domain` admits
/// one by; `Binary64` has no admitted literal spelling. Every other
/// primitive is carried as the text itself and then held to that primitive's own grammar by
/// [`primitive_value`], so a literal that is not a legal `Uuid` answers `None` rather than becoming
/// an assertion no implementation can satisfy.
pub(crate) fn primitive_literal(kind: Primitive, spelling: &str) -> Option<Node> {
    let value = match kind {
        Primitive::Boolean => match spelling {
            "true" => Node::Bool(true),
            "false" => Node::Bool(false),
            _ => return None,
        },
        Primitive::Integer => {
            let number = spelling.parse::<i64>().ok()?;
            if number.to_string() != spelling {
                return None;
            }
            Node::Number(number.into())
        }
        // The grammar `ess-domain` admits a `Decimal` literal by, and the same function (#135).
        Primitive::Decimal => {
            Node::Number(ess_primitives::facts::Number::decimal_literal(spelling)?)
        }
        // No literal is a JSON value here: a literal is one piece of text (beyond10x/ess#138).
        Primitive::Binary64 | Primitive::Json => return None,
        Primitive::String
        | Primitive::Timestamp
        | Primitive::Duration
        | Primitive::Uuid
        | Primitive::Bytes => Node::Text(spelling.to_owned()),
    };
    primitive_value(kind, &value).is_some().then_some(value)
}

/// Whether every declared field has to be supplied, or only the ones named.
///
/// A command's input is [`Total`](Self::Total): a command is invoked with all of it, and a field
/// left out is a call that could not be made. A *comparison* is [`Partial`](Self::Partial): an
/// event's payload, a declared error's fields and a view's row are asserted field by field, because
/// a specification does not determine every value an implementation may legitimately put in one —
/// which is the reading [`ScenarioStep::ExpectError`](crate::ScenarioStep::ExpectError) already
/// states about its own `fields`.
///
/// Nothing else moves with it: a value supplied under a name the surface does not declare, and a
/// value the declared type does not admit, are refused either way.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Completeness {
    /// Every non-optional field must be supplied.
    Total,
    /// Only the fields named are checked, and the rest are not required.
    Partial,
}

/// The facts one candidate input projects, and the guards they decide.
///
/// Holds the IR as well as the facts, because refusing well needs the *types*: "nothing is bound at
/// `amount.vat`" is the same observation whether the path names a field the candidate left out or a
/// field no type declares, and those two need opposite responses.
#[derive(Debug, Clone)]
pub struct InputFacts<'ir> {
    ir: &'ir EssIr,
    command: &'ir ResolvedCommand,
    facts: FactStore,
    scales: Scales,
}

impl<'ir> InputFacts<'ir> {
    /// Declares the ordered scales non-numeric comparisons are read against.
    ///
    /// Empty by default, and that default is a fact about the model rather than a placeholder: the
    /// ESS specification language has no scale vocabulary. A text ordering no scale decides is
    /// decided by the texts' UTF-8 bytes, as in every ESS lane (ess#94); a scale supplied here by
    /// something outside the specification — an AEP protocol, whose `scales:` this takes — decides
    /// first wherever it contains both values. [`Reason::TextNotOrdered`] is left for a declared
    /// `Timestamp` whose text names no instant.
    #[must_use]
    pub fn with_scales(mut self, scales: Scales) -> Self {
        self.scales = scales;
        self
    }

    /// The command whose input this is.
    pub fn command(&self) -> &'ir ResolvedCommand {
        self.command
    }

    /// Every fact the candidate bound, in path order.
    pub fn facts(&self) -> &FactStore {
        &self.facts
    }

    /// Decides `predicate` against this candidate.
    ///
    /// `True` is [`Satisfied`](Decision::Satisfied), `False` is [`Refuted`](Decision::Refuted), and
    /// `Unknown` is [`Unevaluable`](Decision::Unevaluable) — never `Refuted`. The two differ in what
    /// a caller should do next, and there is no reading of §11's "prove or evaluate" on which an
    /// undecidable guard counts as a guard this candidate failed.
    pub fn decide(&self, predicate: &Predicate) -> Decision {
        match predicate.evaluate(self) {
            Truth::True => Decision::Satisfied,
            Truth::False => Decision::Refuted(predicate.outcome(self)),
            Truth::Unknown => {
                let mut causes = Vec::new();
                self.explain(predicate, &mut causes);
                if causes.is_empty() {
                    causes.push(UnknownCause {
                        expression: predicate.to_string(),
                        reason: Reason::Unclassified,
                    });
                }
                Decision::Unevaluable(Unevaluable {
                    predicate: predicate.to_string(),
                    command: self.command.name.to_string(),
                    causes,
                })
            }
        }
    }

    /// Collects every leaf that evaluates to `Unknown`, with the reason it does.
    ///
    /// Walks the whole tree rather than only the leaves that decided the result. A conjunction of
    /// two undecidable leaves is two defects, and reporting one of them sends the reader back for
    /// the other — invariant 3's reasoning, applied to a refusal instead of a validation.
    fn explain(&self, predicate: &Predicate, causes: &mut Vec<UnknownCause>) {
        match predicate {
            Predicate::Always | Predicate::Never => {}
            Predicate::All(children) | Predicate::Any(children) => {
                for child in children {
                    self.explain(child, causes);
                }
            }
            Predicate::Not(inner) => self.explain(inner, causes),
            leaf => {
                if leaf.evaluate(self) == Truth::Unknown {
                    self.explain_leaf(leaf, causes);
                }
            }
        }
    }

    /// The reason one undecided leaf could not be decided.
    fn explain_leaf(&self, leaf: &Predicate, causes: &mut Vec<UnknownCause>) {
        let expression = leaf.to_string();
        let mut push = |reason| {
            causes.push(UnknownCause {
                expression: expression.clone(),
                reason,
            });
        };
        match leaf {
            // The four leaves that read one path: `Unknown` means nothing is bound there.
            // `Defined` is not among them — it reports `False` for an unbound path, by design.
            // A string operator is `Unknown` only when its path is unbound: its literal always
            // resolves, and a resolved value that is not text is `False`. So is a case-insensitive one.
            Predicate::Truthy(path)
            | Predicate::AnyOf { path, .. }
            | Predicate::NoneOf { path, .. }
            | Predicate::TextMatch { path, .. }
            | Predicate::FoldMatch { path, .. } => push(self.explain_path(path)),
            Predicate::Compare {
                left, op, right, ..
            } => {
                let mut unresolved = false;
                for operand in [left, right] {
                    if let Some(path) = operand.fact_path() {
                        if self.observe(path).is_none() {
                            unresolved = true;
                            push(self.explain_path(path));
                        }
                    }
                }
                if unresolved {
                    return;
                }
                // Both sides resolved, so the evaluator's ordering branch is what returned
                // `Unknown`. Its two cases, in its own order: two texts no scale orders, and a
                // pairing that has no ordering at all.
                let (Some(left_value), Some(right_value)) =
                    (self.resolve(left), self.resolve(right))
                else {
                    push(Reason::Unclassified);
                    return;
                };
                if !op.needs_ordering() {
                    push(Reason::Unclassified);
                    return;
                }
                match (&left_value, &right_value) {
                    (FactValue::Text(left_text), FactValue::Text(right_text)) => {
                        push(Reason::TextNotOrdered {
                            left: left_text.clone(),
                            right: right_text.clone(),
                        });
                    }
                    _ => push(Reason::TypesNotOrdered {
                        left: left_value.type_name(),
                        right: right_value.type_name(),
                    }),
                }
            }
            // A quantifier is `Unknown` when the collection it walks is unobserved, which is the
            // same defect `Truthy` reports and is answerable the same way. It is also `Unknown`
            // when a body leaf is, and that case is not classified here: the body reads paths under
            // a binder, and `explain_path` resolves against the command's input types, where no
            // binder exists. Reporting the collection would name the wrong path.
            Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
                if self.cardinality(&quantified.over).is_none() {
                    push(self.explain_path(&quantified.over));
                } else {
                    push(Reason::Unclassified);
                }
            }
            // The same two cases: an unobserved list, or a key read under the binder that is absent
            // or outside its kind, which `explain_path` cannot name for the reason above.
            Predicate::Distinct(distinct) => {
                if self.cardinality(&distinct.over).is_none() {
                    push(self.explain_path(&distinct.over));
                } else {
                    push(Reason::Unclassified);
                }
            }
            Predicate::Always
            | Predicate::Never
            | Predicate::All(_)
            | Predicate::Any(_)
            | Predicate::Not(_)
            | Predicate::Defined(_) => push(Reason::Unclassified),
            // A window over a fact is `Unknown` where the fact is unbound, as `Truthy` is; over
            // `now`, or over text that names no instant, it is not classified.
            Predicate::Window(window) => match window.at.fact_path() {
                Some(path) if self.observe(path).is_none() => push(self.explain_path(path)),
                _ => push(Reason::Unclassified),
            },
        }
    }

    /// One operand's value, or `None` when it reads a path nothing bound.
    fn resolve(&self, operand: &Operand) -> Option<FactValue> {
        match operand {
            Operand::Fact(path) => self.observe(path),
            Operand::Literal(value) => Some(value.clone()),
            // An offset names no value of its own; its base is explained as a path.
            Operand::Offset(_) => None,
            Operand::Derived(derived) => derived.value(self),
        }
    }

    /// Why a path nothing bound is unbound, resolved against the command's input types.
    ///
    /// This is the whole difference between "supply a value" and "fix the specification", and it is
    /// only answerable with the types in hand — which is why [`InputFacts`] carries the IR.
    fn explain_path(&self, path: &FactPath) -> Reason {
        match resolve_path(self.ir, &self.command.input, path) {
            Target::Scalar => Reason::ValueAbsent { path: path.clone() },
            Target::Aggregate(holds) => Reason::PathNotScalar {
                path: path.clone(),
                holds,
            },
            Target::Undeclared(segment) => Reason::PathNotDeclared {
                path: path.clone(),
                segment,
            },
            Target::TooDeep => Reason::TypeTooDeep {
                path: path.clone(),
                limit: MAX_TYPE_DEPTH,
            },
        }
    }
}

impl FactSource for InputFacts<'_> {
    fn fact(&self, path: &FactPath) -> Option<FactValue> {
        self.facts.fact(path)
    }

    fn present(&self, path: &FactPath) -> bool {
        self.facts.present(path)
    }

    fn scales(&self) -> &Scales {
        &self.scales
    }

    /// Every path but a declared `Duration`: an ESS specification declares no scale, and every ESS
    /// lane orders text by bytes — but a `Duration` has no ordering (validate refuses one), and its
    /// ISO 8601 bytes would put `PT10M` below `PT5M`.
    fn orders_text_by_bytes(&self, path: &FactPath) -> bool {
        !declared_as(self.ir, &self.command.input, path, Primitive::Duration)
    }

    /// A path whose declared terminal type is `Timestamp`, through any newtype or `Optional`.
    fn orders_as_instant(&self, path: &FactPath) -> bool {
        declared_as(self.ir, &self.command.input, path, Primitive::Timestamp)
    }

    /// The reference instant a candidate is decided against where a guard reads the current time
    /// (beyond10x/ess#171): a candidate is an offset from it, sent as a `now_offset`.
    fn now(&self) -> Option<ess_primitives::time::Rfc3339Instant> {
        Some(crate::now_offset::reference())
    }
}

/// Where a fact path lands in a set of declared fields.
///
/// Public because the same question is asked of two different surfaces. A guard reads a *command's
/// input*, and an entity invariant read against a *view* asks the identical question of the fields
/// that view publishes: does `total.amount` land on something a fact value can hold, or on nothing
/// at all? One walk answers both, and a second one written beside it would be a second opinion about
/// what `Optional<Money>` exposes.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    /// A primitive or an enum: something a fact value can hold.
    Scalar,
    /// A construct no fact value can hold, named as it reads in a diagnostic.
    Aggregate(&'static str),
    /// The segment that names nothing.
    Undeclared(String),
    /// The walk exceeded [`MAX_TYPE_DEPTH`].
    TooDeep,
}

impl Target {
    /// `true` only where the path lands on something a predicate can compare.
    ///
    /// Which is what "this surface publishes what the predicate reads" means: the other three cases
    /// all end in a predicate that evaluates to `Unknown`, and `Unknown` refuses.
    pub fn is_scalar(&self) -> bool {
        matches!(self, Self::Scalar)
    }
}

/// Resolves a fact path against a set of declared fields, without any value in hand.
///
/// The fields of a command's input, of a view's projection, or of anything else the model declares
/// as a flat list of named, typed members.
pub fn resolve_path(ir: &EssIr, fields: &[ResolvedField], path: &FactPath) -> Target {
    match ess_compiler::expression::resolve_path(ir, fields, path, "conformance input") {
        Ok(resolved) => projection_target(ir, &resolved),
        Err(error) if error.code == ess_primitives::error::ValidationCode::SelfReference => {
            Target::TooDeep
        }
        Err(error) => match error.boundary {
            Some(boundary) => Target::Aggregate(boundary),
            None => Target::Undeclared(error.segment.unwrap_or_else(|| path.to_string())),
        },
    }
}

/// Classifies producer support after semantic resolution; terminal Number alone is insufficient.
pub(crate) fn projection_target(
    ir: &EssIr,
    resolved: &ess_domain::expression::Resolution<ResolvedTypeRef>,
) -> Target {
    if matches!(
        resolved.terminal,
        ResolvedTypeRef::Primitive {
            name: Primitive::Binary64
        }
    ) {
        return Target::Aggregate("an unsupported Binary64 scalar");
    }
    if resolved.access.depth > MAX_TYPE_DEPTH {
        return Target::TooDeep;
    }
    if resolved.access.collection {
        return Target::Aggregate("a collection");
    }
    // A text length is a leaf the evaluator derives, and no suite format carries one to a view
    // row: it is asserted where values are built, on command input and setup.
    if resolved.access.text_length {
        return Target::Aggregate("a text length");
    }
    if resolved.scalar.is_some() {
        return Target::Scalar;
    }
    match &resolved.terminal {
        ResolvedTypeRef::List { .. } => Target::Aggregate("a list"),
        ResolvedTypeRef::Map { .. } => Target::Aggregate("a map"),
        ResolvedTypeRef::Declared { name } => match ir.named_type(name).body {
            ResolvedBody::Struct { .. } => Target::Aggregate("a struct"),
            ResolvedBody::Union { .. } => Target::Aggregate("a union"),
            _ => Target::Aggregate("an aggregate"),
        },
        _ => Target::Aggregate("an aggregate"),
    }
}

/// Whether every checked read is available from the current typed scalar projector.
pub(crate) fn predicate_projectable(
    ir: &EssIr,
    fields: &[ResolvedField],
    predicate: &Predicate,
) -> bool {
    let checked =
        ess_compiler::expression::check_predicate(ir, fields, predicate, "conformance projection");
    let mut presence = BTreeSet::new();
    presence_reads(predicate, &mut presence);
    let sequences = crate::expression_format::binds_sequences(predicate);
    checked.errors.is_empty()
        && checked.reads.iter().all(|read| {
            projection_target(ir, &read.resolution).is_scalar()
                || aggregate_presence(ir, read, &presence)
                || (sequences && sequence_read(ir, read))
                || lifted_text_length(ir, read, predicate)
        })
}

/// Whether `read` is one a runner answers from a row whose sequences are bound element by element
/// (final review decision 1, suite `/40`): a list a `distinct` or a quantifier walks, or a scalar
/// read through one of its elements. Every other read keeps [`projection_target`]'s answer.
fn sequence_read(ir: &EssIr, read: &ess_domain::expression::Read<ResolvedTypeRef>) -> bool {
    let resolution = &read.resolution;
    if resolution.access.depth > MAX_TYPE_DEPTH || resolution.access.text_length {
        return false;
    }
    if read.collection_target {
        return matches!(resolution.terminal, ResolvedTypeRef::List { .. });
    }
    resolution.scalar.is_some()
        && !matches!(
            projection_target(ir, resolution),
            Target::Aggregate("an unsupported Binary64 scalar") | Target::TooDeep
        )
}

/// Whether `read` is a text's `.count` in a predicate that already needs suite `/40`
/// (`docs/design/expression-family-source22.md`, final review decision 1): every runner derives a
/// text's length from the text a row publishes, so `/40` and `/41` carry it in `satisfies` beside
/// `{utf8_bytes: …}`. A predicate needing no `/40` vocabulary keeps refusing it, so a suite that
/// carried none keeps its format and bytes.
pub(crate) fn lifted_text_length(
    ir: &EssIr,
    read: &ess_domain::expression::Read<ResolvedTypeRef>,
    predicate: &Predicate,
) -> bool {
    projection_target(ir, &read.resolution) == Target::Aggregate("a text length")
        && crate::expression_format::reads(predicate)
}

/// Whether `read` is a `defined()` (or `missing()`) of an `Optional` struct, union, list, map or
/// `Json`, which a surface publishes as its presence (beyond10x/ess#176). `presence` is what
/// [`presence_reads`] found in the same predicate.
///
/// The one answer to that question: synthesis projects such a read, an authored `satisfies`
/// admits it, and a suite carrying it takes suite/26 ([`crate::defined_aggregates`]).
pub(crate) fn aggregate_presence(
    ir: &EssIr,
    read: &ess_domain::expression::Read<ResolvedTypeRef>,
    presence: &BTreeSet<FactPath>,
) -> bool {
    read.resolution.optional
        && presence.contains(&read.path)
        && matches!(
            projection_target(ir, &read.resolution),
            Target::Aggregate("a struct" | "a union" | "a list" | "a map" | "an aggregate")
        )
}

/// Every path a `defined()` under `predicate` reads, binder names kept as written.
///
/// An `Optional` aggregate is projected as its presence and nothing else (beyond10x/ess#176), so a
/// surface publishing one publishes what `defined()` asks of it, and no other read.
pub(crate) fn presence_reads(predicate: &Predicate, found: &mut BTreeSet<FactPath>) {
    match predicate {
        Predicate::Defined(path) => {
            found.insert(path.clone());
        }
        Predicate::All(children) | Predicate::Any(children) => {
            for child in children {
                presence_reads(child, found);
            }
        }
        Predicate::Not(inner) => presence_reads(inner, found),
        Predicate::Forall(quantified) | Predicate::Exists(quantified) => {
            presence_reads(&quantified.body, found);
        }
        _ => {}
    }
}

/// Binds one fact per scalar leaf of `value`, guided by `type_ref`.
/// The authored names of an enum's variants.
///
/// A variant's declared wire spelling is what a document carries; these errors are read by the
/// person who wrote the scenario, so they name what that person typed.
fn variant_names(variants: &[ess_domain::types::EnumVariant]) -> Vec<String> {
    variants
        .iter()
        .map(|variant| variant.name().to_owned())
        .collect()
}

/// Binds `value` at `path`, and records it as present where it is an aggregate ([`mark_aggregate`]).
fn project(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    value: &Node,
    path: &FactPath,
    depth: usize,
    facts: &mut FactStore,
    errors: &mut Vec<ShapeError>,
) {
    mark_aggregate(type_ref, value, path, facts);
    project_value(ir, type_ref, value, path, depth, facts, errors);
}

/// A content member written beside a unit variant's tag (ess/22), as the undeclared field it is.
///
/// A unit variant declares nothing beside its tag; a payload variant keeps the shape-only reading
/// the gate has always given a union.
fn unit_variant_payload(
    tag: &str,
    variants: &std::collections::BTreeMap<String, Option<ResolvedTypeRef>>,
    entries: &std::collections::BTreeMap<String, Node>,
    path: &FactPath,
) -> Option<ShapeError> {
    let content = ess_gen::schema::union_content_key(tag);
    let unit = entries
        .get(tag)
        .and_then(Node::as_text)
        .and_then(|label| variants.get(label))
        .is_some_and(Option::is_none);
    (unit && entries.contains_key(content)).then(|| ShapeError::UndeclaredField {
        at: path.to_string(),
        field: content.to_owned(),
    })
}

fn project_value(
    ir: &EssIr,
    type_ref: &ResolvedTypeRef,
    value: &Node,
    path: &FactPath,
    depth: usize,
    facts: &mut FactStore,
    errors: &mut Vec<ShapeError>,
) {
    if depth > MAX_TYPE_DEPTH {
        errors.push(ShapeError::TooDeep {
            at: path.to_string(),
            limit: MAX_TYPE_DEPTH,
        });
        return;
    }
    let wrong = |errors: &mut Vec<ShapeError>, expected: String| {
        errors.push(ShapeError::WrongShape {
            at: path.to_string(),
            expected,
            found: value.type_name(),
        });
    };

    match type_ref {
        // An absent optional binds nothing, which is a fact about the candidate rather than an
        // error in it. The guard reading it then refuses as `ValueAbsent`.
        ResolvedTypeRef::Optional { of } => {
            if !matches!(value, Node::Null) {
                project(ir, of, value, path, depth + 1, facts, errors);
            }
        }
        // Any JSON value, and no fact: a predicate reads nothing from one (beyond10x/ess#138).
        ResolvedTypeRef::Primitive {
            name: Primitive::Json,
        } => {}
        ResolvedTypeRef::Primitive { name } => match primitive_value(*name, value) {
            Some(fact) => facts.set(path.clone(), fact),
            None => wrong(errors, name.to_string()),
        },
        // A list publishes its size as `<path>.count` and each element under `<path>.<index>`,
        // the convention `FactSource::cardinality` reads for an observed collection — so a
        // quantifier and a `.count` guard over command input are decided (ess#94).
        ResolvedTypeRef::List { of } => match value {
            Node::Seq(items) => project_list(ir, of, items.iter(), path, depth, facts, errors),
            _ => wrong(errors, format!("{type_ref}")),
        },
        // A map publishes its size as `<path>.count`, as a list does, so a `.count` guard over one
        // is decided (beyond10x/ess#196), and its values under `<path>.<index>` in key order — what
        // a quantifier binds, as the compiler and Entity Runtime read it — so a quantifier over one
        // is decided (beyond10x/ess#240). No path reaches a key.
        ResolvedTypeRef::Map { value: of, .. } => match value {
            Node::Map(map) => project_list(ir, of, map.values(), path, depth, facts, errors),
            _ => wrong(errors, format!("{type_ref}")),
        },
        ResolvedTypeRef::Declared { name } => {
            let declared = ir.named_type(name);
            match &declared.body {
                // Transparent: a newtype wraps a representation rather than naming a member, so
                // there is no segment for the inside of one and the path does not grow.
                ResolvedBody::Newtype { of, .. } => {
                    project(ir, of, value, path, depth + 1, facts, errors);
                }
                ResolvedBody::Enum { variants } => match value.as_text() {
                    Some(text) if variants.iter().any(|variant| variant == text) => {
                        facts.set(path.clone(), FactValue::text(text));
                    }
                    // A string that is not one of the variants is a different mistake from a
                    // number where a name was wanted, and only the first can be answered by
                    // showing the closed set the model declares. A generated candidate never
                    // reaches it — the witness walk draws from the variants — so this exists for
                    // the surface where a person types the name: `authored`.
                    Some(text) => errors.push(ShapeError::UndeclaredVariant {
                        at: path.to_string(),
                        declared_by: name.to_string(),
                        value: text.to_owned(),
                        variants: variant_names(variants),
                    }),
                    None => wrong(
                        errors,
                        format!("one of {}", variant_names(variants).join(", ")),
                    ),
                },
                // Shape only, as for a list: the tag is a text a fact could hold, and binding it is
                // a decision this gate does not take. A unit variant's content member is refused.
                ResolvedBody::Union { tag, variants } => match value {
                    Node::Map(entries) => {
                        errors.extend(unit_variant_payload(tag, variants, entries, path));
                    }
                    _ => wrong(errors, format!("{name} as a mapping")),
                },
                ResolvedBody::Struct { fields, .. } => {
                    let Some(entries) = value.as_map() else {
                        wrong(errors, format!("{name} as a mapping"));
                        return;
                    };
                    for field in fields {
                        let child = path.child(&field.name);
                        match entries.get(&field.name) {
                            Some(inner) => {
                                project(
                                    ir,
                                    &field.type_ref,
                                    inner,
                                    &child,
                                    depth + 1,
                                    facts,
                                    errors,
                                );
                            }
                            None if admits_absence(ir, &field.type_ref, 0) => {}
                            None => errors.push(ShapeError::MissingField {
                                at: path.to_string(),
                                field: field.name.clone(),
                            }),
                        }
                    }
                    for supplied in entries.keys() {
                        if !fields.iter().any(|field| &field.name == supplied) {
                            errors.push(ShapeError::UndeclaredField {
                                at: path.to_string(),
                                field: supplied.clone(),
                            });
                        }
                    }
                }
            }
        }
    }
}

/// Records a struct, list, map, union or `Json` value as present at `path`.
///
/// None of them binds a fact at its own path — a struct and a list bind their leaves, a map, a
/// union and `Json` nothing — so presence is recorded beside the facts, and an empty one is present
/// too: what `defined()` over an `Optional` one reads (beyond10x/ess#176). `null` is absent. A value
/// of the wrong shape is marked as well, and refused by the walk.
fn mark_aggregate(
    type_ref: &ResolvedTypeRef,
    value: &Node,
    path: &FactPath,
    facts: &mut FactStore,
) {
    let json = matches!(
        type_ref,
        ResolvedTypeRef::Primitive {
            name: Primitive::Json
        }
    );
    if matches!(value, Node::Map(_) | Node::Seq(_)) || (json && !matches!(value, Node::Null)) {
        facts.mark_present(path.clone());
    }
}

/// Binds `<path>.count` and each element of a list, or each value of a map in key order, under
/// `<path>.<index>`.
fn project_list<'n>(
    ir: &EssIr,
    of: &ResolvedTypeRef,
    elements: impl ExactSizeIterator<Item = &'n Node>,
    path: &FactPath,
    depth: usize,
    facts: &mut FactStore,
    errors: &mut Vec<ShapeError>,
) {
    facts.set(path.child("count"), FactValue::count(elements.len()));
    for (index, element) in elements.enumerate() {
        project(
            ir,
            of,
            element,
            &path.child(&index.to_string()),
            depth + 1,
            facts,
            errors,
        );
    }
}

/// The fact value a primitive-typed node projects to, or `None` when the node is the wrong shape.
///
/// Crate-visible rather than private because [`Holds::admits`](crate::Holds::admits) asks the same
/// question of an event's payload. One table, so that a payload and a command input cannot come to
/// different conclusions about whether `1.5` is an `Integer`.
pub(crate) fn primitive_value(primitive: Primitive, value: &Node) -> Option<FactValue> {
    primitive.admits(value)
}

/// Whether `value` is a value of `primitive`: [`primitive_value`], and any value at all for `Json`,
/// which admits every JSON value and projects to no fact (beyond10x/ess#138).
pub(crate) fn primitive_admits(primitive: Primitive, value: &Node) -> bool {
    primitive == Primitive::Json || primitive_value(primitive, value).is_some()
}

/// A candidate that is not a value of the command's declared input type.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ShapeError {
    /// The candidate supplied nothing for a field that is not optional.
    MissingField {
        /// The path of the value that should have carried it; empty at the input's root.
        at: String,
        /// The field's name.
        field: String,
    },
    /// The candidate supplied a field no type declares.
    UndeclaredField {
        /// The path of the value that carried it; empty at the input's root.
        at: String,
        /// The name supplied.
        field: String,
    },
    /// A value is not of the shape its declared type calls for.
    WrongShape {
        /// Where it sits.
        at: String,
        /// What the type calls for.
        expected: String,
        /// What the candidate carried.
        found: &'static str,
    },
    /// A value names something the declared enum does not have as a variant.
    ///
    /// Separate from [`WrongShape`](Self::WrongShape) because the repair is different and the
    /// message can be exact: the model declares a closed set, so the answer is the set rather than
    /// the name of a shape.
    UndeclaredVariant {
        /// Where it sits.
        at: String,
        /// The declared type the value had to be a variant of.
        declared_by: String,
        /// The name supplied.
        value: String,
        /// What the model declares, in declaration order.
        variants: Vec<String>,
    },
    /// Projection walked deeper than [`MAX_TYPE_DEPTH`], which a type referring to itself is the
    /// only way to do.
    TooDeep {
        /// Where it gave up.
        at: String,
        /// The limit it exceeded.
        limit: usize,
    },
    /// An input field whose name cannot be spelled as a fact path, so no predicate could read it.
    ///
    /// Unreachable from a document — the parser holds a field name to
    /// [`Field::PATTERN`](ess_domain::types::Field::PATTERN), which is stricter than a fact path
    /// segment. Reachable from a `CommandSpec` built in code, which is how every fixture in this
    /// workspace's compiler tests is built.
    UnnameableField {
        /// The name that cannot be a path.
        field: String,
    },
}

impl fmt::Display for ShapeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingField { at, field } => {
                write!(f, "{}: nothing supplied for `{field}`", at_or_root(at))
            }
            Self::UndeclaredField { at, field } => {
                write!(f, "{}: `{field}` is not declared there", at_or_root(at))
            }
            Self::WrongShape {
                at,
                expected,
                found,
            } => write!(f, "{at}: expected {expected}, found {found}"),
            Self::UndeclaredVariant {
                at,
                declared_by,
                value,
                variants,
            } => write!(
                f,
                "{}: `{value}` is not a variant of `{declared_by}`; it declares {}",
                at_or_root(at),
                variants.join(", ")
            ),
            Self::TooDeep { at, limit } => {
                write!(f, "{at}: walked deeper than {limit} types")
            }
            Self::UnnameableField { field } => {
                write!(f, "`{field}` cannot be spelled as a fact path")
            }
        }
    }
}

/// How a path reads in a message when it is the input's root.
fn at_or_root(at: &str) -> &str {
    if at.is_empty() {
        "the input"
    } else {
        at
    }
}

/// Every way a candidate failed to be a value of the input's type.
///
/// Accumulates, so a candidate with three wrong fields reports three — invariant 3, on the path a
/// runner takes rather than the one a document does.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ShapeErrors(Vec<ShapeError>);

impl ShapeErrors {
    /// Every mismatch, in the order the input declares the fields it is about.
    pub fn iter(&self) -> impl Iterator<Item = &ShapeError> {
        self.0.iter()
    }

    /// The mismatches whose position `answered` does not claim, or `None` when it claims them
    /// all: for a caller that has already answered what sits at some positions itself — an
    /// authored `{$instance: …}` inside a list (beyond10x/ess#242).
    pub(crate) fn without(self, answered: impl Fn(&str) -> bool) -> Option<Self> {
        let kept: Vec<ShapeError> = self
            .0
            .into_iter()
            .filter(|error| {
                let at = match error {
                    ShapeError::MissingField { at, .. }
                    | ShapeError::UndeclaredField { at, .. }
                    | ShapeError::WrongShape { at, .. }
                    | ShapeError::UndeclaredVariant { at, .. }
                    | ShapeError::TooDeep { at, .. } => Some(at.as_str()),
                    ShapeError::UnnameableField { .. } => None,
                };
                !at.is_some_and(&answered)
            })
            .collect();
        (!kept.is_empty()).then_some(Self(kept))
    }

    /// How many.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// `true` when there are none, which [`flatten`] never returns.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}

impl fmt::Display for ShapeErrors {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for (index, error) in self.0.iter().enumerate() {
            if index > 0 {
                f.write_str("\n")?;
            }
            write!(f, "{error}")?;
        }
        Ok(())
    }
}

impl std::error::Error for ShapeErrors {}

#[cfg(test)]
mod tests {
    use super::*;

    use ess_primitives::facts::Number;

    fn number(value: f64) -> Node {
        Node::Number(Number::new(value).expect("a finite number"))
    }

    #[test]
    fn every_primitive_projects_to_the_one_fact_value_that_can_hold_it() {
        let text = Node::Text("x".to_owned());
        let table: [(Primitive, Node, Option<FactValue>); 10] = [
            (
                Primitive::Boolean,
                Node::Bool(true),
                Some(FactValue::Bool(true)),
            ),
            (
                Primitive::Decimal,
                number(1.5),
                Some(FactValue::Number(Number::new(1.5).expect("finite"))),
            ),
            (
                Primitive::Integer,
                number(2.0),
                Some(FactValue::Number(Number::from(2_i64))),
            ),
            (Primitive::String, text.clone(), Some(FactValue::text("x"))),
            (Primitive::Uuid, text.clone(), None),
            (
                Primitive::Uuid,
                Node::Text("0f8fad5b-d9cb-469f-a165-70867728950e".to_owned()),
                Some(FactValue::text("0f8fad5b-d9cb-469f-a165-70867728950e")),
            ),
            (
                Primitive::Timestamp,
                text.clone(),
                Some(FactValue::text("x")),
            ),
            (
                Primitive::Duration,
                text.clone(),
                Some(FactValue::text("x")),
            ),
            (Primitive::Bytes, text, None),
            (
                Primitive::Bytes,
                Node::Text("AA==".to_owned()),
                Some(FactValue::text("AA==")),
            ),
        ];
        for (primitive, node, expected) in table {
            assert_eq!(
                primitive_value(primitive, &node),
                expected,
                "{primitive} projects from {}",
                node.type_name()
            );
        }
    }

    #[test]
    fn a_primitive_refuses_a_node_of_the_wrong_shape_rather_than_coercing_it() {
        assert_eq!(
            primitive_value(Primitive::Boolean, &Node::Text("true".to_owned())),
            None,
            "the string `true` is not a boolean; accepting it would let a candidate decide \
             `express == true` in a way the system under test does not"
        );
        assert_eq!(
            primitive_value(Primitive::Integer, &number(1.5)),
            None,
            "rounding here would decide `quantity == 1` differently from the system under test"
        );
        assert_eq!(primitive_value(Primitive::String, &number(1.0)), None);
        assert_eq!(primitive_value(Primitive::Decimal, &Node::Null), None);
    }

    #[test]
    fn shape_errors_render_one_per_line_and_name_the_input_root_by_name() {
        let errors = ShapeErrors(vec![
            ShapeError::MissingField {
                at: String::new(),
                field: "currency".to_owned(),
            },
            ShapeError::UndeclaredField {
                at: "amount".to_owned(),
                field: "vat".to_owned(),
            },
        ]);
        let rendered = errors.to_string();

        assert_eq!(errors.len(), 2);
        assert!(!errors.is_empty());
        assert_eq!(rendered.lines().count(), 2);
        assert!(
            rendered.contains("the input: nothing supplied for `currency`"),
            "an empty path is the input itself, not a blank: {rendered}"
        );
        assert!(rendered.contains("amount: `vat` is not declared there"));
    }
}
