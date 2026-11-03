//! The state a toggle action shows: one expression, resolved here and handed to every renderer,
//! so the React project and the terminal cannot disagree about whether a toggle is on.

use crate::filter::{self, Predicate};
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
/// only `bind` entry that is `not <expr>` as a whole and reads a path. Two such entries, an entry
/// whose `not` is one operand of `and` or `or`, or a `not` over literal text name no state. A row
/// field named like the action is never read: a name match is a guess.
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

/// `<expr>` when `text` is `not <expr>` as a whole and `<expr>` reads a path.
fn negated(text: &str) -> Option<String> {
    let Predicate::Not(inner) = filter::parse(text)? else {
        return None;
    };
    if inner.paths().is_empty() {
        return None;
    }
    let rest = text.trim_start().strip_prefix("not")?;
    rest.starts_with(char::is_whitespace)
        .then(|| rest.trim().to_owned())
}
