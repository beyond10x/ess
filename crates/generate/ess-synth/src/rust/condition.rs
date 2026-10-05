//! A binding's event-payload condition in the generated Rust dispatch (ess/22, beyond10x/ess#268).
//!
//! Each conditioned binding whose delivery is generated gets one function in the system crate's
//! `conditions` module, over the typed event, answering in Kleene logic: `Some(true)` invokes,
//! `Some(false)` skips this binding alone, `None` is Unknown. A module rather than a suffix on the
//! transformation's name, because a module lives in the type namespace and a binding's
//! transformation in the value namespace, so no binding name can make the two collide.
//!
//! See [`crate::condition`] for what the target-neutral half decides.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::ir::{EssIr, ResolvedBinding};
use ess_primitives::facts::FactValue;
use ess_primitives::predicate::{CompareOp, Operand, Predicate};

use super::layout::Layout;
use super::name;
use super::port::types_path;
use crate::condition::{End, Step, Walk};

/// The `conditions` module, holding one function per conditioned binding in `bindings`; nothing
/// where none is conditioned, so an unconditioned specification keeps its bytes.
pub(super) fn module<'a>(
    out: &mut String,
    ir: &EssIr,
    layout: &Layout,
    types: &str,
    bindings: impl Iterator<Item = &'a ResolvedBinding>,
) {
    let mut functions = String::new();
    for binding in bindings {
        let Some(condition) = &binding.condition else {
            continue;
        };
        let walks = crate::condition::walks(ir, binding)
            .expect("a resolved condition reads declared paths");
        let event = binding
            .cause
            .event()
            .expect("a condition is on an event cause");
        let _ = writeln!(
            functions,
            "\n    /// The condition of `{}`: `{}` over the `{}` it reacts to.\n    ///\n    /// \
             `Some(true)` invokes, `Some(false)` skips this binding alone, and `None` — a \
             comparison\n    /// reading an absent member — is Unknown: this binding's unmet \
             obligation, never a skip.\n    pub fn {}(event: &{}) -> Option<bool> {{\n        \
             let _ = event;\n        {}\n    }}",
            binding.name,
            condition.plan.predicate,
            event,
            name::value_ident(&binding.name.to_string()),
            types_path(layout, types, event.name()),
            predicate(&condition.plan.predicate, &walks, layout, types),
        );
    }
    if functions.is_empty() {
        return;
    }
    let _ = write!(
        out,
        "\n/// Every binding's event-payload condition (ess/22), evaluated before its \
         transformation, its\n/// invocation record and its port, as the specification orders \
         them.\npub mod conditions {{{functions}}}\n"
    );
}

/// The predicate as an `Option<bool>` expression: Kleene `all`, `any` and `not`.
fn predicate(
    predicate: &Predicate,
    walks: &BTreeMap<String, Walk<'_>>,
    layout: &Layout,
    types: &str,
) -> String {
    let parts = |children: &[Predicate]| {
        children
            .iter()
            .map(|child| self::predicate(child, walks, layout, types))
            .collect::<Vec<_>>()
            .join(", ")
    };
    match predicate {
        Predicate::Always => "Some(true)".to_owned(),
        Predicate::Never => "Some(false)".to_owned(),
        Predicate::All(children) if children.is_empty() => "Some(true)".to_owned(),
        Predicate::Any(children) if children.is_empty() => "Some(false)".to_owned(),
        Predicate::All(children) => format!(
            "{{ let parts = [{}]; if parts.contains(&Some(false)) {{ Some(false) }} else if \
             parts.contains(&None) {{ None }} else {{ Some(true) }} }}",
            parts(children)
        ),
        Predicate::Any(children) => format!(
            "{{ let parts = [{}]; if parts.contains(&Some(true)) {{ Some(true) }} else if \
             parts.contains(&None) {{ None }} else {{ Some(false) }} }}",
            parts(children)
        ),
        Predicate::Not(child) => format!(
            "({}).map(|holds| !holds)",
            self::predicate(child, walks, layout, types)
        ),
        Predicate::Defined(path) => {
            let walk = &walks[&path.to_string()];
            format!("Some({}.is_some())", read(walk))
        }
        Predicate::Compare {
            left: Operand::Fact(path),
            op,
            right: Operand::Literal(FactValue::Text(literal)),
            ..
        } => {
            let walk = &walks[&path.to_string()];
            let equal = match walk.end {
                End::String => format!("value.as_str() == {literal:?}"),
                End::Enum(of) => format!(
                    "matches!(value, {}::{})",
                    types_path(layout, types, of.name()),
                    name::pascal(literal)
                ),
                End::Other => unreachable!("admission compares String and enum leaves only"),
            };
            let test = if *op == CompareOp::Ne {
                format!("!({equal})")
            } else {
                equal
            };
            format!("{}.map(|value| {test})", read(walk))
        }
        other => unreachable!("`{other}` is outside the admitted binding condition"),
    }
}

/// One path as an `Option<&leaf>` expression: absent wherever an Optional level is.
fn read(walk: &Walk<'_>) -> String {
    let mut out = format!("Some(&event.{})", name::value_ident(&walk.root.name));
    for step in &walk.steps {
        match step {
            Step::Optional => out.push_str(".and_then(|value| value.as_ref())"),
            Step::Newtype => out.push_str(".map(|value| &value.0)"),
            Step::Field { name: member, .. } => {
                let _ = write!(out, ".map(|value| &value.{})", name::value_ident(member));
            }
        }
    }
    out
}
