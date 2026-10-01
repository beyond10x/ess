//! Binding expressions, evaluated against what the terminal holds.
//!
//! The grammar the TUI understands is the part of the schema's expression language a fixture run
//! needs: a dotted path from a root (`row`, `state`, `params`, `shell`, `channel`, `draft`,
//! `actor`, `section`, `url`), `not <expr>`, `<a> == <b>`, `<a> != <b>`, and scalar literals. A
//! bare word that is not a root is its own text, as in `row.status == overdue`. A function call
//! (`matches(params)`, `same_as(list)`) is not evaluated here and yields [`None`]; callers decide
//! what an unknown means in their position.

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
    if let Some(quoted) = text
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .or_else(|| {
            text.strip_prefix('\'')
                .and_then(|rest| rest.strip_suffix('\''))
        })
    {
        return Some(Value::String(quoted.to_owned()));
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

/// A row's field: `budget.limit_cents` is the `limit_cents` member of the row's `budget`.
pub(crate) fn field_path(row: &Value, field: &str) -> Value {
    walk(row, &field.split('.').collect::<Vec<_>>())
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
}
