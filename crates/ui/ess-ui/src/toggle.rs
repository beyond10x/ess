//! The state a toggle action shows: one expression, resolved here and handed to every renderer,
//! so the React project and the terminal cannot disagree about whether a toggle is on.

use ess_yaml::Value;

use crate::filter::{self, Operator, Predicate};
use crate::model::{Action, ActionAs, Expr};

/// Where a toggle's state expression comes from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ToggleSource {
    /// The action's own `checked:` expression.
    Checked,
    /// The one `bind` entry written `not <expr>`: the toggle sends the negation of its state.
    Flipped {
        /// The command input that entry binds.
        input: String,
    },
    /// The first key the toggle `sets` (a `sets.*: toggle` expands to `not <key>`).
    Sets {
        /// That key.
        key: String,
    },
}

/// A toggle's state: an expression a renderer evaluates in the action's scope, on when truthy.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToggleState {
    /// The expression.
    pub expr: Expr,
    /// What it was read from.
    pub source: ToggleSource,
}

/// The state of an `as: toggle` action, or `None` when the action is not a toggle or no state
/// can be named.
///
/// In order: `checked:` when written; else for a toggle with `sets`, the first key it sets
/// (the state it has always shown); else for a toggle with `does`, the expression `<expr>` of its
/// only `bind` entry that parses as `not <expr>` as a whole, where `<expr>` reads a path and
/// every path it reads starts at a root the filter grammar admits
/// ([`Predicate::allowed_roots`]). `<expr>` is written in canonical text without redundant
/// parentheses and with every string quoted (`not (row.on)` and `(not row.on)` both name
/// `row.on`; `not (row.stage == 'draft')` names `row.stage == 'draft'`), so the terminal reads it
/// with the filter grammar as the React runtime reads it. Two such entries, an entry whose `not`
/// is one operand of `and` or `or`, or a `not` over literal text (`not yet sent`, or a dotted
/// word such as `item.enabled` that starts at no root) name no state. A row field named like the
/// action is never read: a name match is a guess.
pub fn toggle_state(action: &Action) -> Option<ToggleState> {
    if action.action_as != Some(ActionAs::Toggle) {
        return None;
    }
    if let Some(checked) = &action.checked {
        return Some(ToggleState {
            expr: checked.clone(),
            source: ToggleSource::Checked,
        });
    }
    if let Some(key) = action.sets.keys().next() {
        return Some(ToggleState {
            expr: Expr(key.clone()),
            source: ToggleSource::Sets { key: key.clone() },
        });
    }
    action.does.as_ref()?;
    let mut flipped = action
        .bind
        .iter()
        .filter_map(|(input, value)| negated(&value.0).map(|state| (input, state)));
    let (input, state) = flipped.next()?;
    if flipped.next().is_some() {
        return None;
    }
    Some(ToggleState {
        expr: Expr(state),
        source: ToggleSource::Flipped {
            input: input.clone(),
        },
    })
}

/// The state `text` flips: the operand of a `bind` entry that parses as one negation as a whole
/// (`not x`, `not(x)`, `(not x)`, `not (x)`) and reads a path, every path from an admitted root,
/// written back in canonical text so every renderer reads the same expression however the entry
/// was spaced or parenthesised. A dotted word whose first segment is no root is literal text to
/// both renderers, so an operand that reads one names no state.
fn negated(text: &str) -> Option<String> {
    let Predicate::Not(inner) = filter::parse(text)? else {
        return None;
    };
    if inner.paths().is_empty() || !inner.allowed_roots() {
        return None;
    }
    canonical(&inner)
}

/// How tightly a form binds in the filter grammar: `or` < `and` < `not` and comparisons <
/// primaries. An operand is parenthesised exactly when it binds looser than its position needs.
fn binding(predicate: &Predicate) -> u8 {
    match predicate {
        Predicate::Binary(Operator::Or, ..) => 1,
        Predicate::Binary(Operator::And, ..) => 2,
        Predicate::Not(_) | Predicate::Binary(..) => 3,
        Predicate::Literal(_) | Predicate::Path(_) | Predicate::List(_) => 4,
    }
}

