//! Native typed projections: one function per DAG node, never expanded union paths.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::{ResolvedTypeRef, TypeHandle};
use ess_domain::{
    accessor::{AccessorPlan, Operation},
    QualifiedName,
};

use super::{layout::Layout, name};
use crate::plan::accessor_type;

pub(super) fn expression(
    plan: &AccessorPlan,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    target: &ResolvedTypeRef,
    conversion: Option<&TypeHandle>,
    layout: &Layout,
    types: &str,
) -> Result<String, String> {
    expression_from(
        plan,
        handles,
        target,
        conversion,
        layout,
        types,
        &format!("event.{}", name::value_ident(&plan.root.name)),
    )
}

pub(super) fn expression_from(
    plan: &AccessorPlan,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    target: &ResolvedTypeRef,
    conversion: Option<&TypeHandle>,
    layout: &Layout,
    types: &str,
    root: &str,
) -> Result<String, String> {
    emit_projection(
        plan,
        handles,
        target,
        conversion,
        layout,
        (types, root, false),
    )
}

/// Borrow the whole leaf of a total typed source before checking its collection bound.
pub(super) fn borrowed(
    plan: &AccessorPlan,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    layout: &Layout,
    types: &str,
    root: &str,
) -> Result<String, String> {
    let target = accessor_type(plan.leaf(), handles);
    emit_projection(plan, handles, &target, None, layout, (types, root, true))
}

fn emit_projection(
    plan: &AccessorPlan,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    target: &ResolvedTypeRef,
    conversion: Option<&TypeHandle>,
    layout: &Layout,
    context: (&str, &str, bool),
) -> Result<String, String> {
    let (types, root, borrow) = context;
    let render = |ty: &ResolvedTypeRef| {
        layout
            .absolute_type(ty)
            .replace("crate::", &format!("{types}::"))
    };
    let leaf = accessor_type(plan.leaf(), handles);
    let result = if borrow {
        format!("&{}", render(&leaf))
    } else {
        render(&leaf)
    };
    let mut out = crate::accessor_output::Output::default();
    out.push_str("{\n");
    for (id, node) in plan.nodes.iter().enumerate() {
        out.check()?;
        let source = render(&accessor_type(&node.source, handles));
        let _ = writeln!(
            out,
            "fn project_{id}(value: &{source}) -> Option<{result}> {{"
        );
        match &node.operation {
            Operation::Leaf => out.push_str(if borrow {
                "Some(value)"
            } else {
                "Some(value.clone())"
            }),
            Operation::Missing => out.push_str("let _ = value; None"),
            Operation::Field { field, next } => {
                let _ = write!(
                    out,
                    "project_{next}(&value.{})",
                    name::value_ident(&field.name)
                );
            }
            Operation::Newtype { next } => {
                let _ = write!(out, "project_{next}(&value.0)");
            }
            Operation::Optional { next } => {
                let _ = write!(out, "value.as_ref().and_then(project_{next})");
            }
            Operation::Union { variants, .. } => {
                out.push_str("match value {\n");
                for (label, next) in variants {
                    let _ = writeln!(
                        out,
                        "{source}::{}(inner) => project_{next}(inner),",
                        name::pascal(label)
                    );
                }
                out.push('}');
            }
        }
        out.push_str("\n}\n");
    }
    let mut assigned = conversion.map_or_else(
        || "value".to_owned(),
        |_| format!("{}::from(value)", render(target)),
    );
    let assigned_type = if conversion.is_some() { target } else { &leaf };
    let mut current = target;
    while current != assigned_type {
        let ResolvedTypeRef::Optional { of } = current else {
            unreachable!("admitted accessor assignment")
        };
        assigned = format!("Some({assigned})");
        current = of;
    }
    let _ = write!(
        out,
        "match project_{}(&{root}) {{ Some(value) => {assigned}, None => ",
        plan.start
    );
    if plan.may_miss() {
        out.push_str("None");
    } else {
        out.push_str("unreachable!(\"total typed accessor\")");
    }
    out.push_str(" } }");
    out.finish()
}

pub(super) fn preflight(
    ir: &ess_compiler::ir::EssIr,
    plan: &crate::SynthesisPlan,
    layout: &Layout,
) -> Result<(), crate::TargetFailure> {
    crate::selection::preflight(ir, plan, crate::Target::Rust)?;
    let types = Layout::crate_ident(layout.package());
    let mut bytes = 0_usize;
    for binding in ir.bindings().values() {
        if !plan.is_generated(
            crate::CapabilityKind::BindingTransformation,
            &binding.name.to_string(),
        ) && !crate::selection::prepared_helper(ir, binding)
        {
            continue;
        }
        if let Some(selection) = &binding.selection {
            let source = binding.name.to_string();
            let failure = |reason| {
                crate::accessor_output::failure(ir, crate::Target::Rust, plan, &source, reason)
            };
            let output = super::selection::prelude(ir, binding, selection, layout, &types)
                .map_err(failure)?;
            bytes = bytes.saturating_add(output.len());
            for field in &ir.command(&binding.command).input {
                if let Some(crate::plan::DeterminedInput::Selection {
                    selector,
                    projection,
                    target,
                    ..
                }) = crate::plan::determined_prepared_input(ir, binding, field)
                {
                    bytes = bytes.saturating_add(
                        super::selection::mapping(
                            ir, selection, selector, projection, target, layout, &types,
                        )
                        .map_err(failure)?
                        .len(),
                    );
                }
            }
            if bytes > 32 * 1024 * 1024 {
                return Err(failure(
                    "aggregate selection/accessor generated source exceeds 33554432 bytes".into(),
                ));
            }
        }
        for field in &ir.command(&binding.command).input {
            if let Some(crate::plan::DeterminedInput::Accessor {
                plan: accessor,
                types: handles,
                target,
                conversion,
            }) = crate::plan::determined_prepared_input(ir, binding, field)
            {
                let source = binding.name.to_string();
                let output = expression(accessor, handles, target, conversion, layout, &types)
                    .map_err(|reason| {
                        crate::accessor_output::failure(
                            ir,
                            crate::Target::Rust,
                            plan,
                            &source,
                            reason,
                        )
                    })?;
                bytes = bytes.saturating_add(output.len());
                if bytes > 32 * 1024 * 1024 {
                    return Err(crate::accessor_output::failure(
                        ir,
                        crate::Target::Rust,
                        plan,
                        &source,
                        "aggregate accessor generated source exceeds 33554432 bytes".into(),
                    ));
                }
            }
        }
    }
    Ok(())
}
