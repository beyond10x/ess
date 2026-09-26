---
format: aep.planning-md/2
id: story:predicate-reference-page
kind: story
status: active
title: Every predicate form ESS accepts is on one reference page
relations:
- serves: vision:O2
revision: 3
---
## Outcome

An adopter can look up every predicate form ESS accepts on one reference page, including where
synthesis cannot witness a form.

## Why

GitHub issue beyond10x/ess#92. The grammar is documented only in the Rust doc comment of
`crates/specify/ess-primitives/src/predicate.rs:15-66`; no page under `website/docs` lists
structured `all/any/not`, the map operators, quantifiers or `.count`. The agentplugins ESS skill
says structured forms are refused, which is false.

## Acceptance

- `website/docs/reference/predicates.md` (linked from the reference index/sidebar) lists: compact
  forms (comparison, bare path, `defined(p)`, `not …`), structured `all/any/not/none`, map
  operators (`eq, ne, lt, lte, gt, gte, any_of|in|one_of, none_of|not_in, exists|defined, truthy`,
  and the string operators of #95), quantifiers, `.count`, where each is accepted (`when`,
  `invariants`, view `filter`, `when_subject`), and what synthesis can and cannot witness after
  #93/#94.
- Every example on the page is checked by a test that runs it through validate (and synthesize
  where the page says it synthesizes), so the page cannot drift from the grammar.
- The agentplugins skill correction is recorded as a follow-up for that repository (issues are
  disabled there); this story does not edit it.
