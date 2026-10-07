//! Values whose implementation-owned content was not recorded by an outcome-only history.
use super::super::{input, EssIr, Node, ResolvedBody, ResolvedTypeRef, Undetermined};
use ess_compiler::ir::ResolvedField;
use ess_domain::types::Primitive;
use std::collections::{BTreeMap, BTreeSet};

/// A stable source location, independent of the order in which history search visits it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Origin {
    pub(crate) operation: String,
    pub(crate) branch: String,
    pub(crate) location: Vec<String>,
}

impl Origin {
    pub(crate) fn child(&self, name: &str) -> Self {
        let mut origin = self.clone();
        origin.location.push(name.into());
        origin
    }
}

/// Absence is distinct from an observed null; unknown content never masquerades as a Node.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Value {
    Absent,
    Known(Node),
    Object(BTreeMap<String, Self>),
    Unknown {
        declared: ResolvedTypeRef,
        origin: Origin,
        /// The complete admitted domain when bounded enumeration established one. This is proof
        /// material for universal transfers, never an observed or selected representative.
        domain: Option<Vec<Node>>,
    },
}

impl Value {
    pub(crate) fn require(&self, what: &str) -> Result<Node, Undetermined> {
        match self.concrete() {
            Ok(Some(value)) => Ok(value),
            Ok(None) => Err(Undetermined::NoValue { what: what.into() }),
            Err(()) => Err(Undetermined::Undecidable {
                outcome: "history value read".into(),
                guard: format!("unrecorded generated value needed for {what}"),
            }),
        }
    }
    pub(crate) fn unobserved(&self) -> bool {
        match self {
            Self::Unknown { .. } => true,
            Self::Object(fields) => fields.values().any(Self::unobserved),
            Self::Absent | Self::Known(_) => false,
        }
    }

    /// A concrete value only when every contained member is actually known.
    pub(crate) fn concrete(&self) -> Result<Option<Node>, ()> {
        match self {
            Self::Absent => Ok(None),
            Self::Known(value) => Ok(Some(value.clone())),
            Self::Unknown { .. } => Err(()),
            Self::Object(fields) => {
                let mut result = BTreeMap::new();
                for (name, field) in fields {
                    if let Some(value) = field.concrete()? {
                        result.insert(name.clone(), value);
                    }
                }
                Ok(Some(Node::Map(result)))
            }
        }
    }

    pub(crate) fn presence(&self, ir: &EssIr) -> Option<bool> {
        match self {
            Self::Absent | Self::Known(Node::Null) => Some(false),
            Self::Known(_) | Self::Object(_) => Some(true),
            Self::Unknown { declared, .. } => {
                if only_absent(ir, declared, 0) {
                    Some(false)
                } else {
                    (!nullable(ir, declared, 0)).then_some(true)
                }
            }
        }
    }
}

/// Whether absence/null can occur in the representation, without choosing that possibility.
fn nullable(ir: &EssIr, declared: &ResolvedTypeRef, depth: usize) -> bool {
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return true;
    }
    match declared {
        ResolvedTypeRef::Optional { .. }
        | ResolvedTypeRef::Primitive {
            name: Primitive::Json,
        } => true,
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => nullable(ir, of, depth + 1),
            _ => false,
        },
        _ => false,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Feasibility {
    Inhabited,
    Empty(&'static str),
    Unresolved,
}

/// Establish existence under the complete declared type, including joint constraints.
/// A validated sample is discarded: its content never becomes history authority.
#[cfg(test)]
pub(crate) fn feasibility(ir: &EssIr, declared: &ResolvedTypeRef) -> Feasibility {
    feasibility_cached(ir, declared, &mut BTreeMap::new())
}

#[cfg(test)]
fn feasibility_cached(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    domains: &mut BTreeMap<String, Feasibility>,
) -> Feasibility {
    let mut context = ProofContext::new(domains, 16_384);
    context.prove(ir, declared, 0)
}

struct ProofContext<'a> {
    completed: &'a mut BTreeMap<String, Feasibility>,
    active: BTreeSet<String>,
    budget: input::ProofBudget,
}

