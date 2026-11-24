---
format: aep.planning-md/3
id: story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths
kind: story
status: active
title: A text input that a guard reads, whose type's invariant refuses its base witness, gets candidates of the lengths the invariant admits, so its branch is witnessed without an `example:`.
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#809
relations:
- serves: vision:O2
- derived_from: story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T02:13:03Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T02:13:06Z", actor: "human:timo", revision: 3}
---
# d809: a guard-read text input gets a candidate its invariant admits

Part of `d809-804-the-witness-search-satisfies-every-declared-constraint`.

Size: S-M (about 180 changed lines: 40 source, 140 test).
Fixed suite bytes of committed models: likely none. No committed model or fixture has a guard-read
`String` newtype whose invariant refuses its path-text base; the alphabet-and-invariant fixtures
(`refusal-guard-outside-input-type.yaml`, `wrapped-runs-kept.yaml`,
`caller_swap_value_invariant_witness.rs`, `wrapped_value_invariants*.rs`) admit their base. Not
checked one by one: the 10 alphabet cases of `tests/adversary_e1_reachability.rs`. The byte pins
decide.
Patch-release candidate: no (synthesis; suites of affected models grow).

## Need

beyond10x/ess#809 (and the invariant half of the triage's generic feature). Where a branch's guard
reads a text input whose type is a newtype with an invariant over its length
(`value.count == 1`, `<= 1`, `value.utf8_bytes == 1`), the one candidate synthesis tries follows the
type's alphabet and breaks the invariant, and the branch is refused with ESS-SYNTH-003, although a
value meeting both exists. Every scenario whose route needs the command's creating branch is lost
with it.

## Reproduction

