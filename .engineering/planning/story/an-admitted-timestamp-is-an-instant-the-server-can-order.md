---
format: aep.planning-md/3
id: story:an-admitted-timestamp-is-an-instant-the-server-can-order
kind: story
status: active
title: An admitted Timestamp is an instant the generated server can order
summary: Leap second and long fractions are admitted but unorderable (W1 GEN1 review, backlog)
tags:
- backlog
- defect
- review-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: review-result:adversary-w1-gen1-pass-1
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/ess-synth/src/go/http.rs
- confidence: cited
  path: crates/generate/ess-synth/src/rust/json.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs
- confidence: cited
  path: crates/generate/ess-synth/tests/json_body_closure.rs
- confidence: cited
  path: crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:36Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:37Z", actor: "human:timo", revision: 11, decided_on: {"recorded":{"review_outcome":2}}}
---
## Need

The generated decoder admits a `Timestamp` with a leap second (`23:59:60`) or a fraction past 9 digits,
as `format: date-time` does, but the tree's `rfc3339-instants` vectors and the generated `instant_of`/`instant`
grammar refuse both, so a stored value is Unknown in every Timestamp guard and byte-ordered in views.
Before W1 any text was stored; W1 narrowed the gap. Found by review-result:adversary-w1-gen1-pass-1.

Confirmed on ess 0.92.0 from generated code (no build): for a model with a network component, a
`Timestamp` input and a guard comparing a stored `Timestamp` with `now` (the committed fixture
`crates/verify/ess-conformance/tests/fixtures/now-stored-rows.yaml` plus one `reached_by: network`
component accepting its commands), `ess generate synthesize --target rust` emits:

- `crates/demo-server/src/json.rs`, `timestamp_at`/`date_time`: `second > 60` refused (so `60`
  admitted on the last minute of a UTC day) and a fraction of any number of digits; its own doc
  comment says "a leap second (`23:59:60`) and a fraction past nine digits are admitted, and a
  guard comparing either reads it as unknown".
- `crates/demo-types/src/behaviour.rs`, `instant_of`: `second > 59` and `width > 9` refused.

The generator sources are `crates/generate/ess-synth/src/rust/json.rs` (`timestamp_at`, around
line 839) and `crates/generate/ess-synth/src/rust/behaviour.rs` (`INSTANT_OF`, around line 776).
The Go equivalents are `crates/generate/ess-synth/src/go/http.rs` (`timestampAt`/`dateTime`,
`second > 60`) and `crates/generate/ess-synth/src/go/behaviour.rs` (`instant`, `second > 59`,
`width > 9`). `crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json` lists
`2020-01-01T00:00:60Z` as not valid.

## Acceptance

- `crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json` gains the `parse` rows
  `{"text": "2016-12-31T23:59:60Z", "valid": false}` and
  `{"text": "2020-01-01T00:00:00.1234567891Z", "valid": false}` (beside the existing
  `2020-01-01T00:00:60Z` row), and `cargo test -p ess-primitives --locked --test instant_comparison`
  passes over them.
- The generated Rust `timestamp_at` and the generated Go `timestampAt` refuse each of those three
  texts with a `DecodeError` at the member's path (answered 400 naming the path), as a value and
  as a map key, and admit `2016-12-31T23:59:59.123456789Z`.
- `cargo test -p ess-synth --locked --test adversary_w1_gen1_adversary` passes with the two cases
  the W1 GEN1 pass wrote (`the_timestamp_decoder_answers_the_trees_rfc3339_instant_vectors`,
  `every_timestamp_the_decoder_admits_is_an_instant_the_generated_server_can_order`) added, and
  with `the_timestamp_decoder_admits_exactly_what_the_served_date_time_format_admits` changed to
  "admits what `format: date-time` admits less a leap second and a fraction past nine digits".
- `cargo test -p ess-synth --locked --test adversary_w1_gen1_security` passes with
  `the_emitted_timestamp_reader_admits_exactly_what_format_date_time_admits` held to the same
  narrowed oracle, and `cargo test -p ess-synth --locked --test json_body_closure` passes with its
  accepted body's leap-second stamps replaced by `23:59:59` and a refused case added for
  `2016-12-31T23:59:60Z`.

## Decisions

- The decoders refuse what the instant grammar cannot read, and the instant grammar is unchanged:
  `docs/design/expression-family-source22.md` final review decision 14 already fixes that grammar
  (up to nine fraction digits, no leap second) for Rust, Go and TypeScript over one vector file.
- The served `openapi.json` keeps `type: string, format: date-time` for `Timestamp` (no added
  `pattern`): a pattern would rewrite every projected `Timestamp` schema, while the decoder's 400
  already names the path of a refused text.

## Scope

- cited: crates/generate/ess-synth/src/rust/json.rs
- cited: crates/generate/ess-synth/src/go/http.rs
- cited: crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json
- cited: crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs
- cited: crates/generate/ess-synth/tests/adversary_w1_gen1_security.rs
- cited: crates/generate/ess-synth/tests/json_body_closure.rs

confidence: medium

The story re-pins any suite-bytes or generated-output byte-pin row that its decoder change moves,
and proposes its `CHANGELOG.md` sentence.

## Schedule

Backlog (D398): not waved; considered only after approval and after every predefined wave is released.
Related: story:a-duration-member-is-refused-as-its-format-refuses (the same decoder/grammar question
for `Duration`).

## Repro

The W1 GEN1 adversary pass wrote `the_timestamp_decoder_answers_the_trees_rfc3339_instant_vectors` and
`every_timestamp_the_decoder_admits_is_an_instant_the_generated_server_can_order` in
`crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs` (over `ess-primitives/tests/vectors/rfc3339-instants.json`
and a fixed 604,800-text corpus; 7,304 admitted texts the instant grammar cannot read). They were left out of the
committed file because the item is out of W1.
