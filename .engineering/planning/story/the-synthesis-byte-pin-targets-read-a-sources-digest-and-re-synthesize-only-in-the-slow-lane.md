---
format: aep.planning-md/3
id: story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane
kind: story
status: draft
title: The synthesis byte-pin targets read a committed sources digest in the test step and re-synthesize every model only in the slow lane
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- depends_on: story:a-slow-lane-runs-the-tests-marked-slow-in-task-test-slow-and-a-ci-job-that-gate-carries
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: inferred
  path: crates/edge/ess-xtask/tests/slow_lane.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/enum_presence_guard.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/synthesis_suite_bytes_table_cli_read.rs
revision: 8
---
## Outcome

The two synthesis byte-pin targets and two more tests that synthesize a whole corpus no longer do it in the test step.

- A committed file `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` holds a SHA-256 over the sources of synthesis (the `src/` of the eight crates it runs through, and every model file the tables read). A fast test in the step recomputes it and compares it with the file; it takes under 2 s and synthesizes nothing.
- The four tests that synthesize every model are marked `#[ignore = "slow lane: ..."]` and run in `task test-slow` and the `slow` job (t1). They still compare bytes with the committed tables, as before.
- A change to synthesis or to a model that does not re-stamp the file fails the step, naming the command that writes the stamp.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 5, decision D1 (b): committed generated files with a source digest, compared in the step; full regeneration in the `slow` lane. Rule: "Tests stay fast" (an expensive product is made once, never per test).
Addition A names `synthesis_suite_bytes_table` (60 to 811 s by host load) and `enum_presence_guard` (about 716 s): `synthesize(` called inside the test over the whole corpus.
Consumer gap: none.

## Where

Read at `bf128f9146` (the tree has since moved to `ff7f8e74fb`; the files below are unchanged between them). Re-read a line before editing it.

`crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`:
- Doc comment 1-65 (the re-pin commands at 57 and 64), `header` 78, `root` 92, `digest` 99 (`sha2` over bytes, formatted `<hex>:<len>`), `walk` 109-128 (skips `target`, `node_modules`, `.git`, `.engineering`; collects `.yaml` and `.yml`), `models` 135 (keeps a file only when a line starts with `format: ess/`, line 145; a directory that holds a `system.yaml` is a model of its own), `model_lines` 205, `component_lines` 230, `table` 258, `compare` 286.
- The two tests: `#[test]` 355 / `every_repository_model_keeps_its_synthesized_bytes` 356; `#[test]` 373 / `every_repository_component_keeps_its_synthesized_bytes` 374.
- Doc comment 71-73 on `UNAFFORDABLE` names `task test-slow-probes`, which t1 deletes.

`crates/verify/ess-conformance/tests/synthesis_suite_bytes_table_cli_read.rs`: `#[test]` 106 / `every_pinned_system_directory_is_read_as_the_cli_reads_it` 107.

`crates/verify/ess-conformance/tests/enum_presence_guard.rs`: `corpus()` is called by the test at `#[test]` 708 / `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere` 709; `synthesize(` at 167.

`AGENTS.md` lines 206-218 (the paragraph "A gate of named test targets includes the repository-wide byte pins" and its list at 211-216); line 220-222 is t1's.

`crates/edge/ess-xtask/tests/slow_lane.rs`: the `MARKED` table (created by t1).

`CHANGELOG.md` `## [Unreleased]` line 3, `### Changed` line 5.

## Tests first

Bounds for every command below: no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most). Time every new test with `target/nextest/ci/junit.xml` or `--durations`; name any test over 5 s in the report.

0. Measure first. Run `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-conformance --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard` on the unchanged tree (a tree that has t1) and read each test's time from `target/nextest/ci/junit.xml`. Decision rule: a test over 30 s gets the same treatment as step 3 below; a test of these four under 30 s stays in the step with its time named in the report (the marker is then not placed on it, and `MARKED` does not list it).1. In `synthesis_suite_bytes_table.rs`, new tests (each under 2 s):
   - `the_synthesis_sources_digest_matches_the_committed_stamp`: computes the digest with `sources_digest(&root())` (step 2), reads `tests/fixtures/synthesis-sources.digest`, and compares. Failure message: the stored and computed values, the count of crates and models, and the command `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`. Red today: `failed to read .../synthesis-sources.digest` (the file does not exist and the function is absent: a compile error naming `sources_digest`).
   - `the_sources_digest_moves_with_a_source_byte_a_model_byte_and_a_new_model_and_ignores_a_manifest`: builds a scratch tree (use a `std::env::temp_dir`-free path: `env!("CARGO_TARGET_TMPDIR")` joined with a fixed directory name, deleted first) with two crate `src/` directories and one model file holding `format: ess/1` on a line; asserts the digest changes when one byte of a source changes, when one byte of the model changes, when a second model file is added, and when a source file is renamed; and does not change when a `Cargo.toml` or a `.tsv` is edited. Red today: compile error on `sources_digest`.
