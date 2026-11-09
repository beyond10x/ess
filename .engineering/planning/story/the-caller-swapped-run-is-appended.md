---
format: aep.planning-md/3
id: story:the-caller-swapped-run-is-appended
kind: story
status: draft
title: The caller-swapped run of a caller-written set effect is appended, and a test fails without it
tags:
- defect
- severity-low
- zero-open-issues
relations:
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 4
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

- A test in `crates/verify/ess-conformance/tests/caller_arranged_rows.rs` asserts that a scenario the
  arranged-callers reading adopts carries a second run under swapped callers (a send of the arranging
  command as the other caller after the first run's steps), and goes red when the append at
  `caller.rs:993` is removed.
- Where the append cannot add steps for a model (an early return in `append`), the suite carries a
  note naming the scenario and the reason, as `unswapped` does for an exhausted one; if it cannot add
  steps for any model, the loop is removed and the module docs say so.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/caller.rs`
- cited: `crates/verify/ess-conformance/tests/caller_arranged_rows.rs`
- inferred: `crates/verify/ess-conformance/src/synthesize/caller/arranged.rs`

confidence: medium
