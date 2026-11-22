# R-03 triage

Round R-03 repairs what the two reviews of R-02 found in the tools (`../R-02/review-report.md`:
exceptions E1 to E3, tool defects T16 to T24) and makes the first change to the site itself: one
style rule for links in running text (UXF-004). Search gets a decision, not a build (UXF-001).

## Caveats

- The coordinator fixed the change set in the brief of R-03; this file records it in the skill's
  shape and recomputes the waiting list by the formula.
- Three of the five changes repair the loop (E1, E2, E3), not the site. Change 1 repairs the axe
  measurement, so R-03's gate 5 numbers are not comparable with R-02's: a rise in `axe.*` is the
  repaired measurement and carries decision `R-03-axe` (`decision.md`), not a regression.
- Frequencies come from R-02's `round.json`: pages affected of 139 pages for page findings, task
  runs affected of 22 task runs for task findings, runs affected of 11 runs for a finding about a
  run. R-02 did not see `nested-interactive` and `scrollable-region-focusable` (UXF-021), so
  UXF-007 and UXF-008 keep R-01's frequency (1 of 139) and say so. UXF-017 has no measured count in
  either round; its frequency is the R-01 estimate.
- UXF-023's evidence does not hold up against R-02's raw records: the landing page's 2 nodes per
  light configuration are `label-content-name-mismatch` (UXF-005), the round's `color-contrast`
  nodes are the archive's `h1` (UXF-006). It keeps confidence 1 until R-03 measures it again
  (`../../findings/UXF-023.md`).
- No person, recording or questionnaire in R-01 to R-03; L4 to L6 did not run.

## Did the last change set move its metrics?

R-02's five changes, with the numbers the report reviewer recomputed (`../R-02/review-report.md`,
§ 1 and § 2).

| round | change | target metric | before (R-01) | after (R-02) | moved? |
| --- | --- | --- | --- | --- | --- |
| R-02 | 1 the every-merge gate is the template pages and the anchor cards; the rest is the `slow` lane | the gate step's wall time against 120 s | cut at 120 s, 283 tests not run | 41.6, 29.1, 28.7 s (`gate-7` to `gate-9`) | yes, with T19: timed at 6 workers while `package.json` says 4 |
| R-02 | 2 an unmeasured page is a failure | `coverage` and `unmeasured` | 138 of 139 pages, unmeasured counted clean | 139 of 139 pages, 556 of 556 configurations, 22 of 22 task runs, 417 of 417 measured-only; unmeasured 0 | yes; T11 and T12 partly repaired |
| R-02 | 3 the ratchet is compiled and reads coverage | `cargo xtask ux-ratchet` exists, its tests pass | not compiled, 18 tests never run | compiled, 30 tests pass | yes, with E2 (T17, T21, T22, T24) |
| R-02 | 4 the adapter states what this site has | `gates.gate2_always_visible.pages` | 138 of 138 | 2 of 139 | yes; the 2 are UXF-022 |
| R-02 | 5 the screen-only user is held to each card's limit | `gates.gate4_screen_only.over_limit` | none gated | 10 entries | yes: gate 4 can fail; gate 3 0 and 0 |

## Change set

At most 5 changes (§ 3.3 rule 3). Each is built failing-first in its own commit.

| # | change | covers | target | checked by |
| --- | --- | --- | --- | --- |
| 1 | Gate 5 measures every rule again | T16, E1, UXF-021 (and makes UXF-007, UXF-008 visible) | `gates.gate5_accessibility.by_rule` holds `nested-interactive` and `scrollable-region-focusable` again, or the cause is shown; the gate stays under 120 s | an L1 test on a doctored page with a nested control and an unreachable scroll region, red on the R-02 engine |
| 2 | The ratchet takes no worse value without a decision | T17, T21, T22, T24, E2 | a rising `content` value needs a decision id and is held against `main`; the baseline is written only after the last comparison; a dirty or disagreeing round is refused; `reading_load` has one unit | one Rust test per case in `ux_ratchet.rs`, red on the R-02 code; one for a lane addition with a worse value in the same pass |
| 3 | The gate and the ratchet are wired and tell new failures from known ones | T18, T19, T20, T23, E3 | `task ux-gate`, `ux-slow`, `ux-ratchet`; one worker count; the gate exits 0 on assets of open findings only, 1 on anything else and on an open finding's asset that passes; the report's baseline column and the baseline's `about` from the committed baseline; `round.spec.ts` under 60 s | L1 tests of the expected-failure rule and the report on canned inputs |
| 4 | Links in running text are told apart without colour | UXF-004 | `gates.gate5_accessibility.by_rule.link-in-text-block.nodes` 2,724 to 0 | UXF-004's asset, red before, green after; shots before and after |
| 5 | Search: a decision file, no build | UXF-001 | the yes/no question answered or defaulted | `decision.md` |

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

