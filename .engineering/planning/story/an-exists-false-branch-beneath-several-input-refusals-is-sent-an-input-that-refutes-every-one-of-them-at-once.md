---
format: aep.planning-md/3
id: story:an-exists-false-branch-beneath-several-input-refusals-is-sent-an-input-that-refutes-every-one-of-them-at-once
kind: story
status: active
title: 'An `exists: false` branch beneath several input refusals is sent an input that refutes every one of them at once, whatever the inputs are named.'
tags:
- issues-2026-11-24
refs:
- provider: github
  reference: beyond10x/ess#804
relations:
- serves: vision:O2
- derived_from: story:the-witness-search-and-the-interpreter-s-minting-choose-values-that-satisfy-every-declared-constraint-at-once
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-24T02:31:09Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-11-24T02:31:10Z", actor: "human:timo", revision: 4}
---
# d804: a missing-row branch is sent an input outside every input refusal

Part of `d809-804-the-witness-search-satisfies-every-declared-constraint`.

Size: S (about 150 changed lines: 20 source, 130 test).
Fixed suite bytes of committed models: under DF-D2 A (default), likely none: the added goal is
tried only where `unclaimed`'s plain and per-guard candidates are all claimed, which is a refusal
today; committed fixtures with `exists: false` have at most 3 `when:` refusals
(`typed-text-operands.yaml`, `related-guard-ranked-candidates.yaml`, `now-stored-rows.yaml`,
`explore-input-beside-unknown.yaml`). Under DF-D2 B: unknown, plausibly some rows of
`synthesis-suite-bytes-table.tsv` and `adversary-285-base-digests.tsv` (`tried` counts in refusal
texts).
Patch-release candidate: no (synthesis).

## Need

