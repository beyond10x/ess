//! The document as a site a browser reads.
//!
//! One of the renderers of [`crate::document`], beside [`crate::markdown`]. It emits a complete
//! static website — a page per [`Page`], a stylesheet, and the diagram renderer — so publishing a
//! specification is one command against one binary. There is no Node, no npm and no bundler
//! anywhere in the path, because a specification tool that needs a JavaScript toolchain to show its
//! own output is a specification tool most adopters will not run.
//!
//! # It renders the document, and never re-reads its own text
//!
//! Everything here reads [`Document`] values. Nothing parses markdown, nothing scans for `# ` to
//! recover a title, and nothing rewrites a link somebody else already spelled as a path. That is
//! the whole reason the document layer exists, and it is the one property this module must not
//! quietly give up.
//!
//! # One seam for presentation, and only one
//!
//! [`Style`] is the seam. It has one variant today and its only job is to answer with a
//! stylesheet, so a second look is an added variant and an added `.css` file rather than a rewrite.
//! Everything else in this module — the page shell, the navigation, the block and inline
//! rendering, the diagram runner — is style-independent: it emits the same elements and the same
//! class names whatever [`Style`] is selected, and a style expresses itself entirely in CSS. A
//! second style that needed a different element tree would be evidence this seam is in the wrong
//! place, and the fix would be to move the seam rather than to branch on the style here.
//!
//! # Diagrams are drawn in the browser, from a bundle this crate ships
//!
//! A [`Block::Diagram`] becomes a `<pre class="mermaid">` holding its own source, which is legible
//! on its own before anything runs and is what a reader sees if the script is blocked. The bundle
//! beside it is vendored rather than fetched from a CDN: a site that phones out to draw its
//! diagrams is a site that stops working offline, in an air-gapped review, and on the day the CDN
//! changes a URL.
//!
//! The bundle is loaded by a `<script>` element the module script appends, not by `import`. This is
//! not a style preference. The build publishes itself with a top-level `var` and a final
//! `globalThis` assignment; a top-level `var` in a module is scoped to the module, so under
//! `import` that last line reads a property of `undefined` and throws before Mermaid exists.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use ess_compiler::refs::EssSemanticRef;

use crate::artifact::{Artifact, Generator};
use crate::document::{Block, Document, Inline, Page, PageId, Target};
use crate::provenance::Provenance;

/// The page every site has, and the one the masthead links to.
const INDEX: &str = "index";

/// Where the stylesheet is published, relative to the site root.
const STYLESHEET: &str = "assets/style.css";

/// Where the diagram renderer is published, relative to the site root.
const RENDERER: &str = "assets/mermaid.min.js";

/// Where the diagram renderer's licence is published, relative to the site root.
const RENDERER_LICENCE: &str = "assets/mermaid.LICENSE";

/// How a generated site looks.
///
/// One variant, deliberately. It exists so that a second look is an added variant and an added
/// stylesheet rather than a rewrite of the renderer: nothing else in this module branches on it,
/// and a style says everything it has to say in CSS against the class names
/// [`Site::page`] emits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Style {
    /// A fixed sidebar, a measure of about 74 characters, and a light and dark palette chosen by
    /// the reader's own `prefers-color-scheme`.
    #[default]
    Default,
}

impl Style {
    /// The stylesheet this style is.
    pub fn stylesheet(self) -> &'static str {
        match self {
            Self::Default => include_str!("../assets/default.css"),
        }
    }
}

/// Where each construct is documented, for resolving a [`Target::Construct`].
///
/// A producer that knows only *which command* it means says so, and this is the renderer answering
/// the question it was left. The answer comes from the document itself: a [`Page`] or a
/// [`Block::Section`] carrying [`about`](Page::about) is the place that construct is documented,
/// and there is nothing else to consult.
///
/// The first claim in document order wins. Two places claiming one construct is a producer's
/// duplication rather than a reader's choice, and picking the later one would make the link move
/// whenever a page was reordered.
#[derive(Debug, Clone, Default)]
pub struct ConstructIndex(BTreeMap<EssSemanticRef, Target>);

impl ConstructIndex {
    /// Indexes everything this document says it documents.
    pub fn of(document: &Document) -> Self {
        let mut found: BTreeMap<EssSemanticRef, Target> = BTreeMap::new();
        for page in &document.pages {
            if let Some(about) = &page.about {
                found.entry(about.clone()).or_insert_with(|| Target::Page {
                    page: page.id.clone(),
                });
            }
            index_sections(&page.blocks, &page.id, &mut found);
        }
        Self(found)
    }

