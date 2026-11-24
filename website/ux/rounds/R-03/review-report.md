# R-03 review: report reviewer

W = the worktree root. Paths are under `W/website/ux` unless they start with `crates/` or are
`Taskfile.yml`. `rs:N` is `crates/edge/ess-xtask/src/ux_ratchet.rs:N`. `rj` is
`rounds/R-03/round.json`. `rp` is `rounds/R-03/report.md`. `r03/...` is the round's scratch
directory. Nothing was built, run or written in the worktree. "Inferred" marks a claim read from
code or from two numbers and not proved by running. Helper files (jq, a digest script, extracted
trees) are under `r03/review-r/`; they are not in the repository.

## Verdict

**Baseline: fit with named exceptions.** The 3,556 values in 578 scenarios of `baseline.json` match
`rj` (jq recount). A later round can be held to it. Four exceptions go in its record before R-04:

- X1 (T25, T26): the engine behind `rj` is not attributable to a committed tree.
- X2 (T27): a rerun replaces a worse value and the replaced value is not kept.
- X3 (T28): a failure in the test of an open finding is held whatever its cause.
- X4 (T29): a ratchet decision id ties to every worse value of one update, not to the named ones.

**Gate in `task check`: no.** Conditions:

1. Wall time under 120 s at load 90 (answer (a): move the two heavy table pages and the three
   nested runs out of the gate lane).
2. `ux-gate` and `ux-ratchet` run as one step, with the ratchet reading the gate's `round.json`.
3. Offline: a prebuilt site and installed browsers (`ux-gate` runs `npm ci`, `Taskfile.yml:554`).
4. Known failures matched by test and by message (T28).
5. `layout_shift` out of the gate lane or de-flaked (T27).
6. `ux-slow` split into jobs that each name `slot --timeout` (T31).
7. `crates/edge/ess-xtask/tests/ci_lanes.rs` accepts the new job (the gate is not in any CI lane).

## § 1 Numbers recomputed

Method: jq over `rj` and `baseline.json`, awk over the logs in `r03/`, the engine digest rebuilt
with `r03/review-r/digest.sh` from `git archive`.

Matched (no mismatch):

- 220 task rows, 24 page-sum rows, the runs table (18 runs), the slowest tests, the slowest-pages
  phases, the measured-only rows, the coverage rows, gates 1 to 5.
- 3,556 values in 578 scenarios; 819 improved, 4 worse with `R-03-axe` (`decision.md:55`,
  `baseline.json:2240,2241,2256,2257`).
- Trend rows R-01 to R-03, for the pointer-only sums: actions 131 (R-02, R-03) and 132 (R-01);
  steers 89 (R-02, R-03) and 23 (R-01).
- Engine digest: tree of commit `7db2574098` gives `bc3e06f44faa130b`, equal to `rj` `engine.digest`.
- UXF-021 repeat: 10 of 10 passes (`r03/c3-uxf021-repeat.log`).

Mismatches:

| # | where | text | recomputed |
| --- | --- | --- | --- |
| M1 | `rp:418-419` | the rerun's 0.00004 "is in `round.json`" | `rj` holds 0 (rounded to 4 places, `core/metrics.ts:18-21`); 0.0000423 is only in the scratch `raw.json` of `rerun-shift` |
| M2 | `rp:405` | "load of 40 to 165" | logged 1-minute load 35.07 to 162.10 |
| M3 | `rp:408` | "load 61 to 92" | mixes 1-minute values 69.39 and 61.24 with the 15-minute value 91.73 |
| M4 | `rp:411` | "three runs at load 105" | no load line is kept for gate-1 to gate-3; cannot verify |
| M5 | `trend.md:5` | column "steers (sum)" | the sum is pointer-only (89 for R-02, R-03); all runs give 93 |
| M6 | `rp:410` | L1 time is "most of it the three nested runs" | 42 to 50 % in gate-8 and gate-9 |
| M7 | `findings/UXF-021.md:50` | "2.7 s per viewport" | gate runs show 11.1 to 27 s for the visualise page |
| M8 | `rp:413` | "load 90 to 115" for the UXF-021 repeat | no load line kept; cannot verify |

Six mismatch, two cannot be verified. Largest by size: M7 (2.7 s against 11.1 to 27 s, in a finding,
not in `rp` or `trend.md`). Largest in `rp` or `trend.md`: M2 (5 load units at the low end).
M1 is the one that matters for the ratchet: the text says a value is in the round record and it is not.

