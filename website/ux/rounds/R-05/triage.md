# R-05 triage

Round R-05 first brings the every-merge gate inside its 120 s budget on a loaded host (UXF-025, a
test-budget defect, so it comes before any other change), then makes the ratchet judge the run
and not only its numbers (the report review of R-04, `../R-04/review-report.md`: T34 to T36 and
T41), then takes three changes to the site from the heuristic review
(`../R-04/review-heuristics.md`, "Changes I would put first", rows 1, 2, 3 and 5).

## Caveats

- The coordinator fixed the change set in the brief of R-05; this file records it in the skill's
  shape and recomputes the waiting list by the formula. Change 1 is a test-budget defect and
  change 2 repairs the measurement every later priority is read from, so the set does not follow
  the priority order.
- `main` was merged into the branch before any change (38 pages reworded, facts corrected,
  `website/src/canonical.ts` removed). Page text R-05 measures is `main`'s, so a reading-load or
  action count can move with that merge and not with a change of this round.
- The report review of R-04 found six claims wrong and four without a kept source. Each is
  corrected or marked "not kept" in place (`../R-04/notes.md`, "Corrected in round R-05";
  `../R-04/report.md`, `../R-04/decision.md`, `../R-04/triage.md`, UXF-025, UXF-026).
- The report review's tool defects T34 to T42 carry no finding record, like R-03's T25 to T33;
  they are listed under "Tool defects" below with what this round does with each.
- The heuristic review tables two site observations without a record. They are filed as UXF-027
  (the index lists: keyboard, name, landing) and UXF-028 (the sidebar's bottom edge and the
  unnamed desktop-1440 page).
- Frequencies come from R-04's `round.json`: pages affected of 139 pages, task runs affected of 22,
  runs of 17, configurations of 556 where a finding is about a value per configuration. UXF-017
  has no measured count in any round; its frequency is the R-01 estimate.
- R-04's whole round ran at a 1-minute load of 115 to 170 (`runs[].timing.load`) except
  `gate-12` to `gate-14` and the reruns (20 to 72); every R-04 timing is a loaded timing.
- No person, recording or questionnaire in R-01 to R-04; L4 to L6 did not run.

## Did the last change set move its metrics?

R-04's five changes, with the numbers the report reviewer confirmed (`../R-04/review-report.md`,
§ 1 to § 3) and the heuristic reviewer recomputed (`../R-04/review-heuristics.md`).

| round | change | target metric | before (R-03) | after (R-04) | moved? |
| --- | --- | --- | --- | --- | --- |
| R-04 | 1 the every-merge gate ends inside 120 s at host load 90 | the gate step's wall time, three runs at load 90 | `gate-4` cut at 119.6 s, `gate-5` 100.7 s, `gate-6` 93.6 s | `gate-9` to `gate-11` cut at 118.2, 117.7, 118.3 s at load 115 to 140; `gate-12` to `gate-14` 35.1, 17.9, 51.5 s at load 20 to 72 | no: fit below load about 70 only (review § 3); the tool's share is 17.9 s for 25 browser tests |
| R-04 | 2 a round is attributable to a committed tree, a rerun cannot hide a worse value | runs from another engine refused; the worse of a replaced value held | neither | `engine.runs_agree` true, digest checked, `replaced` `{}` | yes, with T37 (no site identity) and T41 (run status not read) |
| R-04 | 3 a known failure is matched by its message, a decision names its values | expected failures by `failure_pattern`; worse values only when listed | by title; one id took every value | 9 listed keys for 9 worse values, each `R-04-worse` | partly: a decision lists keys, not values (T35); the two-error case left the gate (T39) |
| R-04 | 4 a task that starts on a phone reaches its page in the card's limit | `gates.gate4_screen_only.over_limit` | 10 entries | 5 entries | yes (target 5 or fewer); `gate2_always_visible.pages` 2 to 0 beside it |
| R-04 | 5 a reference row is reachable without scrolling many screens | `ess-31-release` actions at desktop-1280 and mobile-390; `diagnostic-type-009/mobile-390` | 25, 33; 7 | 3, 5; 4 | yes (targets 6 and 5); reading load rose under `R-04-worse` |

## Change set

At most 5 changes (§ 3.3 rule 3). Each is built failing-first in its own commit.

