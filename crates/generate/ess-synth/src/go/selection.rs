//! Native typed list selection with deterministic preflight and occurrence identity.
use super::Emit;
use crate::plan::accessor_type;
use ess_compiler::ir::{ResolvedBinding, ResolvedBody, ResolvedSelectionPlan, ResolvedTypeRef};
use ess_domain::selection::{InputSource, SelectionOperation};
use ess_primitives::predicate::{CompareOp, Operand, Predicate};
use std::{collections::BTreeMap, fmt::Write as _};

pub(super) fn support(emit: &Emit<'_>) -> String {
    let unmet = emit.unmet();
    format!(
        r#"
// SelectionFailureCause is the closed runtime refusal vocabulary.
type SelectionFailureCause string
const (
    // SelectionInvalidInput identifies malformed required typed input.
    SelectionInvalidInput SelectionFailureCause = "invalid_input"
    // SelectionUnsupported identifies a shape outside the finite runtime observation capability.
    SelectionUnsupported SelectionFailureCause = "unsupported"
    // SelectionUnknown identifies an undecidable predicate.
    SelectionUnknown SelectionFailureCause = "unknown"
    // SelectionResource identifies a fixed count or byte limit.
    SelectionResource SelectionFailureCause = "resource"
)
// SelectionFailure identifies the first failing input, occurrence and selector.
type SelectionFailure struct {{
    Cause SelectionFailureCause
    Input int
    Index *int
    Selector *int
}}
func (failure *SelectionFailure) Error() string {{ return "selection failure: " + string(failure.Cause) }}
// TransportFailure preserves both existing host obligations and selection failures.
type TransportFailure struct {{
    Obligation {unmet}
    Selection *SelectionFailure
}}
func (failure *TransportFailure) Error() string {{
    if failure.Selection != nil {{ return failure.Selection.Error() }}
    if failure.Obligation != nil {{ return failure.Obligation.Error() }}
    return "invalid transport failure"
}}
// Truth is represented privately as -1 unknown, 0 false and 1 true.
func selectionAll(values ...int) int {{
    answer := 1
    for _, value := range values {{ if value == 0 {{ return 0 }}; if value < 0 {{ answer = -1 }} }}
    return answer
}}
func selectionAny(values ...int) int {{
    answer := 0
    for _, value := range values {{ if value == 1 {{ return 1 }}; if value < 0 {{ answer = -1 }} }}
    return answer
}}
func selectionBool(value bool) int {{ if value {{ return 1 }}; return 0 }}
func selectionCompare(value string, present bool, expected string, equal bool) int {{
    if !present {{ return -1 }}
    return selectionBool((value == expected) == equal)
}}
func selectionNot(value int) int {{ if value < 0 {{ return -1 }}; return 1-value }}
func selectionIndex(value int) *int {{ return &value }}
"#
    )
}

fn unwrap_list(emit: &Emit<'_>, mut ty: ResolvedTypeRef, mut value: String) -> String {
    while let ResolvedTypeRef::Declared { name } = ty {
        let ResolvedBody::Newtype { of, .. } = &emit.ir.named_type(&name).body else {
            break;
        };
        value = format!("({value}).Value()");
        ty = of.clone();
    }
    value
}

fn scalar(emit: &Emit<'_>, ty: &ResolvedTypeRef, value: &str) -> String {
    let body = match ty {
        ResolvedTypeRef::Optional { of } => format!(
            "if {value} == nil {{ return \"\", false, true }}; return {}",
            scalar(emit, of, &format!("(*{value})"))
        ),
        ResolvedTypeRef::Declared { name } => match &emit.ir.named_type(name).body {
            ResolvedBody::Newtype { of, .. } => {
                format!("return {}", scalar(emit, of, &format!("({value}).Value()")))
            }
            ResolvedBody::Enum { variants } => {
                let arms = variants
                    .iter()
                    .map(|variant| {
                        format!(
                            "case {}: return {variant:?}, true, true",
                            emit.reference_variant(name.name(), variant)
                        )
                    })
                    .collect::<Vec<_>>()
                    .join("\n");
                format!("switch ({value}).(type) {{ {arms}\ndefault: return \"\", false, false }}")
            }
            _ => unreachable!("admitted predicate scalar"),
        },
        ResolvedTypeRef::Primitive { .. } => format!("return {value}, true, true"),
        _ => unreachable!("admitted predicate scalar"),
    };
    format!("func() (string, bool, bool) {{ {body} }}()")
}

fn predicate(predicate: &Predicate, reads: &BTreeMap<String, usize>) -> String {
    match predicate {
        Predicate::Always => "1".into(),
        Predicate::Never => "0".into(),
        Predicate::Defined(path) => format!("selectionBool(present{})", reads[&path.to_string()]),
        Predicate::Compare {
            left: Operand::Fact(path),
            op,
            right: Operand::Literal(value),
        } => {
            let ess_primitives::facts::FactValue::Text(value) = value else {
                unreachable!()
            };
            let id = reads[&path.to_string()];
            format!(
                "selectionCompare(read{id}, present{id}, {value:?}, {})",
                *op == CompareOp::Eq
            )
        }
        Predicate::All(children) | Predicate::Any(children) => format!(
            "{}({})",
            if matches!(predicate, Predicate::All(_)) {
                "selectionAll"
            } else {
                "selectionAny"
            },
            children
                .iter()
                .map(|child| self::predicate(child, reads))
                .collect::<Vec<_>>()
                .join(",")
        ),
        Predicate::Not(child) => format!("selectionNot({})", self::predicate(child, reads)),
        _ => unreachable!("admitted finite selector predicate"),
    }
}