Digest note (inferred from the digests): the committed `website/ux` tree at `0d51e1e891` and at
`be6f7a2c8c` has digest `93f932aacd3f4a09`. The measuring engine was `bc3e06f44faa130b`. Only the
rerun note (`rp:416`) says the tree had moved on; nothing states it in `rj`.

## § 2 R-02 defects T16 to T24 and E1 to E3

E1, E2, E3 are T16, T17, T18 of the R-02 review (`rounds/R-02/review-report.md:14-18`). They are
rated once each here and counted with their T number: 9 distinct defects, 12 labels.

| item | rating | evidence (code) | test that goes red |
| --- | --- | --- | --- |
| T16 / E1 axe loses frame nodes | repaired | `core/contract.ts:79-107` `runAxe` runs the top document in legacy mode and `axe.run` per frame; `:108-119` `axeInFrame`; a frame error fails the test; `r03/probe-axe.log:8` shows `color-contrast` x8 and `nested-interactive` x1 inside the frame that legacy mode alone misses (`:4`) | `specs/l1-probes.spec.ts:54` (frame fixture); `rj` visualise light 3 to 10 nodes, taken under `R-03-axe` |
| T17 / E2 content rise without decision | repaired | `rs:61-75` `Tolerance::Content` 0.0; `rs:298-310` class; `rs:518-597` `update` refuses a worse value without a decision; `rs:441-443` `decided_anew` | `rs:1355`, `rs:1390`, `rs:1685` |
| T18 / E3 nothing runs the gate | partly | `Taskfile.yml:547-574` adds `ux-gate`, `ux-slow`, `ux-ratchet`; `Taskfile.yml:624-643` `check` calls none of them | no test; `ci_lanes.rs` is not extended |
| T19 gate timed at 6 workers | repaired | `adapter.ts:22` `WORKERS = 4`; `package.json:7` `playwright test --workers=4`; `Taskfile.yml` `UX_WORKERS: '4'` and `--workers={{.UX_WORKERS}}` in both tasks; `rj` runs carry `workers` 4 | `specs/l1-adapter.spec.ts:54-59` |
| T20 gate exit code cannot tell a known failure | repaired | `core/findings.ts:83-87` `holdKnownFailures` calls `testInfo.fail(true, ...)` for an open finding's asset; Playwright then exits 0 on that failure, 1 on a new failure and 1 when the asset passes | `specs/l1-findings.spec.ts:71`, `:75`, `:79` (three nested runs, exits 0, 1, 1) |
| T21 baseline written before second comparison | repaired | `rs:1076-1135` `run`: in-memory `update`, second `compare`, file written last | `rs:1448` |
| T22 ratchet ignores `engine.dirty` and `runs_agree` | partly | `rs:378` dirty, `rs:383` runs disagree. Not repaired: `core/round.ts:52` takes `engine` from the first raw only, so a dirty or unknown later run is dropped; `core/round.ts:118` `runs_agree` compares digests only; `core/engine.ts:48` returns `{commit:'unknown', dirty:true}` outside Git, and `rj` shows `dirty:false` with the reruns in it | `rs:1410`, `rs:1428` cover the ratchet side; none covers `mergeRaws` |
| T23 report column and `about` stale | repaired | `about` at `rs:600` is written from the round; the report reads `rj` | `rs:1706` (names the round each lane was taken from) |
| T24 `reading_load` unit | repaired | `rs:298-310` and `rs:301` class `words`; baseline entries carry `words` | `rs:1478`, `rs:1779`, `rs:1916` |

Counts: repaired 9 labels (T16, T17, T19, T20, T21, T23, T24, E1, E2), partly 3 labels (T18, T22,
E3), not repaired 0, cannot tell 0. By distinct defect: 7 repaired, 2 partly.

## § 3 Admissions

### (a) The gate cut at 120 s under load

Facts from `r03/gates*.out` and `gate-*.log`: gate-4 was cut at 119.6 s; gate-5 and gate-6 took
100.7 and 93.6 s at 1-minute load 61 to 92. One run without the cut took 220.7 s (`rp:411`).

Share of the tool's own time (computed from the per-project test seconds in the logs):

