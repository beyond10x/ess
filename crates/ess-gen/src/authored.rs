//! Content an adopter wrote, read into the document IR.
//!
//! Two things a specification's site needs and a specification does not contain: the `README.md`
//! beside the model, which says why it exists, and a page another tool rendered — a plan board, a
//! runbook — that belongs beside the boundary it is about.
//!
//! # This is not the round trip the IR exists to remove
//!
//! [`crate::markdown`] renders the document *out*; this reads foreign markdown *in*. The smell the
//! document IR was introduced to kill was a generator writing markdown and a second generator
//! parsing it back — a structure flattened and then reconstructed inside one program. Nothing here
//! reads bytes this crate produced. An adopter's `README.md` has never been anything but markdown,
//! so parsing it is how it enters, not a value being recovered.
//!
//! # What it does not accept
//!
//! Raw HTML is dropped, not passed through. A generated site that embedded whatever an adopter's
//! markdown happened to contain would inherit that markdown's scripts and its styling, and the two
//! renderers of this IR could then disagree about a page. Footnotes and definition lists are not
//! read either; they have no [`Block`] and inventing one for a page nobody has written yet is a
//! shape this crate would then have to keep.

use pulldown_cmark::{Event, HeadingLevel, Options, Parser, Tag, TagEnd};

use crate::document::{Block, Inline, Target};

/// Adopter-written markdown as blocks of the document IR.
///
/// Total: anything it cannot represent is dropped rather than refused. A `README.md` is prose
/// somebody wrote for a person, and failing a documentation build over a construct in it would
/// make the site hostage to the file least likely to be reviewed.
pub fn blocks(markdown: &str) -> Vec<Block> {
    let mut options = Options::empty();
    options.insert(Options::ENABLE_TABLES);
    options.insert(Options::ENABLE_STRIKETHROUGH);

    Reader::default().read(Parser::new_ext(markdown, options))
}

/// Adopter-written markdown split into the title it opens with and the rest.
///
/// A `README.md` almost always starts with an `# H1`, and a page that kept it would carry two
/// titles — its own and that one. So the leading heading becomes the page's title and the blocks
/// under it become its body; markdown that opens with anything else keeps `fallback`.
pub fn titled(markdown: &str, fallback: &str) -> (Vec<Inline>, Vec<Block>) {
    let mut blocks = blocks(markdown);
    if let Some(Block::Section {
        level: 2,
        title,
        blocks: nested,
        ..
    }) = blocks.first()
    {
        if nested.is_empty() && starts_with_heading(markdown) {
            let title = title.clone();
            blocks.remove(0);
            return (title, blocks);
        }
    }
    (vec![Inline::text(fallback)], blocks)
}

/// Whether the first non-blank line is a level-one heading.
///
/// Read from the source rather than from the parsed blocks, because [`depth`] flattens `#` and
/// `##` onto one level and only the first of the two is a page title.
fn starts_with_heading(markdown: &str) -> bool {
    markdown
        .lines()
        .find(|line| !line.trim().is_empty())
        .is_some_and(|line| line.starts_with("# "))
}

/// The walk from `CommonMark` events to blocks.
///
/// A stack, because a list item holds blocks and a table cell holds inlines, and the parser hands
/// both out as flat begin/end pairs.
#[derive(Default)]
struct Reader {
    blocks: Vec<Block>,
    /// Blocks belonging to a container that has not closed yet.
    open: Vec<Vec<Block>>,
    /// The inline run currently being filled.
    inlines: Vec<Inline>,
    /// Inline runs belonging to an emphasis or a link that has not closed yet.
    nested: Vec<Vec<Inline>>,
    /// Where a link being read points.
    links: Vec<String>,
    /// Table state: the header cells, then each row.
    columns: Vec<Vec<Inline>>,
    rows: Vec<Vec<Vec<Inline>>>,
    row: Vec<Vec<Inline>>,
    in_head: bool,
    /// The language of the fence being read, and its accumulated text.
    fence: Option<(Option<String>, String)>,
    /// The heading being read, and how deep.
    heading: Option<u8>,
    /// Whether the list being read is numbered.
    ordered: Vec<bool>,
}

