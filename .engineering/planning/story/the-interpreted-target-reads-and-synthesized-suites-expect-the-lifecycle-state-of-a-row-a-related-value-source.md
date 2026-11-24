---
format: aep.planning-md/3
id: story:the-interpreted-target-reads-and-synthesized-suites-expect-the-lifecycle-state-of-a-row-a-related-value-source
kind: story
status: active
title: The interpreted target reads, and synthesized suites expect, the lifecycle state of a row a related value source selects with `where:` or as the head of an ordered selection.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#822
relations:
- serves: vision:O2
- depends_on: story:a-related-value-source-that-selects-its-row-with-where-may-read-the-row-s-lifecycle-state-with-field-state-as
- decomposes: epic:generic-row-queries
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:25:05Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:25:07Z", actor: "human:timo", revision: 3}
---
# d822 part 2: a where-selected state is read by the interpreter and synthesis

format: follows part 1 (`ess/38`, DF-D5 A and D77). This unit adds no format of its own.

Part 2 of 2 for beyond10x/ess#822: the interpreted target answers the read part 1 admits, and
synthesis writes the scenario that expects it. Part 1,
`d822-a-where-selected-related-value-reads-the-rows-state`, gave every consumer an arm; this unit
replaces the interpreter's and synthesis's arms (unsupported / obligation) with the read.

Size: M (about 330 changed lines: 40 interpreter, 110 synthesis, 180 test).
Fixed suite bytes of committed models: none (only a model at the chosen format using the read is
affected, and none is committed).
Patch-release candidate: no.

## Need

beyond10x/ess#822: see part 1. Admitted but not read, the construct leaves the event's `state`
unchecked by the suite and the interpreted target unsupported.

## Reproduction

`repro/822/` with `format:` raised to the chosen format, after part 1: synthesis records `Join` an
obligation and the interpreted target answers `joined` unsupported (by part 1's arms). On 0.98.0 the
model is refused at validation (part 1's reproduction).

## Outcome

The interpreted target reads the selected row's state before the outcome, as it reads the `via:`
form's state (`interpret/execute/determined.rs:85-120`) and a selected field
(`interpret/execute.rs:2632-2637`, `row_set::Reader::one`, `determined::selected` at
`determined.rs:666`). Synthesis arranges the selected row in a state and expects that state in the
event, as it expects a selected field.

## Where it lands (read in the source, not run)

Lines at `41dd4aa9e`; part 1 adds the variants these arms match.

- Interpreter: `crates/verify/ess-conformance/src/interpret/execute.rs` (2632-2637) and
  `interpret/execute/determined.rs` (85-120, 666).
- Synthesis: `synthesize/row_set.rs` (103, 295, 1098, 3759, 4968), `synthesize.rs` (11626, 11715,
  13042, 13946, 14510), `synthesize/related.rs` (113), `witness.rs` (1772, 1892): the arms that
  decide a selected field's value, extended to the state.

## Acceptance

1. Red after part 1: synthesis of the reproduction (format raised) writes a `joined` scenario whose
   `Joined` event expects the state of the member the scenario arranged (`Pending`, and `Active`
   where a scenario arranges an active member), and no obligation for `Join`.
2. That suite passes on the interpreted target; an interpreted variant that answers the member's
   state as `Active` always fails the `Pending` scenario.
3. Several rows and no row are answered by the interpreter as for `field: member_id` of the same
   form.
4. Every committed suite keeps its bytes (the byte pins).

## Scope

The interpreter arm and the synthesis arms listed above; nothing outside `ess-conformance`.

## Owned files

- `crates/verify/ess-conformance/src/interpret/execute.rs`, `interpret/execute/determined.rs`
- `crates/verify/ess-conformance/src/synthesize.rs`, `synthesize/row_set.rs`,
  `synthesize/related.rs`, `witness.rs` (the arms part 1 placed, at the sites listed)
- `crates/verify/ess-conformance/tests/a_command_with_two_related_selectors_arranges_both.rs`
  (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `CHANGELOG.md` (completes part 1's bullet: "read by the interpreted target and expected by
  synthesized suites")

## Not yours

- Validation, compiler, IR, the arms outside `ess-conformance`: part 1.
- The other units editing `synthesize.rs` and `witness.rs` in the same pull request (d795, d794,
  d799 suite, d809, the d816 amendment, d819 part 2): different functions; land in the order of
  `MAP.md`, the later rebases.

## Tests first

`crates/verify/ess-conformance/tests/a_command_with_two_related_selectors_arranges_both.rs`
(existing): `a_where_selected_state_is_carried_by_the_event` (acceptance 1, 2) and
`a_where_selected_state_with_no_row_is_answered_as_a_field` (acceptance 3). Each under 5 s expected
(bound 60 s per test, 120 s per step); one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`; `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test a_command_with_two_related_selectors_arranges_both`
- `cargo test -p ess-cli --locked`
- `task test-xtask` (the unit edits `CHANGELOG.md`)
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

Part 1 (the variants and the arms this unit replaces).

## Changelog

Completes part 1's `Added` bullet (one bullet for #822).

## Fit review

See part 1. Specific: no surface of its own; the read part 1 admits, answered by the paths the
`via:` state read and a selected field already take.

## Decisions

- Decision: accept, part 2 of the split.
