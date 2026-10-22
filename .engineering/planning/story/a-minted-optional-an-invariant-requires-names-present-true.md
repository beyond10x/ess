---
format: aep.planning-md/3
id: story:a-minted-optional-an-invariant-requires-names-present-true
kind: story
status: draft
title: 'A broken invariant over a minted Optional names present: true'
summary: 'The ESS-CF-TARGET refusal for an invariant over an Optional minted with generated: true gains a help line, and the specify guide names {generated: true, present: true} (row 46, not a language gap).'
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Reported by adopter B (tracker row 46, 2026-10-23). An outcome sets `token: {generated: true}` on an
`Optional<Token>` field whose entity declares the invariant `defined(token)`. The interpreted run
refuses it as ESS-CF-TARGET: "the model's own outcome leaves demo.grant.Grant … violating its
invariant defined(token)". The specification is wrong, not ESS: `{generated: true, present: true}`
(#545) states that the minted value is always present
(`docs/design/minted-value-sources.md:7-24`, `:34-44`), and with it the adopter's suite passes
150/150. Nothing on the way says so: the refusal (`crates/verify/ess-conformance/src/interpret/execute.rs:363-368`,
raised at `:3211`, `:3218`) carries no hint, `website/docs/guides/specify/` never mentions
`present: true`, and `diagnostics.md` has no ESS-CF-TARGET entry.

## Acceptance

- The broken-invariant refusal, when the invariant reads an `Optional` field an outcome sets with
  `{generated: true}` and no `present: true`, carries a help line naming `present: true`; an
  interpreter test asserts it, and another asserts no such line for any other broken invariant.
- `website/docs/guides/specify/commands-and-outcomes.md` says in one line that a minted `Optional`
  value an invariant requires is declared `{generated: true, present: true}`.
- No format change.
