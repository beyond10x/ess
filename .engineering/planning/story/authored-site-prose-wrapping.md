---
format: aep.planning-md/1
id: story:authored-site-prose-wrapping
kind: story
status: implemented
title: Keep long authored identifiers within narrow viewports
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/generate/ess-gen/assets/default.css
- confidence: cited
  path: docs/design/authored-site-publication.md
revision: 5
---
## Outcome

Long unbroken identifiers in authored HTML prose remain readable without widening
the page beyond a narrow viewport. Fix the shared site stylesheet, not adopter text.

## Evidence

`crates/generate/ess-gen/assets/default.css` applies `overflow-wrap: anywhere` only
to inline code. A bold forty-character hexadecimal revision in an authored
paragraph extends outside a 390-pixel Chromium viewport. The content container
already constrains its measure, but ordinary inline text has no emergency wrap.

## Acceptance

Generated HTML containing long plain, linked and emphasized identifiers fits both
narrow and desktop viewports. Identifier text is unchanged and selectable. Code
listings and diagrams retain their own horizontal scrolling. Verify in Chromium
with JavaScript enabled and disabled; run the workspace and site-build gates.

## Scope

- Cited: `crates/generate/ess-gen/assets/default.css`.
- Cited: `docs/design/authored-site-publication.md` and `CHANGELOG.md`.
- No persisted model change, adopter rewrite, truncation or hidden page overflow.