    /// Where this construct is documented, if the document documents it anywhere.
    ///
    /// Never a [`Target::Construct`], which is what makes [`address`] terminate: the index is built
    /// from pages and sections, and both resolve to a page or an anchor.
    pub fn find(&self, reference: &EssSemanticRef) -> Option<&Target> {
        self.0.get(reference)
    }
}

/// Records every section of these blocks that documents a construct.
///
/// Sections only, because a section is the only block that has an anchor to link to. A section
/// nested inside a list item is not a shape a producer writes and is not walked here.
fn index_sections(blocks: &[Block], here: &PageId, into: &mut BTreeMap<EssSemanticRef, Target>) {
    for block in blocks {
        if let Block::Section {
            anchor,
            about,
            blocks,
            ..
        } = block
        {
            if let Some(reference) = about {
                into.entry(reference.clone())
                    .or_insert_with(|| Target::Anchor {
                        page: here.clone(),
                        anchor: anchor.clone(),
                    });
            }
            index_sections(blocks, here, into);
        }
    }
}

/// The document as a self-contained static website.
///
/// Holds the two things a site has that a document does not: which style to publish it in, and the
/// adopter's own front page if they wrote one.
#[derive(Debug, Clone, Default)]
pub struct Site {
    /// How it looks.
    pub style: Style,
    /// An adopter's own front page, shown above the generated index.
    ///
    /// Empty means the generated overview is the front page, which is the honest default: a
    /// specification whose author wrote no prose about it should not get a blank panel where the
    /// prose would be.
    ///
    /// Blocks and not markdown. A renderer that parsed text would be the second parser in this
    /// crate and exactly the round trip the document IR exists to remove; [`crate::authored`]
    /// reads a `README.md` into blocks once, and every renderer of the document gets it.
    pub front_page: Vec<Block>,
}

impl Site {
    /// Renders a document only if all page and artifact destinations are valid and distinct.
    ///
    /// Use this entry point for authored or deserialized documents. It checks the complete
    /// sequence before callers can collect it into a map, including collisions with site assets.
    pub fn try_render(
        &self,
        document: &Document,
        whole: &Provenance,
    ) -> Result<Vec<Artifact>, String> {
        document.validate_page_ids()?;
        let artifacts = self.render(document, whole);
        crate::artifact::validate_paths(artifacts.iter().map(|artifact| artifact.path.as_str()))?;
        Ok(artifacts)
    }

    /// A site in the default style, with no adopter front page.
    pub fn new() -> Self {
        Self::default()
    }

    /// The same site, opening on blocks an adopter wrote.
    ///
    /// Blocks and not markdown: reading `README.md` is [`crate::authored`]'s job, and a renderer
    /// that took the text would be the second thing in this crate to parse.
    #[must_use]
    pub fn with_front_page(mut self, blocks: Vec<Block>) -> Self {
        self.front_page = blocks;
        self
    }

    /// Every page of the document, plus the assets they refer to.
    ///
    /// This compatibility entry point assumes validated page identities. Use [`Self::try_render`]
    /// for untrusted documents and before collecting results into a map.
    ///
    /// `whole` is the **whole-model** provenance. The stylesheet, the diagram renderer and its
    /// licence derive from no construct at all, and [`crate::artifact::run`] checks every
    /// artifact's stamp against the slice it records — so the only stamp that is both accepted and
    /// honest for a constant is the fail-closed one, the whole model.
    pub fn render(&self, document: &Document, whole: &Provenance) -> Vec<Artifact> {
        let constructs = ConstructIndex::of(document);
        let mut out: Vec<Artifact> = document
            .pages
            .iter()
            .map(|page| self.page(document, page, &constructs))
            .collect();

        out.push(Artifact::new(
            STYLESHEET,
            format!("{}{}", comment(whole), self.style.stylesheet()),
        ));
        // The bundle is included rather than fetched so the site works offline and in a review
        // with no network, and its licence travels with it because MIT requires it to.
        out.push(Artifact::new(
            RENDERER,
            format!(
                "{}{}",
                comment(whole),
                include_str!("../assets/mermaid.min.js")
            ),
        ));
        out.push(Artifact::new(RENDERER_LICENCE, licence(whole)));
        out
    }