pub(super) fn prelude(
    emit: &Emit<'_>,
    binding: &ResolvedBinding,
    selection: &ResolvedSelectionPlan,
) -> Result<String, String> {
    let mut out = crate::accessor_output::Output::default();
    out.push_str(&validators(emit, selection)?);
    let input_types = crate::selection::input_types(emit.ir, selection)?;
    let input_ids = crate::selection::type_ids(&input_types);
    let zero = format!("{}{{}}", emit.reference(binding.command.name()));
    input_prelude(&mut out, emit, binding, selection, &zero)?;
    let _ = writeln!(out, "selected := [{}]int{{}}\nfor index := range selected {{ selected[index] = -1 }}\nselectionBytes := 0\n_ = selectionBytes", selection.plan.selectors.len());
    for (selector, plan) in selection.plan.selectors.iter().enumerate() {
        if matches!(plan.operation, SelectionOperation::First { .. }) {
            let _ = writeln!(
                out,
                "matches{selector} := make([]bool, len(selectionInput{}))",
                plan.input
            );
        }
    }
    for (input, input_plan) in selection.plan.inputs.iter().enumerate() {
        let _ = writeln!(out, "for index, rawItem := range selectionInput{input} {{");
        if input_plan.optional_items {
            let element = crate::selection::element_type(emit.ir, selection, input);
            let container = unwrap_list(emit, element, "rawItem".to_owned());
            let _ = writeln!(out, "itemContainer := {container}; if itemContainer == nil {{ continue }}; item := *itemContainer");
        } else {
            out.push_str("item := rawItem\n");
        }
        out.push_str("_ = item; _ = index\n");
        let validator = input_ids[&accessor_type(&input_plan.item_type, &selection.types)];
        let _ = writeln!(out, "if cause := selectionValidate{validator}(item, &selectionBytes, 0); cause != \"\" {{ return {zero}, &SelectionFailure{{Cause: cause, Input: {input}, Index: selectionIndex(index)}} }}");
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
                    emit,
                    "item",
                )?;
                let id = ids[path];
                let text = scalar(emit, &ty, &format!("value{id}"));
                let _ = writeln!(out, "value{id} := {expr}\nread{id}, present{id}, valid{id} := {text}\nif !valid{id} {{ return {zero}, &SelectionFailure{{Cause: SelectionInvalidInput, Input: {input}, Index: selectionIndex(index), Selector: selectionIndex({selector})}} }}\nselectionBytes += len(read{id}) + {}\nif len(read{id}) > 4096 || selectionBytes > 1048576 {{ return {zero}, &SelectionFailure{{Cause: SelectionResource, Input: {input}, Index: selectionIndex(index), Selector: selectionIndex({selector})}} }}", path.len());
            }
            let expr = predicate(condition, &ids);
            let _ = writeln!(out, "truth := {expr}\nif truth < 0 {{ return {zero}, &SelectionFailure{{Cause: SelectionUnknown, Input: {input}, Index: selectionIndex(index), Selector: selectionIndex({selector})}} }}\nmatches{selector}[index] = truth == 1\n}}");
        }
        out.push_str("}\n");
    }
    for (selector, plan) in selection.plan.selectors.iter().enumerate() {
        match &plan.operation {
            SelectionOperation::First { excluding, .. } => {
                let exclude = excluding
                    .iter()
                    .map(|index| format!("selected[{index}] != index"))
                    .collect::<Vec<_>>()
                    .join(" && ");
                let _ = writeln!(out, "for index, eligible := range matches{selector} {{ if eligible{} {{ selected[{selector}] = index; break }} }}", if exclude.is_empty() { String::new() } else { format!(" && {exclude}") });
            }
            SelectionOperation::FirstPresent { selections } => {
                for earlier in selections {
                    let _ = writeln!(out, "if selected[{selector}] < 0 {{ selected[{selector}] = selected[{earlier}] }}");
                }
            }
        }
    }
    out.finish()
}

pub(super) fn mapping(
    emit: &Emit<'_>,
    selection: &ResolvedSelectionPlan,
    selector: usize,
    projection: &ess_domain::accessor::ProjectionPlan,
    target: &ResolvedTypeRef,
) -> Result<String, String> {
    let input = selection.plan.selectors[selector].input;
    let item = if selection.plan.inputs[input].optional_items {
        let element = crate::selection::element_type(emit.ir, selection, input);
        let container = unwrap_list(
            emit,
            element,
            format!("selectionInput{input}[selected[{selector}]]"),
        );
        format!("*({container})")
    } else {
        format!("selectionInput{input}[selected[{selector}]]")
    };
    let expr = super::accessor::expression_from(
        &projection.0,
        &selection.types,
        target,
        None,
        emit,
        "item",
    )?;
    Ok(format!("func() {} {{ if selected[{selector}] < 0 {{ return nil }}; item := {item}; return {expr} }}()", emit.go_type(target)))
}

