---
format: aep.planning-md/3
id: story:areas-that-interleave-sections-stack-as-the-terminal-does
kind: story
status: draft
title: Areas that interleave sections stack as the terminal does
summary: 'Follow-up of story:a-wide-table-scrolls-inside-its-column: below 48rem React stacks whole areas at their earliest declared section, so areas placing interleaved sections stack a, c, b where the terminal stacks a, b, c (adversary pass 2, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- ess-ui
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:a-wide-table-scrolls-inside-its-column` makes React stack a page's areas below `48rem` in
section declaration order. It orders whole areas by the earliest declared section each one places
(`crates/ui/ess-ui-react/src/emit.rs`, the `PageLayout::Areas` arm).

When areas place interleaved sections, React and the terminal disagree. With `left: [a, c]` and
`right: [b]`:
- React stacks `a, c, b`;
- the terminal goes section by section (`crates/ui/ess-ui-tui/src/profile.rs`) and stacks
  `a, b, c`.

Columns behave the same way. The shipped examples do not interleave.

Above `48rem` the page order for keyboard and screen readers now follows section order, not the
grid's reading order, so on the partner-portal overview focus lands first on `board`, which sits
in the bottom row.

The second adversary pass over the wide-table story found both. Neither is a regression against
what shipped.

## Acceptance

1. A page whose areas or columns place interleaved sections stacks below `48rem` in the order the
   terminal draws it. A generator test pins it, either through the order or through a refusal or
   warning in `ess ui check` that the layout cannot stack in section order.
2. The focus order above `48rem` is stated, in the reference page or the design note: grid
   reading order or section order. A test pins whichever is chosen.
