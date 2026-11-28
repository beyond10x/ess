---
format: aep.planning-md/3
id: review-result:adversary-budget-pass-2
kind: review-result
status: active
title: Adversary pass 2 on the test-budget step 1
relations:
- reviews: story:the-test-step-meets-the-test-budget
revision: 1
---
unit: budget step 1, pass 2 (story:the-test-step-meets-the-test-budget), uncommitted tree ess-wr-budget on ff5c4713e
verdict: NEEDS-CHANGE
cases: executed 571→574, red 3 (571 is 574 minus my 3 cases; I did not run the suite separately without them)
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 22 files in the scratch directory scratch-adv2
needs-coordinator: no

- **Diff:** the same 6 files as before. I only added 3 cases at the end of `crates/edge/ess-xtask/tests/ci_lanes.rs` (+114 lines). The other files are the implementor's.
- **Cases (each run alone first, each failed for its stated reason):**
  1. Two slow-lane tests reached 57.1 s and 55.4 s on CI and have no `slow` override. AGENTS.md:140 says every test whose longest time was 50–60 s has one. Output: `…workbench_views…: 57113 ms on CI, wants 69 s, has None`.
  2. When a doc-test step is killed, `DOC_TEST_TIMES` drops the crate it was killed in, which is the slowest one. With a sample killed output it printed only `classname="ess_conformance"`.
  3. `test-feature-off` has a limit of 664 s, under 1.2× its longest step on CI: 546.2 s of tests plus 10.7 s of listing = 556.9 s, which needs 669 s. AGENTS.md:154 says the listing takes up to 7 s.
- **Suite:** `slot --timeout 30m task test-xtask` ran 574 tests: 571 passed, 3 failed (my three). The step took 24 s of its 120 s. Clippy and rustfmt are clean.
- **Replay of the 13 runs:**
  - `ci` profile, replayed in run order with the new 50–60 s rule: 0 of 9 runs red (pass 1 had 2 of 9).
  - Slow lane: 1 of 8 runs red. In run 45901071526, 11 tests that have an override ran 20–57 % over their earlier maximum, and that is the run where the two tests of finding 1 reached 55 s and 57 s.
- **Untested risk (nextest refusing an override for a missing binary):** this happens only in the fuzz workspace (`operator didn't match any binary IDs`).
  - No partial selection fails: `-p ess-yaml` lists and an archive of `ess-yaml` alone runs under `--profile ci` (both exit 0).
  - The binary names are checked against the whole workspace's package list, so `PACKAGES`, number-semantics, `test-packages` and the hotfix scope would not fail either.
- **Could not break:**
  - The ratchet's spelling check: inline tables, quoted or dotted keys, single-bracket headers, CRLF, literal strings, a filter widened by escapes or `|`, an override with only `platform`, and the fuzz config.
  - All 30 new overrides: each names a test seen on CI, matches exactly one test, and equals min(72, ⌈1.2×max⌉).
  - The JUnit report is removed in every nextest step. No step or upload reads a report another step deleted.
  - `test-feature-off-extract` runs through `test-feature-off`.
  - The 61 s kill works in all three steps. The fuzz `ci` profile loads.
  - The doc-test steps take at most 27 s of their 60 s on CI. The case with no dependency cache at all is not measured.
- **Fixes I would name:**
  - Finding 1: give the two tests `slow` overrides of 69 s and 67 s, or narrow AGENTS.md:140 to the `ci` profile.
  - Finding 2: in `DOC_TEST_TIMES`, when the output ends with a pending `name`, write that crate with the step's time.
  - Finding 3: raise the ceiling to 669 s and change "7 s" to "11.9 s" in AGENTS.md:154.

Everything I wrote is under the scratch directory scratch-adv2:
- the 5 case logs (`adversary_budget_*.log`, `*.rerun.log`)
- `suite.log`, `suite2.log`, `fmt.log`
- `fuzz-rootcfg.log`, `partial-yaml.log`, `archive-partial.log`
- `cases.rs.txt`
- `ci-overrides-now.txt`, `ci-seen-tests.txt`, `slow-max.txt`, `slow-overrides-now.txt`
- `replay2.sh`, `replay2-slow.sh`, `replay2-slow-norule.sh`, `replay2.txt`, `replay2-from2.txt`, `replay2-slow.txt`

The probe's `target/adv2` in the tree was removed by the same command that created it.

```findings
- file: .config/nextest.toml
  line: 888
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the slow profile kills at 60 s but gives no override to two slow-lane tests seen at 57.1 s and 55.4 s on CI, though AGENTS.md promises one to every test of 50 to 60 s and the slow lane gates the merge
- file: Taskfile.yml
  line: 22
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: when a doc-test step is killed at its limit, DOC_TEST_TIMES writes no entry for the crate it was killed in, so the ten slowest leave out the crate that ran over
- file: Taskfile.yml
  line: 489
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the test-feature-off limit of 664 s assumes 7 s of listing, but CI measured 10.7 s on the longest shard (556.9 s), so the limit is below the 1.2x rule (669 s)
```
