---
format: aep.planning-md/3
id: story:no-emitted-document-carries-a-number-wrapper
kind: story
status: draft
title: No emitted document carries a serde_json number wrapper, in either feature setting
tags:
- adopter-report
- defect
refs:
- provider: github
  reference: beyond10x/ess#579
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

No document ESS emits carries a `serde_json` private number wrapper. Every number in every
projection (OpenAPI, JSON Schema, AsyncAPI, suites, IR, generated embedded documents) is written as
a plain JSON or YAML number whether or not `serde_json/arbitrary_precision` is on in the build, and
one corpus test holds every emitter to it.

Findings covered: https://github.com/beyond10x/ess/issues/579, part 1 (a downstream adversarial
review on ess 0.60.0, `ess/25`).

## Fit review

1. **Need.** An enum with an `Integer` attribute is published in `x-ess-attributes` as
   `{"$serde_json::private::Number": "2"}` instead of `2`, in every generated OpenAPI document a
   served component embeds. Minimal reproduction to write first: `demo.levels.Level`, an enum with
   `attributes: [{name: rank, type: Integer}]` and two variants, projected to OpenAPI.
   Requester's expectation (theirs): `"CRIT": 2`.
2. **Class.** Defect. `attributes()` builds a `serde_json::Number`
   (`crates/generate/ess-gen/src/types.rs:1357-1375`); with `arbitrary_precision` unified into the
   build a `Number` serialized through a serializer that is not `serde_json` writes the private
   wrapper map. The feature is turned on by design where exact numbers need it
   (`story:generated-rust-types-make-exact-numbers-a-crate-feature`, implemented, #483), so any
   emitter that holds a `serde_json::Number` and writes through another serializer is exposed, not
   only this one.
3. **Already expressible?** Not a language question; the consumer cannot repair generated output.
4. **Fit.** The generic fix is a property of every emitter rather than a patch of one annotation:
   numbers reach emitted bytes through one conversion that never hands a `serde_json::Number` to a
   foreign serializer, and a test walks every projection of a corpus with the feature on and off and
   requires identical bytes and no wrapper key. Related, not overlapping:
   `story:primitive-canonical-serialization` (the canonical bytes of a number, blocked on a
   decision); this story fixes only the leak.
5. **Second adopter.** Any specification with an `Integer` enum attribute, an integer `const` or
   bound published through a non-`serde_json` writer, in any build where another crate turns on
   `arbitrary_precision`.
6. **Cost.** Bytes change only where the wrapper appeared, each a correction; no format version.
7. **Alternatives.** (a) Change nothing: every consumer of the published OpenAPI reads a wrapper.
   (b) Patch `attributes()` only: the next emitter that holds a `Number` leaks again.
   (c) One conversion plus a corpus test across emitters and both feature settings (chosen).

## Decisions

Accept, redesigned from "fix the attribute values" to "no emitter writes the wrapper, held by a
corpus test in both feature settings".

## Acceptance

- `integer_enum_attribute_is_a_plain_number`: the reproduction's OpenAPI projection carries
  `"values": {"Low": 1, "High": 2}` with `arbitrary_precision` on and off.
- `no_projection_carries_a_number_wrapper`: every projection of every repository example, built with
  the feature on, contains no `$serde_json::private::Number` key and equals the bytes built with it
  off.

## Scope (inferred)

`crates/generate/ess-gen/src/types.rs` (`attributes`), other emitters holding `serde_json::Number`,
an `ess-xtask` or `ess-gen` corpus test, CHANGELOG.
