---
format: aep.planning-md/3
id: story:tests-that-build-into-one-shared-scratch-directory-collide-when-each-test-runs-in-its-own-process-one-fixture-is-built-once-per-run-keyed-by-its-sources
kind: story
status: draft
title: 'Tests that build into one shared scratch directory collide when each test runs in its own process: one fixture is built once per run, keyed by its sources'
summary: 'A OnceLock per test process is no shared fixture under one process per test: transport_free_entry.rs:828, generate_ui_tui.rs:121 and the wasm host lock collide on a fresh target'
tags:
- defect
- gap-reports-2026-11-24
- test-step
relations:
- serves: vision:O2
revision: 1
---
## Summary

Tests that build one shared fixture into one scratch directory under `target/tmp/`, each behind a
`OnceLock`, collide when the runner starts every test in its own process (cargo-nextest): each
process runs the initializer, each initializer empties or writes the same directory, and a test
that was reading what another process built loses it. This is the case `AGENTS.md` § Tests stay
fast names: "A `OnceLock` or other in-process global is not this form under cargo-nextest, which
runs each test in its own process, so the fixture is built once per test". The accepted forms are a
cache on disk keyed by the digest of its sources, or a setup step before the tests.

## Evidence

Found on 2026-11-25 in the whole check of a branch, in a fresh `target/`, with the tests of one
binary starting together. Each failed once and passed on a rerun alone:

- `crates/generate/ess-synth/tests/transport_free_entry.rs:828`, `adversary_pass2_harness`: a
  `OnceLock<PathBuf>` that builds the generated tree and a harness crate into
  `scratch("adversary-pass2")`. Failure: `adversary_pass2_a_declared_tree_with_no_json_is_answered_by_handle`,
  a rustc internal error "expecting a current working directory to exist", building in a directory
  another process had emptied. Rerun of the binary: 8 of 8 pass.
- `crates/edge/ess-cli/tests/generate_ui_tui.rs:121`, `built_desk`: a `OnceLock<PathBuf>` building
  into `scratch("built")`; `scratch()` removes the directory first. Failure:
  `the_built_crate_reads_a_view_from_a_synthesized_gatepass_server`, "can't find bin `desk` at
  `target/tmp/generate-ui-tui/built/app/src/main.rs`". Rerun of the binary: 3 of 3 pass.
- `crates/verify/ess-conformance/tests/support_initial_state/mod.rs:268`: the host's `Cargo.lock` is
  generated when it does not exist; a process that checks while another still generates it builds
  with `--locked` against a lock being written. Seen on the first run in a fresh `target/`
  (slice 1 of `ess-conformance`, and the two host cases `caller_upsert_wasm_matches_actual_native_outcomes`
  and `caller_set_wasm_matches_actual_native_outcomes` of the slow lane, started together, 0.3 s
  and 0.5 s): `cannot update the lock file ... because --locked`.

The branch changed none of these files, and an earlier batch of the same branch passed the first
two. The collision depends on which tests of a binary start at the same moment on a node.

## Red test

A test that starts the initializer of each site from two processes at once (two child processes of
the test binary, or two `cargo nextest` instances on one scratch directory) and expects both to
read a built fixture: red today for each site with the failure above.

## Acceptance

- Each site builds its fixture once per test run, never per test: a cache on disk under `target/`
  keyed by the digest of its sources, written to a temporary name and renamed into place, or a
  setup step before the tests; a second process waits for the first or finds the finished product.
- The red test is green for each of the three sites; the three binaries pass together on a fresh
  `target/` in one run.
- The scratch helper of each file (`scratch()`) no longer removes a directory another process may
  be building in.
- Every other `OnceLock` of a test that builds into a shared directory is listed in the change with
  its fate (fixed, or shown to build into a directory of its own per process).

## Fit review

1. **Class.** A defect of the test step: a rule the repository already states (the test-budget
   section of `AGENTS.md`, the `OnceLock` case), met here by three fixtures.
2. **Border.** `story:the-wasm-host-an-adversary-test-builds-regenerates-its-lock-file-when-the-workspace-version-moves-not-only-when-the-lock-is-missing`
   (the same directory, a different failure: a lock kept across a version bump) and
   `story:tests-that-run-over-60-s-on-main-without-a-branch-of-the-week-run-within-60-s-or-move-to-the-slow-lane`
   (time, not collision).

## Decisions

- None open.