impl<'a> ProofContext<'a> {
    fn new(completed: &'a mut BTreeMap<String, Feasibility>, work: usize) -> Self {
        Self {
            completed,
            active: BTreeSet::new(),
            budget: input::ProofBudget::new(work),
        }
    }
    fn prove(&mut self, ir: &EssIr, declared: &ResolvedTypeRef, depth: usize) -> Feasibility {
        if self.budget.charge(1).is_err() {
            return Feasibility::Unresolved;
        }
        if self.budget.type_ref(declared).is_err() {
            return Feasibility::Unresolved;
        }
        let key = declared.to_string();
        if self.budget.charge(key.len()).is_err() {
            return Feasibility::Unresolved;
        }
        if let Some(proof) = self.completed.get(&key) {
            return *proof;
        }
        if depth > ess_domain::types::MAX_TYPE_DEPTH || !self.active.insert(key.clone()) {
            return Feasibility::Unresolved;
        }
        let proof = prove_domain(ir, declared, self, depth);
        self.active.remove(&key);
        if !matches!(proof, Feasibility::Unresolved) {
            self.completed.insert(key, proof);
        }
        proof
    }
}

fn prove_domain(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    context: &mut ProofContext<'_>,
    depth: usize,
) -> Feasibility {
    if matches!(declared, ResolvedTypeRef::Optional { .. }) {
        return Feasibility::Inhabited;
    }
    if let Some((Some(lower), Some(upper))) = integer_bounds(ir, declared, 0, &context.budget) {
        if lower > upper {
            return Feasibility::Empty("contradictory exact Integer bounds");
        }
    }
    // A complete candidate can settle existence without walking every mandatory dependency.
    // This is especially important for productive recursive shapes and long named chains.
    if let Some(value) = crate::witness::proof_base(ir, declared, &context.budget) {
        if input::validate_typed_value_bounded(ir, declared, &value, &context.budget).is_ok() {
            return Feasibility::Inhabited;
        }
    }
    if let ResolvedTypeRef::Declared { name } = declared {
        match &ir.named_type(name).body {
            ResolvedBody::Struct { fields, .. } => {
                for field in fields {
                    if context.budget.exhausted() {
                        return Feasibility::Unresolved;
                    }
                    if absence(ir, &field.type_ref, 0, &context.budget) == Some(false)
                        && matches!(
                            context.prove(ir, &field.type_ref, depth + 1),
                            Feasibility::Empty(_)
                        )
                    {
                        return Feasibility::Empty("a required struct member has an empty domain");
                    }
                }
            }
            ResolvedBody::Newtype { of, .. }
                if matches!(context.prove(ir, of, depth + 1), Feasibility::Empty(_)) =>
            {
                return Feasibility::Empty("the newtype representation has an empty domain");
            }
            _ => {}
        }
    }
    // Exhaustive finite domains prove both inhabited and empty results. Failure to enumerate a
    // larger/infinite domain is not an emptiness proof.
    if let Some(values) = finite_in(ir, declared, 0, context) {
        if !values.values.is_empty() {
            Feasibility::Inhabited
        } else if values.complete {
            Feasibility::Empty("exhaustive finite-domain validation")
        } else {
            Feasibility::Unresolved
        }
    } else {
        Feasibility::Unresolved
    }
}

const FINITE_LIMIT: usize = 256;

/// Validate an abstract transfer from retained type guarantees and actual known structure.
/// Unknown constraints remain a decision; a representative is never substituted to validate them.
pub(crate) fn validate(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    value: &Value,
) -> Result<(), Undetermined> {
    validate_at(ir, declared, value, 0)
}

