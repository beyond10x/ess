# R-06 triage

Round R-06 first makes the every-merge gate hold its budget where the host now judges a budget, at
a 1-minute load of 60 or less, and say the load beside its time (UXF-025, a test-budget defect, so
it comes before any other change). Then it takes the numbers out of the round's hand-written prose
(the report review of R-05, `../R-05/review-report.md`: T44 to T49, and T39's missing red run),
and three changes to the site, two of them proposed by the heuristic review
(`../R-05/review-heuristics.md`, "Changes I would put first", rows 1 and 4) and one from the
owner notes of UXF-005 (the shared docs theme's release 0.15.0 fixes the landing page's graph).

## Caveats

- The coordinator fixed the change set in the brief of R-06; this file records it in the skill's
  shape and recomputes the waiting list by the formula. Change 1 is a test-budget defect and
  change 2 repairs the record every later round reads, so the set does not follow the priority
  order.
- The host's working rule since 2026-11-23: a test budget (60 s per test, 120 s per step) is judged
  at a 1-minute load of 60 or less; a run over budget at a higher load is reported, not counted as
  a defect. R-05's gate runs are read by that rule below.
- `main` with release 0.98.0 is merged into the branch before change 4, which moves the same lines
  of the install and status pages. Page text R-06 measures after that merge is `main`'s, so a
  reading-load or action count can move with the merge and not with a change of this round.
- The report review of R-05 found eight claims wrong and five without a kept source. Each is
  corrected or marked "not kept" in place (`../R-05/notes.md`, "Corrected in round R-06").
- The report review's tool defects T43 to T50 carry no finding record, like T25 to T42; they are
  listed under "Tool defects" below with what this round does with each.
- The heuristic review tables three observations without a record (index lists without list
  semantics; the visualise model reachable only through the frame; the release number typed in 16
  places). The third is change 4; the first is change 5's fallback; the second waits.
- Frequencies come from R-05's `round.json`: pages affected of 139 pages, task runs affected of 22,
  runs of 15, light configurations of 278 where a finding is about a page-load value. UXF-017 has
  no measured count in any round; its frequency is the R-01 estimate.
- No person, recording or questionnaire in R-01 to R-05; L4 to L6 did not run.

## Did the last change set move its metrics?

R-05's five changes, with the numbers the report reviewer confirmed (`../R-05/review-report.md`,
§ 1 to § 4) and the diagnosis of the gate's time (`../R-05/diagnosis-gate-time.md`).

