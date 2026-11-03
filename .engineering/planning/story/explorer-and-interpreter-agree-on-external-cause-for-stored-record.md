---
format: aep.planning-md/3
id: story:explorer-and-interpreter-agree-on-external-cause-for-stored-record
kind: story
status: implemented
title: The explorer and the interpreter agree on an external cause arranged for a stored record
tags:
- adopter-report
- defect
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/explore_held_state_external.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-03T05:57:21Z", actor: "human:timo", revision: 8}
- {from: "proposed", to: "active", at: "2026-11-03T05:57:22Z", actor: "human:timo", revision: 9}
- {from: "active", to: "implemented", at: "2026-11-03T10:46:40Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}}
---
## Outcome

The Go explorer and the interpreted target expect the same branch when an `external:` cause is
arranged for a stored record whose held state another branch also answers.

## Evidence

An adopter on ess 0.56.0: once an `external:` unauthorized cause is arranged for a stored record,
the explorer and the interpreted target disagree (ESS-CF-OUTCOME). On `main`,
`crates/verify/ess-conformance/src/go/explore.go:1522-1537` offers an eligible external branch
beside `wrong_state`, sorted ahead of it, while the interpreter applies held state before
externals (precedence steps 4 and 6). Related:
`story:refusal-above-held-state-branch-validates`, which may change that precedence.

## Acceptance

- A test runs the Go explorer and the interpreted target over one specification with an external
  branch and a held-state branch on the same command; both expect the same branch for every
  arranged cause.

## Scope

Derived 2026-11-03 by `aep:story-scoper` for wave W2 on b89faaa41. Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance` (Go and TypeScript explorers) — cited
- **Files:** `crates/verify/ess-conformance/src/go/explore.go:2095` (a holding held-state branch returns eligible externals beside itself) and `:1994` (`wrongState` does the same) — cited. The story's `explore.go:1522-1537` now holds `exploreAt` (stale).
- **Also likely:** `crates/verify/ess-conformance/src/ts/explore.ts:1721` (same return) — inferred; `explore_before_row.rs` requires both languages to agree
- **Symbols:** `exploreDecideRow` (`explore.go:1903`), `exploreEligible` (`:2274`) — cited
- **Also likely:** a new target `crates/verify/ess-conformance/tests/explore_held_state_external.rs` with its fixture — inferred (name a suggestion)
- **Documents:** none edited; the order is `docs/design/cross-record-and-stored-field-guards.md:827-854` (held state at step 4, plain externals at step 6) and `:856-863` (the ess/27 rule) — cited
- **Reference, not edited:** the interpreter places plain externals in `Phase::Accepting` (`crates/specify/ess-domain/src/command/precedence.rs:543-549`) and held-state shapes in `Phase::HeldState` (`:532`) — cited
- **Confidence:** medium — code and normative order read, nothing run
- **Would collide with:** any unit editing `exploreDecideRow` or `exploreChoices` in `explore.go`, or `heldBranch` in `explore.ts`. The #235 pin `tests/explore_arranged_external_stranded.rs` covers externals beside the default and wrong-state answers, so the fix gates on a held branch answering — inferred
- **Not established:** the downstream adopter's disagreement is not reproduced (report on 0.56.0); whether wrong-state plus a holding held branch is the same defect.
- **Size:** medium, about 40 lines in the explorers and about 150 for test and fixtures; no format version — inferred
- **Targeted run:** ess-conformance `--test explore_stored_rows`, `explore_before_row`, `explore_external`, `explore_arranged_external_stranded`, `adversary_explore_order_pass1`, `explore_guards_first`, plus the new target (go and node on PATH) — inferred

## Delivered

W2 (D408), 2026-11-03: the generated explorer (Go and TypeScript) no longer expects an arranged `external:` branch where a branch the stored row selects holds, so it agrees with the interpreter's precedence. Unit ab6d56ac01 (with 15ce4749c6); merged 3cc17ed596; frozen-byte tables re-pinned for the unit's fixtures in 446c625b9f. Adversary pass 1: no finding on this story.
