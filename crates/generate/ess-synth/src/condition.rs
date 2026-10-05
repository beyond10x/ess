//! A binding's event-payload condition (ess/22, beyond10x/ess#268, beyond10x/ess#194) in
//! generated dispatch: what every target that delivers bindings needs decided before it writes a
//! line, in one place.
//!
//! `docs/design/conditional-binding-failure-policies.md`, "Payload condition", is the contract, and
//! `ess-conformance`'s interpreter (`interpret/bindings.rs`) is the reference reading of it. The
//! generated dispatch evaluates the condition before the transformation, the invocation record
//! and the port: True invokes, False skips this binding alone, Unknown invokes nothing and is this
//! binding's unmet obligation while the bindings beside it still run. Kleene logic decides `all`,
//! `any` and `not` around an absent member, exactly as the evaluator does.
//!
//! A required input filled from an Optional member the condition proves present (#194) is read
//! through a presence check rather than unwrapped: the transformation answers absent, and the
//! delivery reports the binding's unmet input instead of inventing a value. Under a sound proof
//! the check never fires; it exists so that nothing the proof missed becomes a fabricated input.

use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedBody, ResolvedEvent, ResolvedField, ResolvedTypeRef, TypeHandle,
};
use ess_domain::binding::condition::{ConditionRead, Leaf};

use crate::plan::DeterminedInput;

/// The capability an Unknown condition reports, as the generated `UnmetObligation` spells it.
pub(crate) const UNKNOWN: &str = "binding condition";
/// The capability an absent proved member reports.
pub(crate) const ABSENT: &str = "binding input";

/// One step of a condition path through declared types.
#[derive(Debug, Clone, Copy)]
pub(crate) enum Step<'a> {
    /// An Optional level: absent here makes the whole path absent.
    Optional,
    /// A newtype: its one wrapped value.
    Newtype,
    /// A struct member.
    Field {
        /// The struct that declares it.
        owner: &'a TypeHandle,
        /// The member, as declared.
        name: &'a str,
    },
}

/// What a condition path ends at, as a generated comparison needs it.
#[derive(Debug, Clone, Copy)]
pub(crate) enum End<'a> {
    /// Text.
    String,
    /// A declared enum.
    Enum(&'a TypeHandle),
    /// Anything else: only `defined(...)` reads it.
    Other,
}

/// One condition path, resolved through the IR's declared types.
#[derive(Debug, Clone)]
pub(crate) struct Walk<'a> {
    /// The event field the path starts at.
    pub root: &'a ResolvedField,
    /// Every step after the root field, in order.
    pub steps: Vec<Step<'a>>,
    /// What the path ends at.
    pub end: End<'a>,
}

/// Strips Optional and newtype wrappers off `ty`, recording each as a step.
fn strip<'a>(
    ir: &'a EssIr,
    mut ty: &'a ResolvedTypeRef,
    steps: &mut Vec<Step<'a>>,
) -> &'a ResolvedTypeRef {
    loop {
        match ty {
            ResolvedTypeRef::Optional { of } => {
                steps.push(Step::Optional);
                ty = of;
            }
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => {
                    steps.push(Step::Newtype);
                    ty = of;
                }
                _ => return ty,
            },
            _ => return ty,
        }
    }
}

/// `read` resolved against `event` through the IR: `None` where it does not resolve, which the
/// compiler refused before this IR existed.
pub(crate) fn walk<'a>(
    ir: &'a EssIr,
    event: &'a ResolvedEvent,
    read: &ConditionRead,
) -> Option<Walk<'a>> {
    let (first, rest) = read.members.split_first()?;
    let root = event.field(first)?;
    let mut steps = Vec::new();
    let mut ty = strip(ir, &root.type_ref, &mut steps);
    for member in rest {
        let ResolvedTypeRef::Declared { name: owner } = ty else {
            return None;
        };
        let ResolvedBody::Struct { fields, .. } = &ir.named_type(owner).body else {
            return None;
        };
        let field = fields.iter().find(|field| &field.name == member)?;
        steps.push(Step::Field {
            owner,
            name: &field.name,
        });
        ty = strip(ir, &field.type_ref, &mut steps);
    }
    let end = match (&read.leaf, ty) {
        (Leaf::String, _) => End::String,
        (Leaf::Enum { .. }, ResolvedTypeRef::Declared { name }) => End::Enum(name),
        (Leaf::Enum { .. }, _) => return None,
        (Leaf::Other, _) => End::Other,
    };
    Some(Walk { root, steps, end })
}

/// Every path `binding`'s condition reads, resolved, keyed as the predicate writes them.
pub(crate) fn walks<'a>(
    ir: &'a EssIr,
    binding: &'a ResolvedBinding,
) -> Option<std::collections::BTreeMap<String, Walk<'a>>> {
    let condition = binding.condition.as_ref()?;
    let event = ir.event(binding.cause.event()?);
    condition
        .plan
        .reads
        .iter()
        .map(|(written, read)| Some((written.clone(), walk(ir, event, read)?)))
        .collect()
}

/// How many Optional levels a proved mapping strips off its source before assignment: `0` where
/// the mapping is not one the condition proves (no condition, an Optional target, or a source that
/// is already present).
///
/// Only a binding with a condition gets anything but `0`, so every unconditioned binding keeps the
/// emission it had.
pub(crate) fn proved_levels(
    ir: &EssIr,
    binding: &ResolvedBinding,
    determined: &DeterminedInput<'_>,
    target: &ResolvedTypeRef,
) -> usize {
    if binding.condition.is_none() || target.is_optional() {
        return 0;
    }
    match determined {
        DeterminedInput::Copy { field } => {
            let Some(event) = binding.cause.event() else {
                return 0;
            };
            let Some(declared) = ir.event(event).field(field) else {
                return 0;
            };
            let mut levels = 0;
            let mut ty = &declared.type_ref;
            while let ResolvedTypeRef::Optional { of } = ty {
                levels += 1;
                ty = of;
            }
            levels
        }
        DeterminedInput::Accessor { plan, .. } => usize::from(plan.may_miss()),
        _ => 0,
    }
}

/// Whether `binding`'s generated transformation checks a proved member's presence, and so answers
/// absent rather than an input: some determined input of it is proved by its condition.
pub(crate) fn checks_presence(ir: &EssIr, binding: &ResolvedBinding) -> bool {
    binding.condition.is_some()
        && ir.command(&binding.command).input.iter().any(|field| {
            crate::plan::determined_prepared_input(ir, binding, field).is_some_and(|determined| {
                let target = binding
                    .mapping
                    .iter()
                    .find(|mapping| mapping.target == field.name)
                    .map_or(&field.type_ref, |mapping| &mapping.target_type);
                proved_levels(ir, binding, &determined, target) > 0
            })
        })
}
