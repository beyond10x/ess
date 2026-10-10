---
format: aep.planning-md/3
id: story:diff-rates-narrowed-acceptance-breaking-for-callers
kind: story
status: implemented
title: verify diff rates an added refusal, a required input and a widened refusal guard breaking for callers
tags:
- defect
refs:
- provider: github
  reference: beyond10x/ess#514
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-09T01:14:06Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-09T01:14:06Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-09T01:14:07Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`ess verify diff --compatibility` rates a change that narrows what an existing caller may send as
breaking for callers: an added refusal, an added required input, a refusal guard widened.

## Evidence

https://github.com/beyond10x/ess/issues/514 (ess 0.56.0): all three are rated `unknown` for callers and readers, so
`--fail-on breaking` exits 0. `dimensions` in `crates/verify/ess-diff/src/compatibility.rs`
rates every command change other than added, removed and `OutcomeCompensatesChanged` as
`[U, U, C]`; `CommandChange::OutcomeAdded` carries only the outcome's name, so the rating cannot
tell a refusal from an accepting branch.

## Acceptance

- An added outcome with `error:` is breaking for callers; an added accepting outcome stays
  unknown.
- An added input whose type is not `Optional` is breaking for callers; an added `Optional` input
  keeps its current rating.
- A refusal guard changed so the new guard holds wherever the old one did (decided only where the
  existing `satisfiable` machinery decides, else unknown) is breaking for callers.
- The issue's three revisions give `breaking` for callers and `--fail-on breaking` exits non-zero;
  the change carries enough of the outcome that a reader of the delta re-derives the answer, and
  the delta format version is decided explicitly (a new `ess-diff/N` if the persisted change
  gains a field).
- Every existing `ess-diff` and `ess-cli` diff expectation that moves is named in the PR with why.

## Scope

`crates/verify/ess-diff/src/{compatibility,change,diff}.rs`, `crates/verify/ess-diff/tests/`,
`crates/edge/ess-cli/tests/verify_diff*`.