Highest first. No finding is S1. In the change set and not listed: UXF-001, UXF-004, UXF-021.
Settled by D15: UXF-002, UXF-003. Closed: UXF-014.

| order | finding | severity | frequency (R-02) | confidence | cost | priority | note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UXF-015 tens of key presses to a sidebar link | S2 (3) | 4/22 = 0.18 | 2 (L3 and heuristic 9) | M (2) | 0.55 | raised to S2 in step 0; tie, anchor cards first |
| 2 | UXF-024 a slice prints gates as fail that the merged round passes | S3 (1) | 6/11 runs = 0.55 | 1 | S (1) | 0.55 | tie |
| 3 | UXF-017 sidebar categories and table headers at about 11 px | S3 (1) | about 70/139 = 0.50 | 1 | S (1) | 0.50 | not measured; needs a font-size probe |
| 4 | UXF-013 two extra taps per link on a phone | S3 (1) | 9/22 = 0.41 | 2 (L3 and heuristic 3) | M (2) | 0.41 | every phone pointer run costs more than its desktop run |
| 5 | UXF-010 reference-table rows many screens down | S3 (1) | 4/22 = 0.18 | 2 (L3 and heuristics 8, 10) | M (2) | 0.18 | tie |
| 6 | UXF-011 "Verify conformance" four screens down | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 3) | S (1) | 0.18 | tie |
| 7 | UXF-012 the theme toggle's first press shows no change | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 2) | S (1) | 0.18 | tie |
| 8 | UXF-018 table headers scroll away | S3 (1) | 2/22 = 0.09 | 1 | S (1) | 0.09 | |
| 9 | UXF-006 text below the contrast minimum | S2 (3) | 1/139 = 0.007 | 2 (axe and heuristic 15) | S (1) | 0.043 | R-02: the archive's `h1` only |
| 10 | UXF-007 a control nested in a control | S2 (3) | 1/139 = 0.007 (R-01) | 2 (axe and heuristic 9) | S (1) | 0.043 | tie; R-02 did not see it (UXF-021) |
| 11 | UXF-008 a scroll region the keyboard cannot reach | S2 (3) | 1/139 = 0.007 (R-01) | 2 (axe and heuristic 9) | S (1) | 0.043 | tie; R-02 did not see it (UXF-021) |
| 12 | UXF-022 the sidebar out of view in a gated state on two short pages | S3 (1) | 2/139 = 0.014 | 2 (round.json and heuristic 16) | S (1) | 0.029 | tie |
| 13 | UXF-009 terminal titles cut at 390 px | S3 (1) | 2/139 = 0.014 | 2 (truncation and heuristic 3) | S (1) | 0.029 | tie |
| 14 | UXF-005 graph nodes' names do not contain their label | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | tie; R-02: the landing page only |
| 15 | UXF-016 the Status page names 0.51.0 as the latest release | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | tie; fixed outside this loop |
| 16 | UXF-023 landing page contrast, light theme | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | tie; the raw records do not support it (caveats) |
| 17 | UXF-019 "Get started" below the first screen at 1280 px | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
| 18 | UXF-020 a six-column table hides its last column | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |

R-03's own measurement (`report.md`) may move these frequencies, most of all the axe findings
after change 1; the next round's triage reads them from R-03.
