# R-02 review: report reviewer

W = the worktree's root (its absolute path removed when filed). Paths are under `W/website/ux`
unless they start with `crates/`. `rs:N` is `W/crates/edge/ess-xtask/src/ux_ratchet.rs:N`. `rj` is
`rounds/R-02/round.json`. Nothing was built, run or written in the worktree. "Inferred" marks a
claim I read from code or from two numbers and did not prove by running.

## Verdict

**Fit with named exceptions.** The baseline can hold later rounds to: coverage, gate values and
task runs are measured per configuration, and an unmeasured unit fails. Three exceptions go in the
baseline's record before R-03:

- E1 (T16): the axe measurement lost three rules' nodes against R-01 and the ratchet took the loss
  as an improvement. UXF-007 and UXF-008 are still open and no longer visible in the numbers.
- E2 (T17): `axe.serious_critical_nodes`, the only `content` metric (556 entries), can rise through
  `--update` with no decision id.
- E3 (T18): nothing in the repository runs the gate or the ratchet. "Every merge" is not enforced.

## § 1 Numbers recomputed

Method: `rj`, the 12 kept `raw.json`/`round.json` under `.../ux-loop/r02/out/`, `baseline.json`,
and `git show 0ef693136d:website/ux/baseline.json` (R-01's baseline), with `jq`.

Matches (report line, my value):

- Coverage 139 of 139, 556 of 556, 22 of 22, 417 of 417: `rj` `coverage.pages` 139/139,
  `coverage.measured_only` 417/417. Match.
- Gate 1: 0 of 139, 0 spills. Match (`gates.gate1_no_horizontal_overflow`).
- Gate 2: `pages` 2, `failures` 4, `by_element` docs sidebar 2 and blog sidebar 2, measured only
  136 and 136. Match.
- Gate 3: 0 and 0. Gate 4: `tasks` 18, `completed` 18, `over_limit` has 10 entries, each equal to
  the list on report lines 38-47. Gate 5 keyboard 4 of 4. Truncation 2 pages, 6 texts. Reduced
  motion 0. Match.
- Gate 5 axe: `nodes` 2732, `pages` 105, rules 4 + 2724 + 4 = 2732. Match.
- Runs table (report lines 55-65): equals `rj` `runs[]` (name, workers, wall_s, status, unmeasured)
  for all 11 runs. The ten slowest (largest 20.3 s) equal `rj` `slowest`.
- L3 table: all 22 rows equal `rj` `scenarios` for the task keys. Page-sum rows equal sums of
  `rj` `scenarios` per viewport and theme.
- Three final-engine gate runs (gate-7, 8, 9) give the same values as the full round for all 44
  gate scenarios (compared in `review-r/gate-7.sc.json`, `gate-8.sc.json`, `gate-9.sc.json`,
  `full.sc.json`). The measurement is repeatable on this host.

Does not match:

1. **The "baseline" column is R-01's baseline, not the committed one.** `rounds/R-02/report.md:119`
   onward reads `baseline.json` at report time (`core/round.ts`, `writeReport`), and the report was
   written before `--update --decision R-02-baseline` replaced the file. 27 of the 220 task rows
   (22 task runs) differ from the committed `baseline.json`: 23 where the committed value is
   higher, 4 where it is lower. Largest:
   - `first-conformance-run/mobile-390/keyboard` `screen_only.actions`: report 26, committed 130
     (difference 104).
   - `install-linux-x86_64/mobile-390/keyboard` `screen_only.actions`: report 24, committed 128.
   - `ess-31-release/desktop-1280` `screen_only.steers`: report 1, committed 23.
   - `ess-31-release/mobile-390` `screen_only.time_to_act_steps.max`: report 3, committed 30.
   - The page rows show "–" because R-01 held no page scenario.
2. `baseline.json` `about` still says it was written from the R-01 round (548 of 556). It holds
   578 scenarios and 3,556 values.
3. `findings/UXF-014.md` says "54 tests"; the gate has 56 (29 L1 + 19 L2 + 8 L3 in `rj`
   `runs[8].timing.projects`).
4. `findings/UXF-015.md` carries a table from before the engine change.
5. Not a mismatch but a limit: the report's axe line (2732 nodes against R-01's 2733) reads as
   unchanged. It is not (§ 3, T16).

