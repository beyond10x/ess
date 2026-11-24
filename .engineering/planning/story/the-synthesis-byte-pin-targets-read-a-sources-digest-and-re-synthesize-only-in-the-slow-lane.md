---
format: aep.planning-md/3
id: story:the-synthesis-byte-pin-targets-read-a-sources-digest-and-re-synthesize-only-in-the-slow-lane
kind: story
status: active
title: The synthesis byte-pin targets read a sources digest and re-synthesize only in the slow lane
tags:
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
- depends_on: story:an-ess-http-binding-declares-the-largest-request-body-a-route-reads-and-the-refusal-a-larger-one-gets
- depends_on: story:a-setup-run-of-a-creating-branch-with-several-related-rows-uses-the-field-values-another-scenario-chose
- depends_on: story:a-test-marked-slow-runs-only-in-task-test-slow-under-its-own-profile-and-slow-list-names-it
- depends_on: story:the-openapi-projection-documents-the-request-bound-of-a-bounded-route-and-the-answer-a-larger-body-gets
- depends_on: story:with-more-than-one-job-the-caller-readings-of-a-synthesis-run-on-scoped-threads-and-write-the-same-suite
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
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-11-23T06:02:59Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-11-23T06:03:01Z", actor: "human:timo", revision: 15, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome

The two synthesis byte-pin targets and two more tests that synthesize a whole corpus no longer do it in the test step.

- A committed file `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` holds a SHA-256 over the sources of synthesis (the `src/` of the eight crates it runs through, and every model file the tables read). A fast test in the step recomputes it and compares it with the file; it synthesizes nothing.
- The four tests that synthesize every model are marked `#[ignore = "slow lane: ..."]` and run in `task test-slow` (and the CI `slow` job of t1b). They still compare bytes with the committed tables, as before.
- A change to synthesis or to a model that does not re-stamp the file fails the step, naming the command that writes the stamp.
- This unit lands after every unit of the wave that changes a source the stamp hashes, and stamps the tree that holds them all.

## Why

Parent: `story:the-test-step-meets-the-test-budget`, step 5, decision D1 (b): committed generated files with a source digest, compared in the step; full regeneration in the `slow` lane. Rule: "Tests stay fast" (an expensive product is made once, never per test).
Addition A names `synthesis_suite_bytes_table` (60 to 811 s by host load) and `enum_presence_guard` (about 716 s): `synthesize(` called inside the test over the whole corpus.
Consumer gap: none. Of the 13 byte-pin targets `AGENTS.md` lists, this unit moves 4 to the lane; pinning that all 13 run there is a follow-up (X8(a) on the wave page).

## Where

Read at `80794bbd698` (the files below are unchanged since `bf128f9146`). Re-read a line before editing it.

`crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs`:
- Doc comment 1-65 (the re-pin commands at 57 and 64), `header` 78, `root` 92, `digest` 99 (`sha2` over bytes, formatted `<hex>:<len>`), `walk` 109-128 (skips `target`, `node_modules`, `.git`, `.engineering`; collects `.yaml` and `.yml`), `models` 135 (keeps a file only when a line starts with `format: ess/`, line 145; a directory holding a `system.yaml` is a model of its own), `model_lines` 205, `component_lines` 230, `table` 258, `compare` 286.
- The two tests: `#[test]` 355 / `every_repository_model_keeps_its_synthesized_bytes` 356; `#[test]` 373 / `every_repository_component_keeps_its_synthesized_bytes` 374.
- Doc comment 71-73 names `task test-slow-probes`: t1a has already rewritten it when this unit starts.

`crates/verify/ess-conformance/tests/synthesis_suite_bytes_table_cli_read.rs`: `#[test]` 106 / `every_pinned_system_directory_is_read_as_the_cli_reads_it` 107.

`crates/verify/ess-conformance/tests/enum_presence_guard.rs`: `#[test]` 708 / `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere` 709; `synthesize(` at 167.

