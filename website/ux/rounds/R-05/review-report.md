# R-05 review: report reviewer

Paths are relative to the repository root. `ux/` is `website/ux/`. `rs:N` is
`crates/edge/ess-xtask/src/ux_ratchet.rs:N`. `rj` is `ux/rounds/R-05/round.json`, `rp` is
`ux/rounds/R-05/report.md`, `dm` is `ux/rounds/R-05/decision.md`, `nt` is `ux/rounds/R-05/notes.md`.
The round is branch `ux/docs-site-loop-r01`, head `2b0c77ce84`, commits `4150f0a494..2b0c77ce84`.
Nothing was built, run or written in the repository; the only file I wrote is this one. I read
`ux/baseline.json` at `4150f0a494` and at the head with `jq` and compared them value by value (helper
files `rv-*.json`, `rv-*.tsv` are in the round's scratch directory, not in the repository). "Inferred"
marks a claim read from code or from two numbers and not proved by running.

## Verdict

**Not fit to send as is.** Default action: fix the 8 claims in § 1 (one-line edits; none changes a
decision), put the sources the round cites into the repository or mark them "not kept" (§ 1, second
list), and then send. The baseline itself is sound: 3,556 values in 578 scenarios before and after, no
value net lost, every worse value is listed by value (§ 2). The ratchet's new rules are real and tested
(§ 3).

- Point 2 (`R-05-worse`): 11 reading-load values: accept the baseline, correct the explanation (4 of 11
  need a source); landing layout shift 0: accept, with the gate-run evidence put in the repository; 556
  dark values: accept (a relabel, checked value by value); 17 lane moves: accept with caveats.
- Wrong claims: 8. Claims with no kept source: 5.
- T34, T35, T36, T41 closed; T39 partly (claimed, not proved); T37, T38, T40, T42 open.
- New tool defects: T43 to T50.

## § 1 Numbers recomputed

Method: every digit in `rp` (all tables, 244 task rows and 24 sum rows plus the run table, the gates
table and the notes), `trend.md`, `nt`, `dm`, `triage.md` and the changed finding records was compared
with `rj`, the new and old `baseline.json`, R-04's `round.json`, and the kept logs of the scratch
directory (`gate-*.log`, `ratchet-full-2.log`, `c*-red.log`, `c*-green.log`). The tables of `rp` match
`rj` (the run table row by row, the gates table, the L3 table, the trend row: 66 pointer actions, 21
steers, 27 nodes on 2 pages, 5 over the limit, all recomputed). The errors are in prose.

| # | Claim | Where | Source says | Verdict |
| --- | --- | --- | --- | --- |
| 1 | "Fourteen more are worse inside their tolerance; the ratchet names each in a note" | `nt:20`, `rp:423`, the builder's report item 11 | `ratchet-full-2.log` has 15 `note:` lines; counting `rj` against the new `baseline.json` gives 15 (the 15th is `target-process/mobile-390 reading_load.median` 114 against 113) | Wrong: 15 |
| 2 | "The slow lane's nine jobs ran at 55.92 to 131.15" | `nt:6`, `rp:409` | `rj` holds 11 slow-lane runs (`slow-l2-1..6`, `slow-l3`, `slow-measured`, `slow-nested`, `slow-rerun-a`, `slow-rerun-b`); 55.92 is `slow-rerun-b`'s; the nine without the reruns started at 68.47 to 131.15 | Wrong: say 11 jobs, or 68.47 |
| 3 | "`gate-5`, `gate-6` and `gate-7` started at a 1-minute load of 133.47 to 169.55" | `nt:3`, `rp:406`, `ux/findings/UXF-025.md:104` | `gate-6` started at 133.34 (`rj` `runs[].timing.load.before[0]`); 133.47 is `gate-5`'s | Wrong at the low end: 133.34 |
| 4 | "Eleven reading-load values ... Nine are cards whose path passes the docs introduction ... Three are the two index pages" | `dm:8-11`, `rp:422`, builder's report item 11 | 9 + 3 is 12, not 11. The 11 deltas are +16 (7 values), +13, +23, +11, +28 (`dm` table). Only the +16 values fit "one line of 16 words on `/ess/docs`"; `limitations` +11 and `page-a-view` +28 are cards whose own pages `main` reworded (`git diff 4150f0a494..2b0c77ce84 --stat -- website/docs`: 51 files, `status/limitations.md` among them). The per-step words are in the discarded `.out/` raw files | Wrong as arithmetic; the cause of 4 values is not shown |
| 5 | R-03's gate runs "at a load of 61 to 92, as R-03's review reported (corrected in R-05 from '61 to 69')" | `ux/findings/UXF-025.md:54`, `ux/rounds/R-04/triage.md:24` | `ux/rounds/R-03/review-report.md:53` (M3) says "load 61 to 92" mixes the 1-minute values 69.39 and 61.24 with the 15-minute value 91.73. The 1-minute range was 61 to 69, so the original was right and the step-0 "correction" made it wrong. (The same file, line 93, repeats 61 to 92; the R-04 reviewer took that line) | Wrong, introduced by step 0 |
| 6 | "`../rounds/R-05/notes.md` keeps the lines" of `gate-2` to `gate-4` | `ux/findings/UXF-025.md:90` | `nt` has no line for `gate-2`, `gate-3` or `gate-4`; `grep` for 114.1, 68.05, 131.28 in `nt`, `rp`, `triage.md` finds nothing. The lines are only in the scratch logs | Wrong source (dangling) |
| 7 | "The visualise page's two L2 tests hold 87.8 of the 251 L2 test-seconds (44.3 and 43.5 s in `gate-3`)" | `ux/findings/UXF-025.md:100`, builder's report item 15 | 44.3 and 43.5 are from `gate-3` (L2 sum 247.2); 251.0 is `gate-4`'s sum | Wrong: two runs mixed; 87.8 of 247.2 |
| 8 | "last round" column of the layout-shift rows: 0 | `rp:132`, `rp:144` | R-04 filed the same measurement under dark: sum 0.124 at desktop-1280 and 0.0318 at mobile-390 (`ux/rounds/R-04/round.json`). Today's rows read as a rise from 0 to 0.014 and 0.028; they are falls (T44) | Misleading, not a typo |

Claims with no kept source (the number may be right):

1. `dm:45`: the landing page's layout shift "0 in `gate-5`, `gate-7` and `gate-8` (load 133, 170,
   60)". `rj` holds the landing's value only for `slow-l2-1` and `slow-rerun-b` (`replaced`); the gate
   runs' per-page values were in `.out/`, which is discarded, and no scratch log holds them (T38,
   again). This is also the strongest evidence in the decision, and it cuts the other way (§ 2, item 2).