    /// One page, as a complete HTML document.
    pub fn page(&self, document: &Document, page: &Page, constructs: &ConstructIndex) -> Artifact {
        // How far this page is from the site root, so every asset and every link is relative and
        // the site can be served from any subdirectory — or opened off a disk.
        let up = "../".repeat(page.id.depth());
        let provenance = &page.provenance.provenance;

        let mut out = String::from("<!DOCTYPE html>\n");
        // First in the file, before anything a browser renders: the same stamp the markdown pages
        // carry, in the place a person opening the source looks.
        out.push_str(&provenance.html_comment());
        out.push_str("<html lang=\"en\">\n<head>\n<meta charset=\"utf-8\">\n");
        out.push_str("<meta name=\"viewport\" content=\"width=device-width, initial-scale=1\">\n");
        let _ = writeln!(
            out,
            "<title>{} · {}</title>",
            escape(&plain(&page.title)),
            escape(&document.system)
        );
        let _ = writeln!(out, "<link rel=\"stylesheet\" href=\"{up}{STYLESHEET}\">");
        out.push_str("</head>\n<body>\n<div class=\"shell\">\n");
        out.push_str(&sidebar(document, &page.id, &up));
        out.push_str("<main>\n<article class=\"content\">\n");
        let _ = writeln!(
            out,
            "<h1>{}</h1>",
            inlines(&page.title, &page.id, constructs)
        );
        if page.id.as_str() == INDEX && !self.front_page.is_empty() {
            out.push_str(&body(&self.front_page, &page.id, constructs));
        }
        out.push_str(&body(&page.blocks, &page.id, constructs));
        out.push_str("</article>\n");
        out.push_str(&footer(provenance));
        out.push_str("</main>\n</div>\n");
        out.push_str(&runner(&up));
        out.push_str("</body>\n</html>\n");

        Artifact::sliced(
            format!("{}.html", page.id.as_str()),
            out,
            page.provenance.slice.clone(),
        )
    }
}

/// The masthead and the page list.
fn sidebar(document: &Document, here: &PageId, up: &str) -> String {
    let mut out = String::from("<nav class=\"sidebar\">\n");
    let _ = writeln!(out, "<a class=\"masthead\" href=\"{up}{INDEX}.html\">");
    let _ = writeln!(
        out,
        "<span class=\"system\">{}</span><span class=\"version\">{}</span>",
        escape(&document.system),
        escape(&document.version)
    );
    out.push_str("</a>\n<div class=\"nav\">\n");

    // The index first, then everything else by id. Alphabetical is not a judgement about
    // importance — it is the one order two runs agree on, and ranking pages by significance would
    // be this renderer inventing a claim the document does not make.
    let mut group: Option<&str> = None;
    for page in ordered(document) {
        let nested = page.id.as_str().split_once('/');
        match nested {
            Some((directory, _)) if group != Some(directory) => {
                let _ = writeln!(out, "<div class=\"group\">{}</div>", escape(directory));
                group = Some(directory);
            }
            Some(_) => {}
            None => group = None,
        }
        let class = if nested.is_some() {
            " class=\"nested\""
        } else {
            ""
        };
        // Not a colour or a weight: a reader on a screen reader is told which entry is the page
        // they are on, and the stylesheet takes its highlight from the same attribute.
        let current = if &page.id == here {
            " aria-current=\"page\""
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "<a{class} href=\"{}.html\"{current}>{}</a>",
            escape(&relative(here, &page.id)),
            escape(&plain(&page.title))
        );
    }
    out.push_str("</div>\n</nav>\n");
    out
}

/// The pages in the order a sidebar lists them: the index, then the rest by id.
fn ordered(document: &Document) -> Vec<&Page> {
    let mut pages: Vec<&Page> = document.pages.iter().collect();
    pages.sort_by(|left, right| {
        (left.id.as_str() != INDEX, &left.id).cmp(&(right.id.as_str() != INDEX, &right.id))
    });
    pages
}