fn validate_at(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    value: &Value,
    depth: usize,
) -> Result<(), Undetermined> {
    let unresolved = || Undetermined::Undecidable {
        outcome: "history value validation".into(),
        guard: format!("unrecorded value must satisfy `{declared}`"),
    };
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return Err(unresolved());
    }
    match value.concrete() {
        Ok(Some(known)) => {
            return input::validate_typed_value(ir, declared, &known)
                .map_err(Undetermined::Request);
        }
        Ok(None) => {
            let field = ResolvedField {
                name: "value".into(),
                type_ref: declared.clone(),
                naming: ess_domain::name::Naming::default(),
            };
            return input::bind(ir, &[field], &BTreeMap::new(), input::Completeness::Total)
                .map(|_| ())
                .map_err(|error| Undetermined::Request(error.to_string()));
        }
        Err(()) => {}
    }
    if let Value::Unknown {
        declared: source, ..
    } = value
    {
        return if ess_domain::types::is_assignable(
            &crate::accessor::unresolve(source),
            &crate::accessor::unresolve(declared),
        ) {
            Ok(())
        } else {
            Err(unresolved())
        };
    }
    let Value::Object(values) = value else {
        return Err(unresolved());
    };
    match declared {
        ResolvedTypeRef::Optional { of } => validate_at(ir, of, value, depth + 1),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Struct { fields, invariants } => {
                let mut unresolved = None;
                for field in fields {
                    retain_unresolved(
                        validate_at(
                            ir,
                            &field.type_ref,
                            values.get(&field.name).unwrap_or(&Value::Absent),
                            depth + 1,
                        ),
                        &mut unresolved,
                    )?;
                }
                if values
                    .keys()
                    .any(|key| !fields.iter().any(|field| &field.name == key))
                {
                    return Err(Undetermined::Request(
                        "undeclared abstract struct member".into(),
                    ));
                }
                let facts = super::Facts::new(ir, fields, values)?;
                retain_unresolved(
                    validate_invariants(invariants, &facts, declared),
                    &mut unresolved,
                )?;
                unresolved.map_or(Ok(()), Err)
            }
            ResolvedBody::Newtype { of, invariants, .. } => {
                let mut unresolved = None;
                retain_unresolved(validate_at(ir, of, value, depth + 1), &mut unresolved)?;
                let fields = [ResolvedField {
                    name: "value".into(),
                    type_ref: of.clone(),
                    naming: ess_domain::name::Naming::default(),
                }];
                let wrapped = BTreeMap::from([("value".into(), value.clone())]);
                let facts = super::Facts::new(ir, &fields, &wrapped)?;
                retain_unresolved(
                    validate_invariants(invariants, &facts, declared),
                    &mut unresolved,
                )?;
                unresolved.map_or(Ok(()), Err)
            }
            _ => Err(Undetermined::Request(
                "constructed value is not a struct".into(),
            )),
        },
        _ => Err(Undetermined::Request(
            "constructed value is not a declared struct".into(),
        )),
    }
}

/// Keep an incomplete abstract proof pending so a later definite invalidity can dominate it.
fn retain_unresolved(
    result: Result<(), Undetermined>,
    unresolved: &mut Option<Undetermined>,
) -> Result<(), Undetermined> {
    match result {
        Err(error @ Undetermined::Undecidable { .. }) => {
            unresolved.get_or_insert(error);
            Ok(())
        }
        result => result,
    }
}

fn validate_invariants(
    invariants: &[ess_domain::entity::Invariant],
    facts: &dyn ess_primitives::facts::FactSource,
    declared: &ResolvedTypeRef,
) -> Result<(), Undetermined> {
    use ess_primitives::predicate::Truth;
    let mut unresolved = None;
    for invariant in invariants {
        match invariant.predicate.evaluate(facts) {
            Truth::True => {}
            Truth::False => {
                return Err(Undetermined::Request(format!(
                    "`{declared}` invariant `{}` is false",
                    invariant.statement
                )));
            }
            Truth::Unknown => {
                unresolved.get_or_insert_with(|| Undetermined::Undecidable {
                    outcome: format!("validation of `{declared}`"),
                    guard: invariant.statement.clone(),
                });
            }
        }
    }
    unresolved.map_or(Ok(()), Err)
}

pub(in super::super) fn finite(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    depth: usize,
) -> Option<Vec<Node>> {
    let mut completed = BTreeMap::new();
    let result = finite_in(
        ir,
        declared,
        depth,
        &mut ProofContext::new(&mut completed, 16_384),
    )?;
    result.complete.then_some(result.values)
}

/// A valid candidate proves existence even when other candidates remain undecided. Only a
/// complete enumeration may prove emptiness or supply universally true finite-domain facts.
struct Enumeration {
    values: Vec<Node>,
    complete: bool,
}

