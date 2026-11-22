---
format: aep.planning-md/3
id: story:the-local-gate-catches-what-ci-catches
kind: story
status: draft
title: 'The local gate catches what CI catches: one pinned rustc, the runner''s differences written and reproduced, a watch task that exits non-zero on red'
summary: Pull request 797 cost 71 minutes and a second full CI run because target/tmp exists only in a used local tree; a CARGO_TARGET_TMPDIR check, rust-toolchain.toml read by local and CI, one list of runner differences each reproduced or marked CI only, and a task that watches a pull request's checks
tags:
- tooling
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: .github/workflows/ci.yml
- confidence: inferred
  path: .github/workflows/package.yml
- confidence: inferred
  path: .github/workflows/pages.yml
- confidence: inferred
  path: .github/workflows/release.yml
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: Taskfile.yml
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/mod.rs
- confidence: inferred
  path: crates/edge/ess-xtask/src/main.rs
- confidence: inferred
  path: crates/edge/ess-xtask/tests/ci_lanes.rs
- confidence: inferred
  path: crates/edge/ess-xtask/tests/target_tmpdir.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/external_request_binding.rs
- confidence: inferred
  path: rust-toolchain.toml
revision: 3
---
## Outcome

ESS's local gate fails where its CI fails, as the organization's rule `The local gate equals the
forge` asks. Checkable:

- **(a) A file that exists only in a used local tree cannot make a test pass locally and fail on
  CI.** A repository check, an `ess-xtask` test in the style of `layout.rs` and `no_test_builds.rs`,
  fails when a test file uses `CARGO_TARGET_TMPDIR` in a way that needs the directory to exist
  without creating it first. The local package gate runs tests the way a CI shard does wherever
  the two differ in outcome (decision O3).
- **(b) One pinned rustc.** `rust-toolchain.toml` names one rustc. The local gate and every CI and
  release job that compiles read it; no workflow and no Taskfile line names a rustc version of its
  own. The `msrv` job still checks the `rust-version` `Cargo.toml` declares.
- **(c) The runner's differences are written in one place.** Every difference between a local
  package gate and a CI test shard listed under Need is in one list in the repository. Each entry
  is either reproduced by a named local task or marked "CI only" with the reason. `ci_lanes.rs`
  fails when `ci.yml` sets an environment variable or installs a tool that the list does not name.
- **(d) A red CI run makes a local command exit non-zero.** One task pushes nothing, watches a pull
  request's check runs until all have ended, and exits non-zero when one ended red. It names each
  failed job and its failing task from the check-run annotations.

## Need

### The case of 2026-11-21

Pull request 797 ("A conformance run drives a target process written in any language", merged as
`da544dcdef`) passed its package gate locally and went red on CI run 44885697541. Two cases of
`ess-cli`'s `adversary_process_target_pass1` failed within 10 ms on shards 11/20 and 20/20:
`tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))` returned `NotFound`. A CI shard extracts the
test binaries from an archive into a fresh `target/`, which has no `target/tmp`; local `cargo test`
creates it, and a used local tree has it anyway. The fix, `fc1ffeb859`, added
`fs::create_dir_all(env!("CARGO_TARGET_TMPDIR"))` before the call. It cost 71 minutes, a second full
CI run for the pull request. A full run costs 628 to 672 runner-minutes (runs 44936791749 and
44942406608, measured in `story:the-test-step-meets-the-test-budget`). Nothing local would have
caught it, and nothing today would catch the next one.

### `CARGO_TARGET_TMPDIR` in the tree today

Counted on the branch at 0.97.1 (`grep -rl CARGO_TARGET_TMPDIR crates fuzz --include='*.rs'`):

- **356 `.rs` files** use `CARGO_TARGET_TMPDIR`.
- **4 create the directory itself** with `fs::create_dir_all(env!("CARGO_TARGET_TMPDIR"))` before
  using it: `ess-cli`'s `adversary_process_target_pass1.rs`, `binary64_structural_adversary.rs`,
  `interpreted_target_adversary.rs` and `interpreted_target_adversary_pass2.rs`.
