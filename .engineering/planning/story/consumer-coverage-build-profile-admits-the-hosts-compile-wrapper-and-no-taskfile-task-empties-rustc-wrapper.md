---
format: aep.planning-md/3
id: story:consumer-coverage-build-profile-admits-the-hosts-compile-wrapper-and-no-taskfile-task-empties-rustc-wrapper
kind: story
status: draft
title: Consumer coverage's build profile admits the host's compile wrapper as a measured tool, and no Taskfile task empties RUSTC_WRAPPER
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: AGENTS.md
- confidence: inferred
  path: CHANGELOG.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask/build.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/metadata.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/metadata_tests.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/mod.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/native.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/tests.rs
- confidence: inferred
  path: docs/design/cli-schema-metadata-accounting.md
revision: 11
---
## Outcome

`task test-xtask` and `task consumer-check` build with the host's compile wrapper in place, and
the consumer-coverage build check still refuses a build whose compile path holds a tool nobody
recorded. No task of `Taskfile.yml` sets `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER` or their
`CARGO_BUILD_` forms to the empty string.

## Need

The organization rule "One compile cache" (`~/beyond10x/AGENTS.md`) says no build sets
`RUSTC_WRAPPER` or `CARGO_BUILD_RUSTC_WRAPPER` to the empty string, and that the local gate keeps
the host's wrapper (`rustc-wrapper = "sccache"` in `~/.cargo/config.toml`, line 49). Two tasks of
`Taskfile.yml` at `16f2fc88c2` do:

- `test-xtask` (lines 401-416) sets `RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER=
  CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER=` (lines 413-414) beside
  `RUSTUP_TOOLCHAIN=1.98.1`, `RUSTFLAGS='-C link-arg=-fuse-ld=lld'`, `CARGO_BUILD_JOBS=2` and
  debug 0.
- `consumer-check` (lines 597-611) sets the same four (lines 606-607) with the same profile.

A check in the repository requires that. `validate_build`
(`crates/edge/ess-xtask/src/consumer_coverage/mod.rs:266`) refuses a provider build unless every
name in `WRAPPERS` (line 169: the four above) is in the recorded build environment as exactly `""`
and is also set to `""` in the process that runs the check (`wrapper(key).as_deref() !=
Some("")`, line 286; the build facts come from `crates/edge/ess-xtask/build.rs:100-125`). The
tests of that refusal are `metadata_tests.rs:248-300` (`compiled/current wrapper profile mismatch
RUSTC_WRAPPER`) and `tests.rs:386-398`. `native.rs:191-194` forces the same four to `""` in the
environment of each case it runs.

Why the check exists, read from its history:

- Commit `9f8efa0a40` (2026-09-10, "fix(ci): declare the measured profile for provider tests")
  put the empties into `task test`, with the comment "The metadata authority regression executes
  the compiled provider. Its test build needs the same declared profile as consumer-check,
  including empty wrappers; inheriting a developer or runner profile is not admissible proof."
  The `CHANGELOG.md` entry reads "The repository test runner declares the same compiler and
  wrapper profile as consumer qualification, so the compiled-provider regression runs with
  admissible evidence."
- The measured profile (`MEASURED_PROFILE`, `mod.rs:155`) fixes the target, the linker flag, the
  profile, the opt level, the job count and, with `MEASURED_TOOLCHAIN` `1.98.1`, the compiler. It
  is the declared answer to "what compiled the provider that runs the consumer cases": a recorded
  `CARGO` and `RUSTC` identity (path, version, sha256) that `validate_build` re-reads and compares,
  `provider_executable_sha256` and the source digest of every file. A wrapper between cargo and
  `rustc` is a tool in that compile path that the profile neither names nor hashes, so the profile
  admits none. Explicit empty, not unset, because an unset variable lets the user's cargo
  configuration choose one: an earlier unit's run 2 "exposed inherited Cargo wrapper defaults:
  removing wrapper environment variables selected sccache from ancestor/user configuration"
  (`.engineering/planning/story/counter-reachability-arithmetic-completeness.md`, § "Refreshed
  source and corrected verification environment").
- Commit `ba5d5a48b4` (2026-09-27) changed the tests: "Two consumer_coverage tests failed under any
  non-CI build environment (CARGO_TARGET_DIR, NUM_JOBS, DEBUG, RUSTC_WRAPPER). They prove run-bound
  proof and v3 retained-proof consumption; the environment refusal was only their precondition. They
  now run on injected build and process facts; `Authority::capture` still reads the real
  environment and refuses exactly as before." Since then every test of the qualification logic
  reads `Measured::qualified()` (`mod.rs:181-260`, injected, wrapper facts `""`) through
  `Authority::capture_in` and `invocation_in`; the only calls of the real-environment forms are
  `Authority::capture` (`metadata.rs:218`, from the `consumer-check` run at `mod.rs:763`) and
  `invocation` (`mod.rs:388`, from `mod.rs:82` and `native.rs:50`, the same run). No test calls
  them: a search for `Measured::current`, `Authority::capture(` and `invocation(` in the test
  modules finds only the injected forms.

So the two empties are different:

- `test-xtask`'s is residue. Its tests no longer read the process environment (the commit that made
  them injected is `ba5d5a48b4`; the task's comment still says the metadata regression "executes
  the compiled provider"). Nothing in `ess-xtask`'s test run needs the profile or the empties.
- `consumer-check`'s is the property above, kept in a parked lane (`task check` runs it only with
  `CONSUMER_CHECKS=true`, no CI lane runs it, `AGENTS.md` § "Consumer coverage is opt-in").

