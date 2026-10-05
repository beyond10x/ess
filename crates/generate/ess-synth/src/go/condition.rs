//! A binding's event-payload condition in the generated Go dispatch (ess/22, beyond10x/ess#268).
//!
//! Each conditioned binding whose delivery is generated gets one function in the system package,
//! named by the layout beside its transformation, over the typed event and answering in Kleene
//! logic as an `int8`: `1` invokes, `0` skips this binding alone, `-1` is Unknown. Go has no
//! option type to spell three values with, and an `int8` keeps every part of the predicate an
//! expression, so `all`, `any` and `not` stay one line each.
//!
//! See [`crate::condition`] for what the target-neutral half decides.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::{ResolvedBinding, ResolvedBody};
use ess_primitives::facts::FactValue;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::selection::go_string;
use super::Emit;
use crate::condition::{End, Step, Walk};

/// One condition function per conditioned binding in `bindings`; nothing where none is
/// conditioned, so an unconditioned specification keeps its bytes.
pub(super) fn functions<'a>(
    out: &mut String,
    emit: &Emit<'_>,
    bindings: impl Iterator<Item = &'a ResolvedBinding>,
) {
    for binding in bindings {
        let Some(condition) = &binding.condition else {
            continue;
        };
        let walks = crate::condition::walks(emit.ir, binding)
            .expect("a resolved condition reads declared paths");
        let source = binding.name.to_string();
        let event = binding
            .cause
            .event()
            .expect("a condition is on an event cause");
        let _ = writeln!(
            out,
            "\n// {function} is the condition of `{source}`: `{}` over the `{event}` it reacts \
             to.\n//\n// 1 invokes, 0 skips this binding alone, and -1 — a comparison reading an \
             absent member — is\n// Unknown: this binding's unmet obligation, never a \
             skip.\nfunc {function}(event {}) int8 {{\n\t_ = event\n\treturn {}\n}}",
            condition.plan.predicate,
            emit.reference(event.name()),
            predicate(&condition.plan.predicate, &walks, emit, binding),
            function = emit.layout.condition(&source),
        );
    }
}

/// The predicate as an `int8` expression: Kleene `all`, `any` and `not`.
fn predicate(
    predicate: &Predicate,
    walks: &BTreeMap<String, Walk<'_>>,
    emit: &Emit<'_>,
    binding: &ResolvedBinding,
) -> String {
    let parts = |children: &[Predicate]| {
        children
            .iter()
            .map(|child| self::predicate(child, walks, emit, binding))
            .collect::<Vec<_>>()
            .join(", ")
    };
    match predicate {
        Predicate::Always => "int8(1)".to_owned(),
        Predicate::Never => "int8(0)".to_owned(),
        Predicate::All(children) if children.is_empty() => "int8(1)".to_owned(),
        Predicate::Any(children) if children.is_empty() => "int8(0)".to_owned(),
        Predicate::All(children) => format!(
            "func(parts ...int8) int8 {{ result := int8(1); for _, part := range parts {{ if part \
             == 0 {{ return 0 }}; if part < 0 {{ result = -1 }} }}; return result }}({})",
            parts(children)
        ),
        Predicate::Any(children) => format!(
            "func(parts ...int8) int8 {{ result := int8(0); for _, part := range parts {{ if part \
             == 1 {{ return 1 }}; if part < 0 {{ result = -1 }} }}; return result }}({})",
            parts(children)
        ),
        Predicate::Not(child) => format!(
            "func(part int8) int8 {{ if part < 0 {{ return part }}; return 1 - part }}({})",
            self::predicate(child, walks, emit, binding)
        ),
        Predicate::Defined(path) => {
            let walk = &walks[&path.to_string()];
            let (statements, last) = read(walk, emit, binding, "0");
            format!("func() int8 {{ {statements}_ = {last}; return 1 }}()")
        }
        Predicate::Compare {
            left: Operand::Fact(path),
            op,
            right: Operand::Literal(FactValue::Text(literal)),
            ..
        } => {
            let walk = &walks[&path.to_string()];
            let (statements, last) = read(walk, emit, binding, "-1");
            let (holds, fails) = if *op == CompareOp::Ne {
                ("0", "1")
            } else {
                ("1", "0")
            };
            let test = match walk.end {
                End::String => format!("if {last} == {} {{ return {holds} }}", go_string(literal)),
                End::Enum(of) => format!(
                    "if _, is := {last}.({}); is {{ return {holds} }}",
                    emit.qualify(
                        emit.layout.package_of(of.name()),
                        emit.layout.variant(of.name(), literal)
                    )
                ),
                End::Other => unreachable!("admission compares String and enum leaves only"),
            };
            format!("func() int8 {{ {statements}{test}; return {fails} }}()")
        }
        other => unreachable!("`{other}` is outside the admitted binding condition"),
    }
}

/// One path as Go statements, each absent level answering `absent`, and the variable holding the
/// leaf.
fn read(
    walk: &Walk<'_>,
    emit: &Emit<'_>,
    binding: &ResolvedBinding,
    absent: &str,
) -> (String, String) {
    let mut statements = format!(
        "v0 := event.{}; ",
        super::accessor::root_identifier(emit, binding, &walk.root.name)
    );
    let mut index = 0;
    for step in &walk.steps {
        let next = index + 1;
        match step {
            Step::Optional => {
                let _ = write!(
                    statements,
                    "if v{index} == nil {{ return {absent} }}; v{next} := *v{index}; "
                );
            }
            Step::Newtype => {
                let _ = write!(statements, "v{next} := v{index}.Value(); ");
            }
            Step::Field { owner, name } => {
                let ResolvedBody::Struct { fields, .. } = &emit.ir.named_type(owner).body else {
                    unreachable!("a condition path crosses structs only")
                };
                let mut taken = BTreeMap::new();
                let ident = fields
                    .iter()
                    .map(|field| {
                        (
                            &field.name,
                            super::items::field_ident(&mut taken, &field.name),
                        )
                    })
                    .find(|(declared, _)| declared.as_str() == *name)
                    .expect("declared field")
                    .1;
                let _ = write!(statements, "v{next} := v{index}.{ident}; ");
            }
        }
        index = next;
    }
    (statements, format!("v{index}"))
}
