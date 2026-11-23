# Report review: round R-06

Reviewer: fresh agent, read-only, nothing run (no build, test, server, browser). Sources recomputed
from: `rounds/R-06/round.json` (14 run records with `timing.load`, 578 scenario entries),
`rounds/R-06/probe.json`, `rounds/R-05/round.json`, `baseline.json` as committed with the round and
as committed before it (`git show ba05fcc25^:website/ux/baseline.json`), the scenario cards, the
findings, the commits `fc7300d1d` (change 1) and `ba05fcc25` (the record). No raw per-run
`raw.json` and no recordings are in the tree, so anything below "not recomputable" has no rawer
source than `round.json`. The round has no README and no questionnaire; no person took part.

## Verdict

**Not fit to send as is.** Default action: make the four edits below (all are prose, a column or a
record line; none needs a new measurement), then send.

1. `notes.md` and the report's closing notes say "the runs table above holds their times and
   loads". The table holds no load column and the report prints no load anywhere. Add the column
   (`timing.load` is in `round.json`) or reword the sentence. See T1.
2. The report names two phone scenarios that moved. Three moved (`install-linux-x86_64/mobile-390`
   too), and both phone keyboard runs fell by about 50 key presses; neither is mentioned. See § 1.4.
3. UXF-025 is closed on three gate runs that do not show the round's change did anything. Say so in
   the record, or keep it open until a run does. See § 1.5.
4. `baseline.json` still scopes the `gate` lane to the landing page and the visualise page, which
   the gate no longer measures. See T3.

## § 1 Are the numbers right and meaningful?

### 1.1 Recomputed headline numbers: all agree

| number in the report | recomputed from | result |
| --- | --- | --- |
| coverage 139 of 139 pages, 556 of 556 configurations, 22 of 22 task runs, 417 of 417 measured-only | `scenarios` keys (139 page keys x 4 configurations; 22 task keys) and `measured_only` | same |
| gate 1, 2, truncation, reduced motion: 0 | sums of `layout.spills`, `always_visible.failures`, `layout.truncated_texts`, `motion.running` over the 556 page entries | same, 0 each |
| gate 4: 18 of 18 pointer runs done, 3 over the limit | the 18 pointer entries' `screen_only.actions` against `expect.max_actions` in each `scenarios/*/scenario.json` | same: `page-a-view` mobile 11 (5), `theme-dark-and-back` desktop 3 (2) and mobile 4 (2); every other run is at or under its limit |
| gate 5: 23 serious/critical nodes on 1 page | sum of `axe.serious_critical_nodes`; the only page with a non-zero entry is the visualise page: 1 (desktop dark), 10 (desktop light), 1 (phone dark), 11 (phone light) | same, 23 on 1 page |
| gate 5 by rule 18 / 2 / 2 / 1 | `gates.gate5_accessibility.by_rule` (sums to 23); the phone-light worst shot says 8 + 1 + 1 + 1 = 11, which fits | same |
| per-viewport and theme sums of axe nodes and rules, layout shift | grouped sums of the 556 page entries | same; axe nodes 1, 10, 1, 11 against R-05's 1, 12, 1, 13; layout shift 0.0135 and 0.0148 against 0.0142 and 0.0281 |
| every task-scenario row of the metrics table (220 rows, three columns each) | `round.json`, the committed `baseline.json`, R-05's `round.json`, compared row by row by script | all 220 rows equal in all three columns |
| trend row R-06: pointer actions 63, steers 18, axe 23 / 1 page, over limit 3 | sums over the 18 pointer entries; R-05 gives 66 and 21 | same; the fall of 3 actions and 3 steers is one of each in three phone scenarios (§ 1.4) |
| runs table: wall times, workers, known and new counts | `runs[].timing` | same for all 14 runs |
| ten slowest tests, and the three slowest pages' phases | `slowest`, `slowest_pages` | same |

### 1.2 Gate wall times and loads (three runs)

| run | wall s in report | wall s in `round.json` | 1-minute load before / after (`round.json`) | other loads (5, 15 min) before |
| --- | --- | --- | --- | --- |
| gate-1 | 21.2 | 21.2 | 47.78 / 47.18 | 40.65, 31.07 |
| gate-2 | 18.9 | 18.9 | 40.53 / 49.56 | 40.84, 38.67 |
| gate-3 | 13 | 13 | 29.04 / 30.83 | 38.29, 42.87 |

