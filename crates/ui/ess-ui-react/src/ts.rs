//! TypeScript text: literals, identifiers and an indenting writer.

use std::fmt::Write as _;

use serde_yaml::Value;

/// A string as a TypeScript string literal (JSON quoting is valid TypeScript).
pub(crate) fn string(text: &str) -> String {
    serde_json::to_string(text).unwrap_or_else(|_| "\"\"".to_owned())
}

/// A YAML value as a TypeScript literal, keys in document order.
pub(crate) fn literal(value: &Value) -> String {
    let mut out = String::new();
    write_literal(&mut out, value);
    out
}

fn write_literal(out: &mut String, value: &Value) {
    match value {
        Value::Null => out.push_str("null"),
        Value::Bool(flag) => out.push_str(if *flag { "true" } else { "false" }),
        Value::Number(number) => {
            if let Some(integer) = number.as_i64() {
                let _ = write!(out, "{integer}");
            } else if let Some(integer) = number.as_u64() {
                let _ = write!(out, "{integer}");
            } else {
                match number.as_f64() {
                    Some(float) if float.is_finite() => {
                        let _ = write!(out, "{float}");
                    }
                    _ => out.push_str("null"),
                }
            }
        }
        Value::String(text) => out.push_str(&string(text)),
        Value::Sequence(items) => {
            out.push('[');
            for (index, item) in items.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                write_literal(out, item);
            }
            out.push(']');
        }
        Value::Mapping(mapping) => {
            if mapping.is_empty() {
                out.push_str("{}");
                return;
            }
            out.push_str("{ ");
            for (index, (key, item)) in mapping.iter().enumerate() {
                if index > 0 {
                    out.push_str(", ");
                }
                out.push_str(&string(&key_text(key)));
                out.push_str(": ");
                write_literal(out, item);
            }
            out.push_str(" }");
        }
        Value::Tagged(tagged) => write_literal(out, &tagged.value),
    }
}

/// A mapping key as text.
pub(crate) fn key_text(key: &Value) -> String {
    match key {
        Value::String(text) => text.clone(),
        Value::Bool(flag) => flag.to_string(),
        Value::Number(number) => number.to_string(),
        Value::Null => "null".to_owned(),
        other => serde_yaml::to_string(other)
            .unwrap_or_default()
            .trim()
            .to_owned(),
    }
}

/// `partners.list` → `PartnersList`.
pub(crate) fn pascal(name: &str) -> String {
    let mut out = String::new();
    for part in name.split(['.', '_', '-', '/']) {
        let mut chars = part.chars();
        if let Some(first) = chars.next() {
            out.extend(first.to_uppercase());
            out.push_str(chars.as_str());
        }
    }
    if out.is_empty() || out.starts_with(|character: char| character.is_ascii_digit()) {
        out.insert(0, 'N');
    }
    out
}

/// `nav_expanded` → `navExpanded`, safe as a local identifier.
pub(crate) fn camel(name: &str) -> String {
    let pascal = pascal(name);
    let mut chars = pascal.chars();
    let first = chars.next().map_or('v', |c| c.to_ascii_lowercase());
    let mut out = String::new();
    out.push(first);
    out.push_str(chars.as_str());
    format!("{out}Value")
}

/// A property key: bare when it is an identifier, quoted otherwise.
pub(crate) fn key(name: &str) -> String {
    let identifier = name
        .chars()
        .next()
        .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_');
    if identifier {
        name.to_owned()
    } else {
        string(name)
    }
}

/// An object literal from `(key, value-literal)` pairs, skipping absent values.
pub(crate) fn object<I, K>(entries: I) -> String
where
    I: IntoIterator<Item = (K, Option<String>)>,
    K: AsRef<str>,
{
    let parts: Vec<String> = entries
        .into_iter()
        .filter_map(|(name, value)| value.map(|value| format!("{}: {value}", key(name.as_ref()))))
        .collect();
    if parts.is_empty() {
        "{}".to_owned()
    } else {
        format!("{{ {} }}", parts.join(", "))
    }
}

/// An array literal.
pub(crate) fn array<I: IntoIterator<Item = String>>(items: I) -> String {
    format!("[{}]", items.into_iter().collect::<Vec<_>>().join(", "))
}

/// A map of names to string literals.
pub(crate) fn string_map<'a, I>(entries: I) -> String
where
    I: IntoIterator<Item = (&'a String, &'a String)>,
{
    object(
        entries
            .into_iter()
            .map(|(name, value)| (name.as_str(), Some(string(value)))),
    )
}

/// An indenting line writer.
#[derive(Default)]
pub(crate) struct Writer {
    out: String,
    depth: usize,
}

impl Writer {
    pub(crate) fn line(&mut self, text: impl AsRef<str>) {
        let text = text.as_ref();
        if text.is_empty() {
            self.out.push('\n');
            return;
        }
        for _ in 0..self.depth {
            self.out.push_str("  ");
        }
        self.out.push_str(text);
        self.out.push('\n');
    }

    pub(crate) fn open(&mut self, text: impl AsRef<str>) {
        self.line(text);
        self.depth += 1;
    }

    pub(crate) fn close(&mut self, text: impl AsRef<str>) {
        self.depth = self.depth.saturating_sub(1);
        self.line(text);
    }

    pub(crate) fn indent(&mut self) {
        self.depth += 1;
    }

    pub(crate) fn dedent(&mut self) {
        self.depth = self.depth.saturating_sub(1);
    }

    pub(crate) fn finish(self) -> String {
        self.out
    }
}
