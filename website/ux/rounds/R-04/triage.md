# R-04 triage

Round R-04 first brings the every-merge gate back inside its 120 s budget (UXF-025, a test-budget
defect, so it comes before any other change), then repairs what the report review of R-03 found in
the tools (`../R-03/review-report.md`: exceptions X1 to X4, tool defects T25 to T33), and makes two
changes to the site that the heuristic review put first (`../R-03/review-heuristics.md`, "Changes
I would put first").

## Caveats

- The coordinator fixed the change set in the brief of R-04; this file records it in the skill's
  shape and recomputes the waiting list by the formula. The set does not follow the priority order:
  change 1 is a test-budget defect (the organization's rule puts it before the next feature), and
  changes 2 and 3 repair the measurement that every later priority is read from.
- The report review checked R-03's text claims against `round.json` and the kept logs: six were
  wrong (M1, M2, M3, M5, M6, M7) and are corrected in place; two could not be checked (M4, M8) and
  are marked "not kept" in place. `layout_shift.load` of `/ess/releases/outcome-shapes@mobile-390/dark`
  is 0 in `round.json`; the 0.0253 of its first run is in no committed file (T27).
- UXF-023 is open again: R-03 closed it on a scratch record, and `round.json` alone fits both
  readings of the landing page's rule (`../../findings/UXF-023.md`).
- Frequencies come from R-03's `round.json`: pages affected of 139 pages, task runs affected of 22
  task runs, runs affected of 18 runs, configurations of 556 where a finding is about a value per
  configuration. UXF-017 has no measured count in any round; its frequency is the R-01 estimate.
- R-03's gate runs ran at a 1-minute load of 61 to 92, as R-03's review reported (R-03's `round.json`
  holds no load; corrected in R-05 from "61 to 69"); R-04's gate runs log the load before
  and after each run.
- No person, recording or questionnaire in R-01 to R-03; L4 to L6 did not run.

## Did the last change set move its metrics?

R-03's five changes, with the numbers the report reviewer recomputed (`../R-03/review-report.md`,
§ 1 and § 2) and the heuristic reviewer recomputed (`../R-03/review-heuristics.md`).

| round | change | target metric | before (R-02) | after (R-03) | moved? |
| --- | --- | --- | --- | --- | --- |
| R-03 | 1 gate 5 measures every rule again | `gates.gate5_accessibility.by_rule` | 3 rules; no `nested-interactive`, no `scrollable-region-focusable` | 4 rules; `nested-interactive` 2 nodes and `scrollable-region-focusable` 1 node reported again, 22 `color-contrast` nodes | yes; the rise is the repaired measurement, taken under `R-03-axe` |
| R-03 | 2 the ratchet takes no worse value without a decision | worse values taken without a decision id | a `content` rise taken without one (T17) | 4 worse values, each under `R-03-axe`; 819 improvements taken | yes, with T29: the id covers every worse value of the update, `decision.md` is not read |
| R-03 | 3 the gate and the ratchet are wired and tell new failures from known ones | the gate's exit code; the step's wall time | no task; exit 1 on any known failure | `ux-gate`, `ux-slow`, `ux-ratchet`; `gate-5`, `gate-6` exit 0 with 8 known and 0 new; `gate-4` cut at 119.6 s | partly: T28 (a failure is held by title only), T30 (UXF-025), T31 (`ux-slow` one job, `ignore_error`) |
| R-03 | 4 links in running text told apart without colour | `gates.gate5_accessibility.by_rule.link-in-text-block.nodes`; `gates.gate5_accessibility.nodes` | 2,724; 2,732 | 0; 31 | yes |
| R-03 | 5 search: a decision file, no build | the yes/no question answered or defaulted | none | `../R-03/decision.md`, default no | yes; unanswered, so the default holds |

## Change set

At most 5 changes (§ 3.3 rule 3). Each is built failing-first in its own commit.

| # | change | covers | target | checked by |
| --- | --- | --- | --- | --- |
| 1 | The every-merge gate ends inside 120 s at host load 90 | UXF-025, T30 | three timed gate runs under 120 s, the load logged beside each; the gate lane still covers every page kind and every always-visible element | an L1 test that the gate lane holds no heavy page and no nested Playwright run, red before |
| 2 | A round is attributable to a committed tree, and a rerun cannot hide a worse value | T25, T26, T27, X1, X2 | a round with any run whose engine digest differs from the committed tree's, or whose commit is unknown, is refused; a replaced value is kept beside its replacement and the worse of the two is compared unless a decision names it | one test per case, red on R-03's code |
| 3 | A known failure is matched by its message, and a decision names its values | T28, T29, T31, T33, X3, X4 | an open finding's asset is an expected failure only when the message matches the record's pattern; a decision lists its scenario and metric keys in `decision.md` and the ratchet refuses a worse value it does not list; `ux-slow` one pool job per project; the report's notes carry no number `round.json` lacks | one test per case, red on R-03's code |
| 4 | A task that starts on a phone reaches its page in the card's limit | UXF-013, UXF-011 (or UXF-022, see the change's commit) | `gates.gate4_screen_only.over_limit` 10 entries to 5 or fewer | the findings' L3 assets |
| 5 | A reference row is reachable without scrolling many screens | UXF-010 | `ess-31-release/desktop-1280` and `/mobile-390` `screen_only.actions` 25 and 33 to 6 or fewer; `diagnostic-type-009/mobile-390` 7 to 5 or fewer | the finding's L3 assets |