That is 27 mismatched report cells (one column) plus three stale statements in the record.

Not recomputed: durations of the three gate runs by the pool's own clock (the logs
`gate-times*.log` give 28.7 to 83.4 s across five engines; only gate-7 to 9 are the final engine).

## § 2 R-01 tool defects T1 to T15

| T | rating | evidence (code) | test that goes red |
| --- | --- | --- | --- |
| T1 unmeasured page counts clean | repaired | `specs/l2-layout.spec.ts:42` declares the expectation first; `core/reporter.ts:136-166` computes unmeasured; `:198,216,241,260-282` pass it into each gate; `rs:358-364` makes each unit a `Breach::Unmeasured` | `specs/l1-reporter.spec.ts:53`, `:66`, `:90`; `rs:1247` |
| T2 coverage and content move the ratchet | repaired, with E2 | per-page scenarios replace `site/*` (`rj` 578 scenarios); `rs:320-323,358-417` hold lanes; `rs:277-290` content class | `rs:1236`, `rs:1294`, `rs:1337`, `rs:1377` |
| T3 search as a layout gate | repaired | `adapter.ts:72-74,79-89` (search not watched); `:91-97` matches markup; `specs/l2-layout.spec.ts:29-34` one site check | `specs/l1-adapter.spec.ts:9`, `:40` |
| T4 sidebar at the footer | repaired | `adapter.ts:75-78` `SIDEBAR_STATES = ['top','middle']`; `core/contract.ts:53-73` gates named states only; `end` goes to `measured_only` | `specs/l1-adapter.spec.ts:23`, `:30` |
| T5 archive misclassified | repaired | `adapter.ts:57-67` `BLOG_WITHOUT_SIDEBAR` | `specs/l1-adapter.spec.ts:16` |
| T6 gate 4 cannot fail on cost | repaired | `specs/l3-screen-only.spec.ts:41-43`; `rj` `gates.gate4_screen_only.over_limit` has 10 entries, result `fail` | `specs/l1-bot.spec.ts:30` |
| T7 truth weaker than card | repaired | `core/bot.ts:27-36` `anchor`, `beside`; `:432-458` `judge` | `specs/l1-bot.spec.ts:46`, `:55` |
| T8 bot uses what is not on screen | repaired | `core/bot.ts:115-128` `look` needs `checkVisibility` plus `probeVisible`; `adapter.ts:139-146` revealers only for named controls | `specs/l1-bot.spec.ts:63`, `:75`; `specs/l1-probes.spec.ts:86` |
| T9 gate 3 cannot fail | repaired | `core/bot.ts:377-380` `arrivedCovered`; `rj` `gate3` 0 and 0 on 22 runs | `specs/l1-bot.spec.ts:87` |
| T10 re-runs duplicate | repaired | `core/reporter.ts:113-128` `mergeRecords` keys on viewport and theme; later replaces earlier | `specs/l1-reporter.spec.ts:80` |
| T11 not reproducible from the tree | partly | digest and commit in `rj` `engine`; `core/engine.ts:18-53`; `core/round.ts:110-118` `runs_agree`. Not repaired: `package.json:6-7` has `gate` and `typecheck` only; the slow-lane and round commands are scripts outside the repository (`.../r02/slow-run.sh`, `gate-run.sh`); no `raw.json` is kept in the tree (`.out` is not committed) | `specs/l1-reporter.spec.ts:112`, `:125` cover the digest and the merge, not the commands |
| T12 `round.json` lacks what the report prints | partly | `rj` holds `runs`, `slowest`, `slowest_pages`, `measured_only`, `coverage.by_config`; `workers` read from `config.workers` (`core/reporter.ts:347`). Not repaired: the report's baseline column is read from `baseline.json`, not from `rj` (`core/round.ts` `writeReport`), so it differs from `rj` plus the committed baseline (§ 1, item 1) | none |
| T13 measured-only coverage | repaired | `core/round.ts:81-93` rows carry `expected`; `rj` `coverage.measured_only` 417/417; `specs/measured.spec.ts:18-25` declares and attaches on cut | `specs/l1-reporter.spec.ts:102` |
| T14 ratchet not built, rule differs | repaired | `mod ux_ratchet` registered in `crates/edge/ess-xtask/src/main.rs`; 30 tests pass (`.../r02/ratchet-green-2.log`); `rs:325-355` improvement checked before tolerance; `rs:985-1000` unreadable `origin/main` is an error; `rs:274-298` `reading_load` classed | `rs:1179`, `rs:1196`, `rs:1520`, `rs:1552` |
| T15 regression assets per finding | partly | `rs:903-950` checks the file and the test title; `rs:1619` is its test. Not repaired: UXF-007 and UXF-008 name `L2 /ess/docs/visualise desktop-1280` and `mobile-390`, which also fail on UXF-004 (`link-in-text-block`, 3 nodes on that page, light), so fixing one finding alone leaves its asset red; UXF-016 to UXF-020 have no asset | `rs:1619` (title check only) |

