//! Binding expressions, evaluated against what the terminal holds.
//!
//! The grammar the TUI understands is the part of the schema's expression language a fixture run
//! needs: a dotted path from a root (`row`, `state`, `params`, `shell`, `channel`, `draft`,
//! `actor`, `section`, `url`), `not <expr>`, `<a> == <b>`, `<a> != <b>`, and scalar literals. A
//! bare word that is not a root is its own text, as in `row.status == overdue`. A function call
//! (`matches(params)`, `same_as(list)`) is not evaluated here and yields [`None`]; callers decide
//! what an unknown means in their position.
//!
//! A string that reads no path and calls no function form is literal text, whatever operators
//! or parentheses it holds (`Limit in cents`, `Cost (cents)`); a lone quoted string, `true`,
//! `false`, `null` or number is that value.

use std::collections::BTreeSet;
use std::sync::OnceLock;

use serde_yaml::Value;

/// The roots a path may start from.
pub(crate) const ROOTS: &[&str] = &[
    "row", "state", "params", "shell", "channel", "draft", "actor", "section", "url", "args",
];

/// Resolves the first segment of a path; the rest is walked by [`walk`].
pub(crate) trait Resolve {
    /// The value under `root.<first>`, where `rest` holds the remaining segments. Returning the
    /// value of the whole path is allowed: the evaluator walks only what a resolver leaves.
    fn resolve(&self, root: &str, segments: &[&str]) -> Resolved;
}

/// What a resolver found.
pub(crate) enum Resolved {
    /// The value at the root; the evaluator walks the remaining segments into it.
    Root(Value),
    /// The value of the whole path.
    Whole(Value),
}

/// Evaluates `text`. `None` means the expression is outside the fixture grammar.
pub(crate) fn eval(text: &str, scope: &dyn Resolve) -> Option<Value> {
    let text = text.trim();
    if let Some(value) = scalar(text) {
        return Some(value);
    }
    if !reads(text) {
        return Some(Value::String(text.to_owned()));
    }
    if let Some(rest) = text.strip_prefix("not ") {
        return eval(rest, scope).map(|value| Value::Bool(!truthy(&value)));
    }
    for (operator, equal) in [(" == ", true), (" != ", false)] {
        if let Some((left, right)) = text.split_once(operator) {
            let left = eval(left, scope)?;
            let right = eval(right, scope)?;
            return Some(Value::Bool((display(&left) == display(&right)) == equal));
        }
    }
    if text.contains('(') {
        return None;
    }
    let segments: Vec<&str> = text.split('.').collect();
    if ROOTS.contains(&segments[0]) {
        let rest = &segments[1..];
        return Some(match scope.resolve(segments[0], rest) {
            Resolved::Whole(value) => value,
            Resolved::Root(value) => walk(&value, rest),
        });
    }
    Some(Value::String(text.to_owned()))
}

/// A lone literal: one quoted string, `true`, `false`, `null` or a number.
fn scalar(text: &str) -> Option<Value> {
    for quote in ['"', '\''] {
        if let Some(inner) = text
            .strip_prefix(quote)
            .and_then(|rest| rest.strip_suffix(quote))
        {
            if !inner.contains(quote) {
                return Some(Value::String(inner.to_owned()));
            }
        }
    }
    match text {
        "true" => return Some(Value::Bool(true)),
        "false" => return Some(Value::Bool(false)),
        "null" => return Some(Value::Null),
        _ => {}
    }
    if let Ok(number) = text.parse::<i64>() {
        return Some(Value::Number(number.into()));
    }
    if let Ok(number) = text.parse::<f64>() {
        return Some(Value::Number(number.into()));
    }
    None
}

/// The roots (`row`, `state`, …) and function forms (`same_as`, `matches`) of the schema's
/// `expressions.forms`.
fn forms() -> &'static (BTreeSet<String>, BTreeSet<String>) {
    static FORMS: OnceLock<(BTreeSet<String>, BTreeSet<String>)> = OnceLock::new();
    FORMS.get_or_init(|| {
        let schema: Value =
            serde_yaml::from_str(ess_ui::SCHEMA).expect("the embedded schema is YAML");
        let mut roots = BTreeSet::new();
        let mut functions = BTreeSet::new();
        for form in schema["expressions"]["forms"]
            .as_sequence()
            .into_iter()
            .flatten()
            .filter_map(|entry| entry["form"].as_str())
        {
            let head: String = form
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            match form[head.len()..].chars().next() {
                Some('.') => {
                    roots.insert(head);
                }
                Some('(') => {
                    functions.insert(head);
                }
                _ => {}
            }
        }
        (roots, functions)
    })
}

