---
format: aep.planning-md/3
id: story:go-explorer-draws-read-only-commands
kind: story
status: draft
title: The Go explorer draws read-only commands on a supplied subject
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

The generated explorers (Go `Explore`, and the TypeScript explorer beside it) draw a command whose
outcomes include a `preserves:` outcome on a supplied subject, and check that the subject and every
view are unchanged after it, instead of excluding the command from random sequences.

Closes https://github.com/beyond10x/ess/issues/536.

## Evidence

- https://github.com/beyond10x/ess/issues/536 (ess 0.59.0): in a lifecycle model where most
  commands have an accepted no-op branch (pause when paused, logout when logged out), the explorer
  excludes most of the state machine; a planted defect (unpause from idle leaves the row idle) is
  caught by the fixed suite and invisible to the explorer.
- An earlier adopter hardening OAuth specifications on ess 0.56.0: two specifications had their
  read-only commands excluded.
- Still on `origin/main` (260a3d1e3, read 2026-10-13): `exploreEffectRefusal`
  (`crates/verify/ess-conformance/src/go/explore.go:852-866`) admits only `creates` from observed
  and `moves`/`updates` from supplied; every other effect, `preserves` included, is an exclusion
  reason (called at `explore.go:1023-1027`). The model update after a step
  (`explore.go:2060-2106`) has no `preserves` arm, so drawing the command needs one: the record is
  found and left as it is, and `exploreViews` (`explore.go:2108`) then compares the views.

## Acceptance

Each line names the failing test written first.

- `crates/verify/ess-conformance/tests/explore.rs`,
  `a_preserving_command_on_a_supplied_subject_is_drawn`: the explorer generated for a model with
  `Pause` (moves `Active -> Paused`) and `PauseAgain` (`preserves:` on a `Paused` row) lists both
  commands as explored, `Excluded` names neither, and a run reaches the preserving outcome.
- Same file, `a_target_that_changes_a_preserved_subject_fails_exploration`: a target whose
  preserving command moves the row reports a disagreement naming the step and the view.
- Same file, `the_typescript_explorer_draws_a_preserving_command` (or the TypeScript twin in
  `crates/verify/ess-conformance/src/ts/explore.test.ts`, whichever the existing parity tests use):
  the TypeScript explorer agrees.
- A `preserves:` on an observed subject, and `deletes:`, stay excluded with today's reason text.

## Scope

- cited: `crates/verify/ess-conformance/src/go/explore.go` (`exploreEffectRefusal`, the model
  update after `explorePerform`); `crates/verify/ess-conformance/tests/explore.rs`.
- inferred: `crates/verify/ess-conformance/src/ts/explore.ts` (its effect check and model update
  mirror the Go file); the explorer section of the generated `README.md` if it lists exclusion
  reasons; CHANGELOG `[Unreleased]` "Fixed".

## Plan

- depends_on: none. First of the three explorer stories: it, then
  `story:explorer-model-applies-selected-row-sets-and-draws-decimals`, then
  `story:explorer-excludes-a-refused-step-not-the-command`; all three edit `go/explore.go` and
  `ts/explore.ts`, so they land one after another (or as one unit).
- du estimate: 8G (ess-conformance `--test explore`, Go toolchain cache under the tree's
  `target/go-build`).
- adversary: yes (a model change the explorer judges every target by).