#[allow(
    clippy::too_many_lines,
    reason = "complete bounded enumeration handles every finite type shape in one recursive match"
)]
fn finite_in(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    depth: usize,
    context: &mut ProofContext<'_>,
) -> Option<Enumeration> {
    context.budget.charge(1).ok()?;
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return None;
    }
    let mut complete = true;
    let candidates = match declared {
        ResolvedTypeRef::Primitive {
            name: Primitive::Boolean,
        } => vec![Node::Bool(false), Node::Bool(true)],
        ResolvedTypeRef::Optional { of } => {
            let inner = finite_in(ir, of, depth + 1, context);
            complete = inner.as_ref().is_some_and(|inner| inner.complete);
            let mut values = inner.map_or_else(Vec::new, |inner| inner.values);
            values.push(Node::Null);
            values
        }
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => {
                if let Some((Some(lower), Some(upper))) =
                    integer_bounds(ir, declared, depth + 1, &context.budget)
                {
                    if lower > upper {
                        Vec::new()
                    } else if upper - lower < i128::try_from(FINITE_LIMIT).expect("small limit") {
                        context
                            .budget
                            .charge(usize::try_from(upper - lower + 1).ok()?)
                            .ok()?;
                        (lower..=upper)
                            .map(|value| {
                                i64::try_from(value)
                                    .ok()
                                    .map(|value| Node::Number(value.into()))
                            })
                            .collect::<Option<Vec<_>>>()?
                    } else {
                        return None;
                    }
                } else {
                    let inner = finite_in(ir, of, depth + 1, context)?;
                    complete = inner.complete;
                    inner.values
                }
            }
            ResolvedBody::Enum { variants } => {
                if variants.len() > FINITE_LIMIT {
                    return None;
                }
                let mut values = Vec::new();
                for variant in variants {
                    context.budget.charge(1 + variant.name.len()).ok()?;
                    values.push(Node::Text(variant.name.clone()));
                }
                values
            }
            ResolvedBody::Struct { fields, .. } => {
                let mut objects = vec![BTreeMap::<String, Node>::new()];
                for field in fields {
                    context.budget.charge(1 + field.name.len()).ok()?;
                    let inner = finite_in(ir, &field.type_ref, depth + 1, context)?;
                    complete &= inner.complete;
                    let mut values: Vec<_> = inner.values.into_iter().map(Some).collect();
                    if absence(ir, &field.type_ref, depth + 1, &context.budget)? {
                        values.push(None);
                    }
                    if objects.len().checked_mul(values.len())? > FINITE_LIMIT {
                        return None;
                    }
                    let mut extended = Vec::new();
                    for object in &objects {
                        for value in &values {
                            context.budget.charge(1 + field.name.len()).ok()?;
                            for (key, value) in object {
                                context.budget.charge(key.len()).ok()?;
                                context.budget.value(value).ok()?;
                            }
                            if let Some(value) = value {
                                context.budget.value(value).ok()?;
                            }
                            let mut next = object.clone();
                            if let Some(value) = value {
                                next.insert(field.name.clone(), value.clone());
                            }
                            extended.push(next);
                        }
                    }
                    objects = extended;
                }
                objects.into_iter().map(Node::Map).collect()
            }
            ResolvedBody::Union { .. } => return None,
        },
        ResolvedTypeRef::List { .. }
        | ResolvedTypeRef::Map { .. }
        | ResolvedTypeRef::Primitive { .. } => return None,
    };
    if candidates.len() > FINITE_LIMIT {
        return None;
    }
    let mut admitted = Vec::new();
    for value in candidates {
        match input::validate_typed_value_bounded(ir, declared, &value, &context.budget) {
            Ok(()) => admitted.push(value),
            Err(input::ValidationFailure::Invalid(_)) => {}
            Err(input::ValidationFailure::Unresolved(_)) => {
                complete = false;
            }
        }
        if context.budget.exhausted() {
            complete = false;
            break;
        }
    }
    Some(Enumeration {
        values: admitted,
        complete,
    })
}

/// Bounds are necessary constraints, not a claim that every point in the interval is valid.
/// Unsupported predicates are retained by the complete validator after exhaustive enumeration.
/// Primitive Integer's exact admitted range is i64; source bounds narrow that domain further.
fn integer_bounds(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    depth: usize,
    budget: &input::ProofBudget,
) -> Option<(Option<i128>, Option<i128>)> {
    budget.charge(1).ok()?;
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return None;
    }
    match declared {
        ResolvedTypeRef::Primitive {
            name: Primitive::Integer,
        } => Some((Some(i128::from(i64::MIN)), Some(i128::from(i64::MAX)))),
        ResolvedTypeRef::Declared { name } => {
            let ResolvedBody::Newtype { of, invariants, .. } = &ir.named_type(name).body else {
                return None;
            };
            let mut bounds = integer_bounds(ir, of, depth + 1, budget)?;
            for invariant in invariants {
                budget.predicate(&invariant.predicate, 1).ok()?;
                tighten(&invariant.predicate, &mut bounds);
            }
            Some(bounds)
        }
        _ => None,
    }
}