/// A run of blocks, as the elements they are.
fn body(blocks: &[Block], here: &PageId, constructs: &ConstructIndex) -> String {
    let mut out = String::new();
    for block in blocks {
        match block {
            Block::Prose { text } => {
                let _ = writeln!(out, "<p>{}</p>", inlines(text, here, constructs));
            }
            Block::Section {
                level,
                title,
                anchor,
                blocks,
                ..
            } => {
                // The page title is the `<h1>`, so the first section under it is an `<h2>`. Clamped
                // at six because HTML has no seventh heading and `<h7>` is an unknown element a
                // browser styles as nothing.
                let level = (*level).clamp(2, 6);
                let _ = writeln!(
                    out,
                    "<h{level} id=\"{}\">{}</h{level}>",
                    escape(anchor),
                    inlines(title, here, constructs)
                );
                out.push_str(&body(blocks, here, constructs));
            }
            Block::Quote { blocks } => {
                let _ = writeln!(
                    out,
                    "<blockquote>\n{}</blockquote>",
                    body(blocks, here, constructs)
                );
            }
            Block::List { ordered, items } => {
                let tag = if *ordered { "ol" } else { "ul" };
                let _ = writeln!(out, "<{tag}>");
                for item in items {
                    match item.as_slice() {
                        // A one-paragraph item goes straight into the `<li>`: a `<p>` inside a list
                        // item carries the paragraph's bottom margin, which spaces a tight list out
                        // like a loose one.
                        [Block::Prose { text }] => {
                            let _ = writeln!(out, "<li>{}</li>", inlines(text, here, constructs));
                        }
                        blocks => {
                            let _ = writeln!(out, "<li>\n{}</li>", body(blocks, here, constructs));
                        }
                    }
                }
                let _ = writeln!(out, "</{tag}>");
            }
            Block::Table { columns, rows } => {
                out.push_str("<table>\n<thead>\n<tr>");
                for column in columns {
                    let _ = write!(out, "<th>{}</th>", inlines(column, here, constructs));
                }
                out.push_str("</tr>\n</thead>\n<tbody>\n");
                for row in rows {
                    out.push_str("<tr>");
                    for cell in row {
                        let _ = write!(out, "<td>{}</td>", inlines(cell, here, constructs));
                    }
                    out.push_str("</tr>\n");
                }
                out.push_str("</tbody>\n</table>\n");
            }
            Block::Code { language, text } => {
                let class = match language {
                    Some(language) => format!(" class=\"language-{}\"", escape(language)),
                    None => String::new(),
                };
                let _ = writeln!(out, "<pre><code{class}>{}</code></pre>", escape(text));
            }
            // Not a `<code>` listing: the source is what the renderer reads out of the element and
            // replaces, and until it does it is the diagram's own text, which is legible.
            Block::Diagram { source, .. } => {
                let _ = writeln!(out, "<pre class=\"mermaid\">{}</pre>", escape(source));
            }
            Block::Rule => out.push_str("<hr>\n"),
        }
    }
    out
}

/// A run of inlines.
fn inlines(text: &[Inline], here: &PageId, constructs: &ConstructIndex) -> String {
    let mut out = String::new();
    for inline in text {
        match inline {
            Inline::Text { text } => out.push_str(&escape(text)),
            Inline::Code { text } => {
                let _ = write!(out, "<code>{}</code>", escape(text));
            }
            Inline::Emphasis { text } => {
                let _ = write!(out, "<em>{}</em>", inlines(text, here, constructs));
            }
            Inline::Strong { text } => {
                let _ = write!(out, "<strong>{}</strong>", inlines(text, here, constructs));
            }
            Inline::Link { to, text } => {
                let reads = inlines(text, here, constructs);
                let href = address(to, here, constructs);
                if href.is_empty() {
                    // An unaddressable target would become `href=""`, which is a link back to the
                    // page the reader is already on — worse than no link. The text stays.
                    out.push_str(&reads);
                } else {
                    let _ = write!(out, "<a href=\"{}\">{reads}</a>", escape(&href));
                }
            }
        }
    }
    out
}

/// Where a link goes, as a path this renderer can serve.
///
/// The producer named a page or a construct; the address is computed here, because this is the
/// layer that knows pages are files and that files end in `.html`.
///
/// A [`Target::Construct`] is resolved rather than dropped — this is where this renderer differs
/// from [`crate::markdown::address`], which has no index to consult and drops the link. Resolution
/// recurses exactly once and cannot loop: a [`ConstructIndex`] only ever holds a page or an anchor.
pub fn address(target: &Target, here: &PageId, constructs: &ConstructIndex) -> String {
    match target {
        Target::Page { page } => format!("{}.html", relative(here, page)),
        Target::Anchor { page, anchor } if page == here => format!("#{anchor}"),
        Target::Anchor { page, anchor } => format!("{}.html#{anchor}", relative(here, page)),
        Target::Construct { reference } => match constructs.find(reference) {
            Some(found) => address(found, here, constructs),
            // A construct no page documents is not addressable, and an address invented for it
            // would point at nothing.
            None => String::new(),
        },
        Target::External { url } => url.clone(),
    }
}

