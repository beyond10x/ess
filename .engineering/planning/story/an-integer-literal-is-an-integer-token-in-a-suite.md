---
format: aep.planning-md/3
id: story:an-integer-literal-is-an-integer-token-in-a-suite
kind: story
status: draft
title: A synthesized Integer literal is an integer token from suite ess-conformance/64, not a float
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
- confidence: inferred
  path: crates/verify/ess-conformance/src/coverage.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/coverage_build.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/integer_literal_token.rs
- confidence: cited
  path: website/docs/guides/verify/runners.md
revision: 12
---
## Outcome

From the suite pair `ess-conformance/64` (ordinary) and `ess-conformance/65` (coverage), a
synthesized whole-number literal for an input, field or payload is written as a JSON integer token
(`1`), not as a binary64-style float (`1.0`). A suite that carries a whole-number literal is written
at that pair; lower suite formats keep their bytes, and the runner guide states the normalisation a
runner owes them.

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
6. **Cost.** Canonical suite bytes change for every suite with a whole-number literal, so it needs a
   suite format version (`AGENTS.md`, "Determinism and formats"): `ess-conformance/64` and `/65`.
7. **Alternatives.** (a) Change nothing, document the normalisation only. (b) Integer tokens from a
   new suite pair (chosen), plus the guide note for older suites. (c) A `--integers-as-tokens`
   switch: refused, two spellings of one suite.

## Decisions

Accept, redesigned: integer tokens from a new suite pair, not a switch; lower suites keep their
bytes; the runner guide documents the normalisation for them.

2026-10-19, left out of the 0.63.0 batch. The store cannot tell an Integer `1` from a Decimal `1.0`
(`crates/specify/ess-primitives/src/facts.rs`, `Number::serialize` is the writer, not
`scenario.rs`), so a writer that picks the token by type cannot reproduce an admitted suite byte
for byte (`crates/verify/ess-conformance/tests/system_issued_vocabulary.rs::the_vocabulary_round_trips_byte_for_byte`).
The two routes:

| option | does | costs |
|---|---|---|
| A | every whole number is an integer token from the new pair | breaks "Decimal literals unchanged" |
| B | the shared number type carries an integer-token flag | a change under `docs/design/review-primitive-semantics.md` |

2026-11-10, decided for this epic:

- **Option A.** It needs no type at write time, so a `/64` suite read back writes the same bytes;
  it is the first item of "What stage two changes" in `docs/design/review-primitive-semantics.md`
  (`Serialize` writes `1` for an integral `Number`); and B changes `Number`'s shape, which re-freezes
  the consumer-coverage baseline (same page, "What this costs the consumer-coverage baseline"). A
  whole `Decimal` (`1.0`) is therefore written `1` at `/64`; a fractional one (`19.99`) is unchanged,
  and every typed `Decimal` reader (`f64`, a decimal type) reads an integer token.
- **The suite writer switches, not `Number`.** `Number::serialize` is unchanged, so reports and every
  other artifact keep their numbers; the suite writer (`ConformanceSuite::to_canonical_json` and
  `to_compact_json`, and the coverage writer) writes an integral `Number` (`Number::as_i64` is
  `Some`) as an integer token when the suite is at `/64` or `/65`.
- **A whole-number literal raises the suite.** A suite whose values hold a whole number is written at
  `/64` (coverage `/65`); otherwise the reporter's model stays at `/34` and keeps writing `1.0`. Every
  suite-version pin of such a model moves, which is the cost of fixing the defect.
- The pair originally planned for this (`/52`, `/53`) shipped with other contents; this story uses
  the epic's pair, opened by story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs.

## Acceptance

- `cargo test -p ess-conformance --locked --test integer_literal_token`,
  `an_integer_literal_is_an_integer_token_at_suite_64`: synthesizing
  `.engineering/repro/integer-literal-float/system.yaml` gives `"suite_version":
  "ess-conformance/64"`, and the literals the `/34` suite wrote as `1.0`, `2.0` and `0.0` are written
  `1`, `2` and `0`.
- Same target, `the_coverage_suite_of_an_integer_literal_is_at_65`: the coverage suite of the same
  model (`--suite-format 5`) is at `ess-conformance/65` and writes the same integer tokens.
- Same target, `a_whole_decimal_is_an_integer_token_at_64`: in a `/64` suite a `Decimal` witness `1`
  is written `1` and a `Decimal` literal `19.99` is written `19.99`.
- Same target, `a_suite_below_64_keeps_its_float_spelling`: a `/34` suite holding `"value": 1.0`, read
  with `ConformanceSuite::from_json` and written with `to_canonical_json`, still holds `1.0`.
- Same target, `a_suite_at_64_round_trips_byte_for_byte`: the `/64` suite of the reproduction, read
  with `ConformanceSuite::from_json` and with `AdmittedSuite::from_json` and written back, gives the
  same bytes.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes, and every row it
  moves is a model whose suite holds a whole-number value; a row of a model without one keeps its
  major and digest.
- `website/docs/guides/verify/runners.md` states that below `ess-conformance/64` (coverage `/65`) a
  whole number is written `1.0` and a typed runner normalises it to an integer, and that from `/64`
  it is an integer token; `crates/verify/ess-conformance/src/witness.rs` module doc :121-126 says the
  same and no longer says the value is an `f64`.

## Scope

- cited: `crates/verify/ess-conformance/src/scenario.rs` (`to_canonical_json`, `to_compact_json` :403-421; the suite-version raise :200-340)
- cited: `crates/verify/ess-conformance/src/coverage_build.rs` (`coverage_version` :587)
- inferred: `crates/verify/ess-conformance/src/coverage.rs` (`to_canonical_json` :822)
- cited: `crates/verify/ess-conformance/src/witness.rs` (module doc :121-126)
- cited: `website/docs/guides/verify/runners.md`
- inferred: `crates/verify/ess-conformance/tests/integer_literal_token.rs` (new)

confidence: medium

Shared surfaces: re-pins the suite-bytes rows of models whose suite holds a whole-number value, and
proposes its `CHANGELOG.md` sentence.
