# R-01 review: report reviewer

Reviewed at commit `df384afaa7` (branch `ux/docs-site-loop-r01`). Inputs: `report.md`, `round.json`,
`shots/`, `baseline.json`, `trend.md`, `findings/UXF-001..014`, the engine source (`adapter.ts`,
`core/*.ts`, `specs/*.ts`, `playwright.config.ts`, `scenarios/*/scenario.json`), the ratchet
`crates/edge/ess-xtask/src/ux_ratchet.rs`, and `website/docusaurus.config.ts`. Nothing was run.

## Verdict

**Not fit to send as is.**

Default action: send R-01 with the corrections in § 1 and the caveats in § 4. Treat UXF-004 to
UXF-013 as observations. Hold the baseline and the gate: do not register the ratchet and do not
wire the specs into any task until T1, T2, T3, T6 and T11 are fixed. Then measure again at full
coverage and replace R-01's baseline with that run.

One decision for the coordinator: does the sidebar have to be wholly in view at the end of a page
(T4)? Default: no. Check it at `top` and `middle` and report `end` as measured only.

## § 1 Are the numbers right and meaningful?

### 1.1 Recomputed from `round.json`

`.out/` was deleted, so the raw data is gone and nothing here was recomputed from it. The
recomputation uses `round.json`, the summary that the code under review wrote.

| number in `report.md` | recomputed | difference |
| --- | --- | --- |
| gate 2: 821 failures (548 + 136 + 135 + 2) | 821 | none |
| gate 5: 2,733 nodes | per-site sum 676 + 696 + 670 + 691 = 2,733; per-rule sum 6 + 2,702 + 22 + 2 + 1 = 2,733 | none |
| truncation: 6 texts | 0 + 0 + 3 + 3 | none |
| gates 3 and 4, keyboard: 18/18, 4/4, misses 0 | 18 pointer entries, 4 keyboard entries, every miss 0 | none |
| `trend.md`: actions 132, steers 23 | sums over the 18 pointer scenarios: 132, 23 | none |
| per-task metrics | consistent with `core/metrics.ts`: `time_to_act.max` ≤ 1 + steers everywhere; p95 = 3 exactly where steers > 0; steers 0 gives p95 ≤ 2; UXF-010 and UXF-011 action breakdowns add up (1 + 1 + 24 = 26, 5 + 1 = 6) | none found; the states needed to recompute these are gone |
| `baseline.json`: 246 values in 27 scenarios | the same 246 values as `round.json`; tolerance classes count 135, steps 110, layout_shift 1, matching `tolerance_class()` (rs:197) | none |
| gate rows "0 of 139", "138 of 139", "104 of 139" | `/ess/docs/reference/ess-ui` has a record with 0 configurations (T1), so 138 pages were measured: 0 of 138, **138 of 138 (100 %)**, 104 of 138 | denominator off by 1 |
| "Configurations measured: 548 of 556", printed for the first run | the first run had 184 L2 tests, so at most 368 configurations; 548/556 is the total over three merged runs | mislabelled |
| "L2 over all runs: 3662 s" | 3,662.1 s is the sum of whole-run wall times; L2's own wall time is 1,641.3 + 1,688.5 + 260.5 = **3,590.3 s** | off by 72 s |
| "L3 0 s" in the first run | `round.json` has no `l3` project for that run: L3 did not run there | wrong meaning |
| "workers 2" | a literal in `round.ts:116`; the `gate` script in `package.json:7` passes `--workers=4`; `round.json` records no worker count | unverifiable, possibly wrong |
| per-viewport and per-theme coverage, never printed | search is missing in every configuration, so `layout.always_visible_pages` equals the number of configurations measured: desktop-1280 light 138, **dark 135**, mobile-390 light 138, dark 137, of 139 each | not reported |
| L2 test runs | 184 + 76 + 23 = **283 runs of 278 tests**: 5 tests ran twice (T10) | not reported |
| UXF-002: "136 failures, 134 configurations, all in the `end` state" | one `end` failure per configuration at most, so this needs 2 duplicated configurations or a miscount | inconsistent by 2 |
| measured-only table: "13 pages measured" | 13 of 139 (9 %) | denominator missing |
| `layout_shift.max` 0.018 | 0.0183 in `round.json` and in the baseline | display rounding only |
| L1 7.1 s, L3 105.8 s, run 118.4 s, 6 runs in 31.8 s; the measured-only table | not in `round.json` (T12) | unverifiable |

