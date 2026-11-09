---
format: aep.planning-md/3
id: story:synthesis-sends-whole-unit-instants-of-an-encoded-timestamp
kind: story
status: draft
title: Synthesis sends whole-unit instants of an encoded Timestamp
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#758
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-timestamp-newtype-declares-its-wire-encoding
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
revision: 4
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

- `ess verify conform synthesize` on the reproduction model (view parameters of
  `example.calls.EpochInstant`, `encoding: unix_seconds`) writes every value of that type as a
  whole-second instant, and the filter's boundary scenarios still put one row on each side of each
  bound (`created_at` equal to `created_from` admitted, equal to `created_to` refused).
- `ess verify conform run --target interpreted` passes, and the interpreted target refuses a sent
  value of the type that is not a whole unit as `malformed`, once
  `story:a-value-outside-its-declared-type-is-answered-malformed` has landed (and as an
  `ESS-CF-TARGET` error before it).
- An authored scenario literal of the type that is not a whole unit is refused as `ESS-AUTHOR-015`,
  naming the encoding.
- A model with no encoded `Timestamp` keeps its suite bytes.

## Scope

- cited: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/interpret/values.rs
- inferred: crates/verify/ess-conformance/src/authored.rs
- inferred: crates/verify/ess-conformance/tests/timestamp_encoding.rs
confidence: low

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

## Source

github:beyond10x/ess#758
