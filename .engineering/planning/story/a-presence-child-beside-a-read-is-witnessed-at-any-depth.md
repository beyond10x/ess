---
format: aep.planning-md/3
id: story:a-presence-child-beside-a-read-is-witnessed-at-any-depth
kind: story
status: draft
title: A presence child beside a read of its field is witnessed at any depth and under not
summary: A read isolated beside a presence sibling under not, or a nested presence child, is still refused with ESS-SYNTH-022 (turn 1 lane-2 adversary, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:zi-w1-lane2-adv-1
revision: 1
---
## Need

Turn 1 made isolation three-valued for a presence child beside a read of the same field at the top
of a connective (story:a-connective-child-masked-by-an-earlier-branch-is-not-asked-for,
beyond10x/ess#729): `all: [defined(prompt), prompt starts_with "hold "]` is witnessed by the input
that leaves `prompt` out. Its adversary (review-result:zi-w1-lane2-adv-1) found two shapes that are
still refused with `ESS-SYNTH-022` on a goal that needs the field absent and a read of it decided:

1. the read child isolated beside a presence sibling that must be false, under `not`:
   `{not: {all: [{prompt: {defined: true}}, {prompt: {starts_with: "hold "}}]}}`, and the integer
   form `not: {all: [defined(minimum), minimum > 10]}`. The repository's own `guard-negate`
   mutants of `mutation-identical-answer.yaml` reach it; `mutation-manifest-4-scoped.json` pins
   the refusal (`guard-negate/desk.entry.Open/leading-hold`, `/trailing-hold`).
2. a nested presence child, `any: [level == 1, {all: [defined(minimum), minimum > 10]}]`, where
   `isolating` drops the sibling but the `swapped` mutant clause still asks the read to be decided
   (`crates/verify/ess-conformance/src/synthesize.rs`, `isolating`, `swapped`).

The adversary's three red cases are kept in the turn scratch as `lane2-backlog-red-cases.rs`.

## Acceptance

- Both shapes above are witnessed, not refused, and the interpreted target passes every
  synthesized scenario for them.
- The `guard-negate` rows of the 4-scoped mutation manifest and report move from refused to
  witnessed.
- `website/docs/reference/predicates.md` states the rule at any depth and under `not`.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs` (cited by the review)
- `crates/verify/ess-conformance/src/witness.rs` (cited by the review: `unsatisfiable`)
- `website/docs/reference/predicates.md` (cited by the review)