This is not a defect of the shared compile cache: no hang, no wrong artifact and no error from
sccache is on record in these commits, and the reason to leave it is a reviewed profile, not the
cache. The story therefore stays in this repository and no `slot` story is written.

34 other files under `crates/` name `RUSTC_WRAPPER`; 5 set it to `""` (for example
`crates/generate/ess-synth/tests/binding_condition_runtime.rs:413`) and the rest remove it
(`.env_remove`). They run cargo on a fixture of their own for a child process, which the rule
allows, and are not in this story.

## Fit review

A defect against an organization rule plus a property to carry over, not an adopter request. The
need touches the Taskfile, one build script, one module and its tests; no format, no generated byte
and no `ess/N`. Consumer coverage stays parked: this story changes how its profile treats a
wrapper, not whether the lane runs.

## Design

- First, measure what the build script sees (no code): build `ess-xtask` once with the host's
  wrapper and read `consumer-build.json` in its `OUT_DIR`, then again with the four empties, and
  record which of `RUSTC_WRAPPER`, `RUSTC_WORKSPACE_WRAPPER`, `CARGO_BUILD_RUSTC_WRAPPER` and
  `CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER` the file holds in each run. The design below depends on
  whether cargo hands a configured wrapper to build scripts as `RUSTC_WRAPPER`; this has not been
  measured here (no build was allowed) and is the story's first acceptance line.
- `test-xtask` runs `cargo test --package ess-xtask --locked` with the toolchain pin
  (`RUSTUP_TOOLCHAIN=1.98.1` stays: `rust-toolchain.toml` is the pin) and none of the profile
  variables or empties. The comment lines of that task go.
- `consumer-check` keeps the profile's flags, job count and toolchain, and stops emptying. The
  profile's wrapper rule changes from "empty" to "measured": `build.rs` records the wrapper the
  build used (its path and sha256, in `tools` beside `CARGO` and `RUSTC`), and `validate_build`
  requires, for the wrapper as for the other two tools, that the identity re-read at check time
  equals the recorded one and that the wrapper is the one the host declares. The wrapper becomes an
  input the profile names and hashes, so a changed sccache binary is a profile mismatch exactly as a
  changed `rustc` is.
- `native.rs` stops forcing the four to `""` in each case's environment and passes through what the
  check qualified. `Measured::qualified()` records the wrapper as the host's, and the test of the
  refusal keeps one case: a wrapper that is not the recorded one (`unreviewed-wrapper`,
  `metadata_tests.rs:289`) still fails.
- `build.rs`'s `rerun-if-env-changed` list keeps the four names (a changed wrapper still rebuilds
  the provider).
- Reviewed pins: `metadata.rs` and `mod.rs` changes move the `guard_source_sha256` pins
  (`ba5d5a48b4` moved six of them `07b8f86d` to `c43b868f` with a review section in
  `docs/design/cli-schema-metadata-accounting.md`); the same review section is written for this
  change, and `task test-xtask` shows no other pin moves.

## Acceptance

- The measurement of the first design item is in the pull request: the wrapper names
  `consumer-build.json` holds with the host's wrapper and with the four empties, and the one
  command that produced each.
- `git grep -n 'RUSTC_WRAPPER=\|RUSTC_WORKSPACE_WRAPPER=' -- Taskfile.yml` prints nothing, and
  `git grep -n 'RUSTC_WRAPPER' -- Taskfile.yml` prints only the `env -u` of the harness tasks.
- `task test-xtask` passes on the host with its wrapper, and with `CARGO_TARGET_DIR`,
  `CARGO_BUILD_JOBS=8` and `CARGO_PROFILE_DEV_DEBUG=2` set (the three the tests of `ba5d5a48b4`
  were made independent of), time named in the pull request (60 s bound per test, 120 s the step).
- `CONSUMER_CHECKS=true task consumer-check` on the host passes with the wrapper in place, and fails
  with a message naming the wrapper when `sccache` is replaced by another binary of the same name
  after the provider is built (shown once, by the command and its output).
- The unit test `metadata_tests.rs` `compiled/current wrapper profile mismatch` case still exists
  and still refuses an unrecorded wrapper.
- `ess/AGENTS.md` § "Consumer coverage is opt-in" says the profile admits the host's wrapper, and
  no sentence of the repository says the profile requires an empty one: `git grep -n -i 'empty
  wrapper\|wrappers explicitly empty'` prints nothing.

## Decisions

A. How the profile treats a wrapper.
   - Measure it as a tool (recommended: keeps "nothing unrecorded between source and `rustc`",
     matches the organization rule, costs one more recorded identity and a re-review of the pins).
   - Admit any wrapper without recording it (simplest; the profile then stops claiming anything
     about the compile path beyond `rustc`'s own hash).
   - Keep the empties for `consumer-check` alone, with the organization's exception named in
     `AGENTS.md` (against the rule, which says a reason to leave the shared cache is a story in the
     `slot` repository; this one is not that, so there would be no story to name).

B. Whether `test-xtask` keeps the native profile (lld, 2 jobs, debug 0) after the wrapper goes.
   - Drop it (recommended: the tests are environment-independent since `ba5d5a48b4`, and the gate
     states the pin of the toolchain through `rust-toolchain.toml`).
   - Keep the rest of the profile (a smaller change, a task that still claims an environment its
     tests no longer read).