- **Most of the others are safe by shape**: they join a subdirectory and create it with
  `create_dir_all`, which creates the parent too, or hand a subdirectory to `cargo` or `go` as
  `CARGO_TARGET_DIR` or `GOCACHE`, which create it.
- **One file still has the defect**: `crates/generate/ess-synth/tests/external_request_binding.rs`
  calls the non-recursive `std::fs::create_dir` on a child of `CARGO_TARGET_TMPDIR` at lines 268,
  512 and 640 (`rust_case_with_source`, `go_case_with_source`, `fresh_root`), used by 10 of its
  tests. It passes on CI only when another test in the shard created `target/tmp` before it ran.
  The CI step `Drop the workspace archive` runs `rm -rf target/tmp target/debug` before the
  feature-off tests (`ci.yml`), so that step starts without the directory too.

This count is by pattern (`tempdir_in`, `new_in`, `create_dir(`, `File::create` and `fs::write` on
the root); the check in design step 1 replaces it with a syntax walk.

### The toolchain

- **CI's gate runs rustc 1.98.1**, written 7 times in `ci.yml` (`checks`, `build-tests`,
  `build-feature-off`, `test`, `transport-network`, `output-ownership-macos`, each
  `toolchain: 1.98.1`) and twice in `release.yml` (`hotfix-checks`, `hotfix-tests`).
- **The shipped release binaries are built by an unpinned `stable`**: `package.yml` (the four
  archives), `release.yml`'s `wasm` and `release` jobs and `pages.yml` call
  `dtolnay/rust-toolchain` without a `toolchain:` input. A release is compiled by a rustc the gate
  did not test.
- **A local build uses rustc 1.99.1** (`rustc --version` on the shared development host,
  2026-11-21; `stable` is the default toolchain), so a local `cargo clippy -p` lints with a newer
  Clippy than CI's. Only `task test-xtask` and `task consumer-check` force 1.98.1, through
  `RUSTUP_TOOLCHAIN=1.98.1` (`Taskfile.yml` lines 361 and 554), and `consumer-check`'s profile pins
  `MEASURED_TOOLCHAIN = "1.98.1"` (`crates/edge/ess-xtask/src/consumer_coverage/mod.rs:176`).
- **There is no `rust-toolchain.toml`.** The organization's record of the same day names the loom
  repository's red run on rustc 1.100.0 on CI and 1.99.1 locally.
- **`msrv`** reads `rust-version = "1.85"` from `Cargo.toml` and runs
  `cargo +1.85.0 check --workspace --all-targets --locked` (`ci.yml`, job `msrv`). The `+toolchain`
  argument outranks `rust-toolchain.toml` in rustup's order, so a pin does not change what `msrv`
  checks. It changes what every other job compiles with: a release binary would be built by the
  pinned rustc instead of the newest `stable` of that day.

### What a CI shard does that a local package gate does not

AGENTS.md § Gate tells a session to run `cargo clippy -p` and `cargo test -p` for the crates it
touched, then "push and read the lanes". Read from the files on 2026-11-21:

| # | difference | CI | local package gate | written where | reproduced locally by |
|---|---|---|---|---|---|
| 1 | rustc | 1.98.1; release archives `stable` | default `stable`, 1.99.1 here | `ci.yml`, `release.yml`, `package.yml` | nothing; `cargo +1.98.1` by hand |
| 2 | runner and layout | `cargo nextest run` from an archive extracted into a fresh `target/`, one process per test (`test-shard`, `test-feature-off`) | `cargo test -p`: threads in one process, `target/tmp` exists | `Taskfile.yml` comment above `test-shard`, AGENTS.md | `task test-archive` and `task test-shard`, whole workspace only (about 45 GiB); `task test-packages` uses nextest but no archive |
| 3 | `target/tmp`, `target/debug` | removed before the feature-off step | present | `ci.yml` (`Drop the workspace archive`) | nothing |
| 4 | test build profile | `CARGO_PROFILE_{DEV,TEST}_DEBUG=0`, `..._STRIP=symbols` | `debug = "line-tables-only"` (`Cargo.toml`) | `ci.yml` (`build-tests`, `build-feature-off`) | nothing |
| 5 | linker | lld through `CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUSTFLAGS` | system linker | `ci.yml` line 50, AGENTS.md | CI only by design (AGENTS.md); `test-xtask` uses lld |
| 6 | network | `ESS_TUTORIAL_NETWORK=1` on workspace shards | unset, `tutorial_page`'s `npm` steps skip | `ci.yml`, AGENTS.md | setting the variable by hand |
| 7 | `@types/node` | `ESS_TYPES_NODE` set to `@types/node@26.3.0` | unset: `adversary_287_pass1` fails `TS2688` | `ci.yml`, AGENTS.md | by hand (`npm install --prefix`) |
| 8 | JavaScript tools | `typescript@6.0.3`, `prettier@3.8.1`, `esbuild@0.28.2`, node 22 | what is on `PATH`; here `tsc` and `prettier` from the system, no `esbuild` (`ess-ui-react` cases exit 127) | `ci.yml`, AGENTS.md | by hand |
| 9 | Go | 1.25.10 (`setup-go`) | what is on `PATH`, go1.27.1 here | `ci.yml` only | nothing |
| 10 | harness variables | 7 `ESS_*` variables (`ESS_INTEGER_SPELLINGS_HARNESS` and six more) | unset under bare `cargo test -p` | `Taskfile.yml`, AGENTS.md | `task test-packages PACKAGES=…`, `task test` |
| 11 | `TMPDIR` | `/tmp`, outside any checkout; `$RUNNER_TEMP/ess-ownership` on macOS | the session's, long or inside a checkout (`observed_bindings` test, the a16 socket case) | AGENTS.md | nothing |
| 12 | feature-off build | number-semantics subset on a pull request, full elsewhere | not run by the package gate | `ci.yml`, `Taskfile.yml`, AGENTS.md | `task test-archive-feature-off`, `task test-feature-off` |
| 13 | checkout | clean, full history, tags fetched | a worktree with untracked files, tags as fetched | `ci.yml` | nothing |
| 14 | task runner | `task` 3.52.0 | 3.54.0 here | `ci.yml` | nothing |
| 15 | compile cache | `Swatinem/rust-cache` | `sccache` on port 4303 (AGENTS.md) | AGENTS.md | CI only; no effect on outcome expected |