Priorities of the set by the formula, for the record: UXF-025 S2 (3) × 1/3 gate runs cut × 2 (both
reviews) ÷ M (2) = 1.00; UXF-013 S3 (1) × 9/22 × 2 ÷ M (2) = 0.41; UXF-011 S3 (1) × 2/22 × 2 ÷ S
(1) = 0.18; UXF-010 S3 (1) × 3/22 × 2 ÷ M (2) = 0.14. The tool defects T25 to T33 carry no finding
record; they are the report review's exceptions to the baseline.

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

Highest first. No finding is S1. In the change set and not listed: UXF-010, UXF-011, UXF-013,
UXF-025. Settled by D15: UXF-002, UXF-003. Settled by the default of `../R-03/decision.md` (no
search of its own in this round): UXF-001. Closed: UXF-004, UXF-014, UXF-021.

| order | finding | severity | frequency (R-03) | confidence | cost | priority | note |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | UXF-026 layout shift moves with the host's load | S3 (1) | 278/556 configurations = 0.50 | 2 (report review T27 and heuristic 4) | S (1) | 1.00 | new in R-04 step 0 |
| 2 | UXF-024 a slice prints gates as fail that the merged round passes | S3 (1) | 11/18 runs = 0.61 | 1 | S (1) | 0.61 | the six L2 slices and five L2 reruns |
| 3 | UXF-015 tens of key presses to a sidebar link | S2 (3) | 4/22 = 0.18 | 2 (L3 and heuristic 9) | M (2) | 0.55 | |
| 4 | UXF-017 sidebar categories and table headers at about 11 px | S3 (1) | about 70/139 = 0.50 | 1 | S (1) | 0.50 | not measured; needs a font-size probe |
| 5 | UXF-012 the theme toggle's first press shows no change | S3 (1) | 2/22 = 0.09 | 2 (L3 and heuristic 2) | S (1) | 0.18 | |
| 6 | UXF-018 table headers scroll away | S3 (1) | 2/22 = 0.09 | 1 | S (1) | 0.09 | |
| 7 | UXF-005 graph nodes' names do not contain their label | S2 (3) | 2/139 = 0.014 | 2 (axe and heuristic 7) | S (1) | 0.086 | tie; landing and visualise |
| 8 | UXF-006 text below the contrast minimum | S2 (3) | 2/139 = 0.014 | 2 (axe and heuristic 15) | S (1) | 0.086 | tie; visualise and archive |
| 9 | UXF-007 a control nested in a control | S2 (3) | 1/139 = 0.007 | 2 (axe and heuristic 9) | S (1) | 0.043 | tie; visible again since R-03 |
| 10 | UXF-008 a scroll region the keyboard cannot reach | S2 (3) | 1/139 = 0.007 | 2 (axe and heuristic 9) | S (1) | 0.043 | tie |
| 11 | UXF-022 the sidebar out of view in a gated state on two short pages | S3 (1) | 2/139 = 0.014 | 2 (round.json and heuristic 16) | S (1) | 0.029 | tie; the fallback of change 4 |
| 12 | UXF-009 terminal titles cut at 390 px | S3 (1) | 2/139 = 0.014 | 2 (truncation and heuristic 3) | S (1) | 0.029 | tie |
| 13 | UXF-016 the Status page names 0.51.0 as the latest release | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | tie; no metric measures it |
| 14 | UXF-023 landing page contrast, light theme | S2 (3) | 1/139 = 0.007 | 1 | S (1) | 0.022 | tie; reopened in R-04 step 0 |
| 15 | UXF-019 "Get started" below the first screen at 1280 px | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
| 16 | UXF-020 a six-column table hides its last column | S3 (1) | 1/139 = 0.007 | 1 | S (1) | 0.007 | tie |
