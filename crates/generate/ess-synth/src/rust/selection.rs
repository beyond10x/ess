//! Native selection over declared records, with complete preflight before invocation.
use super::{layout::Layout, name};
use crate::plan::accessor_type;
use ess_compiler::ir::{
    EssIr, ResolvedBinding, ResolvedBody, ResolvedSelectionPlan, ResolvedTypeRef,
};
use ess_domain::selection::{InputSource, SelectionOperation};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};
use std::{collections::BTreeMap, fmt::Write as _};

pub(super) fn support(types: &str) -> String {
    format!(
        r"
/// Deterministic selection failure, reported before the command is invoked.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SelectionFailure {{
    /// Closed failure classification.
    pub cause: SelectionFailureCause,
    /// Input in declaration order.
    pub input: usize,
    /// Original occurrence, when failure concerns one item.
    pub index: Option<usize>,
    /// Selector in declaration order, when applicable.
    pub selector: Option<usize>,
}}
/// The finite runtime refusal vocabulary.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SelectionFailureCause {{
    /// A required declared input is invalid.
    InvalidInput,
    /// The finite runtime cannot inspect this declared shape.
    Unsupported,
    /// A predicate cannot be decided from its declared observations.
    Unknown,
    /// A fixed count or byte bound was exceeded.
    Resource,
}}
/// Transport refusal preserves both host obligations and selection diagnostics.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TransportFailure {{
    /// Existing host implementation obligation.
    Obligation({types}::obligation::UnmetObligation),
    /// Failed selection; the command was not invoked.
    Selection(SelectionFailure),
}}
impl From<{types}::obligation::UnmetObligation> for TransportFailure {{
    fn from(value: {types}::obligation::UnmetObligation) -> Self {{ Self::Obligation(value) }}
}}
impl From<SelectionFailure> for TransportFailure {{
    fn from(value: SelectionFailure) -> Self {{ Self::Selection(value) }}
}}
fn selection_all(values: &[Option<bool>]) -> Option<bool> {{
    if values.contains(&Some(false)) {{ Some(false) }} else if values.contains(&None) {{ None }} else {{ Some(true) }}
}}
fn selection_any(values: &[Option<bool>]) -> Option<bool> {{
    if values.contains(&Some(true)) {{ Some(true) }} else if values.contains(&None) {{ None }} else {{ Some(false) }}
}}
"
    )
}

fn unwrap_list(ir: &EssIr, mut ty: ResolvedTypeRef, mut value: String) -> String {
    while let ResolvedTypeRef::Declared { name } = ty {
        let ResolvedBody::Newtype { of, .. } = &ir.named_type(&name).body else {
            break;
        };
        value = format!("&({value}).0");
        ty = of.clone();
    }
    value
}

fn scalar(ir: &EssIr, layout: &Layout, types: &str, ty: &ResolvedTypeRef, value: &str) -> String {
    match ty {
        ResolvedTypeRef::Optional { of } => format!(
            "({value}).as_ref().and_then(|value| {})",
            scalar(ir, layout, types, of, "value")
        ),
        ResolvedTypeRef::Declared { name: handle } => match &ir.named_type(handle).body {
            ResolvedBody::Newtype { of, .. } => {
                scalar(ir, layout, types, of, &format!("&({value}).0"))
            }
            ResolvedBody::Enum { variants } => {
                let owner = layout
                    .absolute_type(ty)
                    .replace("crate::", &format!("{types}::"));
                let arms = variants
                    .iter()
                    .map(|variant| {
                        format!(
                            "{owner}::{} => {:?}.to_owned()",
                            name::pascal(variant),
                            variant
                        )
                    })
                    .collect::<Vec<_>>()
                    .join(",");
                format!("Some(match {value} {{ {arms} }})")
            }
            _ => unreachable!("admitted scalar"),
        },
        ResolvedTypeRef::Primitive { .. } => format!("Some(({value}).to_owned())"),
        _ => unreachable!("admitted scalar"),
    }
}