`AGENTS.md` lines 206-218: the paragraph "A gate of named test targets includes the repository-wide byte pins" and its list (211-216). Lines 220-222 are t1a's and t1b's.

`crates/edge/ess-xtask/tests/slow_lane.rs`: the `MARKED` table (t1a). `CHANGELOG.md`: the `### Changed` heading under `## [Unreleased]`.

## Tests first

Bounds as in t1a: wall-time figures are the runner's own per-test figure (`target/nextest/ci/junit.xml`), on a warm build at a 1-minute load of 60 or less (`cut -d' ' -f1 /proc/loadavg`, before and after; above 60 the run decides nothing); no test over 60 s, the test step within 120 s, a pool job ends after 10 minutes unless it names its time (`SLOT_TIMEOUT=30m`, 30 at most); no Python for anything that stays. Time every new test; name any over 5 s.

0. Measure first, before any edit. `SLOT_TIMEOUT=30m cargo nextest run --profile ci -p ess-conformance --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard`; read each test's time from `target/nextest/ci/junit.xml`. Decision rule, one threshold: a test of the four over 30 s is marked (step 4); a test under 30 s stays in the step with its time in the report (no marker, no `MARKED` row). The fast tests below exist either way.
1. In `synthesis_suite_bytes_table.rs`, new tests (each under 2 s):
   - `the_synthesis_sources_digest_matches_the_committed_stamp`: computes `sources_digest(&root())` (step 2), reads `tests/fixtures/synthesis-sources.digest`, compares. The failure message names the stored and computed values, the crate and model counts, and the command `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`. Red today: compile error naming `sources_digest`, and no stamp file.
   - `the_sources_digest_moves_with_a_source_byte_a_model_byte_and_a_new_model_and_ignores_a_manifest`: builds a scratch tree under `env!("CARGO_TARGET_TMPDIR")` (a fixed directory name, deleted first) with two crate `src/` directories and one model file holding `format: ess/1` on a line; the digest changes when one byte of a source changes, when one byte of the model changes, when a second model file is added, and when a source file is renamed; it does not change when a `Cargo.toml` or a `.tsv` is edited. Red today: compile error on `sources_digest`.
2. Run both red, then implement.

## Steps

