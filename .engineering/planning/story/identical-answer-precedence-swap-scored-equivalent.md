---
format: aep.planning-md/3
id: story:identical-answer-precedence-swap-scored-equivalent
kind: story
status: draft
title: A precedence swap of two branches with identical answers is scored equivalent
tags:
- defect
refs:
- provider: github
  reference: beyond10x/ess#517
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A precedence swap of two branches whose observable answers are identical (same error, same or no
payload, no state change, no sets, no events) is scored `equivalent` (`ESS-MUTATE-005`), not
`survived`.

## Evidence

https://github.com/beyond10x/ess/issues/517 (ess 0.56.0): `prompt-none-with-other-values` and `prompt-other-values-then-none`
both refuse with `oidc.provider.InvalidRequest` and no payload; the swap is scored survived
though no implementation can kill it. `precedence_sites` in
`crates/verify/ess-conformance/src/mutate.rs` generates the site whenever both branches are
guarded by input alone and agree on having an error; `unsatisfiable_guard` marks a swap
equivalent only when the two guards never overlap.

## Acceptance

- The issue's pair is scored `equivalent` with a reason naming the shared answer.
- Two refusals differing in error, payload or effect keep their current scoring
  (https://github.com/beyond10x/ess/issues/472's case still survives or is killed as before).
- The reason is recorded so a reader of the manifest tells "guards never overlap" from "answers
  identical"; if that changes what the persisted `unsatisfiable_guard` field means, the manifest
  format moves and an old-reader test says so.

## Scope

`crates/verify/ess-conformance/src/mutate.rs`, `crates/verify/ess-conformance/tests/mutate*`.
