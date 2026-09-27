---
format: aep.planning-md/2
id: story:input-guard-overlap-precedence
kind: story
status: draft
title: An input-guarded refusal that overlaps an accepting branch is ordered or refused
refs:
- provider: github
  reference: beyond10x/ess#178
relations:
- decomposes: epic:retrofit-findings-round-3
revision: 1
---
## Scope

Either a stated precedence (an input-guarded refusal wins over an accepting branch it overlaps, witnessed at the overlap point) or `validate` refuses the overlap naming both outcomes.

## Acceptance

The #178 specification either synthesizes a scenario sending `{ticket_id: "", open: false}` that requires `id-required`, or is refused naming `closed` and `id-required`.