2. `ux/findings/UXF-016.md:34`: 0.97.4 "published on 2026-11-22 at 18:37 UTC with its four archives and
   `SHA256SUMS`". The tag's date is 2026-11-22 18:00 +0100 (`git for-each-ref refs/tags/0.97.4`); no
   release-listing output is kept. The status page now says "observed on 23 November 2026"
   (`website/docs/status/where-this-stands.md:13`), a dated observation of the asset list, and the
   L1 test compares version strings only.
3. `dm:615-640`: "the round of `gate-8`" is the source of the baseline's `gate` lane
   ("lane gate from round R-05"); its `round.json` is in `.out/` and not kept.
4. `ux/findings/UXF-025.md:87`: "59 in all". True for change 1; the final gate runs hold 62 tests
   (48 L1, 11 L2, 3 L3: `rj` `gate-8`, `ux/rounds/R-05/round.json` `runs[name=gate-8].timing.projects`).
   Stale, not wrong.
5. Probe-style evidence for "0 under light in `slow-l2-5` and `slow-l2-3`" (`dm:54`) is held
   indirectly: `rj` `replaced` has only the landing page, so the two pages measured the same 0 in the
   first run and in `slow-rerun-b`. That is sound and checkable; I mention it because the sentence does
   not say so.

Step 0 (the six wrong claims of the R-04 review): five are corrected and the corrections hold against
their sources: 16 tests did not pass (15 `timedOut`, 1 `failed`; `ux/rounds/R-04/round.json`
`unexpected` lengths 1+2+3+7+2+1) and 19 reran (12 + 7); R-03's `gate-4` ran 36/19/7 = 62; the
layout shift was measured light and filed dark (`ux/specs/l2-layout.spec.ts` `colorScheme:
THEMES[0]`); change 4 of R-04 touched `website/src/css/custom.css` (commit `5f6ae80d36` does); "nine
worse" is split into 9 and 4 inside tolerance. The sixth, the R-03 load, is now wrong (row 5). The four
"not kept" marks are honest. So step 0 fixed 5 of 6 and made 1 new error by correcting a figure
against a secondary claim without opening the source that the same review flagged.