fn predicate(predicate: &Predicate, reads: &BTreeMap<String, usize>) -> String {
    match predicate {
        Predicate::Always => "Some(true)".into(),
        Predicate::Never => "Some(false)".into(),
        Predicate::Defined(path) => format!("Some(read_{}.is_some())", reads[&path.to_string()]),
        Predicate::Compare {
            left: Operand::Fact(path),
            op,
            right: Operand::Literal(value),
        } => {
            let ess_primitives::facts::FactValue::Text(value) = value else {
                unreachable!()
            };
            format!(
                "read_{}.as_ref().map(|value| value.as_str() {} {value:?})",
                reads[&path.to_string()],
                if *op == CompareOp::Eq { "==" } else { "!=" }
            )
        }
        Predicate::All(children) | Predicate::Any(children) => format!(
            "{}(&[{}])",
            if matches!(predicate, Predicate::All(_)) {
                "selection_all"
            } else {
                "selection_any"
            },
            children
                .iter()
                .map(|child| self::predicate(child, reads))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Predicate::Not(child) => format!("({}).map(|value| !value)", self::predicate(child, reads)),
        _ => unreachable!("admitted finite selector predicate"),
    }
}

pub(super) fn prelude(
    ir: &EssIr,
    binding: &ResolvedBinding,
    selection: &ResolvedSelectionPlan,
    layout: &Layout,
    types: &str,
) -> Result<String, String> {
    let mut out = crate::accessor_output::Output::default();
    out.push_str(&validators(ir, selection, layout, types)?);
    let input_types = crate::selection::input_types(ir, selection)?;
    let input_ids = crate::selection::type_ids(&input_types);
    input_prelude(&mut out, ir, selection, layout, types)?;
    let count = selection.plan.selectors.len();
    let _ = writeln!(out, "let mut selected: [Option<usize>; {count}] = [None; {count}];\nlet mut selection_bytes = 0_usize;");
    for (selector, plan) in selection.plan.selectors.iter().enumerate() {
        if matches!(plan.operation, SelectionOperation::First { .. }) {
            let _ = writeln!(
                out,
                "let mut matches_{selector} = vec![false; selection_input_{}.len()];",
                plan.input
            );
        }
    }
    // Preflight order is input, occurrence, selector; selection happens only after every read.
    for (input, input_plan) in selection.plan.inputs.iter().enumerate() {
        let _ = writeln!(
            out,
            "for (index, item) in selection_input_{input}.iter().enumerate() {{"
        );
        if input_plan.optional_items {
            let element = crate::selection::element_type(ir, selection, input);
            let item = unwrap_list(ir, element, "item".to_owned());
            let _ = writeln!(
                out,
                "let Some(item) = ({item}).as_ref() else {{ continue; }};"
            );
        }
        let validator = input_ids[&accessor_type(&input_plan.item_type, &selection.types)];
        let _ = writeln!(out, "selection_validate_{validator}(item, &mut selection_bytes, 0).map_err(|cause| SelectionFailure {{ cause, input: {input}, index: Some(index), selector: None }})?;");
        for (selector, plan) in selection
            .plan
            .selectors
            .iter()
            .enumerate()
            .filter(|(_, plan)| plan.input == input)
        {
            let SelectionOperation::First {
                predicate: condition,
                reads,
                ..
            } = &plan.operation
            else {
                continue;
            };
            out.push_str("{\n");
            let ids: BTreeMap<_, _> = reads
                .keys()
                .cloned()
                .enumerate()
                .map(|(id, path)| (path, id))
                .collect();
            for (path, projection) in reads {
                let ty = accessor_type(&projection.0.effective_type(), &selection.types);
                let expr = super::accessor::expression_from(
                    &projection.0,
                    &selection.types,
                    &ty,
                    None,
                    layout,
                    types,
                    "*item",
                )?;
                let id = ids[path];
                let text = scalar(ir, layout, types, &ty, "&value");
                let _ = writeln!(out, "let read_{id}: Option<String> = {{ let value = {expr}; {text} }};\nif let Some(value) = &read_{id} {{ selection_bytes = selection_bytes.saturating_add(value.len()).saturating_add({}); if value.len() > 4096 || selection_bytes > 1048576 {{ return Err(SelectionFailure {{ cause: SelectionFailureCause::Resource, input: {input}, index: Some(index), selector: Some({selector}) }}); }} }}", path.len());
            }
            let expression = predicate(condition, &ids);
            let _ = writeln!(out, "matches_{selector}[index] = ({expression}).ok_or(SelectionFailure {{ cause: SelectionFailureCause::Unknown, input: {input}, index: Some(index), selector: Some({selector}) }})?;\n}}");
        }
        out.push_str("}\n");
    }
    for (selector, plan) in selection.plan.selectors.iter().enumerate() {
        match &plan.operation {
            SelectionOperation::First { excluding, .. } => {
                let exclude = excluding
                    .iter()
                    .map(|index| format!("selected[{index}] != Some(index)"))
                    .collect::<Vec<_>>()
                    .join(" && ");
                let _ = writeln!(out, "selected[{selector}] = matches_{selector}.iter().enumerate().find_map(|(index, eligible)| (*eligible{}).then_some(index));", if exclude.is_empty() { String::new() } else { format!(" && {exclude}") });
            }
            SelectionOperation::FirstPresent { selections } => {
                let expr = selections
                    .iter()
                    .map(|index| format!("selected[{index}]"))
                    .collect::<Vec<_>>()
                    .join(".or(");
                let _ = writeln!(
                    out,
                    "selected[{selector}] = {expr}{};",
                    ")".repeat(selections.len().saturating_sub(1))
                );
            }
        }
    }
    let _ = binding;
    out.finish()
}

pub(super) fn mapping(
    ir: &EssIr,
    selection: &ResolvedSelectionPlan,
    selector: usize,
    projection: &ess_domain::accessor::ProjectionPlan,
    target: &ResolvedTypeRef,
    layout: &Layout,
    types: &str,
) -> Result<String, String> {
    let input = selection.plan.selectors[selector].input;
    let item = if selection.plan.inputs[input].optional_items {
        let element = crate::selection::element_type(ir, selection, input);
        let container = unwrap_list(ir, element, format!("&selection_input_{input}[index]"));
        format!("({container}).as_ref().expect(\"selected present item\")")
    } else {
        format!("&selection_input_{input}[index]")
    };
    let expr = super::accessor::expression_from(
        &projection.0,
        &selection.types,
        target,
        None,
        layout,
        types,
        "*item",
    )?;
    Ok(format!("match selected[{selector}] {{ Some(index) => {{ let item = {item}; {expr} }}, None => None }}"))
}

fn validators(
    ir: &EssIr,
    selection: &ResolvedSelectionPlan,
    layout: &Layout,
    namespace: &str,
) -> Result<String, String> {
    let types = crate::selection::input_types(ir, selection)?;
    let ids = crate::selection::type_ids(&types);
    let render = |ty: &ResolvedTypeRef| {
        layout
            .absolute_type(ty)
            .replace("crate::", &format!("{namespace}::"))
    };
    let mut out = crate::accessor_output::Output::default();
    for (id, ty) in types.iter().enumerate() {
        let _ = writeln!(out, "fn selection_validate_{id}(value: &{}, bytes: &mut usize, depth: usize) -> Result<(), SelectionFailureCause> {{ if depth > 128 || *bytes > 1048576 {{ return Err(SelectionFailureCause::Resource); }}", render(ty));
        match ty {
            ResolvedTypeRef::Primitive { name: ess_domain::Primitive::String } => out.push_str("*bytes = bytes.saturating_add(value.len()); if value.len() > 4096 { return Err(SelectionFailureCause::Resource); }\n"),
            ResolvedTypeRef::Primitive { .. } => out.push_str("let _ = value;\n"),
            ResolvedTypeRef::Optional { of } => { let _ = writeln!(out, "if let Some(value) = value {{ selection_validate_{}(value, bytes, depth+1)?; }}", ids[of.as_ref()]); }
            ResolvedTypeRef::List { of } => { let _ = writeln!(out, "if value.len() > 64 {{ return Err(SelectionFailureCause::Resource); }} for item in value {{ selection_validate_{}(item, bytes, depth+1)?; }}", ids[of.as_ref()]); }
            ResolvedTypeRef::Map { .. } => out.push_str("let _ = value; return Err(SelectionFailureCause::Unsupported);\n"),
            ResolvedTypeRef::Declared { name: handle } => match &ir.named_type(handle).body {
                ResolvedBody::Newtype { of, .. } => { let _ = writeln!(out, "selection_validate_{}(&value.0, bytes, depth+1)?;", ids[of]); }
                ResolvedBody::Struct { fields, .. } => for field in fields { let _ = writeln!(out, "*bytes = bytes.saturating_add({}); selection_validate_{}(&value.{}, bytes, depth+1)?;", field.name.len(), ids[&field.type_ref], name::value_ident(&field.name)); },
                ResolvedBody::Enum { variants } => { out.push_str("*bytes = bytes.saturating_add(match value {\n"); for variant in variants { let _ = writeln!(out, "{}::{} => {},", render(ty), name::pascal(variant), variant.len()); } out.push_str("});\n"); }
                ResolvedBody::Union { variants, .. } => { out.push_str("match value {\n"); for (label, child) in variants { let _ = writeln!(out, "{}::{}(inner) => selection_validate_{}(inner, bytes, depth+1)?,", render(ty), name::pascal(label), ids[child]); } out.push_str("}\n"); }
            }
        }
        out.push_str(
            "if *bytes > 1048576 { return Err(SelectionFailureCause::Resource); } Ok(())\n}\n",
        );
    }
    out.finish()
}

fn input_prelude(
    out: &mut crate::accessor_output::Output,
    ir: &EssIr,
    selection: &ResolvedSelectionPlan,
    layout: &Layout,
    types: &str,
) -> Result<(), String> {
    for (input, plan) in selection.plan.inputs.iter().enumerate() {
        let source = if plan.conversion.is_some() {
            format!("prepared_{input}")
        } else {
            match &plan.source {
                InputSource::Field { field } => {
                    format!("&event.{}", name::value_ident(&field.name))
                }
                InputSource::Accessor { plan } => super::accessor::borrowed(
                    plan,
                    &selection.types,
                    layout,
                    types,
                    &format!("event.{}", name::value_ident(&plan.root.name)),
                )?,
            }
        };
        let list = unwrap_list(ir, accessor_type(&plan.list_type, &selection.types), source);
        let _ = writeln!(out, "let selection_input_{input} = {list};\nif selection_input_{input}.len() > 64 {{ return Err(SelectionFailure {{ cause: SelectionFailureCause::Resource, input: {input}, index: None, selector: None }}); }}");
    }
    out.check()
}
