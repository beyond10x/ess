//! The document a specification obliges, before anybody decides what it looks like.
//!
//! Three layers, and this is the middle one:
//!
//! ```text
//! EssIr          the model — what the system is
//!    |  docs.rs
//! ess-docs/1     the document — what is said about it, in what order, about which construct
//!    |
//!    +--> markdown   a repository reads it
//!    +--> site       a browser reads it
//!    +--> whatever   a design system, a wiki, a slide deck
//! ```
//!
//! # Why it exists
//!
//! The middle layer was there all along and was spelled as text. `docs.rs` walked the model and
//! wrote markdown; anything else that wanted the same pages had to read that markdown back. The
//! site projection did exactly that inside one process — scanning for `# ` to recover a title it
//! had just written, doing string surgery to drop a heading, re-extracting two digests from an
//! HTML comment stamped three functions earlier, and rewriting `.md` links to `.html` by guessing
//! at strings. Each of those is a value that was flattened and then reconstructed.
//!
//! # It is not a markdown AST with extra steps
//!
//! Three things make it this system's rather than a worse Pandoc:
//!
//! * [`Block::Section`] and [`Page`] carry [`about`](Page::about) — the construct being documented
//!   — so a consumer can ask *which entity is this section for* rather than parse a heading.
//! * [`Target`] is typed. A link to another page is a page id and an anchor, never a relative path,
//!   so the renderer that knows where it put its pages computes the path and the producer never
//!   does. This is the whole of what `rewrite()` used to be.
//! * [`Block::Diagram`] says what kind of diagram it is, so a renderer can present one as a fenced block,
//!   as a `<pre>` a script draws into, or one day as an inlined SVG.
//!
//! # Prose lives here, presentation does not
//!
//! The sentences are in the document, as [`Inline`] sequences. The alternative — a document of pure
//! facts, with each renderer composing its own wording — reads well until the second renderer, at
//! which point there are two copies of thirty sentence-writing functions and they disagree by the
//! third release. A renderer decides typography, layout and colour. It never decides wording.
//!
//! # What a diagram carries, and what it deliberately does not
//!
//! Mermaid source and a [`DiagramKind`], not a graph. `ess-gen::graph::SystemGraph` is the typed
//! model of the system diagram and stays where it is; carrying a second structural copy through
//! this layer would be a model nothing consumes and everything has to keep in step. The narrowing
//! is deliberate and is the one place this IR is thinner than the layer above it.

use ess_compiler::refs::EssSemanticRef;

use crate::provenance::SlicedProvenance;

/// The format this document is written in.
pub const DOCUMENT_FORMAT: &str = "ess-docs/1";

/// Every page a specification's documentation consists of.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Document {
    /// What this document is written in — always [`DOCUMENT_FORMAT`].
    pub format: String,
    /// The system documented.
    pub system: String,
    /// The version of that system's specification.
    pub version: String,
    /// The pages, in the order a reader meets them.
    pub pages: Vec<Page>,
}

impl Document {
    /// A document of these pages.
    pub fn new(system: impl Into<String>, version: impl Into<String>, pages: Vec<Page>) -> Self {
        Self {
            format: DOCUMENT_FORMAT.to_owned(),
            system: system.into(),
            version: version.into(),
            pages,
        }
    }

    /// The page with this id.
    pub fn page(&self, id: &PageId) -> Option<&Page> {
        self.pages.iter().find(|page| &page.id == id)
    }
}

/// A page's identity, which is also where a renderer files it.
///
/// Slash-separated and without an extension — `domains/acd-routing`, `index` — because the
/// extension is the renderer's decision and the identity is not. It is the only thing a
/// [`Target::Page`] names, which is what stops a link being a path somebody computed.
#[derive(
    Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, serde::Serialize, serde::Deserialize,
)]
#[serde(transparent)]
pub struct PageId(pub String);

impl PageId {
    /// The id as written.
    pub fn as_str(&self) -> &str {
        &self.0
    }

    /// How many directories deep it is, for a renderer computing a relative path.
    pub fn depth(&self) -> usize {
        self.0.matches('/').count()
    }
}

impl From<&str> for PageId {
    fn from(value: &str) -> Self {
        Self(value.to_owned())
    }
}

