//! The document as markdown a repository reads.
//!
//! One of the renderers of [`crate::document`], and the one whose output is a contract: these are
//! the bytes `ess generate --kind docs` has always written, and the port that introduced the
//! document IR is proved by them not moving.
//!
//! # Blocks are flattened, not nested
//!
//! A [`Block::Section`] holds its children, because that is the
//! shape a consumer walks. Markdown has no nesting — a `##` is a line, and what follows it belongs
//! to it by position — so this renderer emits the heading and then its children into the same
//! stream. Each chunk ends with exactly one newline and they are joined by one more, which is where
//! every blank line in the output comes from.

use std::fmt::Write as _;

use crate::artifact::Artifact;
use crate::document::{Block, Document, Inline, Page, PageId, Target};
use crate::provenance::Provenance;

/// Every page of the document, as markdown.
pub fn render(document: &Document) -> Vec<Artifact> {
    document.pages.iter().map(page).collect()
}

/// One page.
pub fn page(page: &Page) -> Artifact {
    let provenance = &page.provenance.provenance;
    let mut contents = provenance_comment(provenance);
    let _ = writeln!(contents, "\n# {}\n", inlines(&page.title, &page.id));

    let chunks: Vec<String> = flatten(&page.blocks, &page.id);
    contents.push_str(&chunks.join("\n"));
    if !contents.ends_with('\n') {
        contents.push('\n');
    }
    contents.push_str(&provenance_footer(provenance));

    Artifact::sliced(
        format!("{}.md", page.id.as_str()),
        contents,
        page.provenance.slice.clone(),
    )
}

/// Every block as a chunk ending in exactly one newline, in reading order.
fn flatten(blocks: &[Block], here: &PageId) -> Vec<String> {
    let mut out = Vec::new();
    for block in blocks {
        match block {
            Block::Prose { text } => out.push(format!("{}\n", inlines(text, here))),
            Block::Section {
                level,
                title,
                blocks,
                ..
            } => {
                out.push(format!(
                    "{} {}\n",
                    "#".repeat(usize::from(*level)),
                    inlines(title, here)
                ));
                out.extend(flatten(blocks, here));
            }
            Block::List { ordered, items } => {
                let mut chunk = String::new();
                for (index, item) in items.iter().enumerate() {
                    // One line per item. A list item holding more than a paragraph is legal in the
                    // IR and is not written by this projection; rendering only its first block
                    // would lose the rest silently, so every block of it is written and the ones
                    // after the first are indented under it.
                    let rendered = flatten(item, here);
                    let marker = if *ordered {
                        format!("{}. ", index + 1)
                    } else {
                        "- ".to_owned()
                    };
                    for (position, part) in rendered.iter().enumerate() {
                        let body = part.trim_end_matches('\n');
                        if position == 0 {
                            let _ = writeln!(chunk, "{marker}{body}");
                        } else {
                            for line in body.lines() {
                                let _ = writeln!(chunk, "  {line}");
                            }
                        }
                    }
                }
                out.push(chunk);
            }
            Block::Table { columns, rows } => {
                let mut chunk = String::new();
                let _ = writeln!(chunk, "| {} |", cells(columns, here).join(" | "));
                let _ = writeln!(
                    chunk,
                    "|{}|",
                    columns.iter().map(|_| "---").collect::<Vec<_>>().join("|")
                );
                for row in rows {
                    let _ = writeln!(chunk, "| {} |", cells(row, here).join(" | "));
                }
                out.push(chunk);
            }
            Block::Code { language, text } => out.push(format!(
                "```{}\n{text}\n```\n",
                language.as_deref().unwrap_or_default()
            )),
            Block::Diagram { source, .. } => out.push(format!("```mermaid\n{source}\n```\n")),
            Block::Rule => out.push("---\n".to_owned()),
        }
    }
    out
}

/// One row of cells, each rendered inline.
fn cells(row: &[Vec<Inline>], here: &PageId) -> Vec<String> {
    row.iter().map(|cell| inlines(cell, here)).collect()
}

/// A run of inlines.
fn inlines(text: &[Inline], here: &PageId) -> String {
    let mut out = String::new();
    for inline in text {
        match inline {
            Inline::Text { text } => out.push_str(text),
            Inline::Code { text } => {
                let _ = write!(out, "`{text}`");
            }
            Inline::Emphasis { text } => {
                let _ = write!(out, "_{}_", inlines(text, here));
            }
            Inline::Strong { text } => {
                let _ = write!(out, "**{}**", inlines(text, here));
            }
            Inline::Link { to, text } => {
                let _ = write!(out, "[{}]({})", inlines(text, here), address(to, here));
            }
        }
    }
    out
}

