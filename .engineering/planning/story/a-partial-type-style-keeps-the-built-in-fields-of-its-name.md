---
format: aep.planning-md/3
id: story:a-partial-type-style-keeps-the-built-in-fields-of-its-name
kind: story
status: draft
title: A partial type style keeps the built-in fields of its name
summary: tokens.type.heading with only a family drops the built-in heading size and weight (turn-2 adversary, outside the tokens acceptance, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-t2-tokens-adv-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

A `tokens.type.<name>` entry replaces the built-in entry of its name whole
(`Tokens::merged` in `crates/ui/ess-ui/src/style.rs`), and `Tokens::filled` fills a field the entry
leaves out from the document's `body`, then from the built-in `body`, never from the built-in entry
of the same name. So `tokens.type.heading: {family: 'Georgia, serif'}` passes `ess ui check`, but
the generated `tokens.css` gives headings the body size and weight, not the built-in 1.05rem and
600: `h2`, `.ui-text-heading`, `.ui-sort` and `.ui-metric-value` lose their heading look. Found by
the turn-2 adversary (`review-result:zi-t2-tokens-adv-1`, finding 3); it follows from the
replace-whole rule the story kept, so it is outside that story's acceptance.

## Acceptance

- A document setting only `tokens.type.heading.family` generates a `tokens.css` whose heading
  size and weight are the built-in heading's.
- A field the document sets still wins over the built-in one.
- The design reference says how a partial type entry is completed.

## Scope

- `crates/ui/ess-ui/src/style.rs` (`Tokens::merged`, `Tokens::filled`)
- `crates/ui/ess-ui-react/tests/tokens.rs`
- `website/docs/reference/ess-ui.md`