Count: 12 repaired, 3 partly (T11, T12, T15), 0 not repaired, 0 cannot tell. One line on T2: the
coverage and content movement is repaired; the content class opens E2.

## § 3 Engine and ratchet review

### A broken page counted as clean

- **Axe frames (T16, inferred cause).** R-01 `gates.gate5_accessibility.by_rule` (R-01 `round.json`)
  held 5 rules, 2733 nodes on 104 pages: `nested-interactive` 2 nodes, `scrollable-region-focusable`
  1, `color-contrast` 22 nodes on 2 pages, `label-content-name-mismatch` 6 on 2 pages.
  R-02 holds 3 rules, 2732 nodes on 105 pages: `color-contrast` 4 nodes on 1 page,
  `label-content-name-mismatch` 4 on 1 page, no `nested-interactive`, no
  `scrollable-region-focusable`. `core/contract.ts:79-82` switched to `setLegacyMode(true)` in R-02.
  `/ess/docs/visualise` shows only `link-in-text-block` in R-02. The probe in
  `.../r02/probe-axe2.log` timed legacy mode (29.7 s) against in-page mode (16.3 s) and did not
  compare findings. So: the loss is measured, the cause (legacy mode and frames) is inferred.
  UXF-007 and UXF-008 (`findings/UXF-007.md:7`, `UXF-008.md:7`) are `status: open`.
- **Dark theme runs two axe rules** (`core/contract.ts:77`, by design, structure checked in light).
  A structure defect that only the dark theme renders is not seen. Inferred, small.
- **Search is not on screen-gated** (`adapter.ts:72-74`): it is one build-time check on
  `TEMPLATE_PAGES` (`specs/l2-layout.spec.ts:29-34`). A search control that is built and hidden
  passes it. The skill's § 4 gate 2 lists search; the adapter drops it with a recorded reason (UXF-001).
- **A path moved into an exception set hides a page.** `adapter.ts:57-59` names two paths with no
  sidebar. A page added there is not checked for a sidebar. It needs an L1 test per entry; none
  guards a new entry.
- **Expected set comes from the built sitemap** (`core/sitemap.ts`). A page missing from the
  sitemap is not expected. The ratchet's lane drop (`rs:1377`) catches a page that left the sitemap
  only with the baseline holding it.

### The ratchet takes a worse value or loses coverage

