---
format: aep.planning-md/3
id: story:fixture-mode-applies-a-confirmed-commands-effect
kind: story
status: implemented
title: Fixture mode applies a confirmed command's declared effect to the rows its views read
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/ui.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/fixtures.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/tests/fixture_command_effect.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/src/lib.rs
- confidence: inferred
  path: crates/ui/ess-ui-test/tests/fixture_command_effect.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/app.rs
- confidence: cited
  path: crates/ui/ess-ui-tui/src/data.rs
- confidence: cited
  path: crates/ui/ess-ui/src/binding.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T01:51:06Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-10T01:51:07Z", actor: "human:timo", revision: 5}
- {from: "active", to: "implemented", at: "2026-11-11T17:04:23Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

In fixture mode a command that succeeds changes the fixture rows the way the model says it does. A
row moved out of a filtered view's state leaves that view, and every section and page that reads
the entity shows the change, with no reload and no timed script. A command whose effect fixture
mode cannot apply says so where the user acted, not just "done". Live (bound) builds do not change.

## Reproduction

Repro: `.engineering/repro/fixture-command-effect/` (`sh run.sh [ess] [playwright-core dir] [chrome]`,
output in `transcript.txt`). It has a `desk.requests.Request` entity with lifecycle `Pending` →
`Approved`, a view `PendingRequests` with `filter: state == Pending`, a command `ApproveRequest`
whose success outcome `moves: desk.requests.Request.approve`, and an inbox list page that reads the
view and offers `Approve` as a row action with a confirm. There are two fixture rows, `r1` and `r2`,
both `Pending`. `ess ui check --model spec` reports 0 errors. Fixture mode is
`ess generate ui --target react` with no `--model`: `--model` binds the app to the served HTTP
surface (`crates/ui/ess-ui-react/src/lib.rs:204`).

- verified (ess 0.92.0; `git diff --stat 0.92.0 4ab2ea1449 -- crates/ui` is empty, so the UI
  crates are the same on main): React, in headless Chrome, approving `r1` through its confirm. Before:
  rows `request-one`, `request-two`. After confirm: notice `desk.requests.ApproveRequest done`. Rows
  are still `request-one`, `request-two`, and no network request was sent.
- verified: terminal renderer, `ess ui test --path ui/desk.ui.yaml ui/approve.test.yaml` fails at
  step 7: `expected 1 rows at pages/requests.inbox/sections/list; it shows 2 [r1, r2]`. The result
  is the same with `--model spec`, whose reads also come from the fixtures.
- verified (code): the fixture adapter's `send` returns `{command, accepted: true, input}` and
  changes nothing (`templates/runtime/data.ts.tmpl:149-150`). `read` serves the generated fixture
  rows as written (`:128-132`). `runCommand` calls `invalidate()` and then notifies
  `<command> done` (`templates/runtime/actions.tsx.tmpl:100-104`), so the re-read gets the same
  rows. `src/fixtures.ts` holds `views`, `derived` (`{same_as}` / `{by_id_from, key}`, no state) and
  `scripts` only, and no command table (`crates/ui/ess-ui-react/src/fixtures.rs:271-326`, `:300`).
- verified (code): fixture channels play timed scripts only. `fixturePlayer` schedules each entry
  at its `at` and loops (`templates/runtime/live.ts.tmpl:47-76`). No command feeds a channel.
- correction to the report: section `live.effect` does have `remove_row` and `patch_row`
  (`crates/ui/ess-ui/src/model.rs:3110-3127`, `templates/runtime/live.ts.tmpl:383,421`;
  terminal `crates/ui/ess-ui-tui/src/app.rs:4600`). It cannot remove the row here because no event
  is ever sent when a command runs.
- verified (code), terminal: `FixtureAdapter::run` accepts every command and only logs it
  (`crates/ui/ess-ui-tui/src/data.rs:278-281`). Unbound, `run_command` notifies
  `<command> accepted (fixture)` and does not invalidate (`crates/ui/ess-ui-tui/src/app.rs:3670-3683`).
- inferred: the same holds for every `does:` path that reaches `runCommand` / `run_command` (forms,
  bulk actions, confirms), because they all share one adapter call.

## Acceptance

- Given the repro, after confirming `Approve` on `r1` the inbox lists only `r2`, without a reload.
  Observed by `crates/ui/ess-ui-test/tests/fixture_command_effect.rs` (terminal, `ess_ui_test::execute`
  with the model, the repro's `approve.test.yaml` steps, expecting `rows: 1` and
  `not_text: request-one`), and by the repro's `check.mjs` printing rows `["request-two…"]`.
- The same command applied through the generated React runtime: after
  `fixtureAdapter.send("desk.requests.ApproveRequest", {id: "r1"})`, a read of `PendingRequests`
  returns only `r2`, and a read of a second, unfiltered view of the same entity returns `r1` with
  `state: Approved`. Observed by `crates/ui/ess-ui-react/tests/fixture_command_effect.rs` (compiles
  the generated runtime and runs it under node via `tests/support`).
- A command whose effect fixture mode cannot apply (no model given, or a reason from Decisions)
  leaves the rows unchanged and shows the notice `<command> done in fixture mode; its effect was not
  applied: <reason>`, never just `<command> done`. React logs the same line with `console.warn`.
  Observed by `crates/ui/ess-ui-react/tests/fixture_command_effect.rs` (notice text and console line)
  and by a unit test in `crates/ui/ess-ui-tui/src/app.rs` (status line text).
- A bound build (`--model` without `--fixtures`) gives the same files as before the change. Observed
  by `crates/ui/ess-ui-react/tests/fixture_command_effect.rs`, which renders the repro bound and
  asserts that `src/fixtures.ts` has no `effects` key and that `runtime/data.ts` has no effect code.
- Reloading the page restores the fixture rows as written. Observed by the React test (a fresh module
  instance reads both rows).

## Scope

- cited: crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl
- cited: crates/ui/ess-ui-react/templates/runtime/actions.tsx.tmpl
- cited: crates/ui/ess-ui-react/src/fixtures.rs
- cited: crates/ui/ess-ui-react/src/lib.rs
- cited: crates/ui/ess-ui-tui/src/data.rs
- cited: crates/ui/ess-ui-tui/src/app.rs
- cited: crates/ui/ess-ui-check/src/model.rs
- cited: crates/ui/ess-ui/src/binding.rs
- inferred: crates/ui/ess-ui-test/src/lib.rs
- inferred: crates/edge/ess-cli/src/ui.rs
- inferred: crates/ui/ess-ui-react/tests/fixture_command_effect.rs
- inferred: crates/ui/ess-ui-test/tests/fixture_command_effect.rs
- confidence: medium (the defect and its seams are verified; how the model reaches fixture mode is a
  design choice, fixed in Decisions)

## Decisions

- Mechanism: the fixture runtime applies the command's declared outcome to its in-memory fixture
  rows. The channel `remove` effect is not used. Reasons:
  - The fixture adapter is the one seam every read and command passes through in both renderers
    (`fixtureAdapter` in `data.ts.tmpl`, `FixtureAdapter` in `ess-ui-tui/src/data.rs`).
  - React already re-reads after every command (`invalidate()`), so a changed row reaches every
    section and page that reads the entity, including after navigation.
  - The model already declares the effect (`moves:` to a target state, `sets:`), so authors declare
    nothing twice.
- Why not the channel: `remove_row` already exists, but fixture channels are fed only by timed
  scripts. A command would need a channel, a `live:` on every section that reads the view, and a map
  from command to event, which again only the model has. And because the re-read after the command
  serves the unchanged fixture rows again, the removed row comes back when the section mounts again.
- The effect table comes from the model, computed by `ess-ui-check` from the compiled model, as the
  binding is. It is not written in the UI document. Fixture mode gets the model through
  `ess generate ui --target react --model <spec> --fixtures`, because `--model` alone means bound
  (live) today and stays that way. `ess ui test --model` already reads fixtures and carries the
  table as it is. `ess ui run --tui` (`--fixtures` conflicts with `--model`) is not part of this
  story.
- What gets applied: exactly one success outcome (no `error:`) with `moves:` and/or `sets:` on the
  view's source entity. The instance is the input field the outcome's `instance:` names, matched to
  the view's identity field. A row whose view `filter:` no longer holds on the changed fields leaves
  that view; otherwise the row is patched.
- Not applied, and reported: no model, more than one success outcome, `creates:`, or a filter that
  reads fields the outcome does not set and the row does not carry.
- Fixture state lasts for the session in memory, and a reload starts from the fixture files.
  Fixture scripts and `fixtures.derived` keep their meaning. A `by_id_from` view reads the changed
  rows.

## Source

Intake from a consumer UI round, 2026-11-10. Reported on ess 0.90.0 and reproduced on 0.92.0 / main
`4ab2ea1449`.
