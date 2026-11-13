---
format: aep.planning-md/3
id: story:a-widget-parameter-in-a-map-key-is-substituted-or-refused
kind: story
status: draft
title: A widget parameter in a map key is substituted or refused
summary: 'Widget expansion never substitutes a parameter written as a map key (sets: {args.target: x}), while ess ui check counts that key as a read of the parameter in an expression position (found by reading in a review)'
tags:
- backlog
- defect
- ess-ui
- from-finding
- review-finding
- severity-low
relations:
- serves: vision:O2
- informed_by: story:ui-check-resolves-the-root-and-section-of-every-binding-path
revision: 1
---
## Defect

Widget expansion substitutes a parameter in a value and never in a map key
(`crates/ui/ess-ui/src/expand.rs:1845`). A widget body that writes `sets: {args.target: x}` keeps
the literal key `args.target` after expansion. `ess ui check` reads the same key as a read of the
parameter in an expression position (`crates/ui/ess-ui-check/src/sites.rs`, the map-key handling
of the first walk), so the use's argument for `target` is judged as an expression and a dotted word
there draws `expr_roots`, for a parameter that is never substituted.

Found by the third review pass of
`story:ui-check-resolves-the-root-and-section-of-every-binding-path`, by reading the code; the
expansion was so before that unit. No test.

## What a design has to settle

Whether a parameter may stand in a map key. If it may, expansion substitutes it and the check's
reading is right. If it may not, the document is refused where the key is written, and the check
does not count the key as a read.

## Acceptance (sketch, to be fixed by the fit review)

- A test, red before, for the settled reading: a `sets` key written as `args.<param>` in a widget
  body is either substituted in both renderers or refused at its path.
- The check judges the argument as the expansion uses it.

## Fit review

Owed. This draft records a finding by reading only.