Rows 2, 3 and 11 have failed a pull request or a local gate already: `fc1ffeb859` (row 2/3),
`story:delivery-trust-fixture-race` (nextest's process-parallel runner, run 36604168638, row 2),
the `observed_bindings` case and `story:the-a16-socket-case-binds-under-any-tmpdir` (row 11).

### How a session learns CI went red

No task, script or `ess-xtask` command waits for a pull request's checks. `Taskfile.yml`,
`.github/scripts/` (only `hotfix-scope.sh`) and `crates/edge/ess-xtask/src` hold nothing that
watches a run. `task release-status` asks GitHub about tags and releases only
(`crates/edge/ess-xtask/src/main.rs`, through `gh`). AGENTS.md describes reading a failed `Gate`
through the check-run annotations endpoint by hand ("Reading a failed `Gate` without the log").

## Design

Each step is one pull request and lands after the one before. Every step that adds or changes a
test keeps the organization's bounds: no test over 60 s, and a test step over 120 s fails.

1. **The `CARGO_TARGET_TMPDIR` check.** A new `crates/edge/ess-xtask/tests/target_tmpdir.rs` walks
   every test file with `syn`, as `no_test_builds.rs` does, and fails when a use of
   `CARGO_TARGET_TMPDIR` needs the directory to exist (`tempdir_in`, `TempDir::new_in`,
   `Builder::tempdir_in`, `File::create`, `fs::write`, non-recursive `create_dir` on the root or a
   direct child) and no `create_dir_all` of the root or of a path under it comes first in the same
   function. It prints each finding as `file:line`. The same change fixes
   `external_request_binding.rs` and every other file the check finds; no allow-list. The test is
   timed before it lands: under 60 s, and `test-xtask` stays within its step budget. Prevents: the
   `fc1ffeb859` class, a directory that exists only in a used tree.
2. **One pinned rustc.** Add `rust-toolchain.toml` with the version decision O1 picks, the
   `rustfmt` and `clippy` components and the `wasm32-unknown-unknown` target. Each `toolchain:`
   input in `ci.yml`, `release.yml`, `package.yml` and `pages.yml` is read from the file, the way
   `msrv` reads `rust-version`. `Taskfile.yml`'s two `RUSTUP_TOOLCHAIN=1.98.1` lines are dropped or
   read from the file, and `MEASURED_TOOLCHAIN` is pinned to it by a test. `ci_lanes.rs` fails when
   a workflow or the Taskfile names a rustc version other than through the file, and `msrv` keeps
   `cargo +<rust-version>.0`. Prevents: a lint or a compile error that only one compiler reports,
   and a release built by a compiler the gate never ran.
3. **The watch task.** `task ci-watch PR=<n>` (an `ess-xtask` subcommand behind it) reads the pull
   request's head commit, polls its check runs until every one has ended, and exits 0 only when
   each ended `success`, `skipped` or `neutral`. Otherwise it exits non-zero and prints, for each
   failed check run, its name and the annotation lines that name a task (`task: Failed to run task
   "<name>"`), read from `/repos/{owner}/{repo}/check-runs/<id>/annotations`. It pushes nothing and
   writes nothing to GitHub. It has a bounded wait and exits non-zero when the bound passes. It is
   tested against canned API answers, like the `prebuilt` step in `ci_lanes.rs`. AGENTS.md § Gate
   says to run it in the background after every push. Prevents: a red run nobody reads until a
   merge is blocked.
4. **The differences, written and reproduced.** The table under Need moves into one list in the
   repository (decision O4). Each row names the local task that reproduces it or says "CI only" and
   why. `ci_lanes.rs` fails when `ci.yml` sets an environment variable at workflow, job or step
   level, or installs a tool or version, that the list does not name. `task test-packages`, the
   local package gate, runs as a shard runs wherever the outcome differs (decision O3): nextest with
   the CI profile `story:the-test-step-meets-the-test-budget` adds (60 s kill), the harness
   variables, and refusal by name when `ESS_TYPES_NODE`, `esbuild` or the pinned Go is missing,
   instead of a later exit 127. AGENTS.md § Gate names `task test-packages` as the package gate in
   place of bare `cargo test -p`. Prevents: a difference nobody wrote down, and a missing tool that
   shows only as a failure on CI or a local failure with no reason given.

When CI goes red after a green local gate from here on, the fix includes the change that makes the
local gate catch the same failure (the organization rule): a new check, a new list row with its
task, or a change to `test-packages`.

## Acceptance

- `target_tmpdir.rs` fails on a fixture test file that calls `tempfile::tempdir_in(env!("CARGO_TARGET_TMPDIR"))`
  with no `create_dir_all` before it, and passes when one is added. It passes on the tree once
  `external_request_binding.rs` is fixed, and runs under 60 s.
- With `target/tmp` removed, `external_request_binding`'s tests and
  `adversary_process_target_pass1`'s two signal cases pass, run one test at a time from the built
  binary (how `fc1ffeb859` reproduced the failure).
- `rust-toolchain.toml` exists and names one rustc. `grep` finds no rustc version in
  `.github/workflows/` or `Taskfile.yml` other than the `msrv` job's read of `rust-version`, and
  `ci_lanes.rs` fails when one is added. `msrv` still runs `cargo +<rust-version>.0 check
  --workspace --all-targets --locked`, and
  `the_msrv_job_checks_every_target_on_the_declared_rust_version` passes.
- A local `rustc --version` in the repository prints the same version the CI jobs print.
- `task ci-watch PR=<n>` on a pull request whose run ended red exits non-zero and prints the failed
  check run's name and its failing task. On a green run it exits 0. Against canned answers, a
  pending, a failed, a cancelled and a timed-out case each give the documented exit.
- The list of differences exists in one place. Every row names a local task or says "CI only" with
  a reason, and `ci_lanes.rs` fails when `ci.yml` gains an environment variable or a tool the list
  does not name.
- `task test-packages PACKAGES=<crate>` refuses by name when a tool a shard has is missing, and
  runs under the 60 s per-test limit.
- AGENTS.md § Gate names the package gate as `task test-packages`, names `task ci-watch`, and
  points to the list.

## Fit review

The skill `.agents/skills/assessing-external-requests/SKILL.md` is written for changes to the
authored language; this request changes ESS's own gate, which no authored document reaches. Its
seven questions are answered for the gate.

1. **The need.** The organization's rule `The local gate equals the forge` (organization
   `AGENTS.md`, 2026-11-21) asks for one pinned rustc read by the local gate and CI, the runner's
   differences written in the repository and used by the local gate, a watch or push script that
   exits non-zero on a red run, and a local-gate change with every fix of a CI-only failure. ESS has
   none of the four (Need). The case is pull request 797: 71 minutes and a second full run of 628
   to 672 runner-minutes.
