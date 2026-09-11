//! Shared finite native-validator type inventory, containing only declared selection input types.
use ess_compiler::ir::{EssIr, ResolvedBody, ResolvedSelectionPlan, ResolvedTypeRef};
use std::collections::{BTreeMap, BTreeSet};

pub(crate) fn input_types(
    ir: &EssIr,
    selection: &ResolvedSelectionPlan,
) -> Result<Vec<ResolvedTypeRef>, String> {
    inventory(ir, selection, false)
}

fn inventory(
    ir: &EssIr,
    selection: &ResolvedSelectionPlan,
    roots: bool,
) -> Result<Vec<ResolvedTypeRef>, String> {
    let mut pending: Vec<_> = selection
        .plan
        .inputs
        .iter()
        .map(|input| {
            crate::plan::accessor_type(
                if roots {
                    &input.list_type
                } else {
                    &input.item_type
                },
                &selection.types,
            )
        })
        .collect();
    let mut seen = BTreeSet::new();
    while let Some(ty) = pending.pop() {
        if !seen.insert(ty.clone()) {
            continue;
        }
        if seen.len() > 4096 || pending.len() > 16384 {
            return Err("selection native type inventory exceeds its finite bound".into());
        }
        match &ty {
            ResolvedTypeRef::Declared { name } => match &ir.named_type(name).body {
                ResolvedBody::Newtype { of, .. } => pending.push(of.clone()),
                ResolvedBody::Struct { fields, .. } => {
                    pending.extend(fields.iter().map(|field| field.type_ref.clone()));
                }
                ResolvedBody::Union { variants, .. } => pending.extend(variants.values().cloned()),
                ResolvedBody::Enum { .. } => {}
            },
            ResolvedTypeRef::Optional { of } | ResolvedTypeRef::List { of } => {
                pending.push((**of).clone());
            }
            ResolvedTypeRef::Map { value, .. } if roots => pending.push((**value).clone()),
            ResolvedTypeRef::Primitive { .. } | ResolvedTypeRef::Map { .. } => {}
        }
    }
    Ok(seen.into_iter().collect())
}

pub(crate) fn type_ids(types: &[ResolvedTypeRef]) -> BTreeMap<ResolvedTypeRef, usize> {
    types
        .iter()
        .cloned()
        .enumerate()
        .map(|(id, ty)| (ty, id))
        .collect()
}

pub(crate) fn element_type(
    ir: &EssIr,
    selection: &ResolvedSelectionPlan,
    input: usize,
) -> ResolvedTypeRef {
    let mut ty =
        crate::plan::accessor_type(&selection.plan.inputs[input].list_type, &selection.types);
    loop {
        match ty {
            ResolvedTypeRef::List { of } => return *of,
            ResolvedTypeRef::Declared { name } => {
                let ResolvedBody::Newtype { of, .. } = &ir.named_type(&name).body else {
                    unreachable!("admitted list alias")
                };
                ty = of.clone();
            }
            _ => unreachable!("admitted required list"),
        }
    }
}

/// A helper supports an owed preparation without claiming to implement that host conversion.
pub(crate) fn prepared_helper(ir: &EssIr, binding: &ess_compiler::ir::ResolvedBinding) -> bool {
    binding.selection.as_ref().is_some_and(|selection| {
        selection
            .plan
            .inputs
            .iter()
            .any(|input| input.conversion.is_some())
    }) && ir
        .command(&binding.command)
        .input
        .iter()
        .all(|input| crate::plan::determined_prepared_input(ir, binding, input).is_some())
}

/// Refuse unsupported source constraints before emitting any native selector artifact.
pub(crate) fn preflight(
    ir: &EssIr,
    plan: &crate::SynthesisPlan,
    target: crate::Target,
) -> Result<(), crate::TargetFailure> {
    for binding in ir.bindings().values() {
        let Some(selection) = &binding.selection else {
            continue;
        };
        let source = binding.name.to_string();
        let types = inventory(ir, selection, true)
            .map_err(|reason| crate::accessor_output::failure(ir, target, plan, &source, reason))?;
        for ty in types {
            let ResolvedTypeRef::Declared { name } = ty else {
                continue;
            };
            let declared = ir.named_type(&name);
            if declared.reading.is_some()
                || matches!(&declared.body, ResolvedBody::Newtype { invariants, .. } | ResolvedBody::Struct { invariants, .. } if !invariants.is_empty())
            {
                return Err(crate::TargetFailure::new(ir, target, plan, vec![crate::TargetFailureCause::new(crate::TargetFailureCode::SelectionConstraint, vec![source, name.name().to_string()], "selection cannot enforce the reachable declared invariant or clock-reading contract; native helpers require unconstrained input types".into())]));
            }
        }
    }
    Ok(())
}
