# R-04 review: report reviewer

Paths are under `website/ux` unless they start with `crates/`, `website/src`, `website/docs` or are
`Taskfile.yml`. `rs:N` is `crates/edge/ess-xtask/src/ux_ratchet.rs:N`; a line marked "about" was
read in the file but not re-checked after the last edit. `rj` is `rounds/R-04/round.json`, `rp` is
`rounds/R-04/report.md`, `dm` is `rounds/R-04/decision.md`. The round is branch
`ux/docs-site-loop-r01`, head `4150f0a494`, commits `0d51e1e891..4150f0a494`. Nothing was built,
run or written in the repository. "Inferred" marks a claim read from code or from two numbers and
not proved by running. Helper jq files (`rv-*.jq`) are in the round's scratch directory, not in the
repository.

## Verdict

**Not fit to send as is.** Default action: fix the five wrong claims (§ 1), add the value-level
decision list (§ 2, item 2), and rewrite the two layout-shift values as "not a good value"
(§ 2, item 3). Then send. The baseline itself is usable: the diff old to new is 9 worse, 46
better, 3501 same, 3556 values before and after, and all 9 worse carry `R-04-worse`.

- Point 2 (decision `R-04-worse`): accept 5 reading-load values, accept 2 keyboard values, needs
  evidence for 2 layout-shift values, accept the lane move with caveats.
- Point 4 (gate budget UXF-025): fit below a stated load; not fit for `task check` as it is.
- Wrong claims: 5. Claims with no kept source: 4.
- New tool defects: T34 to T42.

## § 1 Numbers recomputed

Method: every row of the tables in `rp` and `trend.md` was parsed (244 rows: 220 task rows and 24
sum rows) and compared by jq with `rj`, the new `baseline.json` and the R-03 `round.json`
(`rv-tab.jq`). The first run printed `[]`; I confirmed that the parser had read all 244 rows, so
"no mismatch" is a real result for the tables. The text claims below were checked one by one.

| Claim | Where | Source says | Verdict |
| --- | --- | --- | --- |
| "nine values are worse" than the baseline | `rp` | 9 worse against the old baseline (`rv-diff.jq`), all `R-04-worse`; against R-03's round values 13 are higher (`rv-rb.jq`), 4 of them within tolerance and not in `dm` | Wrong as written: it says "than R-03" in the sentence. Say "9 worse than the baseline, 4 more within tolerance" |
| Layout shift "measured in the dark theme" | `rp`, UXF-026 | `specs/l2-layout.spec.ts` sets `colorScheme: THEMES[0]`, which is `light` (`adapter.ts:52`); the value is filed under `dark` because `core/reporter.ts:197` puts it on `rec.configs[0]` after `mergeRecords` sorts the configs, and `dark` sorts first | Wrong: measured under light, filed under dark (T34) |
| "17 tests timed out and were rerun" | `rp`, builder note | 16 reruns of this kind: 15 `timedOut` and 1 `failed`; 19 reruns in all (`rj` runs) | Wrong: 16 |
| UXF-025 "R-03 63 tests (36/19/8)" | finding record | R-03 gate-4 ran 36/19/7; the A/B run of this round ran 37/19/8 | Wrong |
| UXF-025 and triage "load 61 to 69" | finding, `triage.md` | R-03 review reported 61 to 92; `rj` runs hold 115 to 170 (`timing.load`); no R-04 load figure for the R-03 runs is kept | Wrong |
| "pages changes 1 to 5 did not touch" | `rp` | change 4 edits `website/src/css/custom.css` (`min-height: calc(100vh + 10rem)` for the docs main wrapper, `+16rem` for the blog row at 997 px and wider), which every page uses | Wrong |

All other numbers match: counts of scenarios and values, the 46 better, the sums of reading-load,
the keyboard counts, the gate test counts of R-04, the engine digest of every run, and
`engine.runs_agree: true`. `replaced` in `rj` is `{}`, so no rerun replaced a value.

Claims with no kept source (the number may be right; nothing in the scratch directory or the
repository shows it):

1. The probe "measured 0" for layout shift: `probe-cls-1.log` and `probe-cls-2.log` hold no
   `layout_shift` value.
2. The path `.out/gate-<n>/raw.json` and the "gate-8" row: `.out/` is discarded (T38).
3. "16 passed" counts two known failures as passed.
4. "R-03 63 tests" has no kept R-03 log in this round; only the R-03 review's own counts.

## § 2 The decision `R-04-worse` and the three tool changes

### Item 1: does `dm` list every worse key, and does the ratchet read it

