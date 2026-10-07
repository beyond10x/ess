---
format: aep.planning-md/3
id: story:one-integration-test-binary-per-large-crate
kind: story
status: draft
title: One integration test binary per large crate, so test builds stop linking every dependency 1294 times
relations:
- serves: vision:O2
revision: 1
---
## Finding

On 2026-10-07 one release-gate tree's `target/debug/deps` reached 73G, almost all of it test
binaries. Each file directly under a crate's `tests/` is its own test target, and each one links
every dependency of the crate again. The workspace has 1294 such files (counted 2026-10-07 on the
wave C integration branch):

| crate | `tests/*.rs` |
|---|---|
| ess-conformance | 465 |
| ess-domain | 139 |
| ess-cli | 131 |
| ess-synth | 100 |
| ess-compiler | 60 |
| ess-diff | 59 |
| ess-gen | 49 |
| schema-contract | 39 |
| ess-entity-runtime | 38 |

## Design

One test target per crate: `tests/<crate-suite>/main.rs` declaring one module per former file,
each file moved with `git mv` so its history follows. Shared helpers under `tests/support/` or
`tests/common/` become a module of that target. No test file under `tests/` uses `set_var`,
`remove_var` or `set_current_dir` (grep, 2026-10-07), so `cargo test` running them as threads of one
process does not change their meaning; nextest still runs each test in its own process.

`ess-cli` keeps two targets, because the pull-request feature-off archive selects ess-cli test
binaries by name (`FEATURE_OFF_NUMBER_SEMANTICS` in `Taskfile.yml`:
`binary(/^(binary64|normalization|schema|execution_recovery)/)`): one target for the
number-semantics modules and one for the rest. The filter and `crates/edge/ess-xtask/tests/ci_lanes.rs`
move to the new target names in the same change.

## Acceptance

- Each crate in the table above has at most one integration test target, ess-cli at most two, and
  `cargo nextest list` names every test the old targets named (the count before and after is in the
  pull request).
- An `ess-xtask` check fails, naming the file, when a new `*.rs` file lands directly under the
  `tests/` directory of one of these crates.
- The pull-request feature-off archive still compiles only the number-semantics ess-cli tests, and
  `ci_lanes.rs` still fails when an ess-cli test source naming `arbitrary_precision` falls outside
  that filter.
- The size of `target/debug/deps` after `cargo test --workspace --no-run` on one tree, before and
  after, is in the pull request.

## Sequencing

Lands as the last wave of 0.57.0, after every other branch that adds test files (wave C, the
issue waves, and the other session's open pull requests) has merged: moving 1000+ files conflicts
with each of them. A test file a later branch adds directly under `tests/` is caught by the xtask
check.
