---
format: aep.planning-md/3
id: story:an-integer-literal-is-an-integer-token-in-a-suite
kind: story
status: draft
title: A synthesized Integer literal is an integer token from suite /52, not a float
tags:
- adopter-report
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- informed_by: release-plan:ess-063-downstream
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/integer_literal_token.rs
- confidence: cited
  path: website/docs/guides/verify/runners.md
revision: 4
---
## Outcome

From the next suite format pair (`ess-conformance/52` ordinary, `/53` coverage), a synthesized
literal for an `Integer`-typed input, field or payload is written as a JSON integer token (`1`), not
as a binary64 float (`1.0`). Lower suite formats keep their bytes, and the runner guide states the
normalisation a runner owes them.

## Fit review

Finding covered: downstream specification report of 2026-10-19, finding 17.

1. **Need.** A runner that decodes a request into the types ESS itself generates for Rust (`Integer`
   → `i64`) refuses `"max_concurrency": 1.0`. Reproduction:
   `.engineering/repro/integer-literal-float/system.yaml` (two `Integer` inputs). On 0.60.0, 0.62.0
   and the integration build the suite carries `"value": 0.0`, `1.0`, `2.0` for them (integration
   build: 1× `0.0`, 5× `1.0`, 2× `2.0`). The downstream suite carries `388081.0` for an `Integer`
   input. The Go runner reads numbers as float64, which hides it.
2. **Class.** Defect: the suite hands a typed target a value the type ESS generates for it refuses,
   and `website/docs/guides/verify/runners.md` does not say a runner must normalise it.
   `crates/verify/ess-conformance/src/witness.rs:110-115` says a runner "has to normalise it",
   giving a reason ("which is an `f64`") that `Number`'s exact representation
   (`crates/specify/ess-primitives/src/facts.rs:46-56`) no longer supports; the float rendering is
   kept for byte stability (`facts.rs:173-210`).
3. **Already expressible?** Only as a runner-side conversion (the reporter's workaround).
4. **Fit.** No authored surface. The scenario writer knows each literal's declared type (the
   flattener already refuses a fractional value for an `Integer`, `witness.rs:113`). Every target
   reads the new token: JSON integer tokens decode into float64 in Go and into `number` in
   TypeScript unchanged.
5. **Second adopter.** Any typed runner: Rust serde `i64`, a JVM `long`.
6. **Cost.** Canonical suite bytes change for every suite with an `Integer` literal, so it needs a
   suite format version (`AGENTS.md`, "Determinism and formats"). The unreleased `/52`,`/53` pair
   planned in `release-plan:ess-063-downstream` (suite row) carries it without a further bump.
7. **Alternatives.** (a) Change nothing, document the normalisation only. (b) Integer tokens from
   `/52` (chosen), plus the guide note for older suites. (c) A `--integers-as-tokens` switch:
   refused, two spellings of one suite.

## Decisions

Accept, redesigned: integer tokens from the `/52`,`/53` pair, not a switch; lower suites keep their
bytes; the runner guide documents the normalisation for them. Decimal literals are unchanged.

## Acceptance

- `an_integer_literal_is_an_integer_token_from_suite_52`: a `/52` suite writes `"value": 1` for an
  `Integer` input; a `/50` suite of the same model still writes `1.0`; a `Decimal` literal is
  written as before.
- The runner guide states the rule for suites below `/52`.

## Scope

- `crates/verify/ess-conformance/src/scenario.rs` (literal writer, ~1759) and the suite-version
  selection — inferred.
- `crates/verify/ess-conformance/src/witness.rs` (doc comment ~110) — cited.
- `website/docs/guides/verify/runners.md` — cited.
- Test: new `crates/verify/ess-conformance/tests/integer_literal_token.rs`.

One unit, scheduled with the `/52` pair (after U5 and the port fix the pair's other contents).

## Decisions

2026-10-19, left out of the 0.63.0 batch. The quick-fix unit found that the store cannot tell an
Integer `1` from a Decimal `1.0` (`crates/specify/ess-primitives/src/facts.rs`, `Number::serialize`
is the writer; the scope line naming `scenario.rs` is wrong), so a writer that picks the token by
type cannot reproduce an admitted `/52` suite byte for byte
(`the_vocabulary_round_trips_byte_for_byte`). The two routes:

| option | does | costs |
|---|---|---|
| A | every whole number is an integer token from suite `/52` | breaks this story's "Decimal literals unchanged" |
| B | the shared number type carries an integer-token flag | a change under `docs/design/review-primitive-semantics.md` |

Fresh suites start at `/34`: unless an Integer literal itself raises a suite to `/52`, a downstream
suite keeps writing `1.0`; if it does, about 289 suite-version pins in 138 files move. Open until a
design pass picks A or B.
