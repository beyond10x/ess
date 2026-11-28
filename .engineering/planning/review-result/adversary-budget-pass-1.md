---
format: aep.planning-md/3
id: review-result:adversary-budget-pass-1
kind: review-result
status: active
title: Adversary pass 1 on the test-budget step 1
relations:
- reviews: story:the-test-step-meets-the-test-budget
revision: 1
---
unit: budget step 1 story:the-test-step-meets-the-test-budget, uncommitted tree ess-wr-budget on ff5c4713e
verdict: NEEDS-CHANGE
cases: executed 566→570, red 4
origin: introduced 1 / pre-existing 4 / undecided 0
wrote-outside-worktree: 6 paths in the scratch directory scratch-adv (see below)
needs-coordinator: yes. Findings 1 and 2 count as pre-existing because they also fail on the base, but meeting them is this unit's own job (Do 1, design step 1). Don't move them out of the wave.

- **Diff:** `git diff --stat` lists 6 files, 1616+/11−. The only file I touched is `crates/edge/ess-xtask/tests/ci_lanes.rs`: I added 162 lines at the end, 4 cases and one helper. The other 5 files are the implementor's uncommitted work.
- **Cases:** I ran each case alone first, and each failed for the stated reason. Logs are `scratch-adv/case1.log` and `scratch-adv/adversary_budget_*.log`.
  - 1 and 2: `test-doc`, `test-feature-off-doc` and `fuzz-check` run `cargo test` with a 120 s step limit. A 61 s test in them passes, and they print no ten slowest tests.
  - 3: four valid TOML spellings that give every test 900 s all pass the ratchet.
  - 4: when `test-feature-off` wrote no report, it printed `a_test_of_the_earlier_step 99.000s` as "the ten slowest tests of this run".
- **Suite:** `slot --timeout 30m task test-xtask` ran 570 tests: 566 passed, 4 failed (the four new cases). The step took 24 s of its 120 s.
- **Not a case, contract drift (finding 5):** `AGENTS.md:130` says each step's build runs before the step. In the implementor's own logs, `test-doc` compiles 16 crates and `test-feature-off-doc` 6 crates inside the limit (`scratch/test-doc.log`, `scratch/test-feature-off-doc.log`). The pre-build `cargo build --lib` uses the dev profile and skips dev-dependencies. `test-feature-off` also counts archive extraction, about 345 s of its 825 s ceiling.
- **Same stale report, second path:** when `PACKAGES` includes ess-xtask, `test-packages` runs `task test-xtask` before its deferred print. That print then shows test-xtask's report, and test-packages' own ten slowest are never shown.
- **Kill risk without an override:** I replayed the 1.2x rule over the 13 runs in order (`scratch-adv/replay.txt`). In runs 5–13, 2 of 9 would have gone red, each from a test with no override that was new over 60 s (3 tests and 1 test). No test with an override went past its ceiling. So about 1 run in 5 goes red today. Add overrides for the 30 tests with no override whose CI maximum is 50–60 s, at 1.2x their maximum, at most 72 s (list in `scratch-adv/near.txt`).
- **Could not break:**
  - Every test-running command in `ci.yml` and `release.yml` runs through a task or is named in `DIRECT_TEST_JOBS`.
  - The `binary_id & test(=…)` filters each match one test.
  - A raise or a new override in the canonical spelling is caught.
  - `timeout` kills nextest's process group, and nextest stops its own tests.
  - A failing command keeps its exit status.
  - `test-xtask` gets `XTASK_ENV` through `"$@"` correctly.
- **Fixes I would name:**
  - Findings 1 and 2: give the three `cargo test` steps a 60 s limit (CI uses about 22, 7 and 10 s of them), or list them in `AGENTS.md` as exceptions to the ten-slowest print.
  - Finding 3: parse `nextest.toml` with a TOML parser, or refuse any line that mentions `overrides` or `slow-timeout` in another spelling.
  - Finding 4: `rm -f` the JUnit file at the start of each nextest step.
- **Written outside the worktree:** under the scratch directory scratch-adv: `near.sh`, `first-cross.sh`, `replay.sh`, `ci-pinned.txt`, `near.txt`, `replay.txt`, `case1.log`, three `adversary_budget_*.log` files and `suite.log`.

```findings
- file: Taskfile.yml
  line: 455
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: test-doc, test-feature-off-doc and fuzz-check run cargo test in a 120 s step, so a 61 s test passes, against the acceptance line that a test sleeping 61 s fails its step
- file: Taskfile.yml
  line: 455
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: test-doc, test-feature-off-doc and fuzz-check print no ten slowest tests, against the acceptance line that every test-running step prints them
- file: crates/edge/ess-xtask/tests/ci_lanes.rs
  line: 6834
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: the ratchet reads overrides by exact line text, so a header with a comment or inner spaces, slow-timeout= without spaces, or dotted slow-timeout keys add an all() override of 900 s unnoticed
- file: Taskfile.yml
  line: 473
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: test-slowest prints whatever junit.xml is on disk, so a feature-off run that wrote no report, or test-packages after its nested test-xtask, prints another step's tests as its own
- file: AGENTS.md
  line: 130
  category: contract-drift
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: the claim that each step's build runs before it is false for test-doc and test-feature-off-doc, which compile 16 and 6 crates inside the limit, and for test-feature-off, which extracts its archive inside it
```