/// Where a link goes, as a path this renderer can serve.
///
/// The producer named a page; the address is computed here, because this is the layer that knows
/// pages are files and that files end in `.md`.
pub fn address(target: &Target, here: &PageId) -> String {
    match target {
        Target::Page { page } => format!("{}.md", relative(here, page)),
        Target::Anchor { page, anchor } if page == here => format!("#{anchor}"),
        Target::Anchor { page, anchor } => format!("{}.md#{anchor}", relative(here, page)),
        // A construct with no page of its own is not addressable by this renderer, and an address
        // it invented would point at nothing. The text stays; only the link goes.
        Target::Construct { .. } => String::new(),
        Target::External { url } => url.clone(),
    }
}

/// The path of `to` as written on `from`'s own page.
///
/// A real relative walk rather than `../` repeated: two pages in one directory link to each other
/// by name, which is what the existing pages do and what a reader expects to see in a diff.
fn relative(from: &PageId, to: &PageId) -> String {
    let from_parts: Vec<&str> = from.as_str().split('/').collect();
    let to_parts: Vec<&str> = to.as_str().split('/').collect();
    let from_dirs = &from_parts[..from_parts.len() - 1];
    let to_dirs = &to_parts[..to_parts.len() - 1];

    let shared = from_dirs
        .iter()
        .zip(to_dirs)
        .take_while(|(left, right)| left == right)
        .count();

    let mut path = String::new();
    for _ in shared..from_dirs.len() {
        path.push_str("../");
    }
    for directory in &to_dirs[shared..] {
        let _ = write!(path, "{directory}/");
    }
    path.push_str(to_parts[to_parts.len() - 1]);
    path
}

/// The provenance block every page opens with, as one HTML comment.
///
/// Not a per-line prefix: a prefix cannot close an HTML comment, so four lines each opening one and
/// none closing it leaves a renderer swallowing the rest of the page.
fn provenance_comment(provenance: &Provenance) -> String {
    let mut out = String::from("<!--\n");
    for line in provenance.lines() {
        let _ = writeln!(out, "{line}");
    }
    out.push_str("-->\n");
    out
}

/// The same facts, visible.
///
/// Duplicated on purpose: the comment above is for a tool and a diff, and it is invisible to
/// exactly the person about to edit a generated file by hand and lose the work.
fn provenance_footer(provenance: &Provenance) -> String {
    format!(
        "\n---\n\nGenerated from {} {} · model digest `{}` · contract digest `{}`. Do not \
         edit this file; change the specification and regenerate it with `ess generate`.\n",
        provenance.system,
        provenance.specification_version,
        provenance.source_digest,
        provenance.contract_digest,
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::document::{Blocks, DiagramKind};

    fn here() -> PageId {
        PageId::from("domains/billing-invoice")
    }

    #[test]
    fn a_link_is_addressed_from_the_page_it_is_written_on() {
        let index = Target::Page {
            page: PageId::from("index"),
        };
        assert_eq!(address(&index, &here()), "../index.md");
        assert_eq!(address(&index, &PageId::from("crossings")), "index.md");

        // Two pages in one directory link by name, which is what the existing pages do.
        let sibling = Target::Anchor {
            page: PageId::from("domains/billing-email"),
            anchor: "sendemail".to_owned(),
        };
        assert_eq!(address(&sibling, &here()), "billing-email.md#sendemail");
        assert_eq!(
            address(&sibling, &PageId::from("interactions")),
            "domains/billing-email.md#sendemail"
        );
    }

    #[test]
    fn a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip() {
        let own = Target::Anchor {
            page: here(),
            anchor: "invoice".to_owned(),
        };
        assert_eq!(address(&own, &here()), "#invoice");
    }

    #[test]
    fn a_table_is_written_with_the_separator_a_reader_expects() {
        let mut blocks = Blocks::new();
        blocks.push(Block::Table {
            columns: vec![vec![Inline::text("field")], vec![Inline::text("type")]],
            rows: vec![vec![
                vec![Inline::code("call_id")],
                vec![Inline::code("CallId")],
            ]],
        });
        assert_eq!(
            flatten(&blocks.finish(), &here()).join("\n"),
            "| field | type |\n|---|---|\n| `call_id` | `CallId` |\n"
        );
    }

    #[test]
    fn a_diagram_is_a_fenced_mermaid_block() {
        let mut blocks = Blocks::new();
        blocks.push(Block::Diagram {
            kind: DiagramKind::Lifecycle,
            source: "stateDiagram-v2\n    [*] --> Draft".to_owned(),
        });
        assert_eq!(
            flatten(&blocks.finish(), &here()).join("\n"),
            "```mermaid\nstateDiagram-v2\n    [*] --> Draft\n```\n"
        );
    }

    #[test]
    fn a_section_flattens_into_the_stream_and_its_children_follow_it() {
        let mut inner = Blocks::new();
        inner.sentence("Under it.");
        let mut blocks = Blocks::new();
        blocks.push(Block::Section {
            level: 2,
            title: vec![Inline::text("Views")],
            anchor: "views".to_owned(),
            about: None,
            blocks: inner.finish(),
        });
        assert_eq!(
            flatten(&blocks.finish(), &here()).join("\n"),
            "## Views\n\nUnder it.\n"
        );
    }
}