1. Measure (step 0 above).
2. In `synthesis_suite_bytes_table.rs` add `const SYNTHESIS_CRATES: &[&str]` with the eight crates whose `src/` synthesis runs through: `ess-conformance`, `ess-gen`, `ess-transport`, `ess-http`, `ess-compiler`, `ess-domain`, `ess-primitives`, `ess-yaml` (directories found by one walk of `crates/`, no hard-coded area). The list was checked against the manifests at `80794bbd698`: `ess-conformance` depends on `ess-primitives`, `ess-compiler`, `ess-domain`, `ess-gen` and `ess-yaml`; `ess-gen` on `ess-transport` and `ess-http`; `ess-synth` depends on `ess-conformance`, not the other way, so `ess-synth` is not hashed; neither are `ess-composition`, `ess-cli`, `ess-xtask` or `ess-ui`. Confirm once with `cargo tree -p ess-conformance --edges normal --prefix none` (a read of the graph, no build) and add any other `ess-*` crate it names, with a line in the report. Add `fn sources_digest(root: &Path) -> (String, usize, usize)` returning the hex SHA-256, the crate count and the model count (reuse `sha2`, already imported by `digest`). The digest is over a sorted list of records, hashed one after the other: `S\0<path relative to root, '/' separated>\0<byte length>\0<bytes>` for every file under each crate's `src/` (recursive, any extension), and `M\0<label>\0<byte length>\0<bytes>` for every file `walk` returns that holds a line starting `format: ess/` (extract the predicate of line 145 into `fn is_ess_source(text: &str) -> bool` and share it). Sorting: by path or label as a byte string; `S` records before `M` records. `Cargo.toml`, `Cargo.lock` and `.tsv` tables are not hashed (a version bump re-stamps nothing).
3. Stamp format, one line, newline-terminated: `#\tsources=<64 hex>\tcrates=8\tmodels=<n>`. Write mode: when `SYNTHESIS_SOURCES_STAMP_WRITE=<path>` is set the test writes that line to the path instead of comparing, as the two existing `*_TABLE_WRITE` variables do (read `table` at 258 for the idiom). Add the command to the doc comment of the file (lines 28-65).
4. Order, because the stamp describes a tree whose tables pass: (a) with the tree as it is (every unit of the wave that touches a hashed source has landed), run the two table tests, `every_pinned_system_directory_is_read_as_the_cli_reads_it` and `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere` once, un-ignored, and confirm green; if one is red, a unit left a table unpinned: stop and report which row and which unit. (b) Write the stamp with the command of step 3. (c) Mark the tests the step-0 rule names, among: `#[ignore = "slow lane: re-synthesizes every repository model and compares its suite bytes"]` on `every_repository_model_keeps_its_synthesized_bytes`, `#[ignore = "slow lane: re-synthesizes every repository component and compares its suite bytes"]` on `every_repository_component_keeps_its_synthesized_bytes`, `#[ignore = "slow lane: reads every pinned system directory as the CLI reads it and synthesizes it"]` on `every_pinned_system_directory_is_read_as_the_cli_reads_it`, `#[ignore = "slow lane: synthesizes every corpus suite"]` on `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere`.
5. The two table tests are ignored now, so their re-pin commands (the doc comment lines 55-66 and the `repin` strings passed to `header` and `compare`) must run them: append `-- --ignored <test name>` to each printed `cargo test ... --test synthesis_suite_bytes_table` command. Check that the printed command writes the table (run it once into a scratch path).
6. Add one `MARKED` row per marked file in `crates/edge/ess-xtask/tests/slow_lane.rs`, sorted by path. Run `cargo test -p ess-xtask --locked --test slow_lane`.
7. `SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-conformance` once; report each marked test's time. The table comparisons must pass against the committed tables; no row moved by this unit.
8. `AGENTS.md` lines 206-218: say that the four byte-pin targets of the list (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`) are gated by named targets as before; that in the test step `synthesis_suite_bytes_table` compares a digest of the sources of synthesis and of every model with `tests/fixtures/synthesis-sources.digest`; that the byte comparison itself runs in the slow lane; and that a change under `src/` of the eight crates or to a model file re-stamps with the printed command in the same change. Leave the list 211-216 and every other paragraph. The package-gate table of § Gate is t7's: do not edit it.
9. Do not edit `ci.yml`, `Taskfile.yml`, `nextest.toml`.

## Owned files

`crates/verify/ess-conformance/tests/synthesis_suite_bytes_table.rs` (all but lines 71-73, which t1a edited), `synthesis_suite_bytes_table_cli_read.rs` (the marker line), `enum_presence_guard.rs` (the marker line), the new `fixtures/synthesis-sources.digest`, `crates/edge/ess-xtask/tests/slow_lane.rs` (the rows you add), `AGENTS.md` lines 206-218, `CHANGELOG.md` (one bullet).

## Not yours

- t1a: the machinery and `MARKED`'s first rows. t1b, t3, t4, t5, t6, t7: their files. No other unit touches the stamp file, and no other unit's Gate runs `the_synthesis_sources_digest_matches_the_committed_stamp` (it does not exist before this unit).
- The stamp rule: this unit lands after p3 (the thread units' last source change), c1 (new model fixtures and `related_guard.rs`), h1 (`ess-http`) and h4 (`ess-gen`), and stamps the tree that holds them. It does not depend on h2 and h3 (their changes are under `crates/generate/ess-synth/` and in `.yaml.in` fixtures, which neither the `src/` list nor the model walk reads) nor on the writer units (`ess-composition`). A unit that lands after this one and changes a hashed source or a model file re-stamps with the printed command in its own change; none is expected (p4, p5, w3, t7 touch no hashed source). The sentence "the integrator re-stamps once" is not used.
- Shared: `slow_lane.rs` `MARKED` (t1a first; t3, t4, t5 add rows; keep every row at merge), `CHANGELOG.md` (keep every bullet), `AGENTS.md` (different paragraphs).

## Gate

```console
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 SLOT_TIMEOUT=30m cargo clippy -p ess-conformance --locked --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard -- -D warnings
CARGO_PROFILE_DEV_DEBUG=false CARGO_INCREMENTAL=0 cargo test -p ess-xtask --locked --test slow_lane
SLOT_TIMEOUT=30m task test-slow PACKAGES=ess-conformance
slot task fmt-check
```

No synthesis code is touched, so the "also `ess-cli`" row does not apply. `task test-xtask` is not needed (no `ValidationCode`, diagnostic, format version, `docs/` or `website/` change). `ess-conformance` runs one `--test` at a time.

## Acceptance

1. `the_synthesis_sources_digest_matches_the_committed_stamp` is green, and fails when one byte of a `src/` file of the eight crates or of a model file changes without a re-stamp (check once by hand and record the failure message). It runs without any synthesis: its own time is under 2 s on a warm build at a 1-minute load of 60 or less.
2. `the_sources_digest_moves_with_a_source_byte_a_model_byte_and_a_new_model_and_ignores_a_manifest` is green.
3. Re-stamp, one owner: the stamp file in the commit equals the output of the command of step 3 run on the final tree of this unit (run it once more after the last edit and `git diff --exit-code` the file); no other acceptance re-stamps anything.
4. The two table tests, `every_pinned_system_directory_is_read_as_the_cli_reads_it` and `no_corpus_suite_requires_a_branch_the_precedence_takes_elsewhere` are green un-ignored before the stamp is written (step 4a), and green in `task test-slow PACKAGES=ess-conformance` after; no table row moved.
5. `cargo nextest run --profile ci -p ess-conformance --test synthesis_suite_bytes_table --test synthesis_suite_bytes_table_cli_read --test enum_presence_guard` runs none of the marked tests (they print as ignored) and every remaining test of the three targets ends within 30 s by the runner's figure, on a warm build at a 1-minute load of 60 or less; each test left in the step by the step-0 rule is named with its time.
6. The printed table re-pin commands write their tables (step 5; checked once into a scratch path).
7. `cargo xtask slow-list` lists the marked tests beside t1a's; `cargo test -p ess-xtask --test slow_lane` is green with the rows.

## Decisions

- Names (t1a, repeated): marker `#[ignore = "slow lane: <reason>"]`; `cargo xtask slow-list`; `task test-slow` with `PACKAGES`; profile `slow`; job `slow` (t1b); `MARKED`.
- D1 (b), not reopened: the step compares a digest, the slow lane compares bytes with the committed tables. A skipped slow lane never counts for a release (t1b), so the byte comparison runs in every run that can count.
- A digest stamp, not a copy of the synthesized suites: the tables already hold their digests; the stamp only says whether they can have gone stale.
- The crate list is eight crates; `ess-synth` is not in it (decided from the manifests, so h2 and h3 are not predecessors). The digest excludes manifests and version bumps.
- One rule for the stamp (coordinator X1): this unit lands last among the units that change hashed sources and stamps the tree that holds them; a later unit that changes one re-stamps in its own change.
- The mutation unit test uses a scratch tree under `CARGO_TARGET_TMPDIR`, not a repository file and not `/tmp`.
- The `UNAFFORDABLE` list stays: `conditional-measures-generated.yaml` is pinned by `adversary_244b_window_free_bytes`, now run by the slow lane (t1a).
- No new dependency (`sha2` is already used).

## Changelog

`### Changed`:

- CI: `synthesis_suite_bytes_table` compares a digest of the sources of synthesis and of every model with a committed stamp in the test step; the byte comparison against the committed tables, with two other whole-corpus synthesis tests, runs in the slow lane (story:the-test-step-meets-the-test-budget).
