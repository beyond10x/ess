---
format: aep.planning-md/3
id: review-result:zi-w1-metric-adv-1
kind: review-result
status: active
title: Turn 1 adversary pass 1 on story:metric-error-state-on-failed-read
relations:
- reviews: story:metric-error-state-on-failed-read
revision: 2
---
```
unit: story:metric-error-state-on-failed-read
verdict: weak
new failing cases: 1 (crates/ui/ess-ui-react/tests/metric_states.rs::adversary_a_metric_over_a_view_without_a_fixture_names_404)
mutants survived: none run
```
cases: executed 25→27, red 1 · origin: introduced 1 / pre-existing 1 / undecided 0 · wrote-outside-worktree: 6 · needs-coordinator: the fix belongs in `data.ts.tmpl`, which is outside this story's scope.

**Covers:** commit 76984b48a8 plus my uncommitted cases in `~/.local/state/worktree/trees/b10x/ess/zi-t1-5`.

**Diff stat:** `crates/ui/ess-ui-react/tests/metric_states.rs | 46 +++` (test file only, uncommitted).

**Main finding:**
- **The failure:** when the read fails with a `DataError` whose message does not start with the status code, the metric shows `Error` instead of the code.
- **The run:** `build.sh adv/run1.txt <tree> test -p ess-ui-react --locked --test metric_states adversary_`, exit 101, assertion `the alert names the status 404: Error`.
- **Who reaches it:** the default adapter of every generated project. `fixtureAdapter` is `current` at `data.ts.tmpl:528` and fails a view that has no fixture with `DataError("no fixture answers <view>", 404)` (`data.ts.tmpl:130`). The served adapter does the same for a view no component serves (`:422`, 404) and for a missing path value (`:383`, `:387`, `:401`, 400).
- **Cause:** `useRead` keeps only `error.message` (`data.ts.tmpl:693`) and drops `DataError.status`. So `metric.tsx.tmpl:40` can only search the message with a regex.
- **Fix (not applied):** carry the numeric status in `ReadState` and draw that.
- **Severity:** this is not in the acceptance, which only names a served 501, so it is a warning and a backlog item. It does break the story's Expected line, "names the status".

**Suite run** (after the cases existed): `--test metric_states --test live_composites --test live_binding`, output in `adv/run2.txt`, exit 101.
- `live_binding`: 11/11 pass.
- `live_composites`: 9/9 pass.
- `metric_states`: 6/7 pass; only my 404 case is red.

**Passing case I left in place:** `adversary_a_refresh_after_an_answer_keeps_the_value_then_fails_to_an_alert` is green. No existing test covered this: a refresh after an answer keeps `2` with no busy marker, and a later 500 turns into an alert with no `0`.

**Backlog only, pre-existing, not tested:**
- **TUI renderer:** `crates/ui/ess-ui-tui/src/view.rs:2040-2044` has no failed state for a metric; when `reader_result` is None it draws the null value.
- **List/table section:** `core.tsx.tmpl:448` still shows "Could not load." without the status, as the story already says.

**Attacked and held:**
- Hook order across the early returns: `useUiPath` now runs before both returns.
- TypeScript narrowing through the `unanswered` alias: `tsc --strict` on a copy of the pattern exits 0.
- An aggregate over a failed read: the computed 0 is overridden by the alert.
- A `from:` value that is undefined while the read failed: it falls back to the read, so the alert is the intended result.
- A refresh flashing the loading placeholder: it does not.
- `idle`: a metric with its own `reads` never gets it, and a section that is idle draws its own state instead of its children.

**Paths written outside the worktree:**
- `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t1/metric-error-state-on-failed-read/adv/{narrow.ts,append.rs,run1.txt,run2.txt}`
- `~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/targets/A/tmp/ess-ui-react-metric-states/{adversary-no-fixture,adversary-refresh}`

```findings
- file: crates/ui/ess-ui-react/templates/runtime/composites/metric.tsx.tmpl
  line: 40
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a failed read whose DataError message does not lead with the code (default fixture adapter 404, unserved view 404, missing path value 400) shows "Error" because useRead drops DataError.status at data.ts.tmpl:693
- file: crates/ui/ess-ui-tui/src/view.rs
  line: 2040
  category: contract-drift
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: the TUI metric has no failed or loading state for a failed read, so the two renderers now disagree; outside this story's scope
```