- **E2 content class.** `rs:341-348` and `rs:517-527`: a rise on a `content` metric is
  `Breach::New` in `compare`, and `update` takes it with no decision id. `rs:454` exempts content
  from the comparison with `origin/main`. The only `content` metric is `axe.serious_critical_nodes`
  (556 entries in `baseline.json`). So a page's serious node count can rise from N to M with one
  `--update` and no `--decision`. The rule count (`count`, tolerance 0) still blocks a new rule.
- **No check of the engine** (T22). `rj` records `engine.dirty` and `engine.runs_agree`
  (`core/round.ts:110-118`). `Round` in the ratchet reads `scenarios`, `coverage`, `lane` only.
  A round from a dirty engine, or from runs of different engines, can update the baseline.
- **Update writes before the second compare** (T21). `rs:1054-1059` writes the baseline;
  `rs:1073` compares again. A run that then fails leaves the changed file on disk.
- **Decision id is not looked up.** `--decision` takes any non-empty string (`rs:499-501`). The id
  `R-02-baseline` exists in `rounds/R-02/decision.md:8`. Nothing checks that a decision file with
  that id exists. Inferred from reading `rs:499-581`.
- **One decision covers a whole replacement.** `R-02-baseline` carries 23 worse values, among them
  keyboard `screen_only.actions` 26 to 130 and 24 to 128. `decision.md:8-27` states the reason for
  the replacement (the R-01 tools). It does not discuss the keyboard rise: it is an engine change
  (the bot now reads the screen) and the file says so only in general.
- **Tolerance class mismatch** (T24). `rs:277-298` classes `reading_load` as `words`; the 44
  `reading_load` entries in `baseline.json` carry `steps`, and `baseline.json` `tolerances` defines
  no `words`. Same 5 %. A new `reading_load` key written by `update` gets the other name. Inferred
  effect, not run.
- Tolerances are as § 6 states: counts 0, steps 5 %, layout shift 0.01 (`baseline.json`
  `tolerances`). The largest `layout_shift.load` in the baseline is 0.0144 (278 entries, sum
  0.0416): the tolerance 0.01 is large against the values, so one page can go from 0 to 0.01
  unreported.

### A gate in the skill's § 4 the code does not check

- Gate 2 search: see above.
- Gate 2 sidebar at the end: measured only (D15, `adapter.ts:75-78`), recorded.
- Gate 6 cue truth: `dropped` on this site (report line 34); nothing checks the drop.
- "Every merge": no `Taskfile.yml` or `.github` hit for `ux-ratchet`, `UX_LANE` or `website/ux`
  (`git grep`, T18). `package.json:6` is the only gate command.

### A value that depends on timing or machine load

- Measured values: none found. gate-7, 8, 9 ran under different load (wall 41.6, 29.1, 28.7 s)
  and equal each other and the full round on 44 scenarios.
- Wall time: depends on load. The same gate took 54.8 to 83.4 s on earlier engines
  (`gate-times.log`). `playwright.config.ts:31` ends the gate step at 120 s; a slower host turns
  the step into a timeout. The reporter then lists the cut units as unmeasured (`core/reporter.ts:163`),
  so the cut cannot count as clean.
- Waits: hydration waits up to 30 s (`core/contract.ts` `showTheme`); a slow host that exceeds it
  fails the page. Inferred, not observed.
- `reuseExistingServer: true` (`playwright.config.ts:44`): a stale server on port 4391 serves an
  older build. Inferred.

### The agent's named gap

"With `--update` on a gate-lane run, a scenario that is new in the lane is not compared in the same
pass."

- Confirmed for the first pass. `compare` skips every out-of-scope scenario's values
  (`rs:364-366`: `if !in_scope(scenario) { continue; }`) and reports only `Breach::NewInLane`
  (`rs:393-398`). `update` takes the lane addition (`rs:499-581`) without judging the values.
- Refuted as a silent acceptance. `run` compares again after the write (`rs:1073`), now in scope,
  so a worse value fails there. Cost: the file is already written (T21), and no test feeds a lane
  addition whose values differ from the held ones (`rs:1358` uses equal values).

### Gate 2: does `round.json` support a finding?

