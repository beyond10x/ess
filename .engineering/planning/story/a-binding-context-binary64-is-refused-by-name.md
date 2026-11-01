---
format: aep.planning-md/3
id: story:a-binding-context-binary64-is-refused-by-name
kind: story
status: draft
title: A binding-context Binary64 is refused by name
summary: A binding's delivery-context field typed Binary64 directly is not walked by owner_uses, so it may be admitted where ESS-AUTHOR-036 should refuse it
relations:
- informed_by: review-result:ess-d335-wave-adversary-u3
revision: 1
---
## Problem

A binding's delivery-context field typed `Binary64` directly, not through a declared type, is not
walked by `owner_uses` in `crates/specify/ess-compiler/src/binary64.rs`, so `uncarried_uses` cannot
refuse it. A Binary64 reached through a binding is meant to stay refused as `ESS-AUTHOR-036`
(0.79.0); this position may be admitted instead. Found by the wave D335 U3 adversary pass as an
out-of-scope row, read from the code and not run; it looks pre-existing.

## Acceptance

- A test with a binding whose delivery-context field is typed `Binary64` directly shows today's
  behaviour first; if it is admitted, it is refused as `ESS-AUTHOR-036` naming the binding and the
  field, as a Binary64 reached through a declared type in a binding is.