2. **Class: defect.** The rule binds, and the gate does not meet it. A test passing locally and
   failing on CI is a defect of the gate, not of the change under test.
3. **Can it already be expressed?** In part. AGENTS.md § Gate lists rows 5, 6, 7, 8, 10, 11 and 15
   of the table in prose. `task test-packages` already runs nextest and sets the harness variables.
   `msrv` already reads a version from a file. `ci_lanes.rs` already holds the workflows to the
   Taskfile. Nothing pins the rustc, checks `CARGO_TARGET_TMPDIR`, lists rows 1, 3, 4, 9, 13 or 14,
   or watches a run.
4. **Fit with what is there.** The check sits beside `layout.rs`, `no_test_builds.rs` and
   `host_paths.rs`, the existing syntax-walking repository checks. The workflow pins sit in
   `ci_lanes.rs` beside the `msrv` and `prior-gate` pins. The watch task reads GitHub the way
   `task release-status` does, and stays outside the offline gate for the same reason. The 60 s kill
   comes from `story:the-test-step-meets-the-test-budget`, not from here.
5. **A second adopter.** Every repository under the rule; the organization's record names loom
   (rustc 1.100.0 against 1.99.1), surf and swell with the same shape on the same day.
6. **Cost.** No format, key, diagnostic or generated byte changes. One new `ess-xtask` test, one
   `ess-xtask` subcommand, a file at the root, rewritten `toolchain:` inputs in four workflows,
   new `ci_lanes.rs` pins, a list in the repository, and the fixes the check finds. The release
   binaries change compiler, from the newest `stable` to the pin (decision O2). A session must have
   the pinned toolchain installed; rustup installs it on first use.
7. **What else was considered.**
   - *Change nothing.* Refused: the rule binds.
   - *Run every test in `cargo test` from a fresh `target/` locally.* Refused: it rebuilds about
     45 GiB, and the rule asks for the difference to be reproduced, not for the whole gate to run
     locally.
   - *One `target_tmpdir()` helper that every test must call.* Kept as an option for step 1 if the
     syntax walk proves too loose; it rewrites up to 356 files.
   - *Only document the differences.* Refused: the rule asks that the local gate use them.

## Decisions

- **Adopt.** The organization's rule binds; the outcome is taken as the rule writes it (the
  skill's *accept as proposed*). Decided by the planning session on 2026-11-21.
- **Ordering.** After step 2 and step 1 of `story:the-test-step-meets-the-test-budget`: step 2
  there reshapes `ci.yml`'s jobs and step 1 adds the nextest CI profile with the 60 s kill that
  design step 4 here runs locally. Within this story, the design steps land in the order written,
  smallest first.

### O1 to O5: the options, the recommended one first

- **O1. Which rustc to pin, and how it moves.** (a) 1.98.1, what CI's gate and the consumer profile
  run today: CI's compiler does not change and the development host has it installed. (b) 1.99.1,
  the local `stable` today: CI moves and `consumer-check`'s profile needs a review. (c) The newest
  `stable` when step 2 lands. How it moves: (a) one pull request per move, when a dependency or a
  feature needs it, with `ci_lanes.rs` holding every reader to the file; (b) a move within a fixed
  time after each `stable` release.
- **O2. The release binaries.** (a) Built by the pinned rustc, the compiler the gate tested. (b) Keep
  the newest `stable` for releases and pin only the gate.
