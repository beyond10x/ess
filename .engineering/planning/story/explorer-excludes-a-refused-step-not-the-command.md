---
format: aep.planning-md/3
id: story:explorer-excludes-a-refused-step-not-the-command
kind: story
status: draft
title: An unsupported exploration step excludes the command only in that state
relations:
- depends_on: story:explorer-model-applies-selected-row-sets-and-draws-decimals
revision: 1
---
## Outcome

When a target answers one exploration step `ErrUnsupported`, the explorer stops drawing that
command only in the held state the step was drawn in (per command and subject state), counts the
refusal with its reason, and keeps drawing the command elsewhere, instead of excluding it for the
rest of the run.

Closes https://github.com/beyond10x/ess/issues/537.

## Evidence

- https://github.com/beyond10x/ess/issues/537 (ess 0.59.0): a target that can drive a command in
  every state but one (a callback that exists only while another request is waiting) loses the
  command entirely, and every state reachable only through it; in the reporting run no sequence
  left the initial lifecycle state.
- Still on `origin/main` (260a3d1e3, read 2026-10-13): the sequential run loop calls
  `p.exclude(command.name, "the target does not expose it: "+unsupported.reason)` on the first
  `ErrUnsupported` from `explorePerform`
  (`crates/verify/ess-conformance/src/go/explore.go:2603-2606`); `explorePlan.exclude`
  (`explore.go:1133`) drops the command from `p.commands`. An external arrangement already keys
  its refusal per command and outcome (`p.unarrangeable[command.name+"/"+step.external]`,
  `explore.go:2592-2596`), which is the precedent for a narrower key. The TypeScript explorer
  excludes the same way (`ts/explore.ts:2234`).

## Acceptance

Each line names the failing test written first.

- `crates/verify/ess-conformance/tests/explore.rs`,
  `an_unsupported_step_excludes_the_command_only_in_that_state`: a target that answers
  `ErrUnsupported` for `Answer` only while the row is `Ringing`, and drives it in `Waiting`; the run
  reaches an outcome of `Answer` from `Waiting`, and the report lists one refusal for
  (`Answer`, `Ringing`) with the target's reason and its count.
- Same file, `a_command_unsupported_in_every_state_is_still_excluded`: a target that refuses
  `Answer` everywhere ends with `Answer` in `Excluded`, as today, so `AllowExcluded` and the
  coverage gate read the same.
- Same file, `the_typescript_explorer_narrows_an_unsupported_step` (or the `ts/explore.test.ts`
  twin): the TypeScript explorer agrees.
- A view read refused `ErrUnsupported` keeps today's per-view exclusion (`explore.go:2217`).

## Scope

- cited: `crates/verify/ess-conformance/src/go/explore.go` (the run loop at `:2603-2606`,
  `explorePlan.exclude`, the report's exclusion list); `crates/verify/ess-conformance/tests/explore.rs`.
- inferred: `crates/verify/ess-conformance/src/ts/explore.ts` (`:2234`); the explorer's printed
  summary and its `Exclusion` type if a per-state refusal becomes a new field (a new report field
  is an additive output change; say so in the CHANGELOG); the concurrent explorer
  (`ExploreConcurrent`) if it shares the loop.
- Design call for the implementor, recorded in the PR: the key is (command, held state of the
  addressed row), with "no row" its own state for a creating or unknown-instance step.

## Plan

- depends_on: `story:explorer-model-applies-selected-row-sets-and-draws-decimals` (same files;
  last of the three explorer stories).
- du estimate: 8G (ess-conformance `--test explore`, Go cache under `target/go-build`).
- adversary: yes (changes what an exploration report claims about coverage).