fn validators(emit: &Emit<'_>, selection: &ResolvedSelectionPlan) -> Result<String, String> {
    let types = crate::selection::input_types(emit.ir, selection)?;
    let ids = crate::selection::type_ids(&types);
    let mut out = crate::accessor_output::Output::default();
    for (id, ty) in types.iter().enumerate() {
        let _ = writeln!(
            out,
            "var selectionValidate{id} func({}, *int, int) SelectionFailureCause",
            emit.go_type(ty)
        );
    }
    for (id, ty) in types.iter().enumerate() {
        let _ = writeln!(out, "selectionValidate{id} = func(value {}, bytes *int, depth int) SelectionFailureCause {{ if depth > 128 || *bytes > 1048576 {{ return SelectionResource }}", emit.go_type(ty));
        match ty {
            ResolvedTypeRef::Primitive {
                name: ess_domain::Primitive::String,
            } => out.push_str(
                "*bytes += len(value); if len(value) > 4096 { return SelectionResource }\n",
            ),
            ResolvedTypeRef::Primitive { .. } => out.push_str("_ = value\n"),
            ResolvedTypeRef::Optional { of } => {
                let _ = writeln!(out, "if value != nil {{ if cause := selectionValidate{}(*value, bytes, depth+1); cause != \"\" {{ return cause }} }}", ids[of.as_ref()]);
            }
            ResolvedTypeRef::List { of } => {
                let _ = writeln!(out, "if value == nil {{ return SelectionInvalidInput }}; if len(value) > 64 {{ return SelectionResource }}; for _, item := range value {{ if cause := selectionValidate{}(item, bytes, depth+1); cause != \"\" {{ return cause }} }}", ids[of.as_ref()]);
            }
            ResolvedTypeRef::Map { .. } => out.push_str("_ = value; return SelectionUnsupported\n"),
            ResolvedTypeRef::Declared { name: handle } => match &emit.ir.named_type(handle).body {
                ResolvedBody::Newtype { of, .. } => {
                    let _ = writeln!(out, "if cause := selectionValidate{}(value.Value(), bytes, depth+1); cause != \"\" {{ return cause }}", ids[of]);
                }
                ResolvedBody::Struct { fields, .. } => {
                    let mut taken = BTreeMap::new();
                    for field in fields {
                        let _ = writeln!(out, "*bytes += {}; if cause := selectionValidate{}(value.{}, bytes, depth+1); cause != \"\" {{ return cause }}", field.name.len(), ids[&field.type_ref], super::items::field_ident(&mut taken, &field.name));
                    }
                }
                ResolvedBody::Enum { variants } => {
                    out.push_str("switch value.(type) {\n");
                    for variant in variants {
                        let _ = writeln!(
                            out,
                            "case {}: *bytes += {}",
                            emit.reference_variant(handle.name(), variant),
                            variant.len()
                        );
                    }
                    out.push_str("default: return SelectionInvalidInput\n}\n");
                }
                ResolvedBody::Union { variants, .. } => {
                    out.push_str("switch branch := value.(type) {\n");
                    for (label, child) in variants {
                        let _ = writeln!(out, "case {}: if cause := selectionValidate{}(branch.Value, bytes, depth+1); cause != \"\" {{ return cause }}", emit.reference_variant(handle.name(), label), ids[child]);
                    }
                    out.push_str("default: return SelectionInvalidInput\n}\n");
                }
            },
        }
        out.push_str("if *bytes > 1048576 { return SelectionResource }; return \"\"\n}\n");
    }
    out.finish()
}

fn input_prelude(
    out: &mut crate::accessor_output::Output,
    emit: &Emit<'_>,
    binding: &ResolvedBinding,
    selection: &ResolvedSelectionPlan,
    zero: &str,
) -> Result<(), String> {
    for (input, plan) in selection.plan.inputs.iter().enumerate() {
        let source = if plan.conversion.is_some() {
            format!("prepared_{input}")
        } else {
            match &plan.source {
                InputSource::Field { field } => format!(
                    "event.{}",
                    super::accessor::root_identifier(emit, binding, &field.name)
                ),
                InputSource::Accessor { plan } => super::accessor::expression(
                    plan,
                    &selection.types,
                    &accessor_type(&plan.effective_type(), &selection.types),
                    None,
                    emit,
                    &super::accessor::root_identifier(emit, binding, &plan.root.name),
                )?,
            }
        };
        let list = unwrap_list(
            emit,
            accessor_type(&plan.list_type, &selection.types),
            source,
        );
        let _ = writeln!(out, "selectionInput{input} := {list}\nif selectionInput{input} == nil {{ return {zero}, &SelectionFailure{{Cause: SelectionInvalidInput, Input: {input}}} }}\nif len(selectionInput{input}) > 64 {{ return {zero}, &SelectionFailure{{Cause: SelectionResource, Input: {input}}} }}");
    }
    out.check()
}