No difference in a wall time. The loads are not in the report (T1). All three started and ended
under the host's line of 60, and all three have 0 new failures and 1 known failure (the search
control, UXF-001). What the three runs can and cannot say is in § 1.5.

### 1.3 Ratchet summary

Recomputed by comparing the round's 3,556 values with the baseline committed before the round
(same scope as the tool: all 578 scenarios, no new value):

- improved: 73 (2 axe rules and 2 axe nodes on the landing page's light entries, 25 layout-shift
  values, 44 task values: reading load, and for the phone runs actions, looks, steers, glance);
- worse beyond tolerance: 0;
- worse inside tolerance: 3: `diagnostic-type-009/desktop-1280 reading_load.max` 264 against 256
  (limit 12.8), `target-process/desktop-1280 reading_load.median` 214 against 213, and
  `target-process/mobile-390 reading_load.median` 114 against 113. These equal R-05's values; the
  baseline held lower ones from earlier rounds.
- equal: 3,480. No value is higher than in R-05 (0 of 3,556).

The tool's own log (`ratchet-2.log` in the session scratch, not in the tree) says the same: "took 73
improvement(s) ... 0 worse and 0 dropped under decision none", with the three tolerance notes.

The report does not carry this summary (T2): it says "No value got worse beyond its tolerance"
and nothing about the 3 values that are worse inside it or the 73 taken in. The statement is true.
It is incomplete: the first of the three is 8 words (3 %) above the best value ever held.

### 1.4 The two phone scenarios, and a third

| scenario (phone) | actions R-05 to R-06 (card limit) | steers | glance median | reading load max / median |
| --- | --- | --- | --- | --- |
| `first-conformance-run` | 4 to 3 (3) | 2 to 1 | 3 to 1 | 108 to 92 / 96 to 76 |
| `limitations` | 4 to 3 (3) | 2 to 1 | 3 to 1 | 108 to 87 / 87 to 85 |
| `install-linux-x86_64` (not named by the report) | 4 to 3 (4) | 2 to 1 | 3 to 1 | 120 to 120 / 96 to 76 |
| keyboard `first-conformance-run` / `install-linux-x86_64` (not named) | 131 to 81 / 129 to 80 key presses | 2 to 1 each | | |

- The report's claim for the two named scenarios is right, and the action counts match the change's
  target (4 to 3).
