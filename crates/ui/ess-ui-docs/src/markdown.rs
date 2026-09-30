//! The reference as a Markdown page of the documentation site.
//!
//! The site parses pages as MDX, so text outside code never carries a bare `{`, `}`, `<` or `>`,
//! and the page carries no HTML comment. Anchors are the ids the site derives from heading text,
//! the same ids the HTML page uses.

use std::fmt::Write as _;

use crate::doc::{Block, Inline};
use crate::yaml;

pub(crate) fn render(blocks: &[Block], format: &str) -> String {
    let mut out = String::new();
    let _ = write!(
        out,
        "---\ntitle: {}\nsidebar_label: {}\ndescription: {}\n---\n",
        yaml::quoted(&format!("{format} reference")),
        yaml::quoted(format),
        yaml::quoted(&format!(
            "Every construct of {} {format} document, generated from the schema.",
            crate::doc::article(format)
        )),
    );
    for block in blocks {
        match block {
            Block::Open { .. } | Block::Close => {}
            Block::Heading { level, text } => {
                let _ = write!(
                    out,
                    "\n{} {}\n",
                    "#".repeat(usize::from(*level)),
                    heading(text)
                );
            }
            Block::Label(text) => {
                let _ = write!(out, "\n**{}**\n", escape(text, false));
            }
            Block::Para(inlines) => {
                out.push('\n');
                out.push_str(&inline_text(inlines, false));
                out.push('\n');
            }
            Block::Table { head, rows, .. } => {
                out.push('\n');
                let _ = writeln!(out, "| {} |", head.join(" | "));
                let _ = writeln!(out, "|{}", "---|".repeat(head.len()));
                for row in rows {
                    let cells: Vec<String> = row
                        .iter()
                        .map(|cell| {
                            let text = inline_text(cell, true);
                            if text.is_empty() {
                                " ".to_owned()
                            } else {
                                text
                            }
                        })
                        .collect();
                    let _ = writeln!(out, "| {} |", cells.join(" | "));
                }
            }
            Block::Yaml(value) => {
                out.push_str("\n```yaml\n");
                for line in yaml::block(value) {
                    for tok in line {
                        out.push_str(&tok.text);
                    }
                    out.push('\n');
                }
                out.push_str("```\n");
            }
        }
    }
    out
}

fn inline_text(inlines: &[Inline], table: bool) -> String {
    let mut out = String::new();
    for inline in inlines {
        match inline {
            Inline::Text(text) => out.push_str(&escape(text, table)),
            Inline::Code(code) => out.push_str(&code_span(code, table)),
            Inline::Link { text, anchor } => {
                let _ = write!(out, "[{}](#{anchor})", escape(text, table));
            }
            Inline::Prose(prose) => {
                let prose = prose.trim().replace('\n', " ");
                let parts: Vec<&str> = prose.split('`').collect();
                let balanced = parts.len() % 2 == 1;
                for (i, part) in parts.iter().enumerate() {
                    if i % 2 == 1 && (balanced || i + 1 < parts.len()) {
                        out.push_str(&code_span(part, table));
                    } else {
                        if i % 2 == 1 {
                            out.push_str("\\`");
                        }
                        out.push_str(&escape(part, table));
                    }
                }
            }
        }
    }
    out
}

fn code_span(code: &str, table: bool) -> String {
    let code = code.replace('\n', " ");
    let code = if table {
        code.replace('|', "\\|")
    } else {
        code
    };
    if code.contains('`') {
        format!("`` {code} ``")
    } else {
        format!("`{code}`")
    }
}

/// A heading's text, shown literally, so the id the site derives from it is the
/// [`crate::doc::slug`] of the text every link uses.
///
/// Every character Markdown or MDX reads as syntax is escaped, except an `_` between two letters
/// or digits: `CommonMark` never reads that one as emphasis, and `graph_editor` stays as written.
fn heading(text: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let mut out = String::with_capacity(text.len());
    for (i, &c) in chars.iter().enumerate() {
        let intraword = |at: Option<&char>| at.is_some_and(|c| c.is_alphanumeric());
        let escaped = match c {
            '_' => !(i > 0 && intraword(chars.get(i - 1)) && intraword(chars.get(i + 1))),
            '\\' | '`' | '*' | '[' | ']' | '{' | '}' | '<' | '>' | '~' | '&' | '#' => true,
            _ => false,
        };
        if escaped {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

/// Text with every character Markdown or MDX would read as syntax escaped.
fn escape(text: &str, table: bool) -> String {
    let mut out = String::with_capacity(text.len());
    for c in text.chars() {
        match c {
            '\\' | '`' | '*' | '_' | '[' | ']' | '{' | '}' | '<' | '>' | '~' | '&' | '#' => {
                out.push('\\');
                out.push(c);
            }
            '|' if table => out.push_str("\\|"),
            '\n' => out.push(' '),
            c => out.push(c),
        }
    }
    out
}