/// The path of `to` as written on `from`'s own page.
///
/// A real relative walk rather than `../` repeated: two pages in one directory link to each other
/// by name, which is what a reader expects to see in a diff.
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

/// Everything a run of inlines says, with the markup dropped.
///
/// For `<title>` and for a sidebar entry, both of which are places an element cannot go: a nav
/// entry rendered with its markup would put an `<a>` inside an `<a>` the day a title held a link.
fn plain(text: &[Inline]) -> String {
    let mut out = String::new();
    for inline in text {
        match inline {
            Inline::Text { text } | Inline::Code { text } => out.push_str(text),
            Inline::Emphasis { text } | Inline::Strong { text } | Inline::Link { text, .. } => {
                out.push_str(&plain(text));
            }
        }
    }
    out
}

/// Text as it can safely appear in HTML.
///
/// All five, including the two quote characters: this output is written into attribute values as
/// well as into element content, and an escaper that is correct in one place and not the other is
/// an escaper somebody will use in the wrong place.
fn escape(text: &str) -> String {
    let mut out = String::with_capacity(text.len());
    for character in text.chars() {
        match character {
            '&' => out.push_str("&amp;"),
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&#39;"),
            other => out.push(other),
        }
    }
    out
}

/// The same facts as the head comment, visible.
///
/// Duplicated on purpose, as in [`crate::markdown`]: the comment is for a tool and a diff, and it
/// is invisible to exactly the person about to edit a generated file by hand and lose the work.
fn footer(provenance: &Provenance) -> String {
    format!(
        "<footer class=\"provenance\">\nGenerated from {} {} · model digest <code>{}</code> · \
         contract digest <code>{}</code>. Do not edit this file; change the specification and \
         regenerate it with <code>ess generate</code>.\n</footer>\n",
        escape(&provenance.system),
        escape(&provenance.specification_version),
        escape(&provenance.source_digest),
        escape(&provenance.contract_digest),
    )
}

/// The script that draws the diagrams.
///
/// Three decisions are worth the bytes they cost on every page:
///
/// * The bundle is fetched only when the page has a diagram. It is three and a half megabytes, and
///   most pages have none.
/// * It is loaded through a `<script>` element rather than `import`, because the build publishes
///   itself from a top-level `var` that a module would scope away — see this module's own docs.
/// * `securityLevel: "strict"` because the diagram source comes from a specification, and a
///   renderer that executes what a label says is one specification away from being an exploit.
fn runner(up: &str) -> String {
    format!(
        r#"<script type="module">
const diagrams = document.querySelectorAll("pre.mermaid");
if (diagrams.length > 0) {{
  // A script element, not `import`: the bundle publishes itself with a top-level `var`, and a
  // top-level `var` in a module is scoped to the module, so importing it throws.
  await new Promise((resolve, reject) => {{
    const tag = document.createElement("script");
    tag.src = "{up}{RENDERER}";
    tag.addEventListener("load", resolve);
    tag.addEventListener("error", reject);
    document.head.append(tag);
  }});
  globalThis.mermaid.initialize({{
    startOnLoad: false,
    securityLevel: "strict",
    theme: window.matchMedia("(prefers-color-scheme: dark)").matches ? "dark" : "default",
  }});
  await globalThis.mermaid.run({{ querySelector: "pre.mermaid" }});
}}
</script>
"#
    )
}

/// The provenance as one block comment, for CSS and JavaScript.
///
/// [`crate::artifact::run`] refuses an artifact whose provenance it cannot read, and it reads the
/// same `model digest` and `contract digest` markers whatever the comment syntax around them is —
/// so an asset carries its stamp exactly as a page does.
fn comment(provenance: &Provenance) -> String {
    format!("/*\n{} */\n", provenance.commented(" *"))
}

/// The diagram renderer's licence, with this site's provenance after it.
///
/// After, and never around: the licence text is not ours to edit, and a stamp inserted into it
/// would be a modification of somebody else's licence to satisfy our own check.
fn licence(provenance: &Provenance) -> String {
    let mut out = String::from(include_str!("../assets/mermaid.LICENSE"));
    if !out.ends_with('\n') {
        out.push('\n');
    }
    out.push_str("\n--\nThe licence of `assets/mermaid.min.js`, redistributed with this site.\n");
    for line in provenance.lines() {
        let _ = writeln!(out, "{line}");
    }
    out
}