`dm` lists 9 keys. The old to new diff has 9 worse values. The lists agree key by key. The ratchet
reads `dm`: `decision_of` (rs:665) takes the backticked list items of the section whose `## `
heading names the id; `covers` (rs:711) accepts a worse value only when its
`<scenario> <metric>` or its whole `<scenario>` is listed; `update_with` (about rs:720 to 820)
refuses an unlisted worse value. The tests are `a_decision_takes_only_the_values_it_lists` and
`a_decision_is_read_from_its_section_of_the_decision_file` (about rs:2571 to 2655). What the
decision does not do: it lists keys, not values (T35). A decision that names
`/ess/docs/index@mobile-390/light reading_load.words` takes any later value of it. CI never reads
`dm`: `task ux-ratchet` reads the committed baseline, whose `decision` field holds only the id
(T35).

### Item 2: each accepted worsening

| Items | Say | Reason |
| --- | --- | --- |
| 5 reading-load values on two index pages | accept | The words come from the new intro link (UXF-011, `website/docs/index.md`) and the rewritten lead; the rise is 5 words or fewer per value and the finding says why. One more value on the same pages rose 256 to 265 and sits inside the 5 % tolerance, so it is not listed; list it or say it is within tolerance. The values were not checked against the page text; the reason is inferred from the diff |
| 2 phone keyboard cards, +1 action each | accept | The one added action is the new intro link on the page they start from; the cause is inferred from the diff, not run |
| 2 layout-shift values "measured under load" | needs evidence | The shift is an absolute metric with tolerance 0.01. A value measured under load is not a good value and should not become the baseline. Two causes are open: UXF-026 (the shift of the page that grows after load) and the change-4 CSS. The probes the report cites hold no value (§ 1). Take them out of the baseline with a decision of kind "not measured", or rerun on a host under load 70 and keep the log |
| 8 scenarios leave the gate lane, 4 enter | accept, with caveats | The 8 values stay in the full lane, so no value is lost. Caveats: the "heavy" evidence is load-confounded (the gate ran at load 115 to 170); the A/B run shows no gain in one pair; per-merge coverage of the generated reference pages is lost, and two of them were the pages that failed most often. The 4 that enter are the anchor cards; the lane count rule (`lane-dropped`) took them correctly |

### Item 3: the three tool changes

**(a) Attribution and reruns.** Mostly sound for the engine tree. `engine_digest` and
`committed_engine_digest` (rs:1384) hash `website/ux` minus `NOT_ENGINE`, and `engine.runs_agree`
is checked. A rerun cannot hide a worse value that differs from the first: `replacedValues`
(`core/round.ts`) keeps the replaced value and `held_value` (about rs:515 to 535) takes the larger.
A decision naming the key takes the rerun's value, as intended. Missing, and not a hiding route
today (T37, T41):

- there is no site identity: nothing ties a run to the website commit or the built `build/` tree,
  so a run of an older site with the same engine digest merges;
- there is no run manifest: `runs` is optional in `round.json`, and the ratchet does not read
  status, `unexpected` or what was rerun.

**(b) Known failures.** Matched by message: `holdKnownFailures` and `unmatchedErrors`
(`core/findings.ts`), the front-matter `failure_pattern`, and `expect.soft` per failure in
`specs/l2-layout.spec.ts`. Another failure in the same test fails the gate when it does not match
the pattern, which is the intended behaviour; `failure_pattern_problems` (about rs:1264) checks the
pattern is a valid expression. Gaps: a pattern that names a shared asset (a stylesheet, a font)
matches a different page's failure; some patterns are broad enough to match two kinds of failure;
the UXF-023 pattern matches no failure any more (dead); the nested exit-code tests that held the
two-error case moved to the slow lane (T39), so no gate test runs one test with a known and an
unknown failure together.

**(c) Gate coverage.** The gate lane still covers all 5 page kinds and every always-visible
element. The L1 test is `specs/l1-adapter.spec.ts`, the coverage test at about lines 100 to 108,
which compares `GATE_PAGE_ELEMENTS` (1200, `adapter.ts`) and the template pages against the
sitemap. One gap: no template page pins a wide table, so a table that spills on a page kind not in
the gate is found only in the slow lane.

## § 3 The gate budget (UXF-025)

Verdict: **fit below a stated load; not fit for `task check` as it is.**

From the per-test times of the kept gate logs (`gate-1..14.log`) and the A/B (`ab.out`):

- Tool's own share: 17.9 s of wall time and 61 test-seconds, that is about 15 % of the 120 s
  budget, for 25 browser tests at 4 workers. No single test comes near 60 s on a quiet host.
- The host's share: at 1-minute load 115 to 140 on this 20-core host the same tests ran 4.8 to
  8.9 times slower, so the step ended at 85 to 160 s and `globalTimeout` (120 s,
  `playwright.config.ts`, `gateProjects`) killed it in the worse runs.
- Stated load: fit while the 1-minute load stays at or below about 70 on 20 cores. At 90 and above
  it is not fit (the R-03 review measured the same at 90). Inferred from the ratio; I did not run
  the gate.

