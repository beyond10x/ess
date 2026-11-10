---
format: aep.planning-md/3
id: story:the-caller-swapped-run-is-appended
kind: story
status: active
title: The caller-swapped run of a caller-written set effect is appended, and a test fails without it
tags:
- defect
- severity-low
- zero-open-issues
relations:
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/caller.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/caller/arranged.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/caller_arranged_rows.rs
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:47:10Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:47:12Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Outcome

The caller-swapped run of a scenario arranged by callers chosen for its rows is appended to that
scenario, and a test fails when it is not.

## Problem

The 2026-10-19 adversarial review of the integration branch (finding T6): with the swapped append in
`crates/verify/ess-conformance/src/synthesize/caller.rs` removed (`arranged_by_callers`, the
`for id in adopted` loop, `append(ir, (Some(&id), scenario), again, identities)` at `:993`), the
suite synthesized from the tokens fixture `TOKENS` of
`crates/verify/ess-conformance/tests/caller_arranged_rows.rs` is byte-identical.

Reproduction on 0.92.0: the `TOKENS` model written to `tokens.yaml`, then
`ess verify conform synthesize --path tokens.yaml --out suite.yaml` (2 scenarios, 0 refusals).
`IssueToken/outcome/issued`, sent through the ordinary reading, carries its swapped run (a second
`IssueToken` under the other caller). `RevokeOwned/outcome/revoked`, which only the arranged-callers
reading witnesses, carries one run: four `IssueToken` sends (three as one caller, one as another),
then `RevokeOwned` with the owner selected and once more with an owner nobody holds. No `IssueToken`
is sent again under swapped callers, and no note says the swapped run was left out.

So the loop reaches no `append` that adds steps for this model. Not yet traced which of these holds
(code reading, `caller.rs`):

- the scenario is not among `adopted` because an earlier reading already holds its id (`:962-968`);
- the swapped synthesis has no scenario of that id (`:986-990`);
- `append` returns `Ok(())` without adding steps (`:1360-1366`): for example where
  `shared_selection` (`:1579`) finds the swapped run's `RevokeOwned` sending a `user_id` literal the
  first run already carries, which is checked before `identities.drawn` redraws it. This early return writes no note, so the left-out run
  is silent where an exhausted one is named (`unswapped`, `:994`).

## Acceptance

- `cargo test -p ess-conformance --locked --test caller_arranged_rows` passes a new test
  `the_arranged_callers_reading_appends_its_swapped_run`: in the suite synthesized from `TOKENS`,
  `RevokeOwned/outcome/revoked` holds, after the first run's two `RevokeOwned` sends, a second run
  of four `IssueToken` sends, each under the other caller than the first run's send in the same
  position, followed by its `RevokeOwned` sends.
- With the `append` call at `caller.rs:993` removed, that test fails (the mutation is run once and
  reverted; its output is recorded in the story's evidence).
- The same target passes a new test `a_swapped_run_left_out_is_named`: on a model whose swapped
  run's `instances:` filter pins a stored field to an input that has no fresh value to draw, so
  `append` returns before adding steps, the synthesis notes hold one note naming that scenario and
  the reason the run was left out, as `unswapped` names an exhausted one.
- `cargo test -p ess-conformance --locked --test caller_arranged_rows` passes every existing test
  with no assertion changed.

## Decisions

- The swapped loop is kept and made to append, not removed: the outcome asks for the swapped run,
  `IssueToken/outcome/issued` shows the same swap appending through the ordinary reading, and the
  leading cause is `append` judging `shared_selection` (`:1366`) on the literals before
  `identities.drawn` redraws them; the implementor traces the three candidates above first and
  fixes the one that holds.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/caller.rs` (`arranged_by_callers`, `append`, `shared_selection`, `unswapped`)
- cited: `crates/verify/ess-conformance/tests/caller_arranged_rows.rs`
- inferred: `crates/verify/ess-conformance/src/synthesize/caller/arranged.rs`

confidence: medium

The story re-pins the suite-bytes rows of models whose arranged-callers scenarios gain a swapped
run, and proposes its `CHANGELOG.md` sentence.