Does this round repeat the kind? Yes, three kinds: a count written in words that no check reads
(rows 1, 2, 4: "Fourteen", "nine jobs", "Nine ... Three"; T49); a range copied from the wrong run (row
3); a source named that does not hold the number (row 6, and unsourced items 1 and 3). The previous
review's remedy (T38: keep the evidence, check more than `notes.md`) was put off ("waits" in
`ux/rounds/R-05/triage.md`) and the round hit it four times.

## § 2 The decision `R-05-worse`

Baseline check, old (`4150f0a494`) against new (head): both hold 3,556 values in 578 scenarios. Keys
only in the old file: 556 (278 `<page>@<vp>/dark layout_shift.load`, 278 `.../dark motion.running`).
Keys only in the new file: 556 (the same 278 pages and viewports under `/light`). Nothing else added or
removed. `dropped` grows by 573 (556 + the 17 lane scenarios); 13 earlier entries stay.

### Item 1: 11 reading-load values, by value: accept the baseline, fix the explanation

`dm` lists each as `<scenario> <metric> <value>`; the 11 match the 11 values that `ratchet-full-1.log`
calls "worse than the baseline" (and no others), and each listed value equals `rj`. These are page
words, not layout: the rise is 3 to 15 % on a metric with a 5 % tolerance, caused by `main`'s page text
and change 5's added class names. Accept the values. Needs evidence: the sentence that nine come from
one line on `/ess/docs` (§ 1 row 4): 7 of 11 deltas are +16; `limitations` +11, `page-a-view` +28,
`install-linux-x86_64/desktop-1280/keyboard` +23 and `ess-31-release/mobile-390` +13 do not fit it. Fix
the text to say the rise is `main`'s page text and change 5's, and name only the +16 group.

### Item 2: the landing page's layout shift at 0: accept; the stated cause is not shown

The baseline value of this key was 0 in R-04 (under `dark`) and is 0 now, so the decision moves no
baseline. It exists only so the ratchet does not hold the worse of two runs (0.0948 in `slow-l2-1`, 0
in `slow-rerun-b`; `rj` `replaced`). It is `takes_value` (rs:674) working as designed. But the cause
"a timing value under load" is not shown by the round's own data: per `dm:45` the page measured 0 at
load 133 (`gate-5`) and 170 (`gate-7`) and 0.0948 at 131. One shift in five reads, at no higher load
than two zero reads, is as consistent with an intermittent real shift as with load. Take the 0 (four of
five reads), put the five reads in the repository (T38), and do not write "load" as the cause until
UXF-026's three-read median exists.

### Item 3: 556 dark page-load values dropped, 556 light added: accept (a relabel)

This is a correction of T34 with the same measurements under the right key, not 556 values gone. Per
page, viewport and metric (1,112 values joined):

- 549 are identical.
- 7 differ: `/ess/docs/concepts/overview@desktop-1280` 0.0342 to 0 and
  `/ess/docs/status/roadmap@desktop-1280` 0.0697 to 0 (the two R-04 values taken "under load"; they
  were light-theme values and now read 0 at lower load); `/ess/docs/guides/specify/layout-and-validation@mobile-390`
  0.0006 to 0; and four rises of 0.0001 to 0.0006 (`.../deploy-an-environment@mobile-390`,
  `/ess/docs/guides@desktop-1280`, `.../commands-and-outcomes@mobile-390`,
  `/ess/releases/generated-servers-refuse-what-their-openapi-refuses@desktop-1280`). Largest rise 0.0006
  against the tolerance 0.01. So the ratchet would have passed every pair.
- No dark page-load value was ever measured in the dark theme: `ux/specs/l2-layout.spec.ts` loads
  under `THEMES[0]` (light). Nothing that existed is lost; "layout shift while the page loads dark" is
  not a metric of this tool.

