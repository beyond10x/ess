---
format: aep.planning-md/3
id: story:an-integer-literal-is-an-integer-token-in-a-suite
kind: story
status: draft
title: A synthesized Integer literal is an integer token from suite /52, not a float
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- informed_by: release-plan:ess-063-downstream
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/integer_literal_token.rs
- confidence: cited
  path: website/docs/guides/verify/runners.md
revision: 7
---
## Outcome

From the next suite format pair after the newest released one (`ess-conformance/62` ordinary, `/63`
coverage, at 0.92.0), a synthesized literal for an `Integer`-typed input, field or payload is
written as a JSON integer token (`1`), not as a binary64-style float (`1.0`). Lower suite formats
keep their bytes, and the runner guide states the normalisation a runner owes them.

## Fit review

Finding covered: downstream specification report of 2026-10-19, finding 17.

1. **Need.** A runner that decodes a request into the types ESS itself generates for Rust (`Integer`
   → `i64`) refuses `"max_concurrency": 1.0`. Reproduction (still present on 0.92.0), the model
   `.engineering/repro/integer-literal-float/system.yaml`: `DeclarePool` takes `slots: Integer`,
   `spare: Integer` and `label: String`, refuses `slots < 1`, and creates a `Pool` that stores all
   three, published by a view.

   ```text
   $ ess verify conform synthesize --path . --format json --out suite.json
   "suite_version": "ess-conformance/34"
   "value": 1.0   "value": 2.0   "value": 0.0      (every Integer literal)
   $ ess verify conform run --path . --target interpreted   → 2 passed
   ```

   The Go runner reads numbers as float64, which hides it.
2. **Class.** Defect: the suite hands a typed target a value that the type ESS generates for it
   refuses, and `website/docs/guides/verify/runners.md` does not say a runner must normalise it.
   `crates/verify/ess-conformance/src/witness.rs:121-126` says a runner "has to normalise it",
   giving a reason ("which is an `f64`") that `Number`'s exact representation
   (`crates/specify/ess-primitives/src/facts.rs:46`) no longer supports; the float rendering is
   kept for byte stability (`facts.rs`, `impl Serialize for Number` :173-210).
3. **Already expressible?** Only as a runner-side conversion (the reporter's workaround).
4. **Fit.** No authored surface. Every target reads the new token: JSON integer tokens decode into
   float64 in Go and into `number` in TypeScript unchanged.
5. **Second adopter.** Any typed runner: Rust serde `i64`, a JVM `long`.
6. **Cost.** Canonical suite bytes change for every suite with an `Integer` literal, so it needs a
   suite format version (`AGENTS.md`, "Determinism and formats").
7. **Alternatives.** (a) Change nothing, document the normalisation only. (b) Integer tokens from a
   new suite pair (chosen), plus the guide note for older suites. (c) A `--integers-as-tokens`
   switch: refused, two spellings of one suite.

## Decisions

Accept, redesigned: integer tokens from a new suite pair, not a switch; lower suites keep their
bytes; the runner guide documents the normalisation for them. Decimal literals are unchanged.

2026-10-19, left out of the 0.63.0 batch. The store cannot tell an Integer `1` from a Decimal `1.0`
(`crates/specify/ess-primitives/src/facts.rs`, `Number::serialize` is the writer, not
`scenario.rs`), so a writer that picks the token by type cannot reproduce an admitted suite byte
for byte (`crates/verify/ess-conformance/tests/system_issued_vocabulary.rs::the_vocabulary_round_trips_byte_for_byte`).
The two routes:

| option | does | costs |
|---|---|---|
| A | every whole number is an integer token from the new pair | breaks "Decimal literals unchanged" |
| B | the shared number type carries an integer-token flag | a change under `docs/design/review-primitive-semantics.md` |

Fresh suites start at `/34`: unless an Integer literal itself raises a suite to the new pair, a
downstream suite keeps writing `1.0`; if it does, every suite-version pin of a model with an
Integer literal moves. Open until a design pass picks A or B. The pair originally planned for this
(`/52`,`/53`) shipped with other contents; the pair number is chosen when the design is.

## Acceptance

- `an_integer_literal_is_an_integer_token_in_the_new_suite_pair`: for the model above, a suite at
  the new pair writes `"value": 1` for an `Integer` input; the same model at `/34` (and any lower
  pair) still writes `1.0`; a `Decimal` literal is written as the chosen option says.
- A suite at the new pair round-trips byte for byte through the admitted-suite reader.
- The runner guide states the rule for suites below the new pair.

## Scope

- cited: `crates/specify/ess-primitives/src/facts.rs` (`Number`, `impl Serialize` :173-210)
- cited: `crates/verify/ess-conformance/src/witness.rs` (module doc :121-126)
- cited: `website/docs/guides/verify/runners.md`
- inferred: `crates/verify/ess-conformance/src/scenario.rs` (suite format selection)
- inferred: `crates/verify/ess-conformance/tests/integer_literal_token.rs` (new)

confidence: medium (blocked on the A/B design decision)