beyond10x/ess#804. A command's `exists: false` branch sits beneath several input refusals over
different leaves. Synthesis must send it an input that no input refusal claims; one exists (the
same suite sends it in the default branch's scenario), but the search never asks for every refusal
refuted at once, and whether a candidate is found depends on where the inputs' names sort.

## Reproduction

Reproduced on 0.98.0. Model `repro/804/` (the issue's model, `format: ess/37`): `PostSlip` with
`not-yours` (`writer != caller.principal`), `bad-window` (`expires <= created`),
`bulk-uncarried`, `self-addressed`, then `unknown-bin`
(`when_related: {via: input.bin_id, exists: false}`), `duplicate`, `posted`.

```console
$ ess verify conform synthesize --path . --out suite.json
refused: refusal[ESS-SYNTH-003]: outcome depot.slips.PostSlip/unknown-bin has no scenario `depot.slips.PostSlip/outcome/unknown-bin`
  no candidate of the 71 tried satisfies `none of: writer != principal-262144, expires <= {fact: created} as timestamp, …`
8 scenario(s) (0 authored), 1 refusal(s), written to suite.json
```

`ess verify conform synthesize --path repro/804-from --out …` (input, field and view member
`writer` renamed `from`): `9 scenario(s) (0 authored), 0 refusal(s)`.

## Outcome

`unclaimed` asks the directed search for one more goal: every negated input refusal satisfied
together. The reproduction's `unknown-bin` scenario is written with an input that refutes all four
refusals and a `bin_id` no row carries, and the result is the same under any naming of the inputs.

## Cause (read in the source, not run)

- `without_row` (`crates/verify/ess-conformance/src/synthesize/related_guard.rs:5226`) falls back to
  `unclaimed` (5392-5448), which builds `not(g)` per input refusal (5405-5409) and asks
  `candidates(…, &goals)` (5414-5416).
- `crates/verify/ess-conformance/src/witness.rs`: `enumerate` (1958-2021) walks at most
  `MAX_CANDIDATES` (64, line 164), the first ladder in path order varying fastest; `Directed` runs
  only past 64 (1172-1177). `Directed::goals` (2290-2335) builds per guard "this one alone true",
  "true", "false", then "all false" (2309), never "all true". Over the negated refusals "all true"
  is the input outside every refusal. The issue's source reading of 0.97.0 holds on this tree.

## Acceptance

1. Red today: synthesizing `repro/804/` writes `depot.slips.PostSlip/outcome/unknown-bin` whose
   input refutes all four refusals and names a `bin_id` no row carries; no refusal.
2. The same model with `writer` renamed `from`, `zz_writer` and `a_writer` gives the same scenario
   ids and 0 refusals in each naming (the result does not depend on the name; scenario bodies spell
   the field and are not compared).
3. The suite of the reproduction passes on the interpreted target.
4. Under DF-D2 A, every model whose `unknown-*` branch is witnessed today keeps its suite bytes: the
   byte pins under Gate keep every row.

## Scope

- DF-D2 A (default): `crates/verify/ess-conformance/src/synthesize/related_guard.rs`, `unclaimed`
  (5409-5416): pass `Predicate::All(negated)` as one extra goal, tried after the existing ones.
- DF-D2 B: `crates/verify/ess-conformance/src/witness.rs`, `Directed::goals`, after line 2309:
  `goals.push(guards.iter().map(|g| (*g, true)).collect())`.

## Owned files

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`unclaimed`) under A, or
  `crates/verify/ess-conformance/src/witness.rs` (`Directed::goals`) under B
- `crates/verify/ess-conformance/tests/missing_related_row_beside_input_refusal.rs` (new `#[test]`s)
- `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (rewritten by the stamp
  command under Gate)
- `CHANGELOG.md`

## Not yours

- The default branch refused in a larger model with the same shape (the issue's "Not reproduced"):
  not reproduced, no acceptance here.
- `insert` at `unclaimed:5418`: rewritten to `set_at` by the active
  `story:synthesis-arranges-the-rows-a-when-related-guard-names-through-a-member-of-a-struct-input-and-the-interpreted`;
  a textual neighbour, whichever lands second rebases.
- #805's swapped-callers run (fixed in 0.97.1, `caller.rs`); this refusal is in the first run.

## Tests first

In `crates/verify/ess-conformance/tests/missing_related_row_beside_input_refusal.rs` (existing
target, #731):

- `a_missing_row_branch_beneath_four_refusals_is_witnessed` (acceptance 1, 3).
- `a_missing_row_branch_does_not_depend_on_input_names` (acceptance 2: three namings, one model
  text each).

Each synthesizes small models: under 5 s per test expected (bound 60 s per test, 120 s per step).
Time them; one over 5 s is named in the report.

## Gate

One `--test` at a time; no build-pool job over 10 minutes.

- `task fmt-check`
- `cargo clippy -p ess-conformance --all-targets --locked -- -D warnings`
- `cargo test -p ess-conformance --locked --test missing_related_row_beside_input_refusal`, and the
  in-crate `unclaimed_refusal_tests` (`src/synthesize.rs:23114`) with
  `SLOT_TIMEOUT=30m cargo test -p ess-conformance --locked --lib unclaimed_refusal` (the library's
  test binary compiles all of `synthesize.rs`)
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

DF-D2 (default A proceeds after 15 minutes).

## Changelog

Under `Fixed`:

- `ess verify conform synthesize` witnesses an `exists: false` branch beneath several input
  refusals over different inputs by searching for an input that refutes all of them at once, where
  it was refused as ESS-SYNTH-003 depending on how the inputs were named (#804).

## Fit review

See the feature. Specific: the name dependence is measured on 0.98.0 (`writer` refused, `from`
witnessed); the narrow form reaches only commands refused today, so it adds no suite churn.

## Decisions

- Decision: accept as proposed.
- DF-D2: A (default) the conjunction goal in `unclaimed` only; B in `Directed::goals` for every
  directed search (reaches other branches beneath several refusals, moves pinned bytes).