/// The site is a projection like any other, so `ess generate` reaches it by name and
/// `generate_all` writes it beside the rest.
///
/// The front page a `Site` may carry is not reachable through this trait — `generate` receives only
/// the model. A caller that has an adopter's `README.md` in hand constructs the `Site` itself and
/// calls [`Site::render`].
impl Generator for Site {
    fn name(&self) -> &'static str {
        "site"
    }

    fn describes(&self) -> &'static str {
        "the documentation as a browsable site: pages, navigation, stylesheet and diagrams"
    }

    fn directory(&self) -> &'static str {
        "site"
    }

    fn generate(
        &self,
        ir: &ess_compiler::EssIr,
        mint: &crate::provenance::ProvenanceMint,
    ) -> Vec<Artifact> {
        self.render(&crate::docs::document(ir, mint), &mint.whole().provenance)
    }
}

#[cfg(test)]
mod tests {
    use ess_compiler::refs::EntityRef;
    use ess_domain::name::QualifiedName;

    use super::*;
    use crate::document::{Blocks, DiagramKind};
    use crate::provenance::{ModelSlice, SlicedProvenance};

    fn provenance() -> Provenance {
        Provenance {
            system: "billing".to_owned(),
            specification_version: "v3".to_owned(),
            source_digest: "0".repeat(64),
            contract_digest: "f".repeat(64),
        }
    }

    fn sliced() -> SlicedProvenance {
        SlicedProvenance {
            provenance: provenance(),
            slice: ModelSlice::WholeModel,
        }
    }

    fn entity(name: &str) -> EssSemanticRef {
        EssSemanticRef::from(EntityRef::new(
            QualifiedName::new(name).expect("a valid qualified name"),
        ))
    }

    fn page(id: &str, blocks: Vec<Block>) -> Page {
        Page {
            id: PageId::from(id),
            title: vec![Inline::text(id)],
            about: None,
            provenance: sliced(),
            blocks,
        }
    }

    fn here() -> PageId {
        PageId::from("domains/billing-invoice")
    }

    fn empty() -> ConstructIndex {
        ConstructIndex::default()
    }

    #[test]
    fn checked_rendering_validates_deserialized_page_identities_before_map_collection() {
        for ids in [
            vec!["index", "index"],
            vec!["index", "INDEX"],
            vec!["../escape"],
            vec!["a//b"],
            vec!["plan", "plan.html/child"],
            vec!["assets/style.css/child"],
        ] {
            let document = Document::new(
                "billing",
                "v3",
                ids.iter().map(|id| page(id, Vec::new())).collect(),
            );
            let encoded = serde_json::to_string(&document).unwrap();
            let decoded: Document = serde_json::from_str(&encoded).unwrap();
            assert!(
                Site::new().try_render(&decoded, &provenance()).is_err(),
                "{ids:?}"
            );
        }
    }

    #[test]
    fn checked_rendering_preserves_valid_parent_and_nested_page_bytes() {
        let document = Document::new(
            "billing",
            "v3",
            vec![
                page("index", Vec::new()),
                page("plan", Vec::new()),
                page("plan/board", Vec::new()),
                page("plan.page/board", Vec::new()),
            ],
        );
        assert_eq!(
            Site::new().try_render(&document, &provenance()).unwrap(),
            Site::new().render(&document, &provenance())
        );
    }

    #[test]
    fn a_link_is_addressed_from_the_page_it_is_written_on() {
        let index = Target::Page {
            page: PageId::from(INDEX),
        };
        assert_eq!(address(&index, &here(), &empty()), "../index.html");
        assert_eq!(
            address(&index, &PageId::from("crossings"), &empty()),
            "index.html"
        );

        // Two pages in one directory link by name, not by a round trip through the root.
        let sibling = Target::Anchor {
            page: PageId::from("domains/billing-email"),
            anchor: "sendemail".to_owned(),
        };
        assert_eq!(
            address(&sibling, &here(), &empty()),
            "billing-email.html#sendemail"
        );
        assert_eq!(
            address(&sibling, &PageId::from("interactions"), &empty()),
            "domains/billing-email.html#sendemail"
        );
    }

    #[test]
    fn a_link_into_the_page_it_is_on_is_a_fragment_and_not_a_round_trip() {
        let own = Target::Anchor {
            page: here(),
            anchor: "invoice".to_owned(),
        };
        assert_eq!(address(&own, &here(), &empty()), "#invoice");
    }