fn tighten(
    predicate: &ess_primitives::predicate::Predicate,
    bounds: &mut (Option<i128>, Option<i128>),
) {
    use ess_primitives::{
        facts::FactValue,
        predicate::{CompareOp, Operand, Predicate},
    };
    if let Predicate::All(children) = predicate {
        for child in children {
            tighten(child, bounds);
        }
        return;
    }
    let Predicate::Compare { left, op, right } = predicate else {
        return;
    };
    let (path, number, op) = match (left, right) {
        (Operand::Fact(path), Operand::Literal(FactValue::Number(number))) => (path, number, *op),
        (Operand::Literal(FactValue::Number(number)), Operand::Fact(path)) => (
            path,
            number,
            match op {
                CompareOp::Eq => CompareOp::Eq,
                CompareOp::Ne => CompareOp::Ne,
                CompareOp::Lt => CompareOp::Gt,
                CompareOp::Le => CompareOp::Ge,
                CompareOp::Gt => CompareOp::Lt,
                CompareOp::Ge => CompareOp::Le,
            },
        ),
        _ => return,
    };
    if path.segments() != ["value"] {
        return;
    }
    let Some(value) = number.as_i64().map(i128::from) else {
        return;
    };
    let (lower, upper) = match op {
        CompareOp::Eq => (Some(value), Some(value)),
        CompareOp::Ne => (None, None),
        CompareOp::Lt => (None, Some(value - 1)),
        CompareOp::Le => (None, Some(value)),
        CompareOp::Gt => (Some(value + 1), None),
        CompareOp::Ge => (Some(value), None),
    };
    if let Some(lower) = lower {
        bounds.0 = Some(bounds.0.map_or(lower, |held| held.max(lower)));
    }
    if let Some(upper) = upper {
        bounds.1 = Some(bounds.1.map_or(upper, |held| held.min(upper)));
    }
}

pub(super) fn allows_absence(ir: &EssIr, declared: &ResolvedTypeRef, depth: usize) -> bool {
    absence(ir, declared, depth, &input::ProofBudget::new(16_384)) == Some(true)
}

fn absence(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    depth: usize,
    budget: &input::ProofBudget,
) -> Option<bool> {
    budget.charge(1).ok()?;
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return None;
    }
    match declared {
        ResolvedTypeRef::Optional { .. } => Some(true),
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => absence(ir, of, depth + 1, budget),
            _ => Some(false),
        },
        _ => Some(false),
    }
}

pub(super) fn only_absent(ir: &EssIr, declared: &ResolvedTypeRef, depth: usize) -> bool {
    let mut completed = BTreeMap::new();
    only_absent_in(
        ir,
        declared,
        depth,
        &mut ProofContext::new(&mut completed, 16_384),
    )
}

fn only_absent_in(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    depth: usize,
    context: &mut ProofContext<'_>,
) -> bool {
    if context.budget.charge(1).is_err() {
        return false;
    }
    if depth > ess_domain::types::MAX_TYPE_DEPTH {
        return false;
    }
    match declared {
        ResolvedTypeRef::Optional { of } => {
            matches!(context.prove(ir, of, depth + 1), Feasibility::Empty(_))
        }
        ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => only_absent_in(ir, of, depth + 1, context),
            _ => false,
        },
        _ => false,
    }
}

pub(crate) fn generated(
    ir: &EssIr,
    declared: &ResolvedTypeRef,
    origin: Origin,
    domains: &super::Domains,
) -> Result<Value, Undetermined> {
    // This cache belongs to one invocation and one immutable IR. A resolved type's name thus
    // identifies its complete constraints; origin-specific state and samples never enter it.
    let mut completed = domains.borrow_mut();
    let mut context = ProofContext::new(&mut completed, 16_384);
    match context.prove(ir, declared, 0) {
        Feasibility::Inhabited if only_absent_in(ir, declared, 0, &mut context) => {
            Ok(Value::Absent)
        }
        Feasibility::Inhabited => {
            let domain = finite_in(ir, declared, 0, &mut context)
                .filter(|values| values.complete)
                .map(|values| values.values);
            Ok(Value::Unknown {
                declared: declared.clone(),
                origin,
                domain,
            })
        }
        Feasibility::Empty(proof) => Err(Undetermined::Request(format!(
            "generated domain `{declared}` is empty at {}: {proof}",
            origin.location.join(".")
        ))),
        Feasibility::Unresolved => Err(Undetermined::Undecidable {
            outcome: origin.branch,
            guard: format!(
                "inhabited generated domain `{declared}` at {} was not established",
                origin.location.join(".")
            ),
        }),
    }
}

#[cfg(test)]
mod proof_tests {
    use std::fmt::Write as _;

    use super::*;

