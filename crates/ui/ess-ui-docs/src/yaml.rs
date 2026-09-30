//! A YAML writer for examples and inline values, emitting classified tokens so the HTML page can
//! colour them at generation time and the Markdown page can print them plain.
//!
//! Only what the reference shows is written here; nothing re-reads these bytes as data. A test
//! still holds every example the schema carries to a round trip through the YAML reader.

use std::fmt::Write as _;

use serde_yaml::{Mapping, Value};

/// What a token is, for colouring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Kind {
    /// A mapping key.
    Key,
    /// A string value.
    Str,
    /// A number.
    Num,
    /// `true`, `false` or `null`.
    Lit,
    /// Indicators: `- `, `: `, braces, brackets and commas.
    Punct,
    /// Indentation.
    Space,
}

/// One classified piece of YAML text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct Tok {
    pub(crate) kind: Kind,
    pub(crate) text: String,
}

fn tok(kind: Kind, text: impl Into<String>) -> Tok {
    Tok {
        kind,
        text: text.into(),
    }
}

/// The widest line a nested value is kept on before it is written as a block.
const WIDTH: usize = 96;

/// `value` on one line, in flow style.
pub(crate) fn flow(value: &Value) -> String {
    let mut toks = Vec::new();
    flow_toks(value, Kind::Str, &mut toks);
    toks.into_iter().map(|t| t.text).collect()
}

/// `value` in block style, one token list per line.
pub(crate) fn block(value: &Value) -> Vec<Vec<Tok>> {
    let mut lines = Vec::new();
    emit(value, 0, &mut lines);
    lines
}

/// The plain text of [`block`].
#[cfg(test)]
pub(crate) fn block_text(value: &Value) -> String {
    block(value)
        .into_iter()
        .map(|line| line.into_iter().map(|t| t.text).collect::<String>() + "\n")
        .collect()
}

fn flow_len(value: &Value) -> usize {
    flow(value).chars().count()
}

fn is_collection(value: &Value) -> bool {
    match value {
        Value::Mapping(map) => !map.is_empty(),
        Value::Sequence(seq) => !seq.is_empty(),
        _ => false,
    }
}

fn emit(value: &Value, indent: usize, lines: &mut Vec<Vec<Tok>>) {
    match value {
        Value::Mapping(map) if !map.is_empty() => emit_mapping(map, indent, lines),
        Value::Sequence(seq) if !seq.is_empty() => {
            for item in seq {
                if !is_collection(item) || indent + 2 + flow_len(item) <= WIDTH {
                    let mut line =
                        vec![tok(Kind::Space, " ".repeat(indent)), tok(Kind::Punct, "- ")];
                    flow_toks(item, Kind::Str, &mut line);
                    lines.push(line);
                } else {
                    let mut nested = Vec::new();
                    emit(item, indent + 2, &mut nested);
                    if let Some(first) = nested.first_mut() {
                        first[0] = tok(Kind::Space, " ".repeat(indent));
                        first.insert(1, tok(Kind::Punct, "- "));
                    }
                    lines.extend(nested);
                }
            }
        }
        _ => {
            let mut line = vec![tok(Kind::Space, " ".repeat(indent))];
            flow_toks(value, Kind::Str, &mut line);
            lines.push(line);
        }
    }
}

fn emit_mapping(map: &Mapping, indent: usize, lines: &mut Vec<Vec<Tok>>) {
    for (key, value) in map {
        let mut line = vec![tok(Kind::Space, " ".repeat(indent))];
        flow_toks(key, Kind::Key, &mut line);
        let key_len: usize = line.iter().map(|t| t.text.chars().count()).sum();
        if !is_collection(value) || key_len + 2 + flow_len(value) <= WIDTH {
            line.push(tok(Kind::Punct, ": "));
            flow_toks(value, Kind::Str, &mut line);
            lines.push(line);
        } else {
            line.push(tok(Kind::Punct, ":"));
            lines.push(line);
            emit(value, indent + 2, lines);
        }
    }
}