Caveats: the tool has no rename, so the move is a 556-line drop list plus 556 "new" values (`dm:50-614`,
643 lines in all), and the ratchet compared nothing between old and new key (T43). I did that
comparison by hand above. The decision text says "278 of each" and means 278 pages-viewports per metric;
the headline "556 values" is 278 × 2 metrics, which is right but reads like 556 pages.

### Item 4: 17 gate-lane scenarios to the slow lane: accept, with caveats

The 17 are exactly `lanes.gate` old minus new (12 page configurations of `/ess/releases`,
`/ess/docs/status/where-this-stands`, `/ess/docs/reference/entity-runtime-lowering`; 5 task runs). Their
values stay in the `full` lane (578 scenarios before and after), so no value is lost; what is lost is
the per-merge ratchet on them. What the gate no longer guards is in § 4. The `dm:617` sentence "the
phone keyboard runs and two pointer runs" misses that `first-conformance-run/desktop-1280/keyboard`
also left (3 keyboard runs and 2 pointer runs left).

## § 3 The ratchet's new rules (`rs`, `ux/core/reporter.ts`, tests)

| Rule | Code | Test | Red before? |
| --- | --- | --- | --- |
| a round of one run with a status not `passed`, or with an unexpected test, is refused | `not_passed` rs:598 (`round.timing`) | `a_round_of_one_run_with_an_unexpected_failure_is_refused` rs:2923 | yes: `c2-red-rust.log`, 5 failed of 51 |
| a run of a merged round that was cut, or failed a test no later run passed, is refused; a gate-lane run reported for its time is not judged | `not_passed` rs:598, `lane` filter | `a_run_that_did_not_pass_is_refused_unless_a_later_run_passed_what_it_failed` rs:2871 (failed, timedout, rerun-earlier cases; `update` also refuses) | yes |
| on a rerun the worse value is held unless a decision names scenario, metric and value | `held_value` rs:685, `takes_value` rs:674 | `a_rerun_is_held_at_its_worse_value_unless...` rs:2982, `a_decision_takes_a_worse_value_only_at_the_value_it_names` rs:2946 | yes |
| a worsening inside tolerance prints a note | `within_tolerance` rs:557, printed in `run` rs:1666 | `a_worsening_inside_its_tolerance_is_named` rs:3011 | yes |
| layout shift filed under the theme it loaded in | `ux/core/reporter.ts:205` | `ux/specs/l1-reporter.spec.ts:110` | yes: `c2-red-ts.log`, `Expected: 0.05 Received: undefined` |

Status of T34 to T42:

| Id | Status | Evidence |
| --- | --- | --- |
| T34 layout shift under the wrong theme | closed | the L1 test above, red then green; the real L2 spec sets `loadTheme` (`l2-layout.spec.ts:50`); `rj` shows it filed under light. Residual: without `loadTheme` the old behaviour returns (T45) |
| T35 a decision lists keys, not values | closed for the update path | rs:2946 and rs:2982, red then green; R-04's `decision.md` rewritten with values. Residual: the baseline stores only the decision id, so CI cannot check the value later |
| T36 a worsening inside tolerance is silent | closed at function level | rs:3011 red then green. The print is in `run` (rs:1666) and is not tested; the notes go to the screen, not to `rp` or `rj`, and are lost when the ratchet bails (rs:1713: `ratchet-full-1.log` prints 0 notes, `ratchet-full-2.log` 15). See T48 |
| T37 no site identity, `runs` optional | open | not touched. A round with no `runs` and no `timing` passes `not_passed` (rs:598 judges only what it finds) |
| T38 evidence in the discarded `.out/`; the number check covers `notes.md` only | open, and hit again | § 1 unsourced items 1 and 3, row 6 |
| T39 two-error case left the gate | partly: claimed closed, not proved | `ux/specs/l1-findings.spec.ts:54`. The test feeds a hand-written result to the new pure function `settleKnownFailure` (`ux/core/findings.ts`); its "exit code" is a local lambda (`settled.expectedStatus === result.status`), so it does not run the hook, the reporter or Playwright's exit. `c1-green2.log` shows it green; there is no red run (the function did not exist), and the existing T28 test (`unmatchedErrors`) already covers the same cases. The end-to-end cases stay in the slow lane |
| T40 `ux-slow` stops at the first failing project | open | `Taskfile.yml` not in the round's diff; the `for:` loop still stops at the first non-zero exit |
| T41 run status, unexpected counts and reruns ignored | closed for the cases tested | rs:2871 and rs:2923 red then green. Residual: the reporter's new lists have no test (below) |
| T42 UTC date, `GATE_PAGE_ELEMENTS` source | open | `ux/adapter.ts` unchanged on that line |