/// One page.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct Page {
    /// Where it goes and what links to it.
    pub id: PageId,
    /// Its title, as a heading a renderer places wherever it places headings.
    ///
    /// Separate from [`Self::blocks`] on purpose: a renderer that has to find the title inside the
    /// body is a renderer that finds it by convention, and a convention is what breaks when a page
    /// starts with a paragraph.
    pub title: Vec<Inline>,
    /// The construct this page documents, where it documents one.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub about: Option<EssSemanticRef>,
    /// What it derives from, and the slice that digest is of.
    ///
    /// Carried as a value rather than stamped into the prose, so a renderer decides whether it is a
    /// comment, a footer, both, or a field in something else entirely. The slice travels with it
    /// because `crate::artifact::run` checks the stamped digest against it for every artifact, and
    /// a renderer that dropped it would be shipping a claim nothing verified.
    pub provenance: SlicedProvenance,
    /// The body.
    pub blocks: Vec<Block>,
}

/// One piece of a page.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "block", rename_all = "snake_case")]
pub enum Block {
    /// A paragraph.
    Prose {
        /// What it says.
        text: Vec<Inline>,
    },
    /// A headed run of blocks.
    ///
    /// Nested rather than flat with a level number *and* flat with a level number: `level` is what
    /// a renderer writes, and the nesting is what a consumer walks. Keeping both would let them
    /// disagree, so `level` is derived on the way in and never edited afterwards.
    Section {
        /// How deep, counting the page title as one.
        level: u8,
        /// Its heading.
        title: Vec<Inline>,
        /// The fragment a [`Target::Anchor`] uses to reach it.
        anchor: String,
        /// The construct it documents, where it documents one.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        about: Option<EssSemanticRef>,
        /// What is under it.
        blocks: Vec<Block>,
    },
    /// A list.
    List {
        /// Whether the items are numbered.
        ordered: bool,
        /// The items, each of which may hold more than a paragraph.
        items: Vec<Vec<Block>>,
    },
    /// A table.
    Table {
        /// The header cells.
        columns: Vec<Vec<Inline>>,
        /// The body rows, each the same width as `columns`.
        rows: Vec<Vec<Vec<Inline>>>,
    },
    /// Somebody else's words, quoted.
    ///
    /// A conversion's reason is written by the person who allowed the crossing. A page that set it
    /// in its own voice would be claiming it, and the whole point of requiring a reason is that a
    /// reader can tell who is answerable for it. Blocks rather than inlines, because a reason is
    /// allowed to run to more than one paragraph.
    Quote {
        /// What is quoted.
        blocks: Vec<Block>,
    },
    /// A verbatim block that is not a diagram.
    Code {
        /// What it is written in, for a renderer that highlights.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        language: Option<String>,
        /// The text, without a trailing newline.
        text: String,
    },
    /// A diagram.
    Diagram {
        /// What kind it is.
        kind: DiagramKind,
        /// Its Mermaid source, without a trailing newline.
        source: String,
    },
    /// A thematic break.
    Rule,
}

/// What a diagram is of.
///
/// A renderer may present all three the same way. It is here so that one which does not — an SVG
/// projection that lays out a state machine differently from a flowchart — does not have to read
/// the Mermaid to find out which it has.
#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DiagramKind {
    /// The whole system: actors, commands, events and the bindings between them.
    System,
    /// One entity's lifecycle.
    Lifecycle,
    /// One binding, from its event to the outcomes of the command it invokes.
    BindingFlow,
    /// One interaction between two bounded contexts.
    Interaction,
}

/// A run of text.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "inline", rename_all = "snake_case")]
pub enum Inline {
    /// Plain text.
    Text {
        /// The words.
        text: String,
    },
    /// A name, a path, a value — anything set in a monospaced face.
    Code {
        /// The text, without its delimiters.
        text: String,
    },
    /// Emphasised text.
    Emphasis {
        /// What is emphasised.
        text: Vec<Inline>,
    },
    /// Strongly emphasised text.
    Strong {
        /// What is emphasised.
        text: Vec<Inline>,
    },
    /// A link.
    Link {
        /// Where it goes.
        to: Target,
        /// What it reads as.
        text: Vec<Inline>,
    },
}

impl Inline {
    /// Plain text.
    pub fn text(value: impl Into<String>) -> Self {
        Self::Text { text: value.into() }
    }

    /// Monospaced text.
    pub fn code(value: impl Into<String>) -> Self {
        Self::Code { text: value.into() }
    }

    /// A link reading as one monospaced name.
    pub fn code_link(to: Target, value: impl Into<String>) -> Self {
        Self::Link {
            to,
            text: vec![Self::code(value)],
        }
    }
}