Yes. `rj` `gates.gate2_always_visible`: `pages` 2, `failures` 4, `by_element` "docs sidebar: out of
viewport" 2 and "blog sidebar: out of viewport" 2, no `missing`, no `covered`. The two pages are
`/ess/docs/getting-started` and `/ess/releases/tags` (per the scenario keys in `rj`
`scenarios`: `always_visible.failures` 1 each at `@desktop-1280/light` and `/dark`). The same two pages fail at
1440 and 1024 in both themes (`rj` measured-only). UXF-002 and UXF-003 are `accepted` under D15
and cover the `end` state only, so these middle-state failures have no record. The cause (sidebar
taller than the viewport in the middle state, or the sticky column) is inferred; round.json holds
the geometry, not the reason. A finding needs a class and severity from `triage.md`; both are
free for the coordinator.

## § 4 New tool defects

- **T16** axe in legacy mode lost `nested-interactive` (2 nodes), `scrollable-region-focusable`
  (1) and 18 `color-contrast` nodes against R-01; the ratchet took it as an improvement.
  `core/contract.ts:79-82`. Test missing: an L1 test with a page that holds an iframe and a
  nested control.
- **T17** a `content` rise is taken by `--update` with no decision and skipped by the `origin/main`
  comparison (`rs:517-527`, `rs:454`); only `axe.serious_critical_nodes` is in the class.
- **T18** no Taskfile or CI step runs the gate or the ratchet; `package.json:6` only.
- **T19** the gate was timed with `--workers=6` (`gate-run.sh`, `rj` `runs[8].timing.workers`),
  the repository's script says `--workers=4` (`package.json:6`); 28.7 to 41.6 s is not the time of
  that command.
- **T20** the gate's Playwright step exits 1 while any finding is open (gate-7 to 9 exit 1; the
  site-search test is red by design, `specs/l2-layout.spec.ts:29-34`). Its exit code does not tell
  a new failure from a known one; only the ratchet does.
- **T21** `run` writes the baseline before the second comparison (`rs:1054-1073`); a failing run
  leaves a changed file.
- **T22** the ratchet does not read `engine.dirty` or `engine.runs_agree`.
- **T23** the report's baseline column and `baseline.json` `about` are not functions of `rj`;
  both go stale when the baseline is replaced after the report.
- **T24** `reading_load` is `words` in code (`rs:280`) and `steps` in the 44 baseline entries.

## § 5 Organization bounds

- Test at most 60 s: `playwright.config.ts:30` `timeout: 60_000`. Longest measured test 20.3 s
  (`rj` `slowest[0]`). `specs/round.spec.ts` sets 120 s for the report step: over the 60 s bound by
  its own text; it runs no browser. Report it as an exception to remove.
- Test step at most 120 s: gate `globalTimeout` 120_000 (`playwright.config.ts:31`); final engine
  28.7, 29.1 and 41.6 s. Earlier engines ran 54.8 to 83.4 s, one failed at 134.6 s
  (`red-gate-r01.log`, the R-01 engine, exit 1).
- Pool job at most 10 minutes unless named: each slow job runs `slot --timeout 30m` (`slow-run.sh:10`)
  and ended in 72.4 to 153.3 s; `globalTimeout` for the slow lane is 1_680_000 ms (28 min).
- Measuring and watching scripts: the scripts here wrap `npx playwright test` (a test) in `slot`.
  `serve.sh`, `build-site.sh` and the probes are not prefixed. Not checked: whether `serve.sh`
  starts the server outside `slot`; I did not read its first line.
- No Python: `git ls-files website/ux` holds no `.py` and no `.sh`. The run scripts are in the
  scratch directory, outside the repository.
- No `Taskfile.yml` text for the loop exists (T18), so there is no Taskfile text to check.

## Files

- Report under review: `W/website/ux/rounds/R-02/report.md`, `round.json`.
- Helper files: `.../ux-loop/r02/review-r/`.