impl Reader {
    fn read<'a>(mut self, events: impl Iterator<Item = Event<'a>>) -> Vec<Block> {
        for event in events {
            self.event(event);
        }
        self.blocks
    }

    fn event(&mut self, event: Event<'_>) {
        match event {
            Event::Start(tag) => self.start(tag),
            Event::End(tag) => self.end(tag),
            Event::Text(text) => match &mut self.fence {
                Some((_, body)) => body.push_str(&text),
                None => self.inlines.push(Inline::text(text.to_string())),
            },
            Event::Code(text) => self.inlines.push(Inline::code(text.to_string())),
            // A hard break inside a paragraph is a space: the IR has no line break, and a page that
            // renders one as a paragraph boundary says something the author did not.
            Event::SoftBreak | Event::HardBreak => self.inlines.push(Inline::text(" ")),
            Event::Rule => self.push(Block::Rule),
            // Raw HTML, footnotes and task markers are dropped. See the module documentation.
            _ => {}
        }
    }

    fn start(&mut self, tag: Tag<'_>) {
        match tag {
            Tag::Paragraph | Tag::Item | Tag::TableCell => self.inlines.clear(),
            Tag::Heading { level, .. } => {
                self.inlines.clear();
                self.heading = Some(depth(level));
            }
            Tag::BlockQuote(_) | Tag::List(_) => {
                if let Tag::List(first) = &tag {
                    self.ordered.push(first.is_some());
                }
                self.open.push(std::mem::take(&mut self.blocks));
            }
            Tag::CodeBlock(kind) => {
                let language = match kind {
                    pulldown_cmark::CodeBlockKind::Fenced(name) if !name.is_empty() => {
                        Some(name.to_string())
                    }
                    _ => None,
                };
                self.fence = Some((language, String::new()));
            }
            Tag::Emphasis | Tag::Strong | Tag::Link { .. } => {
                if let Tag::Link { dest_url, .. } = &tag {
                    self.links.push(dest_url.to_string());
                }
                self.nested.push(std::mem::take(&mut self.inlines));
            }
            Tag::Table(_) => {
                self.columns.clear();
                self.rows.clear();
            }
            Tag::TableHead => self.in_head = true,
            Tag::TableRow => self.row.clear(),
            _ => {}
        }
    }

    fn end(&mut self, tag: TagEnd) {
        match tag {
            TagEnd::Heading(_) => {
                let Some(level) = self.heading.take() else {
                    return;
                };
                let title = std::mem::take(&mut self.inlines);
                self.push(Block::Section {
                    level,
                    anchor: anchor(&title),
                    title,
                    about: None,
                    // Flat: the parser gives no nesting and inferring it from levels would be this
                    // module deciding that a `###` after a `##` is inside it, which a document is
                    // free to disagree with.
                    blocks: Vec::new(),
                });
            }
            // A list item's text and a paragraph's are the same block; the parser just wraps one
            // of them in an item that has already opened its container.
            TagEnd::Paragraph | TagEnd::Item => {
                let text = std::mem::take(&mut self.inlines);
                if !text.is_empty() {
                    self.push(Block::Prose { text });
                }
            }
            TagEnd::List(_) => {
                let items = std::mem::take(&mut self.blocks);
                self.blocks = self.open.pop().unwrap_or_default();
                let ordered = self.ordered.pop().unwrap_or_default();
                self.push(Block::List {
                    ordered,
                    items: items.into_iter().map(|block| vec![block]).collect(),
                });
            }
            TagEnd::BlockQuote(_) => {
                let quoted = std::mem::take(&mut self.blocks);
                self.blocks = self.open.pop().unwrap_or_default();
                self.push(Block::Quote { blocks: quoted });
            }
            TagEnd::CodeBlock => {
                if let Some((language, text)) = self.fence.take() {
                    self.push(Block::Code {
                        language,
                        text: text.trim_end_matches('\n').to_owned(),
                    });
                }
            }
            TagEnd::Emphasis | TagEnd::Strong => {
                let text = std::mem::take(&mut self.inlines);
                self.inlines = self.nested.pop().unwrap_or_default();
                self.inlines.push(if matches!(tag, TagEnd::Emphasis) {
                    Inline::Emphasis { text }
                } else {
                    Inline::Strong { text }
                });
            }
            TagEnd::Link => {
                let text = std::mem::take(&mut self.inlines);
                self.inlines = self.nested.pop().unwrap_or_default();
                let url = self.links.pop().unwrap_or_default();
                // Every link an adopter wrote is external as far as this document is concerned.
                // Resolving one into a `Target::Page` would mean guessing that their `./guide.md`
                // is a page of *this* document, and a wrong guess is a link that opens the wrong
                // page — which cannot be told from a right one by looking at it.
                self.inlines.push(Inline::Link {
                    to: Target::External { url },
                    text,
                });
            }
            TagEnd::TableHead => {
                self.in_head = false;
                self.columns = std::mem::take(&mut self.row);
            }
            TagEnd::TableRow => {
                let row = std::mem::take(&mut self.row);
                if !self.in_head {
                    self.rows.push(row);
                }
            }
            TagEnd::TableCell => {
                let cell = std::mem::take(&mut self.inlines);
                self.row.push(cell);
            }
            TagEnd::Table => {
                let columns = std::mem::take(&mut self.columns);
                let rows = std::mem::take(&mut self.rows);
                self.push(Block::Table { columns, rows });
            }
            _ => {}
        }
    }

    fn push(&mut self, block: Block) {
        self.blocks.push(block);
    }
}