    #[test]
    fn a_construct_is_addressed_by_the_section_that_documents_it() {
        let invoice = entity("billing.Invoice");
        let document = Document::new(
            "billing",
            "v3",
            vec![
                page(INDEX, Vec::new()),
                Page {
                    about: None,
                    ..page(
                        "domains/billing-invoice",
                        vec![Block::Section {
                            level: 2,
                            title: vec![Inline::text("Invoice")],
                            anchor: "invoice".to_owned(),
                            about: Some(invoice.clone()),
                            blocks: Vec::new(),
                        }],
                    )
                },
            ],
        );

        let constructs = ConstructIndex::of(&document);
        let target = Target::Construct {
            reference: invoice.clone(),
        };
        assert_eq!(
            address(&target, &PageId::from(INDEX), &constructs),
            "domains/billing-invoice.html#invoice"
        );
        assert_eq!(address(&target, &here(), &constructs), "#invoice");

        // A construct nothing documents is not addressable, and the link is dropped rather than
        // pointed at the page the reader is already on.
        let unknown = Target::Construct {
            reference: entity("billing.Ledger"),
        };
        assert_eq!(address(&unknown, &here(), &constructs), "");
        let mut blocks = Blocks::new();
        blocks.prose(vec![Inline::code_link(unknown, "Ledger")]);
        let rendered = body(&blocks.finish(), &here(), &constructs);
        assert_eq!(rendered, "<p><code>Ledger</code></p>\n");
    }

    #[test]
    fn a_diagram_is_a_pre_the_renderer_draws_into_and_never_a_code_listing() {
        let mut blocks = Blocks::new();
        blocks.push(Block::Diagram {
            kind: DiagramKind::Lifecycle,
            source: "stateDiagram-v2\n    [*] --> Draft".to_owned(),
        });
        let rendered = body(&blocks.finish(), &here(), &empty());
        assert_eq!(
            rendered,
            "<pre class=\"mermaid\">stateDiagram-v2\n    [*] --&gt; Draft</pre>\n"
        );
        assert!(!rendered.contains("<code"), "{rendered}");
    }

    #[test]
    fn a_code_block_is_a_code_listing_and_carries_its_language() {
        let mut blocks = Blocks::new();
        blocks.push(Block::Code {
            language: Some("yaml".to_owned()),
            text: "system: billing".to_owned(),
        });
        assert_eq!(
            body(&blocks.finish(), &here(), &empty()),
            "<pre><code class=\"language-yaml\">system: billing</code></pre>\n"
        );
    }

    #[test]
    fn a_table_is_a_table_with_a_head_and_a_body() {
        let mut blocks = Blocks::new();
        blocks.push(Block::Table {
            columns: vec![vec![Inline::text("field")], vec![Inline::text("type")]],
            rows: vec![vec![
                vec![Inline::code("call_id")],
                vec![Inline::code("CallId")],
            ]],
        });
        assert_eq!(
            body(&blocks.finish(), &here(), &empty()),
            "<table>\n<thead>\n<tr><th>field</th><th>type</th></tr>\n</thead>\n<tbody>\n\
             <tr><td><code>call_id</code></td><td><code>CallId</code></td></tr>\n\
             </tbody>\n</table>\n"
        );
    }

    #[test]
    fn markup_in_text_never_reaches_the_browser_as_markup() {
        let hostile = "<script>alert('x' & \"y\")</script>";
        let mut blocks = Blocks::new();
        blocks.prose(vec![Inline::text(hostile)]);
        blocks.push(Block::Code {
            language: None,
            text: hostile.to_owned(),
        });
        blocks.push(Block::Diagram {
            kind: DiagramKind::System,
            source: hostile.to_owned(),
        });
        let rendered = body(&blocks.finish(), &here(), &empty());

        assert!(!rendered.contains("<script>alert"), "{rendered}");
        assert!(!rendered.contains("alert('x'"), "{rendered}");
        assert_eq!(
            escape(hostile),
            "&lt;script&gt;alert(&#39;x&#39; &amp; &quot;y&quot;)&lt;/script&gt;"
        );

        // A title is escaped in `<title>` and in the sidebar too, which are the two places the
        // markup is dropped rather than rendered.
        let document = Document::new(
            "bill<ing>",
            "v&3",
            vec![Page {
                title: vec![Inline::text(hostile)],
                ..page(INDEX, Vec::new())
            }],
        );
        let rendered = Site::new()
            .page(&document, &document.pages[0], &empty())
            .contents;
        assert!(!rendered.contains("<script>alert"), "{rendered}");
        assert!(
            rendered.contains("<span class=\"system\">bill&lt;ing&gt;</span>"),
            "{rendered}"
        );
    }