- Missed: `install-linux-x86_64/mobile-390` moved the same way, and the card's limit is 4, so it is
  within limit either way. Its asset list (UXF-013 `round_values`) does not name it, and no prose
  says why it moved. Both phone keyboard runs fell by about 50 key presses (UXF-015's metric, "tens
  of key presses to a sidebar link", open): nothing in the report says so. The trend row's "63"
  includes all three.
- The desktop reading load fell in 7 of the 9 desktop pointer scenarios too (the two that did not are
  `target-process` and `theme-dark-and-back`; for example
  `diagnostic-type-009/desktop-1280` median 217 to 194, `go-runner/desktop-1280` median 205 to 182).
  The notes tie the fall to the docs introduction's link change. The triage's own caveat, that the
  merge of `main` can move a reading load or an action count without a change of this round, is
  dropped from the report. The cause of the desktop fall is not recomputable from the record.

### 1.5 UXF-025 and UXF-023: closed with an asset that fails before and passes after?

Skill § 5: no S1 or S2 closes without an asset that fails before the fix and passes after.

- **UXF-025 (S2, closed).** The asset is the L1 test of the gate lane's pages and browser-test count,
  plus the 120 s `globalTimeout`. The commit message reports it red first (count 13 against at most
  9, the coverage test, the budget-line test) and green after, so the asset itself meets the clause.
  But it asserts the lane's configuration, not the finding's metric (the step's wall time under
  load), and the metric did not move because of the change:
  - The record's closing criterion is "gate runs at a load of 60 or less hold the budget". That
    held before the change as well: R-04 `gate-12` 35.1 s (load 53 to 60) and `gate-13` 17.9 s
    (load 20 to 26), R-05 `gate-8` 19.5 s (load 60.06), all passing (the record's own tables).
  - R-06's three runs took 21.2, 18.9 and 13 s at loads 48, 41 and 29, with a lane of 9 browser
    tests in place of 13 and without a third of the test-seconds. They are not faster than
    `gate-8`, so they do not show the change helped; they show a quiet host passes.
  - Above load 60, the only evidence the change helps is the probe, and it is weak: three pairs, B
    over A 0.81, 1.23 and 0.31 (recomputed from `probe.json`, same), loads 130 / 98, 81.5 / 108 and
    67 / 61 (before), so the pairs are not at equal load; the third pair's A ran 145.5 test-seconds
    and B 39.2 for 4 fewer tests; the probe ran on the R-05 engine commit `f9c03e8394c9`, not on
    R-06's.
  - So the closure rests on the host's working rule of 2026-11-23 (judge a budget at load 60 or
    less), not on the change. That can be a fair closure, but the record and the notes say "closed
    on them", which reads as if the three runs proved the fix. Say what it is, or reopen with the
    criterion "a gate run above load 60 that is not cut".
- **UXF-023 (S2, closed "not reproduced").** The recorded asset, `L2 /ess/ desktop-1280`, did fail
  before and pass after, but for a different defect: its failures were UXF-005's
  `label-content-name-mismatch` nodes (the record itself says R-02 never held a contrast node on the
  landing page), which the shared theme's release removed. The asset names only the desktop test;
  the phone twin `L2 /ess/ mobile-390` passed too and is not listed. For its own claim (contrast on
  the landing page, light theme) there was never a red run, so "fails before" cannot be met; the
  closure is honest only as "not reproduced, and never was", and the round values (0 and 0, both
  recomputed) support that. The two landing-page tests now pass where the slow lane expects a known
  failure, and every slow run that holds one exits as a new failure (slow-l2-1, slow-l2-2,
  slow-rerun-a: "0 known, 1 new"). That is the tool working as designed, but the report's table does
  not say that these three "failed" runs failed for a test that passed.

### 1.6 Do the metrics mean what they say this round?

- The "baseline" column is the baseline after the ratchet took this round in, so for every improved
  value "this round" and "baseline" are the same number. The column cannot show what was beaten; the
  "last round" column is the only useful comparison. Only 3 rows differ, and they are the worse
  inside tolerance ones in § 1.3 (T2).
- Gate 4 and gate 5 are red on three open-finding assets and the visualise page's frame. The report
  does not say that all 3 over-limit runs are known (UXF-012, UXF-013) and that the 23 nodes are the
  templates package's frame (UXF-005 to UXF-008); a reader of the Gates table alone sees two hard
  gates failing. The notes name the visualise page, not the over-limit runs.
- The runs table is headed "against the 120 s step budget of the gate lane" and lists slow-lane runs
  of 138.1 s (`slow-l2-1`) and 160.1 s (`slow-measured`) beside it. The slow lane has no 120 s
  budget (it is a named separate lane); nothing marks which runs the budget applies to.
- `slow-l2-6` timed out one test at 60 s at a load of 56.81, under the line, so by the host's rule
  that is a test-budget defect: filed as UXF-029. Correct. The report calls its coverage "unmeasured
  none" because a rerun filled the gap; that is true of the merged round and the rerun's load
  (37.69) differs from the failed run's.
- The round has no recording, no questionnaire and no person (the report says so). Nothing in it
  speaks about users; the L3 numbers are a deterministic bot's.

## § 2 Findings the report misses

1. Three phone scenarios improved, not two, and the keyboard runs improved by about 50 key presses
   each (§ 1.4). One-line cause for each is not given anywhere.
2. The worst-moment shot (`shots/worst-1-mobile-390-light.png`) shows the top of the visualise page
   at 390 px: no magenta outline and none of the 11 axe nodes is in view, although the caption says
   "magenta outline: the first axe nodes". The shot does not show the defect it is cited for.
3. The contact sheets (both themes) show the new in-page links on the phone landing frame ("Run your
   first conformance suite", "Limitations and trust assumptions") and the phone menu still opening
   on the docs panel with "Back to main menu" first. That confirms UXF-013's remaining cost for
   `page-a-view`; the report does not use the frame.
4. UXF-012 and UXF-013 own all three over-limit runs; the report's Gates table does not link a gate
   failure to its finding.
5. The merge of `main` (release 0.98.0) lands between the measured engine commit and the record, and
   so does the shared theme's release bump (`cd70d37c1`). `round.json` holds the engine's commit
   (`e9479fac0601`) but not the site build's, so the 4 axe nodes of the landing page and the
   reading-load falls cannot be tied to a site state from the record (T4).
6. A gate run's own ten slowest tests are not reported: the "ten slowest" table is the slow lane's.
   The budget rule for the test step asks for the step's ten slowest.

## § 3 Tool defects

- **T1.** `website/ux/core/round.ts`, the Runs table: it prints wall, workers and counts but not
  `timing.load`, which every run record holds and `core/reporter.ts` reads on purpose. Input: R-06's
  `round.json` run `gate-1` has `load.before [47.78, ...]`; `report.md` line 46 onward has no load.
  The hand-written note then claims the table holds the loads, and the number check in
  `notesNumbersMissing` does not read such a claim. A round's budget judgement (load at or under 60)
  cannot be checked from the report alone.
- **T2.** `core/round.ts`, the metrics table and report: no ratchet summary (improved, worse beyond
  tolerance, worse inside tolerance) and no list of the worse-inside-tolerance values. The ratchet
  prints them (`ux_ratchet.rs`, `within_tolerance`), the report never does. The baseline column is
  the post-update baseline, so it equals the round for every improvement. Input: R-06
  `diagnostic-type-009/desktop-1280 reading_load.max`, 264 against 256, appears only as two numbers
  in a 220-row table.
- **T3.** `website/ux/baseline.json` (`lanes.gate`, `sources.gate: R-05`), with `adapter.ts`
  `SLOW_LANE_KINDS` and `rounds/R-06/decision.md`. Change 1 removed the landing page and the
  visualise page from the gate lane, but the gate lane's scope in the baseline still lists their 8
  scenarios, and the decision names kinds and tests, not `gate: <scenario>` items (which
  `ux_ratchet.rs`, `decision_of`/`covers`, needs to take a dropped lane scenario). By the source,
  `task ux-gate` then `task ux-ratchet` (default round: the gate's `round.json`) reports those 8
  scenarios as missing values on the next merge, and no L1 test holds `lanes.gate` against
  `GATE_PAGES`. Not run (read-only), so this is a reading of `compare_with`, scope and `Missing`;
  confirm with one `ux-ratchet` run on a gate round before the next merge.
- **T4.** `round.json` has no identity of the measured site (its build commit or lock), only the
  engine's. Input: the R-06 record, whose site changed by the docs-theme release and a merge after
  the engine commit. Same defect as R-05's T37 (listed "waits" in the R-06 triage).
- **T5.** `core/round.ts`/worst-moment shot: the caption promises a magenta outline of the first axe
  nodes; the shot is the top of the page and has none in view. Input: `shots/worst-1-mobile-390-light.png`.
- **T6.** (the round's own list, confirmed from the source and the record) the screen-only bot
  clicks the centre of a link's box (a wrapped link is missed), and the reporter's list test prints
  canned reporter lines into every gate log. Both are open, neither is in a finding record; give them
  records or the next round loses them.
- **T7.** `core/reporter.ts` `budgetLine`: judged only at the end of a run and only to the log (the
  report's Runs table has no "judged / not judged" column), so the report cannot say which of the 14
  runs the 120 s budget applies to (§ 1.6).

## § 4 Caveats to send with the round

- Three of the gate's three runs held the budget at load 29 to 50. They do not show that dropping
  the landing page and the visualise page from the gate lane did anything; the closure of UXF-025
  follows the host's rule (judge at load 60 or less), as the passing runs before this round did.
- Gates 4 and 5 are red on open findings only: 3 over-limit phone and desktop runs (UXF-012,
  UXF-013) and 23 axe nodes on the visualise page (the templates package's frame, UXF-005 to
  UXF-008). No new failure in the measurement itself except the passing landing-page tests, which the
  slow lane counts as new failures.
- 73 values improved, 3 got worse inside their tolerance (the largest: reading load 264 against 256),
  0 beyond. Three phone runs lost one action each; both phone keyboard runs lost about 50 key
  presses. The cause of the desktop reading-load fall is not isolated from the merge of `main`.
- The visualise page's axe nodes and the 2 landing-page nodes of R-05 are the only axe change;
  UXF-023 closes as "not reproduced", with an asset that never failed for its own claim.
- No person, recording or questionnaire took part. The numbers are those of a deterministic
  screen-only bot and of a hermetic layout and axe measurement.
- One slow-lane test (`ess-ui` reference page, phone) ran to its 60 s limit at a load under 60 and
  passed in a rerun: UXF-029, open, no asset yet.
- The baseline's gate lane still lists scenarios the gate no longer measures (T3).