2. Run both red, then implement.

## Steps

1. Measurement (step 0 of `## Tests first`).
2. In `synthesis_suite_bytes_table.rs`, add `const SYNTHESIS_CRATES: &[&str]` with the eight crates whose `src/` synthesis runs through: `ess-conformance`, `ess-gen`, `ess-transport`, `ess-http`, `ess-compiler`, `ess-domain`, `ess-primitives`, `ess-yaml` (their directories are found under `crates/*/<name>/src`; resolve with one directory walk of `crates/`, no hard-coded area). Verify the list before using it: `cargo tree -p ess-conformance --edges normal --prefix none` (a read, no build) must name no other `ess-*` crate that `synthesize` calls into; if one does, add it and say so in the report. Add `fn sources_digest(root: &Path) -> (String, usize, usize)` returning the hex SHA-256 (reuse `sha2`, already imported by `digest`), the crate count and the model count. The digest is over a sorted list of records, hashed one after the other:
   - `S\0<path relative to root, '/' separated>\0<byte length>\0<bytes>` for every file under each crate's `src/` (recursive, any extension),
   - `M\0<label>\0<byte length>\0<bytes>` for every file `walk` returns that holds a line starting `format: ess/` (the same test `models` applies, line 145; share the predicate by extracting it into `fn is_ess_source(text: &str) -> bool`).
   Sorting: by the whole record's path or label as a byte string; `S` records before `M` records. `Cargo.toml`, `Cargo.lock` and `.tsv` tables are not hashed (a version bump must not re-stamp; a version bump changes no synthesized byte because the tables already contain no version).