- The L1 sum rose from 11.5 to 14 test-seconds (R-02 engine) to 38 to 62 (R-03 engine). The rise is
  the three nested Playwright runs of `specs/l1-findings.spec.ts` (each `spawnSync` allows 50 s,
  line 66) and the frame fixture test (`specs/l1-probes.spec.ts:54`).
- The visualise page is two L2 tests (desktop and mobile) that now run axe in the frame: about
  10 test-seconds more.
- Together about 9 to 15 s of wall time is the tool's. The rest swings with load: the same tests that
  took 7 s took 18 to 50 s.
- `spec-versions` and `diagnostics` hold 100 to 133 of the 321 to 400 test-seconds of the gate.

What brings it under 120 s at load 90 (inferred, not run): the two table pages move to the slow
lane and the three nested runs move to the Rust ratchet tests or the slow lane. That leaves about
57 to 90 s of wall time at 4 workers. Raising the cut, adding shards or more workers is not a fix
(`AGENTS.md`, "Tests stay fast": "Raising a timeout, adding shards or waiting longer is not a fix").
UXF-014, the finding for the gate over budget, is closed and was not reopened (T30).

### (b) Reruns from an engine copy outside Git, commit "unknown"

- Attributable by digest: `rerun-essui` and `rerun-shift` carry digest `bc3e06f44faa130b` (`rp:68-69`),
  the digest of commit `7db2574098`, and `rj` `engine.commit` is `7db25740980e` with `runs_agree`
  true. I rebuilt that digest from `git archive`, so the round is attributable to that commit.
- Not attributable to the merged tree: the committed tree has digest `93f932aacd3f4a09` (§ 1).
- The ratchet accepts it. `rs:378` reads the top-level `engine.dirty`, which `core/round.ts:52`
  copied from the first run (`false`). The two reruns reported `dirty:true` (`core/engine.ts:48`) and
  that is lost. Verdict: attributable by digest, accepted by accident, not by a check (T25, T26).

### (c) Layout shift 0.0253 against 0.00004

- `/ess/releases/outcome-shapes@mobile-390/dark` measured 0.0253 in `slow-l2-4` (baseline 0,
  tolerance 0.01 absolute, so worse) and 0.0000423 in `rerun-shift`. `rj` keeps 0.
- `core/reporter.ts:118-133` `mergeRecords`: a later record replaces an earlier one per
  configuration. Nothing records the replaced value; `rj` lists the run by name only.
- Yes, a hole (T27). A load-dependent lower-is-better value can be rerun until it is low, and the
  ratchet cannot see it. The shift is read after hydration under the first theme only
  (`core/reporter.ts:192`, `specs/l2-layout.spec.ts`), so it is a timing value. Inferred: the 0.0253
  is a measuring artefact under load 90 to 160, but no run proves it.

### (d) The four visualise values that got worse under `R-03-axe`

- `decision.md:34-51` names all four scenario and key pairs with their old and new numbers, and the
  node counts by rule (8 + 1 + 1 = 10 at desktop, 8 + 1 + 1 + 1 = 11 at mobile), and the command.
- The ratchet ties them: `baseline.json:2240,2241,2256,2257` each carry `"decision": "R-03-axe"`.
  `rs:518-597` sets the decision on every worse value of the update, so the id would also have
  covered a fifth worse value. The ratchet does not read `decision.md` (T29).
- `rs:447-502` `compare_with_main` then requires an id that `origin/main` does not hold.

### (e) Several assets per finding and an expected failure

- `core/findings.ts:83-87` matches on the test title alone and marks the test an expected failure.
  Any failure in that test is held, whatever its cause. Example: `L2 /ess/docs/visualise desktop-1280`
  is the asset of UXF-005, UXF-006, UXF-007 and others, so a new axe rule or a thrown error on that
  page passes the gate (inferred from `findings.ts`, not run).
- The ratchet catches a new measured value (a new rule raises `axe.serious_critical_rules`, `rs:371-439`),
  not an error thrown in the test body or a changed test that no longer reaches its assertions.
- `rs:844-856`: `asset_path` and `asset_title` read the first `::` and do not split on `; `, and
  `rs:897` `title_matches` lets a `${...}` title match any `L2 ...` text, so the second and later
  assets of UXF-006, 010 to 013 and 022 are not checked against their spec file (T28).
- A closed finding's asset (UXF-023: `L2 /ess/ desktop-1280`) is not an expected failure of its own;
  it passes only while UXF-005 holds the same title.

### (f) UXF-023 closed as "not reproduced"