The reporter's `unexpected_tests` and `passed` lists (`ux/core/reporter.ts:379-387`) have no TypeScript
test (`grep unexpected_tests` finds only the writer). It is a hole a wrong report could pass through,
in one direction:

- Safe directions are guarded: an empty `passed` makes a rerun fail to rescue a failed test (refused);
  `unexpected` and `unexpected_tests` of different lengths, or status `failed` with no named test, are
  refused (`not_passed`, rs:598-622).
- Unsafe direction: if a reporter change put a failed test into `passed` (for example by calling
  `outcome()` differently), `later` (rs:612) would count a rerun that never passed and the round would be
  taken. Today `passed` already holds tests that failed as expected (a known failure's outcome is
  `expected`, line 386), so a "passed" list is "met its expectation", not "passed"; `rj` `slow-rerun-b`
  lists `[l2-1] L2 /ess/ desktop-1280` as passed, and that test is a known failure. A later run in which a
  test fails with a stated (known) error thus rescues an earlier unexpected failure of the same test.
  That is arguably right (the test measured), but it is not what the field name says.
- The real R-05 data is consistent with the writer: `slow-l2-5` names its one timed-out test in both
  lists and `slow-rerun-a` passes it (`rj`).

## § 4 The gate lane after the cut from 24 to 13 browser tests

The L1 test is `ux/specs/l1-adapter.spec.ts:114`, "the template pages of the gate cover every page
template and every always-visible element of the sitemap". It requires each template path to be in the
sitemap, `kinds(templates)` equal to `kinds(sitemap)`, and the always-visible elements (navbar; theme
toggle and sidebar at desktop; menu button at phone; the sidebar states top and middle) of the gate
pages equal to those of the sitemap at both gating viewports. The five template pages are
`/ess/docs/visualise` (doc without sidebar), `/ess/releases/conformance-in-any-language` (blog),
`/ess/releases/archive` (blog without sidebar), `/ess/docs/start/install` (doc) and `/ess/` (landing)
(`ux/adapter.ts` `TEMPLATE_PAGES`), the five kinds `pageKind` (adapter.ts:69) returns. So yes: every
page kind and every always-visible element is still covered, at both gating viewports. The count test
(`l1-adapter.spec.ts:102`) is red at 24 (`c1-red.log`) and green at 13.

What the 11 moved tests (6 L2 + 5 L3; 24 minus 13) guarded that no gate test guards now. The L1 test
sees kinds and element names, not pages, so none of this is visible to it:

1. A page with a wide table at 390 px (gate 1, spill, and UXF-020, "a six-column table hides its last
   column"): `entity-runtime-lowering` was the only generated table page left in the gate (R-04's review
   already noted that no template page pins a wide table; now not even this one does). A table spill
   shows up in the slow lane only.
2. The release list `/ess/releases` (a blog index with its sidebar and list of posts) and the status
   page: same kind as pages that stay, different DOM. A regression of the list's layout, or of the
   status page's table, is not seen per merge. UXF-016's L1 test (the version string) is in the gate.
3. Phone keyboard access of the menu (gate 5, keyboard half at mobile-390): both phone keyboard runs
   left (`first-conformance-run/mobile-390/keyboard`, `install-linux-x86_64/mobile-390/keyboard`). The
   gate keeps one keyboard run, at desktop-1280.
4. Gate 4 on two runs: `first-conformance-run/mobile-390` (4 actions against a limit of 3, an open
   failure) and `install-linux-x86_64/desktop-1280` (pointer). The gate keeps `install` at mobile-390
   and `first-conformance-run` at desktop-1280, one run per card.
