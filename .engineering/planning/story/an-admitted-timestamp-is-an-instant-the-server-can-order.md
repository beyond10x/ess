---
format: aep.planning-md/3
id: story:an-admitted-timestamp-is-an-instant-the-server-can-order
kind: story
status: draft
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
revision: 4
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
`crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json` lists `2020-01-01T00:00:60Z`
as not valid.

## Acceptance

- One RFC 3339 grammar is chosen for leap seconds and fractions past nine digits, written in the
  design page the `rfc3339-instants` vectors cite, and the vectors state it for
  `2016-12-31T23:59:60Z`, `2020-01-01T00:00:60Z` and `2020-01-01T00:00:00.1234567891Z`.
- The generated Rust decoder (`timestamp_at`), the generated `instant_of`, the Go equivalents and
  the served `openapi.json` agree on those vectors: every text the decoder admits, `instant_of`
  reads as an instant. Either the decoder refuses what the instant grammar cannot read (answered
  400 naming the path), or the instant grammar reads it (a leap second as the following instant,
  extra fraction digits truncated or rounded as the design says).
- `crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs` gains the two red cases the W1
  GEN1 pass wrote (`the_timestamp_decoder_answers_the_trees_rfc3339_instant_vectors`,
  `every_timestamp_the_decoder_admits_is_an_instant_the_generated_server_can_order`), green.

## Scope

- cited: crates/generate/ess-synth/src/rust/json.rs
- cited: crates/generate/ess-synth/src/rust/behaviour.rs
- cited: crates/specify/ess-primitives/tests/vectors/rfc3339-instants.json
- cited: crates/generate/ess-synth/tests/adversary_w1_gen1_adversary.rs
- inferred: crates/generate/ess-synth/src/go/ (the Go decoder and instant grammar)
- inferred: crates/generate/ess-openapi/src/lib.rs (if the served schema narrows `format: date-time` with a pattern)

confidence: medium

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
