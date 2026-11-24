---
format: aep.planning-md/3
id: story:a-guard-read-text-input-whose-invariant-joins-a-length-rule-with-a-prefix-or-a-suffix-gets-a-candidate-the
kind: story
status: active
title: A guard-read text input whose invariant joins a length rule with a prefix or a suffix gets a candidate the invariant admits
tags:
- external-request
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#809
relations:
- serves: vision:O2
- derived_from: story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once
- depends_on: story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T03:51:57Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-11-24T03:51:57Z", actor: "human:timo", revision: 3}
---
Size: S-M. Step 12b of the one pull request, after step 12 (D84).
Fixed suite bytes of committed models: none move for a model without a text newtype whose invariant
joins a length rule with a prefix or a suffix.
Patch-release candidate: no (the hotfix for #809 is on its own branch; this is its follow-up).

## Need

Follow-up of beyond10x/ess#809, found by the review of the hotfix wave. A text newtype whose
invariant combines `min_length` or `max_length` (a length rule) with `starts_with` or `ends_with`,
or all three, and that a guard reads, gets no candidate the invariant admits. Release 0.98.0 and the
hotfix fix for #809 both refuse the branch the guard decides (`ESS-SYNTH-003`, "no candidate of the 1
tried"), where a length window alone, or a prefix alone, is witnessed. The input has values the
invariant admits, so the refusal is wrong.

## Reproduction

Two models of the review's tests, in `crates/verify/ess-conformance/tests/string_guards_beside_invariant_input.rs`
(built by that file's `one_letter` helper, a `Code` newtype read by a guard on the optional input
`pick`; generic names):

- A length window and a prefix: invariants `value.count >= 2`, `value.count <= 3` and
  `{value: {starts_with: "Q"}}`. `QA` and `QAB` satisfy all three.
- An exact length beside a prefix and a suffix: `value.count == 2`, `{value: {starts_with: "A"}}`
  and `{value: {ends_with: "Z"}}`. `AZ` is the one text.

On 0.98.0 and on the hotfix branch the scenario `depot.racks.PlaceShelf/outcome/unknown-pick` is not
written and is refused as `ESS-SYNTH-003` with `pick = "IBVD"` as the candidate tried. The controls
pass: a length window alone is witnessed, and an invariant no text satisfies (`value.count >= 3`,
`value.count <= 2`) stays a refusal and synthesis ends.

## Outcome

A guard-read text input whose invariant joins a length rule with a prefix or a suffix, or all
three, gets a candidate every conjunct of the invariant admits, so its branch is witnessed. An
invariant no text satisfies stays a refusal of the branch.

## Cause area (read, not traced)

`crates/verify/ess-conformance/src/witness.rs`: the candidate ladder the #809 fix added, in
`invariant_ladders` (about line 5247 on `hotfix/issues-2026-11-24`, commit `77b06ddf3`), with
`admitted_witness`, `compositions` and `text_length_ladder` beside it. The ladder holds values built
from one conjunct at a time (the literals and a composition of the affixes, and lengths of the
window); a value that meets the length rule and the affix together is in none of them. The
implementor reads the branch and names the exact function in the red test's commit.

## Acceptance

1. Red first: the two tests of the review whose names contain `length_window_and_a_prefix` and
   `length_a_prefix_and_a_suffix`
   (`a_guard_read_code_under_a_length_window_and_a_prefix_is_witnessed`,
   `a_guard_read_code_under_a_length_a_prefix_and_a_suffix_is_witnessed`) fail on the hotfix branch
   and pass after the change, unchanged. The review's diff of that test file holds them with two
   controls and the helper `assert_unknown_pick_written_and_passing`.
2. The two controls of that diff stay green: `a_guard_read_code_under_a_length_window_alone_is_witnessed`
   and `a_guard_read_code_no_text_satisfies_stays_refused` (an invariant no text satisfies stays a
   refusal, and synthesis ends with no hang and no panic).
3. The four `ess-conformance` byte pins do not move for a model without such an invariant
   (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
   `adversary_related_values_all_bytes`, `external_beside_held_guard`); the slow lane agrees.
4. The synthesis-sources stamp is current.
5. Each new test ends within 60 s, expected under 5 s; time them and name any over 5 s.

## Scope

- `crates/verify/ess-conformance/src/witness.rs` (the ladder named above)
- `crates/verify/ess-conformance/tests/string_guards_beside_invariant_input.rs` (the review's tests)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest`
- `CHANGELOG.md` (one bullet)

## Gate

One at a time; every test under 60 s, the test step under 120 s, no pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test string_guards_beside_invariant_input`
- the byte pins of acceptance 3, each alone
- the sources stamp, rewritten in this change:
  `SYNTHESIS_SOURCES_STAMP_WRITE="$PWD/crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest" cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table the_synthesis_sources_digest_matches_the_committed_stamp`
- `task test-xtask` (`CHANGELOG.md` changed)

## Depends on

- `story:a-text-input-that-a-guard-reads-whose-type-s-invariant-refuses-its-base-witness-gets-candidates-of-the-lengths`
  (the #809 fix whose ladder this widens)
- Derived from `story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once`.

## Changelog

Under `Fixed`:

- `ess verify conform synthesize` witnesses a branch whose guard reads a text input whose invariant
  joins a length rule with a prefix or a suffix; the branch was refused as `ESS-SYNTH-003` although
  the invariant admits values (beyond10x/ess#809).

## Decisions

- Decision: accept as a defect, built in the one pull request as step 12b, after step 12 (D84,
  2026-11-24).