Why not fit for `task check`: `task check` calls none of the UX tasks (`Taskfile.yml`: `ux-gate`,
`ux-slow`, `ux-ratchet` are separate), `ux-gate` runs `npm ci` and needs a prebuilt site, which is
network and a build, and no CI lane runs it. The local gate and the forge cannot be equal for it.

The one change that would make it fit: **cut the gate's browser tests from 25 to about 12** (move
the keyboard anchor cards and half of the L2 pages to the slow lane, keep one page per kind), or
else run the gate only on a CI runner with a fixed core count. Then the tool's share is about
7 s and the host's inflation of 8.9 times still ends under 120 s.

## § 4 Findings hygiene

- No S1 or S2 finding was closed this round. Closed: UXF-010, UXF-011 and UXF-022 (all S3).
  UXF-023 (S2) was reopened; UXF-025 (S2) is new and open. So the "S1 and S2 closed with a
  regression asset red before, green after" check has nothing to hold; for the S3 closes the red
  and green logs exist (`c1-`, `c3-`, `c4-`, `c5-` red and green logs in the scratch directory).
- The assets of the closed findings run in the slow lane only (T39 shape). A regression of UXF-022
  would not fail a merge.
- UXF-013 "owned by the theme" has component paths. I verified them in the primary checkout's
  `website/node_modules` for Docusaurus 3.10.2. The docs-system claim (`dist/theme`) was not
  verifiable: the installed 0.6.0 has no `dist/theme`.
- Nothing was closed on a scratch file: the closes cite repository assets. The CSS fix of UXF-022
  pads short pages with `min-height`; it had no yes/no decision in `dm`, though it changes every
  page's height.

## § 5 New tool defects (numbered after T33)

| Id | Defect | Where |
| --- | --- | --- |
| T34 | The layout-shift value is filed under the wrong theme label: measured under light, stored on the first sorted config (`dark`). The baseline holds the wrong label | `core/reporter.ts:197`, the sort in `mergeRecords`, `specs/l2-layout.spec.ts`, `adapter.ts:52` |
| T35 | A decision lists keys, not values, and CI never reads `decision.md`; a later worse value of a listed key passes | `rs:665` `decision_of`, `rs:711` `covers`, `rs:574` `decided_anew` |
| T36 | A worsening within tolerance (5 % words, 0.01 shift) is silent: 4 values rose against R-03 and no report line names them | `rs:398` to `512` `compare_with` |
| T37 | No site identity and no run manifest; `runs` is optional in `round.json`, so a round with no runs is accepted | `core/engine.ts`, `core/round.ts`, `rs:538` `unattributable` |
| T38 | Evidence lives in `.out/`, which is discarded; the note-number check covers `notes.md` only, not `report.md` or finding records | `core/round.ts` `notesNumbersMissing` |
| T39 | The nested exit-code tests moved to the slow lane; the gate has no end-to-end case with two errors in one test, one known and one not | `specs/nested-findings.spec.ts`, `core/findings.ts` `unmatchedErrors` |
| T40 | `ux-slow` stops at the first failing project, so later lanes are not measured; its comment contradicts the `slot` prefix on the command | `Taskfile.yml` `ux-slow` |
| T41 | The ratchet ignores run status, `unexpected` counts and what was rerun; a run that failed to finish judges as a pass on the values it did record | `rs:1440` to `1545` `run`, `core/round.ts` |
| T42 | Minor: the round date is UTC, so a round near midnight CET is dated a day off; `GATE_PAGE_ELEMENTS = 1200` is an arbitrary threshold with no stated source | `core/round.ts`, `adapter.ts` |

## Caveats

- I ran nothing. The load ratios, the 25 to 12 cut and "fit below load 70" are read from logs and
  inferred; none was proved by a run.
- The line numbers marked "about" were read once; the function names are the stable reference.
- The cause of the 5 reading-load values and of the 2 keyboard values is read from the diff of the
  pages; the page text was not rendered.
- The host's load was 115 to 170 during the whole round, so every timing in this round is a
  loaded timing. The tool's own share is the least certain number.
- The docs-system theme path of UXF-013 is unverified; Docusaurus's is verified.

## Files

- `rounds/R-04/report.md`, `round.json`, `decision.md`, `notes.md`, `triage.md`, `baseline.json`,
  `trend.md`
- `crates/edge/ess-xtask/src/ux_ratchet.rs`
- `core/engine.ts`, `core/findings.ts`, `core/reporter.ts`, `core/round.ts`
- `specs/l1-adapter.spec.ts`, `specs/l2-layout.spec.ts`, `specs/nested-findings.spec.ts`,
  `adapter.ts`, `playwright.config.ts`
- `Taskfile.yml`, `website/src/css/custom.css`, `website/docs/index.md`
- `rounds/R-03/review-report.md` (the form)