/// Where a link goes.
///
/// Typed, and this is the point of the whole layer. A producer names *what* it is linking to; the
/// renderer, which is the only thing that knows where it put its pages and what extension they
/// have, works out the address. `docs.rs` used to compute `../domains/acd-routing.md#dispatch`, and
/// the site projection had to unpick it.
#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "target", rename_all = "snake_case")]
pub enum Target {
    /// Another page of this document.
    Page {
        /// Which one.
        page: PageId,
    },
    /// A section of another page, or of this one.
    Anchor {
        /// Which page.
        page: PageId,
        /// Which section of it.
        anchor: String,
    },
    /// The construct's own section, wherever the renderer put it.
    ///
    /// Distinct from [`Anchor`](Self::Anchor): a producer that knows only *which command* it means
    /// says so, and a renderer that has filed commands somewhere unusual resolves it its own way.
    Construct {
        /// Which construct.
        #[serde(rename = "ref")]
        reference: EssSemanticRef,
    },
    /// Somewhere outside this document.
    External {
        /// The address, as written.
        url: String,
    },
}

/// A builder for a run of blocks, so a producer reads as a sequence of statements.
///
/// `docs.rs` was written against `body: &mut String` and `writeln!`. This is the same shape with
/// the same ergonomics, which is what keeps the port a change of constructor rather than a
/// restructuring of two thousand lines.
#[derive(Debug, Default)]
pub struct Blocks(Vec<Block>);

impl Blocks {
    /// An empty run.
    pub fn new() -> Self {
        Self::default()
    }

    /// Adds a paragraph.
    pub fn prose(&mut self, text: Vec<Inline>) -> &mut Self {
        self.0.push(Block::Prose { text });
        self
    }

    /// Adds a paragraph of one plain sentence.
    pub fn sentence(&mut self, text: impl Into<String>) -> &mut Self {
        self.prose(vec![Inline::text(text)])
    }

    /// Adds any block.
    pub fn push(&mut self, block: Block) -> &mut Self {
        self.0.push(block);
        self
    }

    /// Adds every block of another run.
    pub fn extend(&mut self, blocks: Vec<Block>) -> &mut Self {
        self.0.extend(blocks);
        self
    }

    /// Whether nothing has been added.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// The blocks.
    pub fn finish(self) -> Vec<Block> {
        self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn provenance() -> SlicedProvenance {
        SlicedProvenance {
            provenance: crate::provenance::Provenance {
                system: "billing".to_owned(),
                specification_version: "v3".to_owned(),
                source_digest: "0".repeat(64),
                contract_digest: "f".repeat(64),
            },
            slice: crate::provenance::ModelSlice::WholeModel,
        }
    }

    #[test]
    fn a_page_id_says_how_deep_it_is_so_a_renderer_can_reach_the_root() {
        assert_eq!(PageId::from("index").depth(), 0);
        assert_eq!(PageId::from("domains/acd-routing").depth(), 1);
    }

    #[test]
    fn a_document_round_trips_through_its_own_format() {
        let document = Document::new(
            "billing",
            "v3",
            vec![Page {
                id: PageId::from("domains/billing-invoice"),
                title: vec![Inline::text("Invoicing")],
                about: None,
                provenance: provenance(),
                blocks: {
                    let mut blocks = Blocks::new();
                    blocks.prose(vec![
                        Inline::text("It reads "),
                        Inline::code_link(
                            Target::Anchor {
                                page: PageId::from("domains/billing-invoice"),
                                anchor: "invoice".to_owned(),
                            },
                            "Invoice",
                        ),
                        Inline::text("."),
                    ]);
                    blocks.push(Block::Diagram {
                        kind: DiagramKind::Lifecycle,
                        source: "stateDiagram-v2\n    [*] --> Draft".to_owned(),
                    });
                    blocks.finish()
                },
            }],
        );

        let written = serde_json::to_string(&document).expect("writes");
        let read: Document = serde_json::from_str(&written).expect("reads");
        assert_eq!(read, document);
        assert_eq!(read.format, "ess-docs/1");
        assert!(read
            .page(&PageId::from("domains/billing-invoice"))
            .is_some());
    }

    #[test]
    fn a_link_names_what_it_points_at_and_never_a_path() {
        // The property the whole layer exists for. A producer cannot express `../foo.md` here, so
        // it cannot leave a renderer to unpick one.
        let written = serde_json::to_string(&Target::Page {
            page: PageId::from("crossings"),
        })
        .expect("writes");
        assert_eq!(written, r#"{"target":"page","page":"crossings"}"#);
        assert!(!written.contains(".md"), "{written}");
    }
}