    fn model(types: &str) -> EssIr {
        let mut source = format!(
            "format: ess/20\nsystem: demo\nversion: v1\ndomain: demo.proof\ntypes:\n{types}"
        );
        let raw = ess_domain::spec::RawSpecFile::parse(&source).unwrap();
        source.push_str("events:\n  - name: demo.proof.Values\n    fields:\n");
        for (index, declared) in raw.types.iter().enumerate() {
            writeln!(
                source,
                "      - {{name: field{index}, type: {}}}",
                declared.name
            )
            .unwrap();
        }
        let raw = ess_domain::spec::RawSpecFile::parse(&source).unwrap();
        let spec = ess_domain::Specification::assemble([(
            ess_domain::system::Source::new("proof.yaml"),
            raw,
        )])
        .unwrap();
        ess_compiler::resolve::compile(&spec, &ess_compiler::source::SourceMap::new()).unwrap()
    }

    fn kind(ir: &EssIr, name: &str) -> ResolvedTypeRef {
        ir.events()
            .values()
            .flat_map(|event| &event.fields)
            .find(|field| field.type_ref.to_string() == format!("demo.proof.{name}"))
            .unwrap()
            .type_ref
            .clone()
    }

    fn container_model() -> EssIr {
        model(
            "  - name: demo.proof.Containers\n    kind: struct\n    fields: [{name: items, type: List<Boolean>}, {name: members, type: 'Map<String, Boolean>'}]\n",
        )
    }