/// Appends `value` in flow style; a string is classified as `string_kind` (keys are strings too).
fn flow_toks(value: &Value, string_kind: Kind, out: &mut Vec<Tok>) {
    match value {
        Value::Null => out.push(tok(Kind::Lit, "null")),
        Value::Bool(b) => out.push(tok(Kind::Lit, b.to_string())),
        Value::Number(n) => out.push(tok(Kind::Num, n.to_string())),
        Value::String(s) => out.push(tok(string_kind, scalar(s))),
        Value::Sequence(seq) => {
            out.push(tok(Kind::Punct, "["));
            for (i, item) in seq.iter().enumerate() {
                if i > 0 {
                    out.push(tok(Kind::Punct, ", "));
                }
                flow_toks(item, Kind::Str, out);
            }
            out.push(tok(Kind::Punct, "]"));
        }
        Value::Mapping(map) => {
            out.push(tok(Kind::Punct, "{"));
            for (i, (key, item)) in map.iter().enumerate() {
                if i > 0 {
                    out.push(tok(Kind::Punct, ", "));
                }
                flow_toks(key, Kind::Key, out);
                out.push(tok(Kind::Punct, ": "));
                flow_toks(item, Kind::Str, out);
            }
            out.push(tok(Kind::Punct, "}"));
        }
        Value::Tagged(tagged) => {
            out.push(tok(Kind::Punct, format!("{} ", tagged.tag)));
            flow_toks(&tagged.value, string_kind, out);
        }
    }
}

/// A string as a YAML scalar: plain where that reads back as the same string, double-quoted
/// otherwise.
fn scalar(s: &str) -> String {
    if needs_quotes(s) {
        quoted(s)
    } else {
        s.to_owned()
    }
}

/// A string as a double-quoted YAML scalar, which reads back as the same string whatever it holds.
pub(crate) fn quoted(s: &str) -> String {
    let mut quoted = String::from("\"");
    for c in s.chars() {
        match c {
            '"' => quoted.push_str("\\\""),
            '\\' => quoted.push_str("\\\\"),
            '\n' => quoted.push_str("\\n"),
            '\t' => quoted.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(quoted, "\\u{:04x}", u32::from(c));
            }
            c => quoted.push(c),
        }
    }
    quoted.push('"');
    quoted
}

fn needs_quotes(s: &str) -> bool {
    const RESERVED: &[&str] = &[
        "true", "false", "null", "~", "yes", "no", "on", "off", "y", "n",
    ];
    let Some(first) = s.chars().next() else {
        return true;
    };
    s != s.trim()
        || RESERVED.contains(&s.to_ascii_lowercase().as_str())
        || s.parse::<f64>().is_ok()
        || s.starts_with("0x")
        || s.starts_with("0o")
        || s.starts_with('.')
        || "-?:,[]{}#&*!|>'\"%@`".contains(first)
        || s.contains(": ")
        || s.contains(" #")
        || s.ends_with(':')
        || s.chars().any(|c| ",[]{}".contains(c) || c.is_control())
}

#[cfg(test)]
mod tests {
    use super::{block_text, flow};
    use serde_yaml::Value;

    fn schema() -> Value {
        serde_yaml::from_str(ess_ui::SCHEMA).expect("the schema is YAML")
    }

    #[test]
    fn every_example_and_field_reads_back_as_the_value_it_was_written_from() {
        let schema = schema();
        let mut checked = 0;
        for (name, construct) in schema["constructs"].as_mapping().expect("constructs") {
            for value in [&construct["example"], &construct["fields"]] {
                let block: Value =
                    serde_yaml::from_str(&block_text(value)).expect("the block is YAML");
                assert_eq!(&block, value, "{name:?} in block style");
                let flowed: Value = serde_yaml::from_str(&flow(value)).expect("the flow is YAML");
                assert_eq!(&flowed, value, "{name:?} in flow style");
                checked += 1;
            }
        }
        assert!(checked > 100, "only {checked} values were checked");
    }

    #[test]
    fn strings_that_would_read_back_as_something_else_are_quoted() {
        for s in [
            "true", "null", "1", "1.5", "", " x", "a: b", "- x", "{x}", "a, b", "*", "Tip: x",
        ] {
            let text = flow(&Value::from(s));
            assert!(text.starts_with('"'), "{s:?} is written {text}");
            let back: Value = serde_yaml::from_str(&text).expect("YAML");
            assert_eq!(back, Value::from(s));
        }
        assert_eq!(
            flow(&Value::from("row.status == overdue")),
            "row.status == overdue"
        );
    }
}
