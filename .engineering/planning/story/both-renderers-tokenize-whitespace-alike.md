---
format: aep.planning-md/3
id: story:both-renderers-tokenize-whitespace-alike
kind: story
status: draft
title: Both renderers tokenize whitespace alike
summary: React splits expressions on ASCII whitespace only while the filter grammar accepts a no-break space (UI-adopter wave 1 U2 pass-4 note, backlog)
tags:
- backlog
- from-finding
- review-finding
relations:
- informed_by: review-result:w086-u2-adversary-pass-4
revision: 1
---
## Need

The two renderers' tokenizers treat whitespace differently. The U2 adversary pass 4 of UI-adopter wave 1 found it (review-result:w086-u2-adversary-pass-4, INFEASIBLE note, introduced). The red case is a hand-built expression; no input from a downstream adopter is known to reach it.

- `crates/ui/ess-ui-react/templates/runtime/expr.ts.tmpl:48`: React's tokenizer splits only on ASCII whitespace. It cannot tokenize a no-break space (U+00A0), so `evaluate` returns the text, which is truthy, and every toggle is drawn on.
- The filter grammar the terminal uses accepts that space as a separator, reads `not row.off`, and draws off, on.
- `checked_unreadable` is silent, because the filter grammar reads the text with admitted roots.

The case from pass 4 was not committed:

```rust
"{name: tc, as: toggle, label: Charlie, does: posts.SetT, bind: {id: row.id}, checked: \"not\\u00a0row.off\"}",
// rows [{off: true}, {off: false}]: the terminal draws off, on; React draws on, on.
```

The case is part of the pass-4 `a_checked_the_check_passes_marks_each_row_as_the_terminal_does` (React) and `a_checked_the_renderers_disagree_on_warns` (check, `"not row.off"` with U+00A0).

## Acceptance

- Both tokenizers split on the same set of whitespace, or `ess ui check` refuses whitespace the React tokenizer does not read.
- The case above draws the same marks in both renderers.

## Schedule

Backlog: not waved; considered only after every predefined wave is released.
