---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-list-of-references-guard-beside-other-related-guards-in-declaration-order-and-the
kind: story
status: active
title: Synthesis witnesses a list-of-references guard beside other related guards, in declaration order, and the generated targets and Entity Runtime name what they do not read
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-guard-over-the-rows-a-list-of-references-names-shares-a-command-with-other-related-guards-its-missing-row
- depends_on: story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:52:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:52:04Z", actor: "human:timo", revision: 3}
---
## Need

`g10-list-beside-related` admits a list guard beside other related guards. Synthesis arranges a list
guard in a strategy of its own, "apart from the one-row strategy of `related_guard`, which leaves it
out" (`crates/verify/ess-conformance/src/synthesize/related_list.rs:1-13`), and arranges every other
run of the command with an empty list. On a command that also names another row, neither strategy
arranges the other's rows: the list refusal's scenario would meet a missing or refusing other row
first, and the other branches' scenarios would never send a non-empty list. Without this unit the
admitted command synthesizes with refusals or with scenarios that test less than the precedence
rule says.

## Outcome

On the reproduction of `g10-list-beside-related` (`no-order`, `unknown-line`, `order-closed`,
`planned`), `ess verify conform synthesize` writes a scenario for every branch with 0 refusals and
the suite passes on the interpreted target:

- `unknown-line` is sent an order that exists in a state no refusal over it selects, and a list
  whose first element names a created line and whose second names none; the refusal publishes the
  second element;
- `order-closed` and `planned` are sent a list naming two created lines;
- `no-order` is sent a missing order and a list holding a missing line, so a target that reads the
  list first fails; where `unknown-line` is declared first, the pair is witnessed the other way;
- the order is the precedence plan's, so a target answering the other order fails a scenario.

Generated Rust and Go keep the command an owed obligation whose text names the list guard. Entity
Runtime refuses it as `RelatedGuardUnsupported`, as it refuses every `when_related:`.

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/related_list.rs` (existing binary) gains
  `a_list_guard_beside_another_related_row_is_witnessed_in_declaration_order`: one synthesis of
  the reproduction, then the interpreted run. On the base (after `g10-list-beside-related`) it
  fails with a refusal or a failing scenario; after, 0 refusals and every scenario passes. The same
  test, with an interpreted target whose plan reads the list guard first (`with_phase_order`, the
  plan's one test seam), sees the `no-order` scenario fail.
- The same for a list guard beside a row-set refusal (`when_related: {entity, where, exists:
  true}`), one test, one synthesis.
- Models without the pair keep their suite bytes: the repository-wide byte pins
  (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard` in `ess-conformance`; `go`,
  `served_codec_names` in `ess-synth`) are unchanged, and
  `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` is re-stamped by the
  task, never by hand.
- `crates/generate/ess-synth`: a case in the existing `related_guard_several_rows.rs` asserts the
  reproduction's obligation names "a guard over the rows the list `input.line_ids` names"
  (`crates/generate/ess-synth/src/determined.rs:226-235`), whichever branch is checked first.
- `crates/generate/ess-entity-runtime`: a case in an existing binary asserts the reproduction is
  refused as `RelatedGuardUnsupported` naming the command.
- Every new test under 5 s or named with its time; one synthesis per test; no new test binary.
- `website/docs/reference/predicates.md`, the list section's synthesis paragraph, says what a
  list guard beside other rows is sent.

## Fit review

Part of the feature of `g10-list-beside-related` (report item 10); its fit review applies. This unit
is the synthesis and target half. No suite major moves: no step vocabulary is added (the suite
major follows vocabulary, `.engineering/waves/wave-2026-11-24.md` § "Decision on the format").

## Decisions

None of its own. Generated Rust and Go reading a list of rows is
`story:generated-behaviour-reads-several-related-rows` (active) territory and stays out; this unit
only pins the obligation's text.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/related_list.rs` (468 lines; the strategy
  split at 1-13, `empty`)
- inferred: `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (hand the other rows'
  arrangement to the list strategy, or the list to the several-rows strategy)
- inferred: `crates/verify/ess-conformance/src/synthesize/row_set.rs` (a list beside a row set)
- inferred: `crates/verify/ess-conformance/src/synthesize/precedence.rs`
- cited: `crates/verify/ess-conformance/tests/related_list.rs`,
  `crates/generate/ess-synth/tests/related_guard_several_rows.rs` (existing binaries)
- inferred: an existing `crates/generate/ess-entity-runtime/tests/` binary (one case)
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- Size: M (about 450 changed lines).
- Overlap: s3 and s4 of the wave of 2026-11-24 change `synthesize.rs`, `synthesize/threads.rs` and
  `ess-conformance`'s `input_path_addressing.rs`; this unit lands after s4.

## Depends on

- `g10-list-beside-related`.
- `story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal`
  (s4, the last synthesis unit of the wave of 2026-11-24, shared files).

## Source

A downstream application, 2026-11-24, report item 10 (its G1).
