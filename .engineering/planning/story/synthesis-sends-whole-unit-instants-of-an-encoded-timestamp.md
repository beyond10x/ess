---
format: aep.planning-md/3
id: story:synthesis-sends-whole-unit-instants-of-an-encoded-timestamp
kind: story
status: active
title: Synthesis sends whole-unit instants of an encoded Timestamp
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#758
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-timestamp-newtype-declares-its-wire-encoding
- depends_on: story:a-value-outside-its-declared-type-is-answered-malformed
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/values.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/timestamp_encoding.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:46:35Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:46:37Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

An encoded `Timestamp` newtype (`story:a-timestamp-newtype-declares-its-wire-encoding`) holds whole
seconds (`unix_seconds`) or whole milliseconds (`unix_millis`). Synthesis must send such values
only, and the interpreted target must hold the type to them, or a suite would send an instant the
wire cannot carry. The ladders around a time boundary already step by one second
(`docs/design/generic-language-features.md`, synthesis cost table, row "time";
`crates/verify/ess-conformance/src/witness.rs`), but a `now_offset` value or a fixture instant with
a fraction would break `unix_seconds`.

## Acceptance

The reproduction model is the parent story's: entity `example.calls.Call` (`created_at:
Timestamp`) and a view filtered `created_at >= param.created_from and created_at <
param.created_to`, both parameters of `example.calls.EpochInstant` (`kind: newtype, of: Timestamp,
encoding: unix_seconds`). Tests run with `cargo test -p ess-conformance --locked --test
timestamp_encoding`.

- `every_encoded_instant_is_a_whole_second`: every value of `example.calls.EpochInstant` in the
  suite `ess verify conform synthesize` writes for the model has no fractional second.
- `the_filter_boundaries_keep_a_row_on_each_side`: in the same suite the filter's boundary scenarios
  hold a row with `created_at` equal to `created_from`, which the view returns, and a row with
  `created_at` equal to `created_to`, which it does not.
- `the_suite_passes_the_interpreter`: `ess verify conform run --target interpreted` on that suite
  passes every scenario.
- `a_fractional_instant_is_answered_malformed`: the interpreted target sent `created_from =
  "2026-01-01T00:00:00.5Z"` answers `TargetError::Malformed` (story:a-value-outside-its-declared-type-is-answered-malformed),
  its path naming `created_from` and its constraint naming `unix_seconds`, and runs no branch.
- `a_fractional_authored_literal_is_refused`: an authored scenario literal of
  `example.calls.EpochInstant` with a fractional second is refused as `ESS-AUTHOR-015`, the text
  naming `unix_seconds`.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes with no moved
  row: no pinned model declares an encoded `Timestamp`.

## Scope

- cited: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/interpret/values.rs
- inferred: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/tests/timestamp_encoding.rs
confidence: low

Shared surfaces: proposes its `CHANGELOG.md` sentence; moves no suite-bytes row.

## Fit review

1. **Need.** The suite sends only values the declared wire form can carry. Requester's syntax
   (theirs): "and into synthesized reads".
2. **Class.** Gap, following the parent story.
3. **Already expressible?** No (parent story).
4. **Fit.** A type's constraints already bound what synthesis sends and what an authored literal
   may be (`ESS-AUTHOR-015`, `website/docs/reference/formats.md`, `ess-scenario/6`). The whole-unit
   rule is one more such constraint.
5. **Second adopter.** The webhook event of the parent story, whose binding condition compares a
   `unix_millis` instant with a stored one.
6. **Cost.** No suite format: values stay `Timestamp` literals. Suites of models without the key
   keep their bytes.
7. **Alternatives.** (a) Round at the adapter: refused, a rounded value is not the value the
   scenario claims. (b) Generate whole units only: chosen.

## Decisions

accept, redesigned, as the parent story.

2026-11-10: the interpreted target answers a fractional instant `malformed` only, after
story:a-value-outside-its-declared-type-is-answered-malformed (now a `depends_on` edge), because
an `ESS-CF-TARGET` error in between would be a second answer for one request that a later story
then replaces.

## Source

github:beyond10x/ess#758
