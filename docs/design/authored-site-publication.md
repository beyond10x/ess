# Authored Site Publication

## Authority

The model owns generated documentation. An author explicitly selects supplementary
Markdown pages and UTF-8 downloads. Selection is not permission to crawl sibling
files. Site assembly belongs to the CLI; Markdown parsing and HTML rendering stay
in `ess-gen`.

## Link Resolution

Keep the source path and selected output path for every authored input until its
Markdown has been parsed. Resolve link events against this explicit input map,
using a URI parser for paths, queries, fragments and encoded names. Output-relative
links are presentation values; no source filesystem identity enters persisted
`ess-docs/1`. Reject ambiguous source mappings and output collisions.

`--front-page PATH` overrides the model-adjacent README without relocating it.
`--include PAGE=PATH` keeps its existing meaning. `--asset OUTPUT=PATH` declares
a UTF-8 file to publish verbatim. Binary assets are not silently decoded; the
current artifact API carries strings. All outputs participate in the existing
containment/collision preflight before any output is written.

An explicitly selected source document maps to its page, including a `README.md`
directory index. Queries and fragments survive the mapping. Absolute network
URLs and fragment-only links retain their meaning. Other local targets remain
untouched for compatibility, unless `--strict-links` is set: then missing targets
are source-located errors and generation writes nothing. Strictness never grants
additional file access or silently publishes an undeclared file.

## Authored Diagrams

A fenced code block whose exact language is `mermaid` uses the same escaped
`pre.mermaid` HTML and bundled renderer as `Block::Diagram`. The authored source
does not establish a model-specific `DiagramKind`; it remains `Block::Code` in
the document IR. No format revision or guessed classification is needed.

## Prose Layout

The content container permits emergency wrapping for unbroken identifiers in
ordinary, linked and emphasized prose, not only inline code. Preserve source text;
do not truncate identifiers or hide page overflow. Preformatted listings and
diagram source retain their own horizontal scrolling and whitespace behavior.

## Verification

Cover nested pages, relocated front pages, README indexes, declared downloads,
encoded filenames, queries/fragments, external links, absent targets and complete
output preflight. Render authored and generated diagrams in a browser, verify
source remains legible with JavaScript disabled, and compare repeated output.
Library tests hold ordinary fences unchanged. The workspace and site-build gates
remain required before publication.
Check long identifiers at narrow and desktop viewports, including emphasized text
and links, with JavaScript enabled and disabled.