5. The per-merge ratchet of the 17 scenarios' values (`lanes.gate`).

What holds: kinds, always-visible elements, both gating viewports, gate 3 (`completion_miss`), the axe
rules on the five pages, one keyboard run, one pointer run per card. The cap test counts the constants
(`TEMPLATE_PAGES.length * GATING.length + GATE_TASK_RUNS.length`), not what Playwright lists, so a new
spec file in a gate project would not be counted (T50). `rj` `gate-8`: 48 L1 + 11 L2 + 3 L3 = 62 tests
(the 11th L2 test is the browserless search check); so "13 browser tests" is right, and the report's
"11" for L2 is not a contradiction.

The cause of the cut and whether it ends inside 120 s is not judged here.

## § 5 Findings hygiene

- UXF-016 (S2, closed): the asset is a Playwright test in the repository
  (`ux/specs/l1-site.spec.ts:14`). Red before (`c3-red.log`, `Expected: "0.97.4"`, `Received:
  "0.97.3"`), green after (`c35-green.log`). Sound. The fix is a hand edit of
  `website/docs/status/where-this-stands.md`; the test fails again at the next release until a person
  edits it (the record says so). The record title still says 0.51.0 and 0.97.1 (history of R-01; not
  wrong). The publication time and "four archives" have no kept source (§ 1, unsourced 2).
- UXF-009 (S3, closed): the asset is `l2-layout.spec.ts :: L2 /ess/ mobile-390` and `... /ess/docs/visualise
  mobile-390`. Those two tests still fail (as known failures of UXF-005 and UXF-008 for other messages), so
  the asset does not "pass after" as a test; what changed is that its truncation error is gone. The
  before and after are measured numbers (6 truncated texts to 0, `ux/rounds/R-04/round.json` and `rj`), and
  a truncation message would now match no open finding's pattern and fail as a new failure. The record
  says this. It is S3, so the S1/S2 rule does not apply; no red-then-green asset run exists.
- UXF-005 to UXF-008 (open): each has an "Owner (R-05)" section naming the package
  `@beyond10x/ess-ui-templates`, the component `<EssPresentation>`, the pinned commit
  `655a7e58bc66...` (matches `website/package.json:19`), and that its `main` changed nothing under
  `web/` since. I checked: `git diff 655a7e58bc66 origin/main -- web` in the package is empty. UXF-005
  also names `@beyond10x/docs-system` `charts.js` at pin `5fd348a9` (matches `package.json:18`) and its
  commit `7fcf56d` in releases 0.15.0 and 0.15.1 (I confirmed the tags and that the pin is an ancestor).
  The four notes are one pasted paragraph: they name the component but not which element renders each
  rule (nested interactive control, scrollable region, label mismatch). Note: a fix for the landing page's
  graph exists in a release and was left; it is a reversible pin move that would take 2 of the 6
  `label-content-name-mismatch` nodes off gate 5. A judgement for the builder, not a defect.
