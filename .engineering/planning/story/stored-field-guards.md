---
format: aep.planning-md/2
id: story:stored-field-guards
kind: story
status: active
title: An outcome can be guarded by the addressed entity's stored fields
relations:
- serves: vision:O2
revision: 3
---
## Outcome

"Express parcels over 20 kg are refused at dispatch" is a checked ESS guard over the addressed
entity's stored fields.

## Why

GitHub issue beyond10x/ess#75, rule 1. The design is
`docs/design/cross-record-and-stored-field-guards.md` as reworked in PR #90 (merged into this
wave's integration branch): `when_subject: {predicate: …}` beside today's `{field, equals}`,
refusal branch taking its subject from sibling branches, goal-directed input choice through `sets:`
input mappings, a typed fact source over the arranged row, ess/8.

## Acceptance

- Everything the design's acceptance and format sections require, including ess/8 as a new source
  major with unchanged bytes for models that do not use the construct, the new IR variant beside
  the unchanged one, and `cargo xtask schema`.
- Red first: the parcels example from the design validates, synthesizes a refusal scenario
  (21 kg, Express) and a success scenario (20 kg), and a mutant that ignores the stored weight
  fails the suite.
- Predicates in `when_subject` may use every form the #93/#94/#95 stories admit, including
  `defined` over an Optional stored field (the design's open question 2 is answered by #93).
- A `creates` outcome copies declared command input into the created entity's fields where the
  design's `sets:` mapping says so (the SYNTH-005 prerequisite reported in #96).

## Out of Scope

Rule 2 (non-overlap, cross-record constraints) — out of scope per the design. The duplicate-identity
refusal on `creates:` is left for a later story.