| round | change | target metric | before (R-04) | after (R-05) | moved? |
| --- | --- | --- | --- | --- | --- |
| R-05 | 1 the gate fits its 120 s at host load 115 and above | the gate step's wall time | `gate-9` to `gate-11` cut at 117.7 to 118.3 s, load 115 to 140 | `gate-5` to `gate-7` cut at 116.2 to 117.2 s, load 133 to 170; `gate-8` 19.5 s at load 60.06 | no at load 115 and above (every run started there was cut, 0 new failures); held at the load the host now judges by (`gate-8`) |
| R-05 | 2 the ratchet judges the run, not only the numbers | a failed or cut run refused; a rerun's worse value held; a decision names its value; a note for a worsening inside tolerance; layout shift under its theme | none of the five | all five, each red then green | yes (T34, T35, T36, T41 closed); T39 claimed, without a red run; new T43 to T50 |
| R-05 | 3 the visualise frame, else the status and install pages agree on the latest release | `gates.gate5_accessibility.nodes`, else an L1 test | 31 nodes; status 0.97.3, install 0.97.4 | 27 nodes (change 4's); both 0.97.4, the test red then green | the fallback, yes (UXF-016 closed); the frame is the templates package's (UXF-005 to UXF-008) |
| R-05 | 4 text not cut at 390 px; the archive heading has contrast | `gates.truncation.texts`; gate-5 nodes | 6; 31 | 0; 27 | yes |
| R-05 | 5 an index list is a labelled navigation, a jump lands visibly | `glance_cost.p95` of `ess-31-release/mobile-390` and `diagnostic-type-009/mobile-390` | 3, 3 | 3, 3 | no: on a phone the first step is the menu, glance 3 already (first round of this change) |

## Change set

At most 5 changes (§ 3.3 rule 3). Each is built failing-first in its own commit.

| # | change | covers | target | checked by |
| --- | --- | --- | --- | --- |
| 1 | The gate holds its budget where the budget is judged, and says the load | UXF-025 | first the diagnosis' probe (three interleaved pairs, B/A); if it holds: the four known-failing browser tests to the slow lane, the gate prints the load beside its wall time and says on an over-budget run whether the load was above 60 | the probe; the L1 count and coverage tests under a recorded decision; a test of the budget line, red before |
| 2 | No number in a round's prose that the record does not hold | T49, T44, T45, T46, T47, T48, T39 | notes refuse digits and spelled numbers; finding records' numeric lines written by the round stage; "not measured" for a missing last-round value; no theme fallback; `passed` without known failures; a round without `runs` or `timing` refused; notes printed on a failure; T39's test red on the code before its fix | one test per item, red on today's code |
| 3 | The first screen sends a phone reader on without the menu | UXF-013, UXF-019 | `screen_only.actions` of `first-conformance-run/mobile-390` and `limitations/mobile-390` 4 to 3; the `/ess/docs` first screen's reading load not above R-04's | an L1 test on the built pages, red before; the L3 runs |
| 4 | The newest release is written once | new (follows UXF-016) | hand-typed newest-release numbers in the docs from 16 to 0 | an L1 test that fails on a hand-typed newest-release number, red before |
| 5 | The landing page's graph passes hard gate 5 | UXF-005 to UXF-008 | gate-5 nodes from 27 down by the landing's 4; no regression on any other page; else the index lists as `ul` (UXF-027) | the gate and the slow lane; the findings' assets |

Priorities of the set by the formula, for the record: UXF-025 S2 (3) × 3/4 gate runs cut × 2 (both
reviews and the diagnosis) ÷ M (2) = 2.25; UXF-013 S3 (1) × 3/22 × 2 ÷ M (2) = 0.14; UXF-019 S3 (1)
× 1/139 × 1 ÷ S (1) = 0.007; the release number S3 (1) × 1/22 × 1 ÷ S (1) = 0.045; UXF-005 S2 (3)
× 2/139 × 2 ÷ S (1) = 0.086.

## Tool defects of the R-05 report review

| id | defect | R-06 |
| --- | --- | --- |
| T37 | no site identity, `runs` optional | change 2 (the `runs` half, with T47) |
| T38 | evidence in the discarded `.out/`; the number check covers `notes.md` only | change 2 (no number in hand-written prose) |
| T39 | the two-error case has no red run | change 2 |
| T40 | `ux-slow` stops at the first failing project | waits |
| T42 | the round date is UTC; `GATE_PAGE_ELEMENTS` has no stated source | waits |
| T43 | no rename: a re-keyed value is compared with nothing | waits |
| T44 | the "last round" column reads 0 where the last round has no value | change 2 |
| T45 | `loadTheme` optional; the fallback re-creates T34 | change 2 |
| T46 | `passed` holds known failures; no TypeScript test of the lists | change 2 |
| T47 | the ratchet skips a run without `timing` and a round without `runs` | change 2 |
| T48 | tolerance notes printed only on success | change 2 |
| T49 | the notes number check is set membership and reads no spelled number | change 2 |
| T50 | the gate count test checks constants, not the listed tests | waits |

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

Highest first. No finding is S1. In the change set and not listed: UXF-005 to UXF-008, UXF-013,
UXF-019, UXF-025, UXF-027 (change 5's fallback). Settled by D15: UXF-002, UXF-003. Settled by the
default of `../R-03/decision.md` (no search of its own): UXF-001. Closed: UXF-004, UXF-009,
UXF-010, UXF-011, UXF-014, UXF-016, UXF-021, UXF-022.

| order | finding | severity | frequency (R-05) | confidence | cost | priority | note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UXF-024 a slice prints gates as fail that the merged round passes | S3 (1) | 9/15 runs = 0.60 | 1 | S (1) | 0.60 | six L2 slices, two reruns, `gate-6` |
| 2 | UXF-015 tens of key presses to a sidebar link | S2 (3) | 4/22 = 0.18 | 2 (L3 and heuristic 9) | M (2) | 0.55 | 21, 23, 129, 131 key presses |
| 3 | UXF-017 sidebar categories and table headers at about 11 px | S3 (1) | about 70/139 = 0.50 | 1 | S (1) | 0.50 | not measured; needs a font-size probe |
| 4 | UXF-026 layout shift moves with the host's load | S3 (1) | 30/278 light configurations = 0.11 | 2 (report review and heuristic 4) | S (1) | 0.22 | |
| 5 | UXF-012 the theme toggle's first press shows no change | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 2) | S (1) | 0.18 | |
| 6 | UXF-018 table headers scroll away | S3 (1) | 2/22 = 0.09 | 1 | S (1) | 0.09 | |
| 7 | UXF-028 the sidebar's bottom edge; an unnamed desktop-1440 page | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
| 8 | UXF-020 a six-column table hides its last column | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
| 9 | UXF-023 landing page contrast, light theme | S2 (3) | 0/139 = 0 | 1 | S (1) | 0 | no contrast node on the landing page in R-05 |
