- The host's load decided this round's times. `gate-9`, `gate-10` and `gate-11` ran at a
  1-minute load of 115.29 to 140.34 and were cut at the 120 s bound (118.2, 117.7 and 118.3 s of
  wall time, status `timedout`), with no new failure. `gate-12`, `gate-13` and `gate-14` ran at
  20.01 to 71.61 and passed in 35.1, 17.9 and 51.5 s. UXF-025 stays open: the gate does not hold
  its bound on a loaded host.
- The slow lane's six L2 jobs ran at 117.83 to 170.96; 16 of their tests did not pass (15 timed
  out, 1 failed: `runs[].timing.unexpected`) and ran again, with 3 more, 19 tests in all, in
  `slow-rerun-a` and `slow-rerun-b` at 54.25 to 68.73 and passed (24.6 and 5.8 s of wall time).
- Gate 4 lists 5 runs over their card's limit. `ess-31-release`, `diagnostic-type-009` and
  `target-process` left the list (changes 4 and 5); the phone navigation of
  `first-conformance-run`, `limitations` and `page-a-view` is the shared theme's (UXF-013).
- Gate 2 passes: the sidebar stays in view in the middle state on short pages (UXF-022).
- Nine values are worse than the baseline and carry `R-04-worse` (`decision.md`): the reading
  load of the two index pages, one keyboard action on two phone cards, and one layout shift each
  on two pages measured under load. Against R-03's round values 13 are higher: the 9, and 4 more
  inside their tolerance that no decision names. That two probe runs measured the two shifts as 0
  is not kept: the probe logs hold no layout-shift value.

## Corrected in round R-05

The report review of this round (`review-report.md`, § 1) found six claims wrong and four without
a kept source. Each is corrected or marked here, in `report.md`'s copy of these notes, in
`decision.md` and in the finding records it named.

| claim | where | corrected to | source |
| --- | --- | --- | --- |
| "Nine values are worse" read as worse than R-03 | these notes | 9 worse than the baseline; 4 more higher than R-03 inside their tolerance | the review's `rv-diff` and `rv-rb` of `round.json` and the baseline |
| the layout shift "measured in the dark theme" | `decision.md`, UXF-026 | measured under the light theme, filed under `dark` by the reporter (T34) | `specs/l2-layout.spec.ts` (`colorScheme: THEMES[0]`), `core/reporter.ts` |
| "17 tests timed out and were rerun" | the builder's report | 16 tests did not pass (15 timed out, 1 failed); 19 reran | `round.json`, `runs[].timing.unexpected` |
| R-03's gate "63 tests (36/19/8)" | UXF-025 | `gate-4` ran 62: 36, 19, 7 | `../R-03/round.json`, `runs[name=gate-4].timing.projects` |
| R-03's gate runs "at load 61 to 69" | UXF-025, `triage.md` | not wrong: the 1-minute values were 61.24 to 69.39 (restored in R-06; R-05 wrote "61 to 92", which adds the 15-minute value 91.73); R-03's `round.json` holds no load | `../R-03/review-report.md`, row M3 |
| "two pages that changes 1 to 5 did not touch" | `decision.md` | change 4 edits `website/src/css/custom.css`, which every page uses | commit `5f6ae80d36` |
| two probe runs "measured 0" | `decision.md`, these notes | not kept | none |
| `.out/gate-<n>/raw.json` and the `gate-8` row | UXF-025 | not kept: `.out/` is discarded and `gate-8` is in no `round.json` | none |
| `gate-9` "16 passed" | UXF-025 | not kept: the count holds two known failures | none |
| "R-03 63 tests" | UXF-025 | replaced by `../R-03/round.json`'s 62 | `../R-03/round.json` |
