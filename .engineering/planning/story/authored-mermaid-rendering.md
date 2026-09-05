---
format: aep.planning-md/1
id: story:authored-mermaid-rendering
kind: story
status: implemented
title: Render authored Mermaid fences in generated sites
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/tests/authored_site.rs
- confidence: cited
  path: crates/generate/ess-gen/src/html.rs
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
revision: 5
---
## Outcome

Authored Markdown Mermaid fences render as diagrams in generated HTML sites, using
the same locally bundled renderer and source fallback as generated diagrams.

## Evidence

In ESS 0.18.0, an included authored page with a fenced `mermaid` flowchart becomes
`<pre><code class="language-mermaid">...</code></pre>`. The browser bootstrap in
`crates/generate/ess-gen/src/html.rs:567-583` selects only `pre.mermaid`, while
the generated `Block::Diagram` path emits that class at line 412. Consequently
the authored flowchart is readable source but never renders, without a JavaScript
error. This is a presentation mismatch, not an invalid domain specification.

## Acceptance

A site containing authored and generated Mermaid diagrams renders both with the
bundled asset, preserves escaped readable source when JavaScript is unavailable,
and leaves non-Mermaid fenced code unchanged. Browser verification covers a nested
authored page and reports rendering failures instead of counting a code block as
a rendered diagram. Repeated generation is byte-stable.

## Scope

- Cited: `crates/generate/ess-gen/src/html.rs` diagram markup and browser bootstrap.
- Inferred: authored Markdown ingestion must retain diagram intent or the HTML
  renderer must recognize its emitted fence markup.
- Excluded: changing Mermaid syntax, adding remote scripts, or modifying domain IR.

This standalone renderer gap introduces no new domain entity and does not decompose
an epic. No implementation is claimed by filing it.