### 1.2 Does each number mean what its name says?

- **Gate 1, no overflow (0).** The probe is sound for what it checks. Blind spot: text clipped by an
  ancestor's `overflow: hidden` is neither a spill (`probes.ts:72` skips it because the parent
  clips) nor a truncation (`:80` looks only at the element's own clipping). The probe also runs
  only at the top of the page.
- **Gate 2, always visible (138 pages).** This counts no layout failure of its own. Every
  configuration fails on `search: missing`, so the number is coverage (T3). The 271 sidebar
  failures are one stock-theme behaviour in the `end` state (T4). The 2 `missing` are the adapter's
  mistake (T5).
- **Gate 3, completion (pass).** With these cards it cannot fail (T9).
- **Gate 4, screen-only user (18/18).** It checks only that each route is reachable. It ignores
  cost: 15 of 22 runs exceed their card's action bar (T6). Its truth is weaker than the cards'
  wording (T7), and it finds targets from the DOM, not from the screen (T8).
- **Gate 5, axe (2,733 nodes).** Real violations. 99 % of the nodes are one CSS rule, and the count
  scales with content: every new prose link adds about 4 nodes (T2). The dark theme runs the colour
  rules only (`contract.ts:63`), so the dark and light counts cover different rules. Dark is also
  missing 3 desktop configurations, so "dark 102 pages against light 104" is partly coverage, not
  theme.
- **Keyboard (4/4).** The runs take 21 to 26 actions where the pointer takes 2 to 4. The report
  lists these runs against the pointer bar and files nothing on them (§ 2).
- **`site/load` `layout_shift.max`.** The summed shift during load, maximum over pages. The skill's
  key is `layout_shift.per_minute`; the rename is fine, but say so.
- **Ratchet keys `site/*`.** These are counts of pages and nodes. They rise when content is added and
  fall when coverage drops (T2). They do not measure the UI.

## § 2 Findings the report misses

There are no recordings, so these come from `round.json` and the source.

1. **The hard gates fail on every measured page today.** Gate 2 fails 138/138 measured pages and
   gate 5 fails 104. Wired into every merge, the gate blocks every merge, and a new failure on a page
   that is already red cannot be seen. The report presents the gate results without saying this.
2. **Keyboard cost is about 10× the pointer cost.** `first-conformance-run` takes 23 actions by
   keyboard and 2 by pointer at desktop-1280; `install-linux-x86_64` takes 21 and 2. The Tab path
   runs through the navbar and the sidebar first. No finding was filed (candidate S3, class
   `input`).
3. **15 of 22 task runs are over their card's bar** (11 of 18 pointer runs), yet the gate row says
   "pass". UXF-010 to UXF-013 cover 7 of these tasks; nothing states the count.
4. **The report never names the unmeasured page**, `/ess/docs/reference/ess-ui`. It appears only as
   "548 of 556"; UXF-001 and UXF-014 name it.
5. **Measured-only viewports cover 13 of 139 pages.** UXF-002 cites "12 of the 13" as evidence.
6. **Dark-theme numbers are not comparable with light:** different axe rules, and 3 to 4 fewer
   configurations.

## § 3 Tool defects

- **T1: an unmeasured page counts as clean.** The record is set before the first configuration is
  measured (`specs/l2-layout.spec.ts:46`) and attached after a timeout (`:19-21`). `pages` counts
  records (`core/reporter.ts:219`), and every gate count goes through `pagesWith` (`:92`), which is
  0 for an empty record. Input: `/ess/docs/reference/ess-ui` passed 60 s inside its first
  `checkConfig` at both viewports, so its record has `configs: []`. That page is inside "0 of 139"
  and "104 of 139". Playwright's exit status fails it; the report, `round.json` and the ratchet
  count it as passing.
- **T2: coverage and content move the ratchet.** The `site/*` keys count configurations
  (`reporter.ts:77-84`). The ratchet ignores `timing.coverage` (rs:82, `Round` reads only
  `scenarios`). A drop counts as an improvement, and `update()` takes it with no decision
  (rs:255, 335-347).
  - Input 1, R-01 itself: R-02 at full coverage reads worse by 1 to 4 on `always_visible_pages` in
    all four site scenarios, with no change to the site.
  - Input 2: a run that times out on 20 pages "improves" and is taken silently.
  - Input 3: a docs pull request that adds one page or one prose link fails a count tolerance of 0.
- **T3: search is a product gap encoded as a per-configuration layout gate.** `adapter.ts:69`
  watches search. `docusaurus.config.ts:25-34` configures no search plugin, no search theme and no
  `algolia`. The docs template at its pinned commit `5fd348a9` adds only its own plugin
  (`dist/product-site.js:364`). So "missing" is true of the site and UXF-001 is right. Counting it
  548 times makes gate 2 permanently red and hides every other gate-2 failure.
  Also `/search/i` (`adapter.ts:69`) matches any button whose name contains "search" ("Research"),
  so a false "present" is possible later. Fix: one site-level check "search exists", outside gate 2.
- **T4: "sidebar out of viewport at end" measures the footer.** `end` is
  `scrollHeight − innerHeight` (`core/contract.ts:38`), which puts the footer in view.
  `probeVisible` requires the whole `nav` inside the viewport (`probes.ts:133`). The stock theme's
  sticky sidebar column ends where the footer starts. So every doc or release page longer than a
  screen fails, once each, in `end`: 135 pages, 271 failures. The geometry is measured correctly.
  The gate is the adapter's choice, and it fires while the reader is at the footer. UXF-002 and
  UXF-003 at S2 follow from that choice.
- **T5: the archive is misclassified.** `pageKind` maps every `/ess/releases/*` page to `blog`
  (`adapter.ts:59`), and `:74` then expects the blog sidebar. `/ess/releases/archive` renders none
  by design, so "blog sidebar: missing ×2" is the adapter's. It needs an exception like
  `DOCS_WITHOUT_SIDEBAR` (`:53`).
- **T6: gate 4 cannot fail on cost.** Nothing compares actions with `expect.max_actions`: the spec
  asserts `reason`, `completed` and gate 3 only (`specs/l3-screen-only.spec.ts:32-34`), and
  `round.ts:140` only prints the bar. Input: `ess-31-release` takes 26 and 35 actions against a bar
  of 6, and the gate passes. The keyboard runs print the pointer bar (23 against 3).
- **T7: the truth is weaker than the card.** `Truth` (`core/bot.ts:19-23`) has no `anchor`, so
  `truth.anchor` in `diagnostic-type-009`, `install-linux-x86_64` and `page-a-view` is read by
  nothing. Neither "its release cell beside it" (`ess-31-release`) nor "the 009 row of the classes
  table" is checked. Input: drop the release column from the format table, and `ess-31-release`
  still passes. For the four h1 targets, completion is the URL alone.
- **T8: the bot uses what is not on screen.** `scrollsTo` and `scroll` (`bot.ts:151-155, 196-202`)
  take the direction and the exact gesture count from the DOM box of an element out of view.
  `steer` (`:159-166`) chooses between scrolling and opening a container because it knows a hidden
  copy exists. Skill § 7 says "may use only what is visible". Effect: gate 4 fails on reachability
  only, never on findability. Scroll counts are a lower bound (24 gestures straight to the `ess/31`
  row).
- **T9: gate 3 cannot fail on this card set.** `completion_miss` is counted only after a link named
  exactly like the target (`bot.ts:315-323`). `false_complete` is counted only when the target is
  usable on an intermediate page (`:256-259`). Of the 9 cards, only `page-a-view` ("A view can be
  paged" has an exact table-of-contents entry) can produce a `completion_miss`. A cell target or an
  h1 target in view cannot.
- **T10: re-runs duplicate configurations.** `mergeByPath` appends configurations without keying on
  viewport and theme (`reporter.ts:157`). There were 283 runs of 278 tests, so a test cut after its
  light pass and run again counts light twice. The 136/134 in UXF-002 points at this. The step that
  merged the runs is not in the tree.
- **T11: the round cannot be reproduced from the tree.**
  - No code writes `timing.continuations`, `l3Timing.continuation` or the merged coverage 548/556;
    only `round.ts` reads them, and `round.spec.ts:57` reads one `.out/round.json`.
  - The report's commit comes from `git rev-parse HEAD` (`round.spec.ts:120`) and is
    `6b580819542f`, which holds no `website/ux` at all.
  - L3 came from a later run "after a fix to the screen-only user". `bot.ts` and `probes.ts` both
    carry mtime 14:16, and `probeVisible` is also gate 2's probe, so the L2 numbers may come from an
    earlier `probes.ts` than the committed one.
  - Fix: write an engine digest into `round.json`, and put the merge step in the round command.
- **T12: `round.json` does not hold what the report prints.** `round.spec.ts:125` writes `measured`
  without `raw.l3Timing` and without `raw.measured`; skill § 3.2 says it holds "the same numbers".
  The report's run-time lines also mislabel:
  - "workers 2" is a literal (`round.ts:116`);
  - "L2 over all runs" sums whole-run wall times (`:120`);
  - the merged coverage is printed under the first run (`:116`);
  - a project that did not run prints "0 s" (`:93`).
- **T13: measured-only coverage is hidden, and its definition differs.**
  - A cut test attaches nothing (`specs/measured.spec.ts:28`, at the end only), and the run has a
    240 s global timeout with `UX_ROUND` (`playwright.config.ts:23`).
  - The table prints "13" with no denominator (`round.ts:163-174`).
  - It drops `missing` (`round.ts:170`) while gate 2 counts it (`reporter.ts:80, 108`).
- **T14: the ratchet is not built, and one rule differs from § 6.**
  - `crates/edge/ess-xtask/src/main.rs:3-20` declares no `mod ux_ratchet`, so the file is never
    compiled, its 18 tests never run, and `cargo xtask ux-ratchet` does not exist. Yet
    `baseline.json` `about` says that command wrote it.
  - An improvement within the 5 % steps tolerance is neither a breach nor taken by `--update`
    (rs:255, 333-347). Example: `reading_load.max` 256 → 245 is never taken, and a later 268
    passes. § 6 says "an improved value becomes the new baseline in the same merge".
  - `main_baseline()` returns `None` silently when `origin/main` cannot be read (rs:570-578), which
    skips rule 4 on a shallow CI checkout.
  - `reading_load` is a word count classed as steps (rs:197). § 6 assigns it no class; decide that
    explicitly.
- **T15: regression assets cannot show red → green per finding.**
  - UXF-001 to UXF-009 each name a whole L2 test, and every L2 test fails on `search: missing`.
    After a fix of UXF-004 alone, its asset is still red, so § 5 ("fails before the fix and passes
    after") cannot be met until T3 is fixed.
  - `check_finding` checks only that the file exists (rs:481-488), not the test title.

## § 4 Caveats to send with the round

- **The raw data is gone.** `.out/` (`raw.json`, the 3 L2 runs, the separate L3 runs, the ten
  slowest tests, the measured-only run) was deleted with the build cache, and the round has no
  recordings, no questionnaire and no last round. Unverifiable as a result:
  - every per-page figure in UXF-002 to UXF-009;
  - the per-task states behind the L3 metrics;
  - the L1 and L3 run times;
  - the measured-only table;
  - whether configurations were duplicated (T10);
  - which `probes.ts` produced the L2 numbers (T11).
- No person took part. L4, L5 and L6 did not run; L7 is this review and the heuristic review.
- `/ess/docs/reference/ess-ui` is unmeasured, and dark is missing 3 desktop and 1 mobile
  configurations. The baseline's `site/*` values therefore hold partial coverage and will read
  worse at full coverage (T2).
- Gate 2's result comes from the adapter's choices (search, sidebar in `end`, archive), not from a
  layout regression (T3 to T5). UXF-001 (no search) stands as a product finding.
- Gate 4 passing means "every route is reachable", not "findable within the card's bar" (T6 to T8).
- The site was measured under a host load average of 90 to 176 (UXF-014), so the timings are high.

## § 5 The gate's cost, and the cut for 120 s

These are the organization's bounds that the loop must meet: no test over 60 s, a test step within
120 s, a pool job within 10 minutes unless it names its time (at most 30 minutes).

### Where the time goes, by design

- **L2:** 139 pages × 2 gating viewports = 278 tests. Each test has:
  - a new context with every request routed through the hermetic handler;
  - `goto` until `load`, plus a hydration wait of up to 10 s;
  - for each of 2 themes, `showTheme` (up to 3 s; if that fails, a reload, `networkidle` and up to
    5 s more);
  - a full DOM walk (`getComputedStyle` and a box for every element);
  - 3 scroll states × 4 to 5 watched elements;
  - an axe run.

  That is 556 axe runs per gate, 278 full WCAG and 278 colour-only. Both kinds include
  `color-contrast`, the costliest rule, so the "colour" pass is not much cheaper.
- **L3:** 22 runs. `find()` waits up to 2 s for `attached` on every `click` iteration when the
  control is not in the DOM (the closed mobile menu); see `bot.ts:267`. `settle()` waits for
  `networkidle` (at least 500 ms quiet, at most 3 s) after each action.
- **Measured:** 283 L2 runs took 2,639.7 + 2,473.8 + 409.2 = 5,522.7 s of test time, a mean of
  19.5 s per test (14.3 s in the first run). L3 took 105.8 s wall, plus 31.8 s for the 6 runs it
  left; L3 alone is over 120 s.
- **The full page set cannot meet 120 s by tuning.** Even at 5 s per test and 4 workers it needs
  278 × 5 / 4 ≈ 350 s.
- **The full sweep also breaks the pool and per-test bounds.** 3,662 s across three runs is more
  than the 30-minute maximum for one job, and `ess-ui` alone is over 60 s per test.

### The smallest cut

- Per merge, take the L2 page list from a committed sample instead of the sitemap
  (`l2-layout.spec.ts:12`). The sample is 7 template pages, one per layout: `/ess/`, `/ess/docs`,
  `/ess/docs/start/install`, `/ess/docs/visualise`, `/ess/releases`, one release post, and
  `/ess/releases/archive`.
- Run L3 for the two anchor cards only (`l3-screen-only.spec.ts:12`): 8 runs.
- Run the full sitemap and every card only with `UX_ROUND` set, or in a `slow` lane when `website/`
  changed. That lane still needs `ess-ui` under 60 s (for example one test per viewport and theme)
  and splitting into pool jobs.
- Estimate at R-01's loaded rates with 4 workers:

  | part | arithmetic | wall time |
  | --- | --- | --- |
  | L2, 14 tests | 14 × 14.3 s / 4 | ≈ 50 s |
  | L3, 8 runs | about 12.5 s per run / 4 | ≈ 25 s |
  | L1 and server start | | ≈ 10 s |
  | total | | ≈ 85 s |

- Time the sample before adopting it: the ten-slowest list was deleted.
- The cut fixes the time only. While search is missing (T3), the sampled gate is still red on every
  page.