Reproduced on 0.98.0. Model `repro/809/` (the issue's model, `format: ess/37`): `Code` newtype of
`String`, `alphabet: "ABCDEFGHIJKLMNOPQRSTUVWXYZ"`, `invariants: [value.count == 1]`; input
`pick: Optional<Code>`; branch `unknown-pick` with
`when: {all: [defined(pick), not: {exists: {in: layout.slots, as: s, that: s.code == pick}}]}`.

```console
$ ess verify conform synthesize --path . --suite-format 5 --out suite.json
refused[ESS-SYNTH-003]: … no candidate of the 1 tried satisfies `(defined(pick) and not (exists s in layout.slots: (s.code == {fact: pick}))), within the declared input types (pick = "IBVD" meets it, but its declared type Optional<depot.racks.Code> refuses that value (invariant value.count == 1 is False); …)`
2 selected scenario(s), 0 authored source(s), 3 refusal occurrence(s)
```

With `example: Z` on the input `pick` only, a model of its own run with `--path repro/809-example`:
`5 selected scenario(s), 0 authored source(s), 0 refusal occurrence(s)`.

## Outcome

The candidate search gives a guard-read text leaf whose base its type refuses a ladder of values
the type admits (lengths the invariant allows, characters from the alphabet), beside whatever
ladder its guards give it. The reproduction synthesizes the scenarios it synthesizes with
`example: Z`, without one.

## Cause (read in the source, not run)

Paths under `crates/verify/ess-conformance/src/witness.rs`.

- `Builder::primitive` (7376-7419) takes the path text and `declared_spelling` (7365-7374) maps it
  into the alphabet (`into_alphabet`, 5170): "pick" becomes "IBVD". Invariants are only recorded
  (`Builder::declared`, 7823-7834).
- `ground_invariants` (5344, skip at 5359) moves a refused base onto an admitted value only for a
  leaf no guard reads (#511).
- `invariant_ladders` (5282) adds the `value.count` and `utf8_bytes` length ladders (5318-5326) only
  when no ladder exists for the path yet (5288). `search_with` gives `pick` a ladder first through
  `equality_copies` (1042, 5431), copied from `slots.0.code`'s base, which the type refuses too.
- `admitted_inputs` (3090, called at 1181) then drops every candidate holding the refused value;
  one survives (most likely `pick` left out), so "1 tried".

## Acceptance

1. Red today: synthesizing `repro/809/` writes the scenario
   `depot.racks.PlaceShelf/outcome/unknown-pick` with a `pick` of one letter of the alphabet that no
   slot of the sent `layout` carries, and no ESS-SYNTH-003.
2. The same with the invariant written `value.count <= 1` and `value.utf8_bytes == 1` (the issue's
   table rows 4 and 5).
3. The same with `pick` declared `Code` instead of `Optional<Code>` (red today: the plan check ran
   it on 0.98.0, "no candidate of the 0 tried").
4. A guard that compares the leaf with a literal the type admits keeps that literal among the
   candidates tried (the guard-derived ladder is extended, not replaced).
5. The two ESS-SYNTH-013 refusals of the reproduction fall away with the scenarios the creating
   branch's route gains, as they do with `example: Z`; if one stays, the report names it.
6. Synthesis time of the reproduction is measured before and after (the issue reports 12 s growing
   to 38 s on a large model with `example:`); a jump on the reproduction is fixed in this unit.
7. The byte pins under Gate keep every row; a moved row names #809 beside its re-pin.

## Scope

- `crates/verify/ess-conformance/src/witness.rs`, `invariant_ladders` (5288-5292): do not skip a
  path that already has a ladder when its base fails `admits_base` (6701). For such a path EXTEND
  the existing ladder, never replace it: today the function ends a path with `ladders.insert(…)`
  (5313-5318) and `text_length_ladder` appends, so dropping the skip alone would overwrite the
  guard-derived ladder (the `equality_copies` one). Append the invariant's alternatives and length
  ladders after the existing choices, and drop from the ladder only the values the type refuses.
  `search_with` is not changed.

## Owned files

- `crates/verify/ess-conformance/src/witness.rs` (`invariant_ladders`)
- `crates/verify/ess-conformance/tests/string_guards_beside_invariant_input.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `CHANGELOG.md`

## Not yours

- `witness::fields` and minting: `story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes`, with the d816 amendment (`d816-a-generated-value-of-a-length-constrained-newtype-is-minted`).
- The ESS-SYNTH-013 message for an input-only invariant: active
  `story:an-input-only-type-invariant-is-checked-by-a-refused-input`.
- The list-fill block of `search_with` (1134-1167): draft
  `story:a-negated-list-literal-is-witnessed-beside-any-other-leaf`.

## Tests first

In `crates/verify/ess-conformance/tests/string_guards_beside_invariant_input.rs` (existing target,
#511):

- `a_guard_read_optional_code_of_one_letter_is_witnessed` (acceptance 1).
- `a_guard_read_code_under_a_byte_or_upper_bound_invariant_is_witnessed` (acceptance 2, 3).
- `a_literal_the_guard_compares_with_stays_a_candidate` (acceptance 4).

Each synthesizes one small model: under 5 s expected (bound 60 s per test, 120 s per step). Time
them; one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test string_guards_beside_invariant_input`, then
  `--test string_alphabet_and_length`, `--test wrapped_value_invariants`,
  `--test adversary_e1_reachability`
- `cargo test -p ess-cli --locked`
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
- `task test-xtask` (the unit edits `CHANGELOG.md`)

## Depends on

Nothing. Lands before the d816 amendment of
`story:the-interpreter-mints-a-newtype-under-a-disjunction-of-prefixes` (same file).

## Changelog

Under `Fixed`:

- `ess verify conform synthesize` tries values a text input's type admits where a guard reads the
  input and the type's invariant over its length refuses the base value, so the branch is witnessed
  without an `example:`, where it was refused as ESS-SYNTH-003 naming a value its own type refuses
  (#809).

## Fit review

See the feature. Specific: class defect, cited by the refusal itself ("pick = "IBVD" meets it, but
its declared type … refuses that value"); the `example:` idiom works but must be repeated per input
and leaves the search blind; cost: suites of models refused today.

## Decisions

- Decision: accept as proposed.
