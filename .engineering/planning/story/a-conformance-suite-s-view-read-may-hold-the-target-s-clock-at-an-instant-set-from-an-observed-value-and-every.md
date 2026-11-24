---
format: aep.planning-md/3
id: story:a-conformance-suite-s-view-read-may-hold-the-target-s-clock-at-an-instant-set-from-an-observed-value-and-every
kind: story
status: active
title: A conformance suite's view read may hold the target's clock at an instant set from an observed value, and every runner and the interpreted target execute it.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#819
relations:
- serves: vision:O2
- depends_on: story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:09Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:10Z", actor: "human:timo", revision: 4}
---
# d819 part 1: a view read step holds the read clock

format: a suite format, at the suite pair the one pull request opens once (DF-D6 A and D77); no source format.

Part 1 of 2 for beyond10x/ess#819, split by layer: this unit gives the suite a view read whose
clock is held and makes every runner execute it; part 2,
`d819-a-view-ordering-a-minted-timestamp-against-now-is-read`, makes synthesis write it. Nothing
writes the new step until part 2 lands, so this unit moves no suite byte.

Size: M (about 380 changed lines: 70 suite model and Rust runner, 40 interpreter, 60 Go runner, 50
TypeScript runner, 30 docs, 130 test).
Fixed suite bytes of committed models: none (no synthesis change).
Patch-release candidate: no (suite format).

## Suite version

The step is written as `ess-conformance/66` (ordinary) and `/67` (coverage), opened by story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet; `/64` and `/65` are written since 0.95.0, and `story:conformance-holds-missing-keeps-and-null-clears` takes `/66` and `/67` too.

## Need

beyond10x/ess#819: see part 2. The suite holds the target's clock only right before an
`execute_command` step (`hold_now`, `runner.rs:2496`; `story:a-suite-fixes-the-targets-clock`,
implemented, planned a request clock for views and built only command steps). A view whose filter
orders a field against `now` can only be checked on both sides of a value if the read's clock is
held too.

## Outcome (DF-D6 A)

An `expect_view` step may carry a held clock, an instant written as an offset from an observed
value (`ScenarioValue::Observed {event, field}`, `scenario.rs:1965-1970`) or from the scenario's
held instant. The Rust runner holds the target's clock at that instant before the read, through the
existing `hold_now` path; the interpreted target answers the read at that instant; the Go and
TypeScript runners do the same. A suite without the field keeps its version and bytes.

## Acceptance

Suite pair (D77, 2026-11-24): this story's suite vocabulary is written at `ess-conformance/66` (ordinary) and `/67` (coverage), opened by story:the-suite-pair-ess-conformance-66-and-67-is-open-read-wherever-64-and-65-are-and-written-by-nothing-yet; where this text names `ess-conformance/64` and `/65`, "the new suite pair", "the next suite pair" or a pair "after `/63`" as the pair this story writes, read `/66` and `/67` (`/64` and `/65` opened in 0.93.0 and are written since 0.95.0).

1. A suite written by the test with an `expect_view` that holds the clock 1 s past an observed
   `due_at` and a second that holds it 1 s before passes on the interpreted target over the part-2
   reproduction model (`repro/819/`), and fails on an interpreted variant whose filter ignores
   `due_at` (the read checks the rule).
2. The same suite runs under the Go and TypeScript runners with the same verdicts, or each refuses
   the step by name; neither ignores the hold.
3. A suite with the held clock at `ess-conformance/62` is refused naming the version that carries
   it (`/66`).
4. Every committed suite keeps its version and bytes (the byte pins).
5. `website/docs/reference/formats.md` (suite rows) names the step and the version.

## Scope

- `crates/verify/ess-conformance/src/scenario.rs`: the held clock on `ExpectView`, read and written
  only from the suite version above.
- `crates/verify/ess-conformance/src/runner.rs`: hold the clock before the read (`hold_now` 2496,
  the observed value resolved as at 3426-3436).
- `crates/verify/ess-conformance/src/interpret/views.rs` (the read answers at the held instant) and
  the interpreted target's `hold_now` (`target.rs:418`, `process.rs:954`).
- `crates/verify/ess-conformance/src/go/mod.rs`, `src/ts/runtime.ts`: the same step.
- `website/docs/reference/formats.md`, `CHANGELOG.md`.

## Owned files

- `crates/verify/ess-conformance/src/scenario.rs` (`ExpectView`)
- `crates/verify/ess-conformance/src/runner.rs` (the view read step)
- `crates/verify/ess-conformance/src/interpret/views.rs` (the read's instant)
- `crates/verify/ess-conformance/src/go/mod.rs`, `crates/verify/ess-conformance/src/ts/runtime.ts`
- `crates/verify/ess-conformance/tests/now_view_filter.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `website/docs/reference/formats.md`, `CHANGELOG.md`

## Not yours

- Synthesis writing the step, the ESS-SYNTH-005 hint: part 2.
- The `/64`/`/65` claims of a missing member and a `null` clear:
  `story:conformance-holds-missing-keeps-and-null-clears` (shares the pair, as above).
- `interpret/views.rs` row builder: also edited by
  `d799-a-cleared-view-member-is-claimed-and-published-in-its-declared-presence` (449-458, a
  different block); whichever lands second rebases.

## Tests first

In `crates/verify/ess-conformance/tests/now_view_filter.rs` (existing):
`a_view_read_with_a_held_clock_reads_both_sides_of_an_observed_instant` (acceptance 1),
`a_held_view_clock_below_its_suite_version_is_refused` (acceptance 3). The Go and TypeScript cases
go beside the existing runner cases of `tests/current_time_guard.rs`, skipped with a printed reason
where `go` or `node` is not on `PATH`, as they are today. Each under 5 s expected (bound 60 s per
test, 120 s per step); one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`; `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test now_view_filter`, then `--test current_time_guard`
- `cargo test -p ess-cli --locked` (`epic_formats_open`, `coverage_cli` pin the suite pair)
- `task test-xtask` (format version, docs, `CHANGELOG.md`)
- byte pins: `ess-conformance` `--test synthesis_suite_bytes_table`,
  `--test synthesis_suite_bytes_table_cli_read`, `--test adversary_related_values_all_bytes`,
  `--test external_beside_held_guard`; `ess-synth` `--test served_codec_names`, `--test go`;
  `ess-openapi` `--test adversary_ess_issues_2_u6`; `ess-compiler` `--test
  string_alphabet_and_length`; `ess-cli` `--test tutorial_page`
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`;
  re-run after every rebase, never merge the stamp by hand
- the byte comparison, once per unit, its measured time named in the report:
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table -- --ignored every_repository_model_keeps_its_synthesized_bytes`

## Depends on

DF-D6 (A). The suite pair as above (shared with
`story:conformance-holds-missing-keeps-and-null-clears`, or opened by this unit).

## Changelog

Under `Added`:

- A conformance suite from `ess-conformance/64` may hold the target's clock for a view read, at an
  instant set from a value an earlier step observed; the Rust, Go and TypeScript runners and the
  interpreted target execute it (#819).

## Fit review

See part 2. Specific: the cell `story:a-suite-fixes-the-targets-clock` planned (a request clock for
views) and did not build; reuses `hold_now` and `Observed`.

## Decisions

- Decision: accept, redesigned (part 1 of the requester's first suggestion, in suite form).
- DF-D6: default A; under B or C this unit is not built. Taken: A (D77).
