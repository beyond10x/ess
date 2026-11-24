---
format: aep.planning-md/3
id: story:the-interpreted-target-mints-a-generated-true-value-of-a-text-newtype-whose-invariant-fixes-its-length-so
kind: story
status: archived
title: 'The interpreted target mints a `{generated: true}` value of a text newtype whose invariant fixes its length, so scenarios that create such a row run instead of ending unsupported.'
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#816
relations:
- serves: vision:O2
- derived_from: story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once
- depends_on: story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths
revision: 2
transitions:
- {from: "draft", to: "archived", at: "2026-11-24T02:30:36Z", actor: "human:timo", revision: 2}
---
# d816: a generated value of a length-constrained newtype is minted

Part of `d809-804-the-witness-search-satisfies-every-declared-constraint` (#816 item 4 only; the
other items of #816 belong elsewhere, see the triage).

Size: S (about 120 changed lines: 25 source, 95 test).
Fixed suite bytes of committed models: none expected. `witness::fields` returns `WitnessGap` for
such a field today, so a model whose suite holds such a value cannot exist; synthesis callers of
`fields` (event payload and delivery-context bases) change only where they fail today. The byte
pins decide.
Patch-release candidate: no (interpreter and synthesis base values; small, but it changes what the
interpreted target answers).

## Need

beyond10x/ess#816, item 4. A `String` newtype with an invariant fixing its length
(`value.count == 52`) validates, but the interpreted target cannot mint a value of it for
`{generated: true}`: every scenario that creates such a row ends `unsupported` (40 of 123 in the
reporter's suite). The reporter moved the length into a comment to get the scenarios back.

## Reproduction

Reproduced on 0.98.0. Model `repro/816-4/` (`format: ess/37`): `NoteId` newtype of `String`,
`invariants: [value.count == 52]`, the identity of `Note`; `WriteNote` creates a `Note` with
`note_id: {generated: true}` in its event.

```console
$ ess verify conform synthesize --path . --out suite.json
2 scenario(s) (0 authored), 0 refusal(s), written to suite.json
$ ess verify conform run --path . --suite suite.json --target interpreted --report-format 2 --report-out report.json
  unsupported depot.notes.WriteNote/outcome/written
      ESS-CF-TARGET: cannot expose invoking `depot.notes.WriteNote`: the model leaves a value of `depot.notes.NoteId` to the implementation, and its type has no shape the interpreter can mint
  2 scenarios: 0 passed, 0 failed, 0 error, 2 unsupported
```

## Outcome

`witness::fields` grounds a field whose base its type refuses onto the first value of the field's
invariant ladder the type admits, as `ground_invariants` does for a command input no guard reads
(#511). `interpret::values::mint`, which reads `fields` at a further distinction per tick, then
mints a 52-character value, distinct per tick, and the reproduction's two scenarios pass.

## Cause (read in the source, not run)

- `crates/verify/ess-conformance/src/interpret/values.rs`, `mint` (60-93): for anything but a bare
  `Uuid` it calls `crate::witness::fields(…, Distinction::further(tick))` and maps its error to
  `Undetermined::NoValue`.
- `crates/verify/ess-conformance/src/witness.rs`, `fields` (1203-1229): `builder.member` builds the
  path-text base and the field is refused at 1219-1224 ("has no base value its declared invariants
  admit"); no invariant ladder is consulted. `invariant_ladders` (5282) and `ground_invariants`
  (5344) are reached only from the command search (`search_with`, line 927).

## Acceptance

1. Red today: the suite of `repro/816-4/` passes on the interpreted target, 2 scenarios, 0
   unsupported.
2. Two `WriteNote` in one scenario mint two different `note_id`s, each of 52 characters.
3. The same with an alphabet beside the invariant (`alphabet: "0123456789abcdef"`) mints only
   characters of the alphabet.
4. A type whose invariants no ladder value meets (e.g. `value.count == 52` and
   `value.count == 53`) still ends `unsupported` with the same message.
5. The byte pins under Gate keep every row.

## Scope

- `crates/verify/ess-conformance/src/witness.rs`, `fields` (1203-1229): before refusing at 1219,
  build the field's invariant ladder (`invariant_ladders` over a builder holding this one field) and
  take its first admitted value at the distinction; refuse only if none is admitted.

## Owned files

- `crates/verify/ess-conformance/src/witness.rs` (`fields`)
- `crates/verify/ess-conformance/tests/wrapped_value_invariants.rs` (new `#[test]`s)
- `CHANGELOG.md`

## Not yours

- `invariant_ladders` itself (d809, lands first; reuse what it leaves).
- Generated Rust and Go minting (`ess-synth`): not reported; if they mint such values wrongly, a
  story of its own.
- #816 items 1-3 and 5-7 (triage: #554, #796's neighbour, D7, a design, #564).

## Tests first

In `crates/verify/ess-conformance/tests/wrapped_value_invariants.rs` (existing target):

- `a_generated_fixed_length_identity_is_minted_and_passes` (acceptance 1, 2).
- `a_generated_fixed_length_value_keeps_its_alphabet` (acceptance 3).
- `an_unsatisfiable_generated_value_stays_unsupported` (acceptance 4).

Each synthesizes and runs one small model on the interpreted target: under 5 s expected (bound 60 s
per test, 120 s per step).

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test wrapped_value_invariants`, then
  `--test string_alphabet_and_length`
- `cargo test -p ess-cli --locked`
- byte pins as in d809 (synthesis: `synthesis_suite_bytes_table`,
  `synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`,
  `external_beside_held_guard`; `ess-synth` `served_codec_names`, `go`; `ess-openapi`
  `adversary_ess_issues_2_u6`; `ess-compiler` `string_alphabet_and_length`; `ess-cli`
  `tutorial_page`; `task test-slow`).

## Depends on

d809 (same file; land after it).

## Changelog

Under `Fixed`:

- The interpreted target mints a `{generated: true}` value of a text newtype whose invariant fixes
  its length, where every scenario creating such a value ended `unsupported` (#816).

## Fit review

See the feature. Class: defect; the reporter's workaround (moving the length into a comment) drops
a checked figure. Cost: interpreter answers for a shape that answers nothing today.

## Decisions

- Decision: accept as proposed.