- **O3. How the package gate reproduces the shard layout.** (a) The step 1 check is the guard, and
  `test-packages` also starts its run from an empty `target/tmp`, as the feature-off step does.
  This costs rebuilding what tests cache there; the run must stay within the 120 s step budget for
  the touched crates. (b) The step 1 check only. (c) `test-packages` builds a nextest archive of
  the touched crates and runs it extracted, as a shard does.
- **O4. Where the list of differences lives.** (a) A table in AGENTS.md § Gate that `ci_lanes.rs`
  parses. (b) A data file such as `.github/ci-differences.yaml` that `ci_lanes.rs` reads and
  AGENTS.md points to.
- **O5. The watch task's GitHub route.** (a) `gh`, read-only, as `task release-status` does.
  (b) `b10x-gates api`, the route AGENTS.md describes for annotations.

### O1 to O5: taken

Each took its recommended option on 2026-11-21, on the operator's rule that a default proceeds;
none changes a ruleset, a published release's artifacts or the architecture.

- **O1: (a)** pin rustc 1.98.1, and (a) it moves by one pull request per move.
- **O2: (a)** release archives are built by the pinned rustc, from the first release after the
  pin lands; archives already published are not rebuilt.
- **O3: (a)** the check of design step 1 is the guard, and `test-packages` starts from an empty
  `target/tmp`.
- **O4: (a)** the list of differences is a table in `AGENTS.md` § Gate that `ci_lanes.rs` parses.
- **O5: (a)** the watch task reads through `gh`, read-only.

## Out of scope

- The test budget itself: the 60 s kill, the 120 s step bound, the plan-only skip, the `slow` lane
  and shared fixtures (`story:the-test-step-meets-the-test-budget`).
- Running the whole `task check` locally before a push; AGENTS.md keeps CI as the owner of the full
  gate.
- The `TMPDIR` length defect of the a16 socket case
  (`story:the-a16-socket-case-binds-under-any-tmpdir`) and the delivery-trust fixture race
  (`story:delivery-trust-fixture-race`): their own stories. Their rows go in the list.
- Larger runners, macOS output-ownership witnesses beyond their toolchain input, Website and Atlas.
- Pushing, merging or retrying a run from the watch task.

## Overlap

- `story:the-test-step-meets-the-test-budget` (active): owns the nextest CI profile and the step
  bounds this story's step 4 uses; edits the same `ci.yml` jobs and `ci_lanes.rs`. Lands first.
- `story:the-a16-socket-case-binds-under-any-tmpdir` (draft): a local-only failure from a long
  `TMPDIR` (row 11).
- `story:delivery-trust-fixture-race` (draft): a CI-only failure from nextest's process-parallel
  runner (row 2, run 36604168638).
- `story:one-integration-test-binary-per-large-crate` (draft): changes the archive layout that row
  2 describes.
- `task:test-builds-take-less-disk` (draft): disk of the test build that makes a local shard run
  impractical today.
- Implemented and not reopened: `story:test-scratch-directories-are-removed-on-drop` (scratch
  under `TMPDIR` is removed on drop), `task:binary64-structural-fixture-isolation` (the
  `create_dir_all` idiom `fc1ffeb859` copied), `story:release-gate-green-at-the-tag`,
  `story:ci-test-shards-each-extract-half-the-test-binaries`.

No epic covers CI or the gate; like its siblings above, this story serves `vision:O2` directly.

## Scope

- inferred: `rust-toolchain.toml` (new)
- inferred: `.github/workflows/ci.yml`, `release.yml`, `package.yml`, `pages.yml` (`toolchain:`
  inputs)
- inferred: `Taskfile.yml` (`test-packages`, `test-xtask`, `consumer-check`, the new `ci-watch`)
- inferred: `crates/edge/ess-xtask/tests/target_tmpdir.rs` (new), `crates/edge/ess-xtask/tests/ci_lanes.rs`
- inferred: `crates/edge/ess-xtask/src/main.rs` (the watch subcommand),
  `crates/edge/ess-xtask/src/consumer_coverage/mod.rs` (`MEASURED_TOOLCHAIN`)
- inferred: `crates/generate/ess-synth/tests/external_request_binding.rs` and the other files step 1
  finds
- inferred: `AGENTS.md` § Gate