- UXF-006 (open): owner note and archive part present (commit `1df979e6e5`, not `a229a58b6c`, which the
  builder's report says correctly). `regression_asset` shrank to the two visualise tests. After the
  record commit the "Before" numbers cite R-04's `round.json` (fixed in `2edaf3cd1a`), good.
- UXF-025 (open) and UXF-027 (open) carry this round's numbers: UXF-025 the runs of `rj` plus three runs
  not in `rj` (source dangling, § 1 row 6); UXF-027 `glance_cost.p95` 3 at mobile-390 for both cards and
  2 at desktop-1280 (`rj`: matches). UXF-027's asset is named "none yet; proposed: ..." though the test
  exists and passes; fine for an open record.
- Nothing is closed on a scratch file: both closes cite repository assets and `round.json` numbers.
  The two `gate-4 in R-05's round.json` citations (a run that is in no `round.json`) were corrected to
  R-04's in the record commit.
- UXF-026's metric line and body now say light (T34). It still says "the dark theme rose from R-02 to
  R-03", which are old values under the old key; acceptable as history.

## § 6 New tool defects (numbered after T42)

| Id | Defect | Where |
| --- | --- | --- |
| T43 | No rename operation: a value that moves to another scenario key is a drop plus a "new" value, so the ratchet compares nothing between the old and the new key; R-05's 556-key move needed a 556-line decision list (`dm` 643 lines) and a by-hand comparison to show nothing got worse | `rs:892` `update_with` (the `Lost` and `New` arms), `dm:50-614` |
| T44 | The "last round" column reads 0 when the previous round holds the scenarios but not the metric (`v[metric] ?? 0`), so a re-keyed metric shows as a rise from 0 (layout shift: R-04 0.124 and 0.0318 under dark) | `ux/core/round.ts:357`; `rp:132`, `rp:144` |
| T45 | `loadTheme` is optional and the fallback `c === rec.configs[0]` re-creates the T34 bug when a record lacks it; no test sets a record without it or checks that the L2 spec always sets it | `ux/core/reporter.ts:205`, `ux/specs/l2-layout.spec.ts:50` |
| T46 | `passed` is "outcome expected", which includes known failures that failed as expected; the name says "passed", and the rerun rescue reads it as a pass; no TypeScript test of `passed` or `unexpected_tests` | `ux/core/reporter.ts:379-387`, `rs:612` |
| T47 | `not_passed` skips a run with no `timing` and a round with no `runs`: T37 stays open and the new rule has the same hole (a round.json written without the lists is accepted) | `rs:598-660` |
| T48 | The tolerance notes are written to a string that is printed only when the ratchet succeeds: none on a bail; never in `rp`, `rj` or the baseline. The count has to be taken by hand and was taken wrong (§ 1 row 1) | `rs:1665-1668`, `rs:1713` |
| T49 | The notes number check is set membership: a digit string passes when the same number appears anywhere in `round.json`; spelled numbers ("Fourteen", "nine jobs", "Eleven ... Nine ... Three") and code spans are not read; it covers `notes.md` only, not `decision.md`, finding records or `triage.md` (T38 unchanged) | `ux/core/round.ts:409-438` |
| T50 | The gate count test checks constants, not the tests Playwright lists, so a gate spec outside `TEMPLATE_PAGES` and `GATE_TASK_RUNS` is not counted; `kinds` compare labels, so two pages of one kind with different DOM look identical | `ux/specs/l1-adapter.spec.ts:102-112`, `ux/adapter.ts:69` |

## Caveats

- I ran nothing. The red and green results are read from the kept logs of the scratch directory
  (`c1-red.log`, `c2-red-rust.log`, `c2-red-ts.log`, `c3-red.log`, `c5-red.log`, `c35-green.log`,
  `c2-green-rust.log`); the run table is read from `rj`.
- Item 4 of § 1 (the cause of the +11, +28, +23 and +13) is inferred from the deltas and the diff of
  `website/docs`; the page text was not rendered and the per-step words are not kept.
- The `unmeasured` column of `rp`'s run table shows 1 for each of the six L2 slices although each
  measured all of its pages (and 2 for `slow-l5`); the report does not say what it counts
  (`ux/core/round.ts:306`, `coverage.unmeasured.length`). I did not resolve it.
- The gate's time budget, and why `gate-2` and `gate-4` (load 68 to 72 at the start) took 114 s while
  `gate-8` (load 60) took 19.5 s, are for the separate diagnosis.
- `ux/trend.md` is at `website/ux/trend.md`, not `website/ux/rounds/trend.md`.

## Files

- `ux/rounds/R-05/report.md`, `round.json`, `decision.md`, `notes.md`, `triage.md`, `ux/baseline.json`
  (old and new), `ux/trend.md`, `ux/rounds/R-04/review-report.md`, `ux/rounds/R-03/review-report.md`
- `crates/edge/ess-xtask/src/ux_ratchet.rs`
- `ux/core/reporter.ts`, `ux/core/round.ts`, `ux/core/findings.ts`, `ux/adapter.ts`
- `ux/specs/l1-adapter.spec.ts`, `l1-findings.spec.ts`, `l1-reporter.spec.ts`, `l1-site.spec.ts`,
  `l2-layout.spec.ts`
- `ux/findings/UXF-005.md` to `UXF-009.md`, `UXF-016.md`, `UXF-025.md`, `UXF-026.md`, `UXF-027.md`
- `Taskfile.yml` (`ux-gate`, `ux-slow`, `ux-ratchet`)
