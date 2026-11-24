# R-02 triage

Round R-02 changes the measuring tools only. The review of R-01 (`../R-01/review-report.md`) found
them unfit: tool defects T1 to T15, and a gate that ran about 30 minutes against its 120 s budget
(UXF-014). The site is not changed in this round.

## Caveats

- The change set comes from the report review, not from site findings: all five changes are
  repairs of the loop. The coordinator chose them; this file records them in the skill's shape.
- R-01's numbers were measured by the tools under repair, and its raw output is gone. They are not
  a base to compare R-02 with: R-02 replaces the baseline under a decision (`decision.md`).
- Frequencies in the waiting list come from R-01, the only round so far: pages affected of 139
  pages for page findings, task runs affected of R-01's 22 task runs for task findings. Costs are
  the heuristic reviewer's where it gave one, otherwise estimated here (S one rule or one file,
  M one component or a plugin).
- No person, no recording and no questionnaire in R-01 or R-02; L4 to L6 did not run.

## Did the last change set move its metrics?

| round | change | target metric | moved? |
| --- | --- | --- | --- |
| R-01 | none: R-01 was the first round | – | – |

## Change set

At most 5 changes (§ 3.3 rule 3).

| # | change | covers | target | checked by |
| --- | --- | --- | --- | --- |
| 1 | The every-merge gate is a fixed set of template pages at the two gating viewports and both themes, plus the two anchor cards; the full sitemap and all 9 cards move to a lane named `slow`, split into pool jobs | UXF-014 | the gate's test step ends within 120 s, 3 timed runs; no test over 60 s, `/ess/docs/reference/ess-ui` included | Playwright `globalTimeout` 120 s on the gate lane, `timeout` 60 s per test; the reporter's run time and ten slowest tests |
| 2 | An unmeasured page is a failure, never a pass | T1, T10, T11, T12, T13 | a page or configuration that timed out or did not run counts as unmeasured and fails its gate; reruns replace, never add; `round.json` holds every number the report prints, the coverage with denominators and the engine's commit and digest | L1 tests of the reporter on canned records (a cut test, a rerun, a run that never started) |
| 3 | The ratchet is compiled and reads coverage | T2, T14, T15 (second half) | `cargo xtask ux-ratchet` exists; lost coverage is a breach; a new page is "new", not "worse"; any improvement must be taken; `main`'s baseline unreadable is an error | the module's unit tests, run in the crate's test build |
| 4 | The adapter states what this site has | T3, T4, T5, T15 (first half); UXF-001 to UXF-003 | search is one site finding (UXF-001) with one asset, not 548 failures; the sidebar is gated at the top and the middle of a page (D15, `decision.md`); the archive expects no sidebar | an L2 test that is red while no search control is built and green when one is; L1 tests of the gated states |
| 5 | The screen-only user is held to each card's limit | T6, T7, T8, T9; UXF-010 to UXF-013 | gate 4 fails a run over its card's action limit; the card's anchor and "beside" conditions are checked; scrolling is decided from the screen alone; gate 3 can fail | L1 tests on doctored pages: over the limit, wrong anchor, a target under a sticky bar after a jump (`completion_miss`), a control found only by scanning |

## Waiting for the next round

By the formula of § 3.3 (`priority = severity weight × frequency × confidence ÷ cost`), highest
first. No finding is S1. UXF-014 is in this change set; UXF-002 and UXF-003 are settled by D15 for
the gate (the end-of-page state is reported, not gated) and wait for nothing.

| order | finding | severity | frequency | confidence | cost | priority | note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UXF-004 links told apart by colour only | S2 (3) | 102/139 = 0.73 | 2 (axe and heuristic 15) | S (1) | 4.40 | one site-wide style rule |
| 2 | UXF-001 no search | S2 (3) | 138/139 = 0.99 | 2 (gate 2 and heuristics 3, 10) | M (2) | 2.98 | after change 4: one finding, one asset |
| 3 | UXF-017 sidebar categories and table headers at about 11 px | S3 (1) | about 70/139 = 0.50 | 1 | S (1) | 0.50 | needs a font-size probe first |
| 4 | UXF-013 two extra taps per link on a phone | S3 (1) | 9/22 = 0.41 | 2 (L3 and heuristic 3) | M (2) | 0.41 | anchor cards |
| 5 | UXF-015 about 20 Tab presses to a sidebar link | S3 (1) | 4/22 = 0.18 | 2 (L3 and heuristic 9) | M (2) | 0.18 | tie, anchor cards first |
| 6 | UXF-010 reference-table rows many screens down | S3 (1) | 4/22 = 0.18 | 2 (L3 and heuristics 8, 10) | M (2) | 0.18 | tie |
| 7 | UXF-011 "Verify conformance" four screens down | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 3) | S (1) | 0.18 | tie |
| 8 | UXF-012 the theme toggle's first press shows no change | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 2) | S (1) | 0.18 | tie |
| 9 | UXF-018 table headers scroll away | S3 (1) | 2/22 = 0.09 | 1 | S (1) | 0.09 | |
| 10 | UXF-006 text below the contrast minimum | S2 (3) | 2/139 = 0.014 | 2 (axe and heuristic 15) | S (1) | 0.086 | |
| 11 | UXF-005 graph nodes' names do not contain their label | S2 (3) | 2/139 = 0.014 | 1 | S (1) | 0.043 | tie |
| 12 | UXF-007 a control nested in a control | S2 (3) | 1/139 = 0.007 | 2 (axe and heuristic 9) | S (1) | 0.043 | tie |
| 13 | UXF-008 a scroll region the keyboard cannot reach | S2 (3) | 1/139 = 0.007 | 2 (axe and heuristic 9) | S (1) | 0.043 | tie |
| 14 | UXF-009 terminal titles cut at 390 px | S3 (1) | 2/139 = 0.014 | 2 (truncation and heuristic 3) | S (1) | 0.029 | |
| 15 | UXF-016 the Status page names 0.51.0 as the latest release | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | being fixed outside this loop, in the documentation sweep |
| 16 | UXF-019 "Get started" below the first screen at 1280 px | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
| 17 | UXF-020 a six-column table hides its last column | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |

R-02's own measurement (`report.md`) may move these frequencies; the next round's triage reads
them from R-02.