3. Stamp format, one line, newline-terminated: `#\tsources=<64 hex>\tcrates=8\tmodels=<n>`. Write mode: when `SYNTHESIS_SOURCES_STAMP_WRITE=<path>` is set the test writes that line to the path instead of comparing, as the two existing `*_TABLE_WRITE` variables do (read `table` at 258 for the idiom). Add the new test's re-pin command to the doc comment of the file (lines 28-65), next to the two others.
4. Order, because the stamp must describe a tree whose tables pass: (a) with the tree as it is, run the two table tests and `every_pinned_system_directory_is_read_as_the_cli_reads_it` and `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere` once, un-ignored, and confirm green; if one is red the tree is not clean, stop and report. (b) Write the stamp with the command from step 3. (c) Mark the four tests: `#[ignore = "slow lane: re-synthesizes every repository model and compares its suite bytes"]` on `every_repository_model_keeps_its_synthesized_bytes`, `#[ignore = "slow lane: re-synthesizes every repository component and compares its suite bytes"]` on `every_repository_component_keeps_its_synthesized_bytes`, `#[ignore = "slow lane: reads every pinned system directory as the CLI reads it and synthesizes it"]` on `every_pinned_system_directory_is_read_as_the_cli_reads_it`, `#[ignore = "slow lane: synthesizes every corpus suite"]` on `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere`. Apply the step-0 rule: omit a marker on a test the measurement put under 30 s.
5. Add the marked tests' rows to `MARKED` in `crates/edge/ess-xtask/tests/slow_lane.rs`, one row per file, sorted by path (t1 created the table; do not touch its other rows). Run `cargo test -p ess-xtask --locked --test slow_lane`.
6. Run `task test-slow PACKAGES=ess-conformance` with `SLOT_TIMEOUT=30m` once and report the wall time and each of the four times. The two table tests must be green: the slow lane compares against the committed tables exactly as before. If one is red, re-pin only the row the change claims (none is expected: this unit changes no synthesis).
7. Rewrite the doc comment at `synthesis_suite_bytes_table.rs:71-73`: the slow probe is now run by `task test-slow` (`#[ignore = "slow lane: ..."]` in `adversary_244b_window_free_bytes`), not `task test-slow-probes`.
8. `AGENTS.md` lines 206-218: say that the four byte-pin targets of the list (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`) are gated by named targets as before; that in the test step `synthesis_suite_bytes_table` compares a digest of the sources of synthesis and of every model with `tests/fixtures/synthesis-sources.digest`; that the byte comparison itself runs in the `slow` lane; and that a change under `src/` of the eight crates or to a model file re-stamps with the printed command in the same change. Leave the list lines 211-216 and the rest as they are. The package-gate table of § Gate is t7's: do not edit it.
9. Do not edit `ci.yml`, `Taskfile.yml` or `nextest.toml` (t1 owns them).

## Owned files

`crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`, `synthesis_suite_bytes_table_cli_read.rs` (marker line only), `enum_presence_guard.rs` (marker line only), the new `fixtures/synthesis-sources.digest`, and `AGENTS.md` lines 206-218.

## Not yours

- t1: `slow_lane.rs`, `Taskfile.yml`, `ci.yml`, `nextest.toml`, `AGENTS.md` 220-222. t3: `tutorial_page.rs`. t4: `generated_view_queries.rs`. t5: the three adversary files. t6: `test-xtask`. t7: `AGENTS.md` § Gate lists and the package-gate table.
- Shared: `crates/edge/ess-xtask/tests/slow_lane.rs` (`MARKED`; t1 lands first, you add rows, a merge keeps every row), `CHANGELOG.md` (one bullet each; keep both at merge), `AGENTS.md` (different paragraphs).
- The stamp file is shared in effect with every later change to `src/` of the eight crates, including the units of this wave: the integrator re-stamps once after the last merge of the wave (`SYNTHESIS_SOURCES_STAMP_WRITE=...`) and commits it in the merge commit. A unit that changes one of those crates (t5 does not; t7 does not) re-stamps in its own change.

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-conformance --locked --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked --test slow_lane
SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-conformance
slot task fmt-check
```

`ess-conformance` synthesis is touched in no way, so the "also `ess-cli`" row of the package-gate table does not apply. `task test-xtask` is not needed (no `ValidationCode`, diagnostic, format version, `docs/` or `website/` change). `ess-conformance` is run one `--test` at a time: its full test build is the largest in the workspace.

## Acceptance

1. `the_synthesis_sources_digest_matches_the_committed_stamp` is green, runs under 2 s, and fails when one byte of a `src/` file of the eight crates or of a model file changes without a re-stamp (check once by hand and record the failure message).
2. `the_sources_digest_moves_with_a_source_byte_a_model_byte_and_a_new_model_and_ignores_a_manifest` is green.
3. `cargo nextest run --profile ci -p ess-conformance --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard` ends within 30 s and runs none of the four marked tests (they print as ignored).
4. `cargo xtask slow-list` lists the four tests (or fewer, per the step-0 rule, each omission named in the report with its time) beside t1's six.
5. `task test-slow PACKAGES=ess-conformance` is green and the two table comparisons still pass against the committed tables; no table row moved.
6. `cargo test -p ess-xtask --test slow_lane` is green with the new `MARKED` rows.
7. `grep -n "test-slow-probes" crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs` finds nothing.

## Decisions

- Names (from t1, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow`; table `MARKED`.
- D1 (b), not reopened: the test step compares a digest, the slow lane compares bytes with the committed tables.
- A digest stamp, not a copy of the synthesized suites: the suites are large and the tables already hold their digests; the stamp only says whether the tables can have gone stale.
- The sources hashed are the `src/` of the eight crates and the model files; the digest excludes manifests and version bumps, so a release does not re-stamp. A source outside this set that changes synthesis is a defect of the list and is added to `SYNTHESIS_CRATES`; step 2's `cargo tree` check is how the implementor finds it today.
- The mutation unit test uses a scratch tree under `CARGO_TARGET_TMPDIR`, not a repository file and not `/tmp`.
- The tables, their `*_TABLE_WRITE` commands and the `UNAFFORDABLE` list are unchanged. `conditional-measures-generated.yaml` stays excluded and pinned by `adversary_244b_window_free_bytes`, now run by the slow lane.
- The stamp is a plain file beside the tables in `tests/fixtures/`, in the style of `adversary-244b-base-digests.tsv` (one `#` header line).
- No new dependency; `sha2` is already used by `digest`.
- Churn cost is accepted: whoever changes `src/` of these crates, or a model, re-stamps with the printed command. The integrator re-stamps at the end of the wave.

## Changelog

`### Changed`:

- CI: `synthesis_suite_bytes_table` compares a digest of the sources of synthesis and of every model with a committed stamp in the test step; the byte comparison against the committed tables, with two other whole-corpus synthesis tests, runs in the `slow` lane.