/// Whether `text`, outside its quoted strings, reads a path (`row.stage`) or calls a function
/// form (`matches(params)`). A string that reads neither is literal text, as `ess-ui-check`
/// decides: `Limit in cents`, `Cost (cents)` and `Terms and conditions` are not expressions.
fn reads(text: &str) -> bool {
    let (roots, functions) = forms();
    let mut quote = None;
    let mut word = String::new();
    for character in text.chars().chain([' ']) {
        if let Some(open) = quote {
            if character == open {
                quote = None;
            }
            continue;
        }
        if character.is_ascii_alphanumeric() || "_.-".contains(character) {
            word.push(character);
            continue;
        }
        let path = word.split_once('.').is_some_and(|(root, rest)| {
            roots.contains(root) && rest.split('.').all(|segment| !segment.is_empty())
        });
        if path || (character == '(' && functions.contains(&word)) {
            return true;
        }
        word.clear();
        if character == '"' || character == '\'' {
            quote = Some(character);
        }
    }
    false
}

/// The value at `segments` inside `value`, or `Null`.
pub(crate) fn walk(value: &Value, segments: &[&str]) -> Value {
    let mut at = value;
    for segment in segments {
        match at.get(*segment) {
            Some(next) => at = next,
            None => return Value::Null,
        }
    }
    at.clone()
}

/// Whether a value counts as true in a condition.
pub(crate) fn truthy(value: &Value) -> bool {
    match value {
        Value::Null => false,
        Value::Bool(flag) => *flag,
        Value::Number(number) => number.as_f64().is_some_and(|number| number != 0.0),
        Value::String(text) => !text.is_empty() && text != "false",
        Value::Sequence(items) => !items.is_empty(),
        Value::Mapping(entries) => !entries.is_empty(),
        Value::Tagged(tagged) => truthy(&tagged.value),
    }
}

/// A value as one line of text.
pub(crate) fn display(value: &Value) -> String {
    match value {
        Value::Null => String::new(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::String(text) => text.clone(),
        Value::Sequence(items) => items.iter().map(display).collect::<Vec<_>>().join(", "),
        Value::Mapping(entries) => {
            if let (Some(amount), Some(currency)) = (value.get("amount"), value.get("currency")) {
                return format!("{} {}", display(amount), display(currency));
            }
            entries
                .iter()
                .map(|(key, value)| format!("{}: {}", display(key), display(value)))
                .collect::<Vec<_>>()
                .join(", ")
        }
        Value::Tagged(tagged) => display(&tagged.value),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct Row(Value);

    impl Resolve for Row {
        fn resolve(&self, root: &str, _: &[&str]) -> Resolved {
            match root {
                "row" => Resolved::Root(self.0.clone()),
                _ => Resolved::Root(Value::Null),
            }
        }
    }

    #[test]
    fn paths_comparisons_and_negation_evaluate() {
        let row = Row(serde_yaml::from_str(
            "{status: overdue, amount: {amount: 5, currency: EUR}}",
        )
        .unwrap());
        assert_eq!(eval("row.status == overdue", &row), Some(Value::Bool(true)));
        assert_eq!(
            eval("row.status != overdue", &row),
            Some(Value::Bool(false))
        );
        assert_eq!(eval("not state.paused", &row), Some(Value::Bool(true)));
        assert_eq!(display(&eval("row.amount", &row).unwrap()), "5 EUR");
        assert_eq!(eval("matches(params)", &row), None);
        assert_eq!(eval("Open", &row), Some(Value::String("Open".into())));
    }

    /// #353: a string that reads no path and no function form is literal text; a lone scalar
    /// literal keeps its value.
    #[test]
    fn a_string_reading_no_path_is_literal_text() {
        let row = Row(serde_yaml::from_str("{stage: won}").unwrap());
        let text = |text: &str| Some(Value::String(text.into()));
        for literal in [
            "Limit in cents",
            "Cost (cents)",
            "Terms and conditions",
            "not now",
            "a == b",
        ] {
            assert_eq!(eval(literal, &row), text(literal), "{literal}");
        }
        assert_eq!(eval("\"Cost (cents)\"", &row), text("Cost (cents)"));
        assert_eq!(eval("'Limit in cents'", &row), text("Limit in cents"));
        assert_eq!(eval("false", &row), Some(Value::Bool(false)));
        assert_eq!(eval("null", &row), Some(Value::Null));
        assert_eq!(eval("42", &row), Some(Value::Number(42.into())));
        assert_eq!(eval("row.stage == won", &row), Some(Value::Bool(true)));
        assert_eq!(eval("row.stage == \"won\"", &row), Some(Value::Bool(true)));
        assert_eq!(eval("not row.stage", &row), Some(Value::Bool(false)));
    }
}
