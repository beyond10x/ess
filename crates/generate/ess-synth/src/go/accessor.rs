//! Native typed projections: node functions preserve pointer levels and branch absence.
use std::collections::BTreeMap;
use std::fmt::Write as _;

use super::Emit;
use crate::plan::accessor_type;
use ess_compiler::ir::{ResolvedTypeRef, TypeHandle};
use ess_domain::{
    accessor::{AccessorPlan, Operation},
    QualifiedName, TypeRef,
};

pub(super) fn expression(
    plan: &AccessorPlan,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    target: &ResolvedTypeRef,
    conversion: Option<&TypeHandle>,
    emit: &Emit<'_>,
    root: &str,
) -> Result<String, String> {
    expression_from(
        plan,
        handles,
        target,
        conversion,
        emit,
        &format!("event.{root}"),
    )
}

pub(super) fn expression_from(
    plan: &AccessorPlan,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    target: &ResolvedTypeRef,
    conversion: Option<&TypeHandle>,
    emit: &Emit<'_>,
    root: &str,
) -> Result<String, String> {
    let leaf = accessor_type(plan.leaf(), handles);
    let result = emit.go_type(&leaf);
    let mut out = crate::accessor_output::Output::default();
    let _ = writeln!(out, "func() {} {{", emit.go_type(target));
    // Go local functions need all names declared before the shared DAG's assignments.
    for (id, node) in plan.nodes.iter().enumerate() {
        out.check()?;
        let _ = writeln!(
            out,
            "var project{id} func({}) ({result}, bool)",
            emit.go_type(&accessor_type(&node.source, handles))
        );
    }
    for (id, node) in plan.nodes.iter().enumerate() {
        out.check()?;
        let source = emit.go_type(&accessor_type(&node.source, handles));
        let _ = writeln!(
            out,
            "project{id} = func(value {source}) ({result}, bool) {{"
        );
        match &node.operation {
            Operation::Leaf => out.push_str("return value, true"),
            Operation::Missing => {
                let _ = write!(out, "var zero {result}; return zero, false");
            }
            Operation::Field { field, next } => {
                let ident = field_identifier(emit, handles, &node.source, &field.name);
                let _ = write!(out, "return project{next}(value.{ident})");
            }
            Operation::Newtype { next } => {
                let _ = write!(out, "return project{next}(value.Value())");
            }
            Operation::Optional { next } => {
                let _ = write!(
                    out,
                    "if value == nil {{ var zero {result}; return zero, false }}; return project{next}(*value)"
                );
            }
            Operation::Union { variants, .. } => {
                let TypeRef::Named(owner) = &node.source else {
                    unreachable!("union accessor")
                };
                out.push_str("switch branch := value.(type) {\n");
                for (label, next) in variants {
                    let _ = writeln!(
                        out,
                        "case {}: return project{next}(branch.Value)",
                        emit.reference_variant(owner, label)
                    );
                }
                out.push_str("default: panic(\"invalid native union accessor value\")\n}");
            }
        }
        out.push_str("\n}\n");
    }
    let _ = writeln!(out, "value, available := project{}({root})", plan.start);
    if plan.may_miss() {
        out.push_str("if !available { return nil }\n");
    } else {
        out.push_str("if !available { panic(\"total typed accessor\") }\n");
    }
    if let Some(to) = conversion {
        let key = format!("{leaf} -> {target}");
        let function = emit.qualify(emit.layout.package_of(to.name()), emit.layout.convert(&key));
        let _ = write!(out, "return {function}(value)\n}}()");
        return out.finish();
    }
    let mut depth = 0;
    let mut current = target;
    while current != &leaf {
        let ResolvedTypeRef::Optional { of } = current else {
            unreachable!("admitted accessor assignment")
        };
        let previous = if depth == 0 {
            "value".to_owned()
        } else {
            format!("wrapped{}", depth - 1)
        };
        let _ = writeln!(out, "wrapped{depth} := &{previous}");
        depth += 1;
        current = of;
    }
    let value = if depth == 0 {
        "value".to_owned()
    } else {
        format!("wrapped{}", depth - 1)
    };
    let _ = write!(out, "return {value}\n}}()");
    out.finish()
}

pub(super) fn preflight(
    ir: &ess_compiler::ir::EssIr,
    plan: &crate::SynthesisPlan,
    layout: &super::layout::Layout,
) -> Result<(), crate::TargetFailure> {
    crate::selection::preflight(ir, plan, crate::Target::Go)?;
    let emit = Emit::new(ir, layout, layout.system(), None);
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
                crate::accessor_output::failure(ir, crate::Target::Go, plan, &source, reason)
            };
            let output = super::selection::prelude(&emit, binding, selection).map_err(failure)?;
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
                        super::selection::mapping(&emit, selection, selector, projection, target)
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
                types,
                target,
                conversion,
            }) = crate::plan::determined_prepared_input(ir, binding, field)
            {
                let source = binding.name.to_string();
                let root = root_identifier(&emit, binding, &accessor.root.name);
                let output = expression(accessor, types, target, conversion, &emit, &root)
                    .map_err(|reason| {
                        crate::accessor_output::failure(
                            ir,
                            crate::Target::Go,
                            plan,
                            &source,
                            reason,
                        )
                    })?;
                bytes = bytes.saturating_add(output.len());
                if bytes > 32 * 1024 * 1024 {
                    return Err(crate::accessor_output::failure(
                        ir,
                        crate::Target::Go,
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

pub(super) fn root_identifier(
    emit: &Emit<'_>,
    binding: &ess_compiler::ir::ResolvedBinding,
    field: &str,
) -> String {
    let mut taken = BTreeMap::new();
    emit.ir
        .event(binding.cause.event().expect("generated event capability"))
        .fields
        .iter()
        .map(|f| (&f.name, super::items::field_ident(&mut taken, &f.name)))
        .find(|(name, _)| name.as_str() == field)
        .expect("declared event field")
        .1
}

fn field_identifier(
    emit: &Emit<'_>,
    handles: &BTreeMap<QualifiedName, TypeHandle>,
    source: &TypeRef,
    field: &str,
) -> String {
    let TypeRef::Named(owner) = source else {
        unreachable!("struct accessor")
    };
    let ess_compiler::ir::ResolvedBody::Struct { fields, .. } =
        &emit.ir.named_type(&handles[owner]).body
    else {
        unreachable!("struct accessor")
    };
    let mut taken = BTreeMap::new();
    fields
        .iter()
        .map(|f| (&f.name, super::items::field_ident(&mut taken, &f.name)))
        .find(|(name, _)| name.as_str() == field)
        .expect("declared field")
        .1
}
