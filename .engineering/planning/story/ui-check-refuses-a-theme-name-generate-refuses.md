---
format: aep.planning-md/3
id: story:ui-check-refuses-a-theme-name-generate-refuses
kind: story
status: draft
title: ess ui check refuses a theme name generate refuses
summary: 'Follow-up of story:light-and-dark-themes-render-differently: generate refuses an unsafe theme name but ui check admits it, and the default theme''s name is checked by neither (adversary pass 1, backlog)'
tags:
- backlog
- defect
- ess-ui
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`ess ui generate` refuses a theme name that could break the `[data-theme="..."]` selector
(`token_grammar::name_problem`, `crates/ui/ess-ui-react/src/assets.rs`), but `ess ui check`
never checks theme names (`crates/ui/ess-ui-check/src/style.rs`). A document with a theme named
`"a b"` checks clean and then fails to generate; the default theme's name is not checked by
either.

Found by adversary pass 1 of story:light-and-dark-themes-render-differently (pre-existing).

## Acceptance

1. `ess ui check` refuses every theme name `generate` would refuse, the default's included, at the
   theme's path, with the same words; a case in `crates/ui/ess-ui-check/tests/checks.rs` pins it.
2. A name `check` admits, `generate` admits.