    #[test]
    fn the_sidebar_groups_the_nested_pages_and_marks_the_page_the_reader_is_on() {
        let document = Document::new(
            "billing",
            "v3",
            vec![
                page("topology", Vec::new()),
                page("domains/billing-invoice", Vec::new()),
                page(INDEX, Vec::new()),
                page("domains/billing-email", Vec::new()),
            ],
        );
        let rendered = sidebar(&document, &here(), "../");

        assert_eq!(
            rendered
                .lines()
                .filter(|line| line.starts_with("<a") && !line.contains("masthead"))
                .collect::<Vec<_>>(),
            vec![
                "<a href=\"../index.html\">index</a>",
                "<a class=\"nested\" href=\"billing-email.html\">domains/billing-email</a>",
                "<a class=\"nested\" href=\"billing-invoice.html\" aria-current=\"page\">domains/billing-invoice</a>",
                "<a href=\"../topology.html\">topology</a>",
            ]
        );
        assert_eq!(
            rendered
                .matches("<div class=\"group\">domains</div>")
                .count(),
            1
        );
    }

    #[test]
    fn every_emitted_file_says_what_it_was_generated_from() {
        let document = Document::new(
            "billing",
            "v3",
            vec![
                page(INDEX, Vec::new()),
                page("domains/billing-invoice", Vec::new()),
            ],
        );
        let artifacts = Site::new().render(&document, &provenance());

        assert_eq!(
            artifacts
                .iter()
                .map(|artifact| artifact.path.as_str())
                .collect::<Vec<_>>(),
            vec![
                "index.html",
                "domains/billing-invoice.html",
                STYLESHEET,
                RENDERER,
                RENDERER_LICENCE,
            ]
        );
        for artifact in &artifacts {
            let digests = Provenance::read_digests(&artifact.contents)
                .unwrap_or_else(|| panic!("`{}` carries no readable provenance", artifact.path));
            assert_eq!(digests.source_digest, "0".repeat(64), "{}", artifact.path);
            assert_eq!(digests.contract_digest, "f".repeat(64), "{}", artifact.path);
        }
    }

    #[test]
    fn a_page_reaches_its_stylesheet_and_its_renderer_from_wherever_it_sits() {
        let document = Document::new(
            "billing",
            "v3",
            vec![
                page(INDEX, Vec::new()),
                page("domains/billing-invoice", Vec::new()),
            ],
        );
        let site = Site::new();

        let root = site.page(&document, &document.pages[0], &empty()).contents;
        assert!(root.contains("href=\"assets/style.css\""), "{root}");
        assert!(root.contains("\"assets/mermaid.min.js\""), "{root}");

        let nested = site.page(&document, &document.pages[1], &empty()).contents;
        assert!(nested.contains("href=\"../assets/style.css\""), "{nested}");
        assert!(nested.contains("\"../assets/mermaid.min.js\""), "{nested}");
    }

    #[test]
    fn an_adopters_front_page_goes_above_the_index_and_nowhere_else() {
        let document = Document::new(
            "billing",
            "v3",
            vec![
                page(INDEX, Vec::new()),
                page("domains/billing-invoice", Vec::new()),
            ],
        );
        let site = Site::new().with_front_page(crate::authored::blocks("Read **this** first.\n"));

        let index = site.page(&document, &document.pages[0], &empty()).contents;
        assert!(
            index.contains("<p>Read <strong>this</strong> first.</p>"),
            "the adopter's prose is rendered, not shown as source: {index}"
        );

        let other = site.page(&document, &document.pages[1], &empty()).contents;
        assert!(!other.contains("Read <strong>"), "{other}");
    }

    #[test]
    fn the_default_style_is_the_stylesheet_that_is_published() {
        assert_eq!(Style::default().stylesheet(), Style::Default.stylesheet());
        let published = Site::new()
            .render(&Document::new("billing", "v3", Vec::new()), &provenance())
            .into_iter()
            .find(|artifact| artifact.path == STYLESHEET)
            .expect("a stylesheet");
        assert!(published.contents.ends_with(Style::Default.stylesheet()));
    }
}