| # | change | covers | target | checked by |
| --- | --- | --- | --- | --- |
| 1 | The gate fits its 120 s at host load 115 and above | UXF-025, T39 | the gate lane's browser tests from 25 to about 12; three timed gate runs under 120 s with the load logged; one cheap gate test runs a known and an unknown failure in one test | an L1 test that the gate lane holds at most its budgeted browser tests, red before; the L1 coverage test of page kinds and always-visible elements stays green |
| 2 | The ratchet judges the run, not only the numbers | T41, T35, T36, T34 | a round with an unexpected failure or a run that did not pass is refused; a rerun's worse value is held unless a decision lists scenario, metric and value; a decision names the value it takes; a worsening inside tolerance is printed; layout shift filed under the theme it was measured in | one test per case, red on R-04's code |
| 3 | The visualise page's frame (UXF-005 to UXF-008), or, when its owner must fix it, the status page and the install page agree on the latest release | UXF-005 to UXF-008, else UXF-016 | `gates.gate5_accessibility.nodes` 31 to 8 or fewer, else an L1 test that the two pages name one release | the findings' assets; a new L1 test for UXF-016, red before |
| 4 | Text is not cut at 390 px and the archive heading has contrast | UXF-009, UXF-006 | `gates.truncation.texts` 6 to 0; `gates.gate5_accessibility.nodes` down by 4 | the findings' L2 assets |
| 5 | An index list is a labelled navigation and a jump lands visibly | UXF-027 (follows UXF-010) | `glance_cost.p95` 3 to 2 on `ess-31-release/mobile-390` and `diagnostic-type-009/mobile-390`; the list is one named landmark | an L1 test on the built pages, red before |

Priorities of the set by the formula, for the record: UXF-025 S2 (3) × 3/6 gate runs cut × 2 (both
reviews) ÷ M (2) = 1.50; UXF-027 S3 (1) × 4/22 × 1 ÷ S (1) = 0.18; UXF-009 S3 (1) × 2/139 × 2 ÷ S
(1) = 0.029; UXF-006 S2 (3) × 2/139 × 2 ÷ S (1) = 0.086; UXF-016 S2 (3) × 1/139 × 1 ÷ S (1) =
0.022; UXF-005 to UXF-008 as listed below.

## Tool defects of the R-04 report review

| id | defect | R-05 |
| --- | --- | --- |
| T34 | layout shift filed under the wrong theme | change 2 |
| T35 | a decision lists keys, not values | change 2 |
| T36 | a worsening inside tolerance is silent | change 2 |
| T37 | no site identity, `runs` optional | waits; change 2 reads `runs` where present |
| T38 | evidence in the discarded `.out/`; the number check covers `notes.md` only | waits |
| T39 | the two-error case left the gate | change 1 |
| T40 | `ux-slow` stops at the first failing project | waits |
| T41 | the ratchet ignores run status, unexpected counts and reruns | change 2 |
| T42 | the round date is UTC; `GATE_PAGE_ELEMENTS` has no stated source | waits |

## Waiting for the next round

By the formula of § 3.3, verbatim:

```
priority = severity weight × frequency × confidence ÷ cost

severity weight  S1 = 9, S2 = 3, S3 = 1
frequency        calls affected ÷ calls in the round   (0 to 1)
confidence       2 when two sources agree (e.g. a metric and the heuristic review,
                 or a recording and the survey), else 1
cost             S = 1, M = 2, L = 4
```

Highest first. No finding is S1. In the change set and not listed: UXF-005 to UXF-009, UXF-016,
UXF-025, UXF-027. Settled by D15: UXF-002, UXF-003. Settled by the default of
`../R-03/decision.md` (no search of its own): UXF-001. Closed: UXF-004, UXF-010, UXF-011,
UXF-014, UXF-021, UXF-022.

| order | finding | severity | frequency (R-04) | confidence | cost | priority | note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UXF-026 layout shift moves with the host's load | S3 (1) | 278/556 configurations = 0.50 | 2 (report review T34 and heuristic 4) | S (1) | 1.00 | change 2 moves the value to its theme; the load dependence stays |
| 2 | UXF-024 a slice prints gates as fail that the merged round passes | S3 (1) | 10/17 runs = 0.59 | 1 | S (1) | 0.59 | six L2 slices, two reruns, `gate-9`, `gate-11` |
| 3 | UXF-015 tens of key presses to a sidebar link | S2 (3) | 4/22 = 0.18 | 2 (L3 and heuristic 9) | M (2) | 0.55 | |
| 4 | UXF-017 sidebar categories and table headers at about 11 px | S3 (1) | about 70/139 = 0.50 | 1 | S (1) | 0.50 | not measured; needs a font-size probe |
| 5 | UXF-012 the theme toggle's first press shows no change | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 2) | S (1) | 0.18 | |
| 6 | UXF-013 the phone menu opens on the docs panel | S3 (1) | 3/22 = 0.14 | 2 (L3 and heuristic 3) | M (2) | 0.14 | owned by the framework's theme |
| 7 | UXF-018 table headers scroll away | S3 (1) | 2/22 = 0.09 | 1 | S (1) | 0.09 | |
| 8 | UXF-023 landing page contrast, light theme | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | tie |
| 9 | UXF-028 the sidebar's bottom edge; an unnamed desktop-1440 page | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie; new in R-05 step 0 |
| 10 | UXF-019 "Get started" below the first screen at 1280 px | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
| 11 | UXF-020 a six-column table hides its last column | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