/// `predicate` as text that [`filter::parse`] reads back as the same tree, with no parentheses
/// the tree does not need. `None` for a literal the grammar cannot spell.
fn canonical(predicate: &Predicate) -> Option<String> {
    let at = |operand: &Predicate, needs: u8| {
        let text = canonical(operand)?;
        Some(if binding(operand) < needs {
            format!("({text})")
        } else {
            text
        })
    };
    Some(match predicate {
        Predicate::Literal(value) => literal(value)?,
        Predicate::Path(segments) => segments.join("."),
        Predicate::List(items) => format!(
            "[{}]",
            items
                .iter()
                .map(canonical)
                .collect::<Option<Vec<_>>>()?
                .join(", ")
        ),
        Predicate::Not(inner) => format!("not {}", at(inner, 3)?),
        Predicate::Binary(op, left, right) => {
            let (word, left_needs, right_needs) = match op {
                Operator::Or => ("or", 1, 2),
                Operator::And => ("and", 2, 3),
                Operator::Equal => ("==", 4, 4),
                Operator::NotEqual => ("!=", 4, 4),
                Operator::In => ("in", 4, 4),
                Operator::Contains => ("contains", 4, 4),
            };
            format!(
                "{} {word} {}",
                at(left, left_needs)?,
                at(right, right_needs)?
            )
        }
    })
}

/// A scalar literal as the grammar spells it. A string is always quoted: a bare word reads back
/// as the same string in the filter grammar, but the renderers read a bare root name (`draft`,
/// `state`, `row`, ...) as a path, so only a quoted string is text to every reader.
fn literal(value: &Value) -> Option<String> {
    match value {
        Value::Null => Some("null".to_owned()),
        Value::Bool(value) => Some(value.to_string()),
        Value::Number(number) => number.as_f64().map(|number| number.to_string()),
        Value::String(text) => ['\'', '"']
            .into_iter()
            .find(|quote| !text.contains(*quote))
            .map(|quote| format!("{quote}{text}{quote}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::{canonical, negated};
    use crate::filter;

    /// The class behind round 1's F2 and F3 and round 2's F1: the state is read off the parse
    /// tree, never sliced from the source text, so every spelling of one negation names one state,
    /// that state parses back to the operand's tree, and every string in it is quoted (a bare
    /// root name such as `draft` or `state` is a path to the renderers).
    #[test]
    fn every_spelling_of_one_negation_names_the_operand_in_canonical_text() {
        let cases = [
            ("not row.enabled", "row.enabled"),
            ("not(row.enabled)", "row.enabled"),
            ("(not row.enabled)", "row.enabled"),
            ("not (row.enabled)", "row.enabled"),
            ("  not    row.enabled  ", "row.enabled"),
            ("((not ((row.enabled))))", "row.enabled"),
            ("not not row.enabled", "not row.enabled"),
            ("not (not (row.enabled))", "not row.enabled"),
            ("not (row.state == 'on')", "row.state == 'on'"),
            ("not (row.stage == 'state')", "row.stage == 'state'"),
            ("not row.state=='on air'", "row.state == 'on air'"),
            (
                "not (row.a and (row.b or row.c))",
                "row.a and (row.b or row.c)",
            ),
            (
                "not ((row.a and row.b) or row.c)",
                "row.a and row.b or row.c",
            ),
            (
                "not (row.a or (row.b or row.c))",
                "row.a or (row.b or row.c)",
            ),
            (
                "not (row.kind in [a, 'b c', 2, true])",
                "row.kind in ['a', 'b c', 2, true]",
            ),
            ("not ((not row.a) == row.b)", "(not row.a) == row.b"),
            ("not (row.note == \"it's\")", "row.note == \"it's\""),
            ("not (row.n == 1.5)", "row.n == 1.5"),
            ("not (row.n != -2)", "row.n != -2"),
            ("not (row.label == 'true')", "row.label == 'true'"),
        ];
        for (text, state) in cases {
            assert_eq!(negated(text).as_deref(), Some(state), "{text}");
            let Some(filter::Predicate::Not(inner)) = filter::parse(text) else {
                panic!("{text} is one negation");
            };
            assert_eq!(
                filter::parse(state).as_ref(),
                Some(&*inner),
                "{text} -> {state}"
            );
            assert_eq!(
                canonical(&inner).as_deref(),
                Some(state),
                "{state} is a fixed point"
            );
        }
    }

    /// Anything that is not one negation of an expression reading a path, every path from an
    /// admitted root, names no state.
    #[test]
    fn anything_but_one_negation_of_a_path_names_no_state() {
        for text in [
            "row.enabled",
            "not row.a and row.b",
            "(not row.a) or row.b",
            "not yet sent",
            "not 'x'",
            "not item.enabled",
            "not (row.a and item.b)",
            "not (row.kind in [a, item.b])",
            "not (row.a",
            "not",
            "",
        ] {
            assert_eq!(negated(text), None, "{text}");
        }
    }
}