/// A heading's depth, clamped to what a page under a title can hold.
fn depth(level: HeadingLevel) -> u8 {
    match level {
        HeadingLevel::H1 | HeadingLevel::H2 => 2,
        HeadingLevel::H3 => 3,
        HeadingLevel::H4 => 4,
        HeadingLevel::H5 => 5,
        HeadingLevel::H6 => 6,
    }
}

/// A fragment for a heading, from its text.
fn anchor(title: &[Inline]) -> String {
    let mut out = String::new();
    for inline in title {
        match inline {
            Inline::Text { text } | Inline::Code { text } => out.push_str(text),
            Inline::Emphasis { text } | Inline::Strong { text } | Inline::Link { text, .. } => {
                out.push_str(&anchor(text));
            }
        }
    }
    out.chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() {
                character.to_ascii_lowercase()
            } else {
                '-'
            }
        })
        .collect::<String>()
        .trim_matches('-')
        .to_owned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_leading_title_becomes_the_page_title_and_not_a_second_heading() {
        let (title, body) = titled("# The ACD boundary\n\nWhy it exists.\n", "fallback");
        assert_eq!(title, vec![Inline::text("The ACD boundary")]);
        assert!(matches!(body.as_slice(), [Block::Prose { .. }]), "{body:?}");

        // Markdown that opens with prose keeps the caller's title rather than inventing one.
        let (title, body) = titled("Just prose.\n", "fallback");
        assert_eq!(title, vec![Inline::text("fallback")]);
        assert_eq!(body.len(), 1);
    }

    #[test]
    fn a_paragraph_keeps_its_inline_structure() {
        let blocks = blocks("Plain, `code`, *emphasis* and **strong**.\n");
        let [Block::Prose { text }] = blocks.as_slice() else {
            panic!("expected one paragraph, got {blocks:?}");
        };
        assert!(matches!(text[1], Inline::Code { .. }));
        assert!(matches!(text[3], Inline::Emphasis { .. }));
        assert!(matches!(text[5], Inline::Strong { .. }));
    }

    #[test]
    fn a_heading_becomes_a_section_with_an_anchor() {
        let blocks = blocks("## What it covers\n");
        let [Block::Section { level, anchor, .. }] = blocks.as_slice() else {
            panic!("{blocks:?}");
        };
        assert_eq!(*level, 2);
        assert_eq!(anchor, "what-it-covers");
    }

    #[test]
    fn a_top_level_heading_is_demoted_because_the_page_title_is_the_first() {
        // Two `<h1>` elements on one page is a document with two titles, and a screen reader
        // announces both.
        let blocks = blocks("# A title\n");
        let [Block::Section { level, .. }] = blocks.as_slice() else {
            panic!("{blocks:?}");
        };
        assert_eq!(*level, 2);
    }

    #[test]
    fn a_table_keeps_its_header_apart_from_its_rows() {
        let blocks = blocks("| a | b |\n|---|---|\n| 1 | 2 |\n| 3 | 4 |\n");
        let [Block::Table { columns, rows }] = blocks.as_slice() else {
            panic!("{blocks:?}");
        };
        assert_eq!(columns.len(), 2);
        assert_eq!(rows.len(), 2, "the header is not a row");
    }

    #[test]
    fn a_fence_keeps_its_language_and_loses_its_trailing_newline() {
        let blocks = blocks("```console\n$ ess validate\n```\n");
        let [Block::Code { language, text }] = blocks.as_slice() else {
            panic!("{blocks:?}");
        };
        assert_eq!(language.as_deref(), Some("console"));
        assert_eq!(text, "$ ess validate");
    }

    #[test]
    fn a_link_an_adopter_wrote_stays_theirs() {
        let blocks = blocks("See [the guide](./guide.md).\n");
        let [Block::Prose { text }] = blocks.as_slice() else {
            panic!("{blocks:?}");
        };
        let Some(Inline::Link { to, .. }) = text.get(1) else {
            panic!("{text:?}");
        };
        assert_eq!(
            *to,
            Target::External {
                url: "./guide.md".to_owned()
            },
            "resolving it into a page of this document would be a guess, and a wrong link cannot \
             be told from a right one by looking at it"
        );
    }

    #[test]
    fn raw_html_is_dropped_rather_than_passed_through() {
        let blocks = blocks("<script>alert(1)</script>\n\nAfter.\n");
        let rendered = format!("{blocks:?}");
        assert!(!rendered.contains("script"), "{rendered}");
        assert!(rendered.contains("After."), "{rendered}");
    }

    #[test]
    fn a_list_becomes_items_and_a_quote_becomes_a_quote() {
        let listed = blocks("- one\n- two\n");
        let [Block::List { ordered, items }] = listed.as_slice() else {
            panic!("{listed:?}");
        };
        assert!(!ordered);
        assert_eq!(items.len(), 2);

        let quoted = blocks("> quoted\n");
        assert!(
            matches!(quoted.as_slice(), [Block::Quote { .. }]),
            "{quoted:?}"
        );
    }
}
