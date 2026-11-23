- The gate lane runs 13 browser tests (change 1, UXF-025), about half of R-04's. It still does not
  hold its 120 s on a loaded host: `gate-5`, `gate-6` and `gate-7` started at a 1-minute load of
  133.34 to 169.55 and were cut at 116.2 to 117.2 s of wall time (status `timedout`, no new
  failure); `gate-8` started at 60.06 and passed in 19.5 s. UXF-025 records three earlier runs on
  the engine of change 1, which this `round.json` does not hold. UXF-025 stays open.
- The slow lane's eleven jobs (nine and two reruns) started at 55.92 to 131.15. One test timed out, `L2
  /ess/docs/reference/ess-ui desktop-1280` in `slow-l2-5` (load 78.21), and passed in
  `slow-rerun-a`; the ratchet takes `slow-l2-5` because a later run of the round passed the test
  it failed (T41).
- Truncated texts went from 6 to 0 and the release archive's contrast nodes from 4 to 0 (change 4,
  UXF-009 closed, UXF-006's archive part); gate 5 still fails on the visualise page's frame and the
  landing page's graph (UXF-005 to UXF-008), owned by the templates package and the shared docs
  theme.
- The index lists are named navigations (change 5, UXF-027). `glance_cost.p95` of
  `ess-31-release/mobile-390` and `diagnostic-type-009/mobile-390` stays 3: on a phone the first
  step opens the menu, and that step is already outside the primary region. UXF-027 stays open on
  its metric.
- Eleven reading-load values are worse than the baseline beyond their tolerance and carry
  `R-05-worse` (`decision.md`); seven of them rose by 16 words, the length of `main`'s added line on the docs introduction; the other four rose with `main`'s reworded pages and change 5's class names (not shown per step: the per-step words were in the discarded `.out/`).
  Fifteen more are worse inside their tolerance; the ratchet names each in a note (T36).
- `layout_shift.load` and `motion.running` are filed under the light configuration (T34): 556 dark
  values dropped and 556 light values new under `R-05-worse`. The two R-04 shifts taken under load
  (`../R-04/decision.md`) measure 0 under light, and in `slow-rerun-b` at load 55.92. The landing page's
  one shift of 0.0948 in `slow-l2-1` at load 131.15 measured 0 in `slow-rerun-b`, and `R-05-worse`
  takes that 0 by name.

## Corrected in round R-06

The report review of this round (`review-report.md`, § 1) found eight claims wrong and five without
a kept source. Each is corrected or marked in place: in these notes, in `report.md`'s copy of them,
in `decision.md`, in the finding records it named and in `../R-04/`.

| # | claim | where | corrected to | source |
| --- | --- | --- | --- | --- |
| 1 | "Fourteen more are worse inside their tolerance" | these notes | fifteen | `ratchet-full-2.log` (round scratch): 15 `note:` lines; `round.json` against `../../baseline.json` |
| 2 | "The slow lane's nine jobs ran at 55.92 to 131.15" | these notes | eleven jobs (nine and two reruns) started at 55.92 to 131.15 | `round.json`, `runs[].timing.load.before` |
| 3 | `gate-5` to `gate-7` "started at 133.47 to 169.55" | these notes, UXF-025 | 133.34 to 169.55 (`gate-6`) | `round.json`, `runs[name=gate-6].timing.load.before[0]` |
| 4 | "Eleven … Nine … Three" reading-load values from the docs introduction | `decision.md`, these notes | seven rose by 16 words (the added line); four rose with the reworded pages and change 5 | `decision.md` table, the deltas |
| 5 | R-03's gate runs "at a load of 61 to 92" | UXF-025, `../R-04/triage.md`, `../R-04/notes.md` | 61 to 69, the 1-minute values; R-05's step 0 had changed it wrongly | `../R-03/review-report.md`, row M3 |
| 6 | "`../rounds/R-05/notes.md` keeps the lines" of `gate-2` to `gate-4` | UXF-025 | not kept in the repository; the scratch logs hold them, `diagnosis-gate-time.md` § 1.1 quotes them | none in the repository |
| 7 | the visualise pair "87.8 of the 251 L2 test-seconds" | UXF-025 | 87.8 of 247.2, both from `gate-3` | the scratch log of `gate-3` |
| 8 | "last round" 0 in the two light layout-shift rows | `report.md` | not measured under this key; R-04 filed it under dark (0.124, 0.0318) | `../R-04/round.json` |

Without a kept source, marked "not kept" where they stand: the landing page's three gate-run reads
of 0 (`decision.md`); 0.97.4's publication time and asset list (UXF-016); the round file of
`gate-8` the gate lane's baseline was taken from (`decision.md`); "59 in all", stale for the final
engine's 62 (UXF-025); the two R-04 shifts at 0 under light, held indirectly by `replaced`
(`decision.md`).

The review's new tool defects, numbered after T42:

| id | defect | where |
| --- | --- | --- |
| T43 | no rename: a value moved to another scenario key is a drop plus a new value, compared with nothing | `crates/edge/ess-xtask/src/ux_ratchet.rs`, `update_with` |
| T44 | the "last round" column reads 0 where the last round has no value under the key | `core/round.ts`, `prevGroup` |
| T45 | `loadTheme` is optional and the fallback to the first configuration re-creates T34 | `core/reporter.ts`, `summarize` |
| T46 | `passed` holds known failures; no TypeScript test of `passed` or `unexpected_tests` | `core/reporter.ts`, `onTestEnd` |
| T47 | the ratchet skips a run without `timing` and a round without `runs` | `ux_ratchet.rs`, `not_passed` |
| T48 | tolerance notes are printed only when the ratchet succeeds | `ux_ratchet.rs`, `run` |
| T49 | the notes number check is set membership, reads no spelled number and covers `notes.md` only | `core/round.ts`, `notesNumbersMissing` |
| T50 | the gate count test checks constants, not the tests Playwright lists | `specs/l1-adapter.spec.ts` |