    fn container_field<'a>(ir: &'a EssIr, field: &str) -> &'a ResolvedTypeRef {
        let ResolvedBody::Struct { fields, .. } = &ir
            .types()
            .get(&"demo.proof.Containers".parse().unwrap())
            .unwrap()
            .body
        else {
            panic!("the admitted container declaration is a struct");
        };
        &fields
            .iter()
            .find(|held| held.name == field)
            .unwrap()
            .type_ref
    }

    #[test]
    fn proof_list_exact_partial_budget_cannot_validate_an_unchecked_invalid_suffix() {
        let ir = container_model();
        let kind = container_field(&ir, "items");
        let value = Node::Seq(vec![Node::Bool(true), Node::Text(String::new())]);
        let sufficient =
            input::validate_typed_value_bounded(&ir, kind, &value, &input::ProofBudget::new(8));
        assert!(
            matches!(sufficient, Err(input::ValidationFailure::Invalid(_))),
            "complete validation must reject the text: {sufficient:?}"
        );
        let partial =
            input::validate_typed_value_bounded(&ir, kind, &value, &input::ProofBudget::new(6));
        assert!(
            matches!(partial, Err(input::ValidationFailure::Unresolved(_))),
            "an unchecked suffix cannot be Valid: {partial:?}"
        );
    }

    #[test]
    fn proof_list_exact_final_budget_is_valid_but_an_unchecked_valid_suffix_is_unresolved() {
        let ir = container_model();
        let kind = container_field(&ir, "items");
        let value = Node::Seq(vec![Node::Bool(true), Node::Bool(false)]);
        let exact = input::ProofBudget::new(8);
        assert_eq!(
            input::validate_typed_value_bounded(&ir, kind, &value, &exact),
            Ok(())
        );
        assert!(
            exact.exhausted(),
            "the last checked child consumes the exact budget"
        );
        let partial =
            input::validate_typed_value_bounded(&ir, kind, &value, &input::ProofBudget::new(6));
        assert!(
            matches!(partial, Err(input::ValidationFailure::Unresolved(_))),
            "validity still requires checking the valid suffix: {partial:?}"
        );
    }

    #[test]
    fn proof_map_exact_partial_budget_cannot_validate_an_unchecked_invalid_suffix() {
        let ir = container_model();
        let kind = container_field(&ir, "members");
        let value = Node::Map(BTreeMap::from([
            ("a".into(), Node::Bool(true)),
            ("b".into(), Node::Text(String::new())),
        ]));
        let sufficient =
            input::validate_typed_value_bounded(&ir, kind, &value, &input::ProofBudget::new(10));
        assert!(
            matches!(sufficient, Err(input::ValidationFailure::Invalid(_))),
            "complete validation must reject the text: {sufficient:?}"
        );
        let partial =
            input::validate_typed_value_bounded(&ir, kind, &value, &input::ProofBudget::new(8));
        assert!(
            matches!(partial, Err(input::ValidationFailure::Unresolved(_))),
            "an unchecked suffix cannot be Valid: {partial:?}"
        );
    }

    #[test]
    fn proof_map_exact_final_budget_is_valid_but_an_unchecked_valid_suffix_is_unresolved() {
        let ir = container_model();
        let kind = container_field(&ir, "members");
        let value = Node::Map(BTreeMap::from([
            ("a".into(), Node::Bool(true)),
            ("b".into(), Node::Bool(false)),
        ]));
        let exact = input::ProofBudget::new(10);
        assert_eq!(
            input::validate_typed_value_bounded(&ir, kind, &value, &exact),
            Ok(())
        );
        assert!(
            exact.exhausted(),
            "the last checked child consumes the exact budget"
        );
        let partial =
            input::validate_typed_value_bounded(&ir, kind, &value, &input::ProofBudget::new(8));
        assert!(
            matches!(partial, Err(input::ValidationFailure::Unresolved(_))),
            "validity still requires checking the valid suffix: {partial:?}"
        );
    }

    #[test]
    fn proof_cache_keeps_only_completed_proofs_and_recovers_after_depth_work_and_active_hits() {
        let ir = model("  - {name: demo.proof.Value, kind: newtype, of: Boolean}\n");
        let value = kind(&ir, "Value");
        let mut cache = BTreeMap::new();
        assert_eq!(
            ProofContext::new(&mut cache, 1).prove(&ir, &value, 0),
            Feasibility::Unresolved
        );
        assert!(cache.is_empty());
        let mut context = ProofContext::new(&mut cache, 16_384);
        assert_eq!(context.prove(&ir, &value, 33), Feasibility::Unresolved);
        assert!(context.active.is_empty());
        context.active.insert(value.to_string());
        assert_eq!(context.prove(&ir, &value, 0), Feasibility::Unresolved);
        context.active.clear();
        assert!(context.completed.is_empty());
        assert_eq!(context.prove(&ir, &value, 0), Feasibility::Inhabited);
        assert!(context.active.is_empty());
        assert_eq!(context.completed.len(), 1);
        assert_eq!(
            feasibility_cached(&ir, &value, &mut cache),
            Feasibility::Inhabited
        );
        assert_eq!(feasibility(&ir, &value), Feasibility::Inhabited);
    }

    #[test]
    fn proof_validation_false_dominates_unknown_in_members_and_enclosing_constraints() {
        let ir = model(
            r"
  - name: demo.proof.Value
    kind: struct
    fields: [{name: maybe, type: Optional<Boolean>}, {name: known, type: Boolean}]
    invariants: ['maybe == true']
  - name: demo.proof.Parent
    kind: struct
    fields: [{name: child, type: demo.proof.Value}, {name: known, type: Boolean}]
    invariants: ['known == false']
  - name: demo.proof.Wrapped
    kind: newtype
    of: demo.proof.Value
    invariants: ['value.known == false']
",
        );
        let unknown = Node::Map(BTreeMap::from([("known".into(), Node::Bool(true))]));
        let invalid = Node::Map(BTreeMap::from([
            ("maybe".into(), Node::Bool(false)),
            ("known".into(), Node::Bool(true)),
        ]));
        assert!(matches!(
            input::validate_typed_value_proof(&ir, &kind(&ir, "Value"), &unknown),
            Err(input::ValidationFailure::Unresolved(_))
        ));
        let cases = [
            (
                kind(&ir, "Parent"),
                Node::Map(BTreeMap::from([
                    ("child".into(), unknown.clone()),
                    ("known".into(), Node::Bool(true)),
                ])),
            ),
            (kind(&ir, "Wrapped"), unknown.clone()),
            (
                ResolvedTypeRef::List {
                    of: Box::new(kind(&ir, "Value")),
                },
                Node::Seq(vec![unknown.clone(), invalid.clone()]),
            ),
            (
                ResolvedTypeRef::List {
                    of: Box::new(kind(&ir, "Value")),
                },
                Node::Seq(vec![invalid.clone(), unknown.clone()]),
            ),
            (
                ResolvedTypeRef::Map {
                    key: Primitive::String,
                    value: Box::new(kind(&ir, "Value")),
                },
                Node::Map(BTreeMap::from([
                    ("a".into(), unknown.clone()),
                    ("b".into(), invalid.clone()),
                ])),
            ),
            (
                ResolvedTypeRef::Map {
                    key: Primitive::String,
                    value: Box::new(kind(&ir, "Value")),
                },
                Node::Map(BTreeMap::from([
                    ("a".into(), invalid),
                    ("b".into(), unknown),
                ])),
            ),
        ];
        for (kind, value) in cases {
            assert!(
                matches!(
                    input::validate_typed_value_proof(&ir, &kind, &value),
                    Err(input::ValidationFailure::Invalid(_))
                ),
                "{kind}: {value:?}"
            );
        }
    }

    #[test]
    fn abstract_validation_false_dominates_earlier_unknown_constraints_and_members() {
        let ir = model(
            r"
  - name: demo.proof.Value
    kind: struct
    fields: [{name: maybe, type: Optional<Boolean>}, {name: known, type: Boolean}]
    invariants: ['maybe == true', 'known == false']
  - name: demo.proof.Maybe
    kind: struct
    fields: [{name: maybe, type: Optional<Boolean>}]
    invariants: ['maybe == true']
  - name: demo.proof.Parent
    kind: struct
    fields: [{name: child, type: demo.proof.Maybe}, {name: known, type: Boolean}]
    invariants: ['known == false']
  - name: demo.proof.Siblings
    kind: struct
    fields: [{name: first, type: demo.proof.Maybe}, {name: second, type: Boolean}]
",
        );
        let origin = Origin {
            operation: "proof".into(),
            branch: "abstract validation".into(),
            location: vec!["maybe".into()],
        };
        let unknown_optional = Value::Unknown {
            declared: ResolvedTypeRef::Optional {
                of: Box::new(ResolvedTypeRef::Primitive {
                    name: Primitive::Boolean,
                }),
            },
            origin,
            domain: None,
        };
        let maybe = Value::Object(BTreeMap::from([("maybe".into(), unknown_optional.clone())]));
        let cases = [
            (
                kind(&ir, "Value"),
                Value::Object(BTreeMap::from([
                    ("maybe".into(), unknown_optional),
                    ("known".into(), Value::Known(Node::Bool(true))),
                ])),
            ),
            (
                kind(&ir, "Parent"),
                Value::Object(BTreeMap::from([
                    ("child".into(), maybe.clone()),
                    ("known".into(), Value::Known(Node::Bool(true))),
                ])),
            ),
            (
                kind(&ir, "Siblings"),
                Value::Object(BTreeMap::from([
                    ("first".into(), maybe),
                    ("second".into(), Value::Known(Node::Text("invalid".into()))),
                ])),
            ),
        ];
        for (declared, value) in cases {
            assert!(
                matches!(
                    validate(&ir, &declared, &value),
                    Err(Undetermined::Request(_))
                ),
                "a later definite invalidity must dominate an earlier unknown: {declared}"
            );
        }
    }

    #[test]
    fn proof_validation_invariant_order_and_incomplete_finite_domains_preserve_truth() {
        for predicates in [
            "['maybe == true', 'known == false']",
            "['known == false', 'maybe == true']",
        ] {
            let ir = model(&format!(
                "  - name: demo.proof.Value\n    kind: struct\n    fields: [{{name: maybe, type: Optional<Boolean>}}, {{name: known, type: Boolean}}]\n    invariants: {predicates}\n"
            ));
            let value = Node::Map(BTreeMap::from([("known".into(), Node::Bool(true))]));
            assert!(matches!(
                input::validate_typed_value_proof(&ir, &kind(&ir, "Value"), &value),
                Err(input::ValidationFailure::Invalid(_))
            ));
            let mut cache = BTreeMap::new();
            let result = finite_in(
                &ir,
                &kind(&ir, "Value"),
                0,
                &mut ProofContext::new(&mut cache, 16_384),
            )
            .unwrap();
            assert!(
                !result.complete,
                "known=false and absent maybe remains unresolved"
            );
            assert!(
                !result.values.is_empty(),
                "known=false and maybe=true proves existence"
            );
            assert_eq!(
                feasibility(&ir, &kind(&ir, "Value")),
                Feasibility::Inhabited
            );
        }
    }

    #[test]
    fn proof_named_chains_bound_work_without_caching_exhaustion_as_empty() {
        let mut declarations = String::new();
        for index in 0..64 {
            let next = if index == 63 {
                "Boolean".into()
            } else {
                format!("demo.proof.S{}", index + 1)
            };
            writeln!(
                declarations,
                "  - {{name: demo.proof.S{index}, kind: struct, fields: [{{name: next, type: {next}}}]}}"
            )
            .unwrap();
        }
        let ir = model(&declarations);
        let mut cache = BTreeMap::new();
        let mut context = ProofContext::new(&mut cache, 16_384);
        assert_eq!(
            context.prove(&ir, &kind(&ir, "S0"), 0),
            Feasibility::Unresolved
        );
        assert!(context.active.is_empty());
        assert!(!context.completed.contains_key(&kind(&ir, "S0").to_string()));
        assert_eq!(
            feasibility_cached(&ir, &kind(&ir, "S63"), &mut cache),
            Feasibility::Inhabited
        );
        assert_eq!(feasibility(&ir, &kind(&ir, "S63")), Feasibility::Inhabited);
    }
}