- `rj` scenarios `/ess/@desktop-1280/light` and `/ess/@mobile-390/light` hold `axe.serious_critical_nodes`
  2 and `axe.serious_critical_rules` 1. That is the same count R-02 had, so `rj` alone does not
  show that the rule changed.
- The rule id is only in scratch: `.out/gate-9/raw.json` lists one rule on `/ess/` mobile-390 light,
  `label-content-name-mismatch`, 2 nodes, and no `color-contrast` in any of the 36 gate-9 axe
  entries (grep count 0). `findings/UXF-023.md:12` and `:44` cite that run.
- Verdict: supported by the kept raw record, not by `round.json` and not by anything in the
  repository (`raw.json` is not committed). Partly.

## § 4 New tool defects

- **T25** `core/round.ts:52` takes `engine` from the first raw: a dirty or unknown engine of a later
  run disappears from `rj`; the ratchet's dirty check (`rs:378`) sees `false`. Test missing for `mergeRaws`.
- **T26** `core/round.ts:118` `runs_agree` compares run digests, never the digest against the
  committed tree. `rj` `engine.digest` `bc3e06f44faa130b` is not the tree's `93f932aacd3f4a09`.
- **T27** `core/reporter.ts:118-133` a rerun silently replaces a worse value; `layout_shift`
  0.0253 became 0 with no record of the first value.
- **T28** `core/findings.ts:83-87` holds a failure by test title only; `rs:844-856,897` do not check
  the second asset of a finding or a templated title.
- **T29** `rs:518-597` a decision id covers every worse value in the update; `decision.md` is not
  read.
- **T30** The gate exceeds 120 s at load (gate-4 cut at 119.6 s). UXF-014 is closed and not reopened.
  `specs/l1-findings.spec.ts:66` runs three nested Playwright runs inside the gate.
- **T31** `Taskfile.yml:556-567` `ux-slow` runs 8 Playwright projects in one task with `ignore_error`
  and no `slot --timeout`. Inferred: under `slot task ux-slow` the whole task is one pool job, ended
  at 10 minutes (the 8 projects ran 72 s to 153 s each in R-02, 8 x about 2 minutes is near that
  limit). A failed project does not fail the task, so only `ux-ratchet` can.
- **T32** `Taskfile.yml:624-643` `check` calls neither `ux-gate` nor `ux-ratchet` (T18 stays partly).
- **T33** The report's hand-written notes (`rp:401-425`) and `findings/UXF-021.md:50` hold numbers
  that `round.json` or the logs do not give (M1 to M8). The notes are not a function of `rj`.

## § 5 Organization bounds

- Test at most 60 s: `playwright.config.ts` `timeout: 60_000`; the heavy reference pages ran into
  it and were rerun (`rp` Reruns note). `specs/round.spec.ts` splits round and report stages; each
  test is under 60 s.
- Test step at most 120 s: gate `globalTimeout` 120_000. R-03 gate runs took 93.6 to 119.6 s, one
  cut and one uncut at 220.7 s. Over the bound at load 90 and above: defect (T30).
- Pool job at most 10 minutes unless named: `ux-slow` names none (T31). `ux-gate` is one job;
  its time at load is 94 to 120 s.
- Measuring and watching scripts run without `slot`: the loop's `serve.sh` starts the server outside
  `slot` (correct, a server). Builds and tests inside the Taskfile are not prefixed; the host rule
  asks `slot` on the whole `task` call.
- No Python: `git ls-files website/ux` lists 107 files, no `.py` and no `.sh`.

## Caveats

- Time numbers in (a) are computed from `r03/gates*.out` and `gate-*.log`; the 57 to 90 s estimate
  is inferred and was not run.
- Load values are 1-minute values from `r03` logs; gate-1 to gate-3 and the repeat have none.
- No test, server or browser was run for this review.

## Files

- Under review: `website/ux/rounds/R-03/report.md`, `round.json`, `triage.md`, `decision.md`,
  `website/ux/baseline.json`, `website/ux/trend.md`.
- Code read: `website/ux/core/{contract,findings,engine,round,reporter,metrics,probes}.ts`,
  `website/ux/specs/*.spec.ts`, `website/ux/adapter.ts`, `website/ux/playwright.config.ts`,
  `crates/edge/ess-xtask/src/ux_ratchet.rs`, `Taskfile.yml`.
- Helper files: `r03/review-r/` (outside the repository).
