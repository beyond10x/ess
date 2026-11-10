---
format: aep.planning-md/3
id: story:a-value-expression-chooses-between-two-values-by-a-condition
kind: story
status: active
title: A value expression chooses between two values by a condition (if c then a else b)
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#756
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- depends_on: story:text-functions-compute-values
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/expression_values.rs
- confidence: inferred
  path: crates/generate/ess-synth/tests/expression_values.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression/value.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/expr.rs
- confidence: inferred
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/conditional_value.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: inferred
  path: website/docs/guides/specify/values-and-views.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:42:24Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":3}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:25Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":3}}}
---
## Need

A computed value depends on a condition over the facts it reads: a secret of 12 characters or more
is answered `"secret:"` + its first 4 + 12 `*` + its last 4, a shorter one `"secret:"` + 20 `*`. A
value expression cannot compare or choose: the grammar has no comparison and no conditional
(`crates/specify/ess-primitives/src/expr.rs:1455`, "is no part of an expression"), although the
language design decides `if c then a else b`, both arms one type
(`docs/design/generic-language-features.md:94`).

Minimal reproduction (0.92.0, `ess/36`), the view field of `story:text-functions-compute-values`:

```text
expr: 'if secret == "a" then "x" else "y"'
-> ESS-SPEC-012 `if secret == "a" then "x" else "y"`: `=` is no part of an expression; it is written
   with paths, literals, `+ - * /`, parentheses and function calls
```

This story depends on story:text-functions-compute-values, which adds `len`, `prefix`, `suffix`
and `repeat`, and on story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs, which
opens `ess/37`.

## Acceptance

- `cargo test -p ess-conformance --locked --test conditional_value` passes, asserting that under
  `format: ess/37` `ess specify validate` exits 0 on a model with the mask
  `if len(secret) < 12 then "secret:" + repeat("*", 20) else "secret:" + prefix(secret, 4) +
  repeat("*", 12) + suffix(secret, 4)` as a view field, `if amount >= 1000 then 0 else 5` as a
  `sets:` value, and the same expression as an event payload value.
- The same test asserts that arms of two types (`if amount >= 1000 then 0 else "x"`) are refused as
  `type_mismatch` naming both arm types, and a condition that is not Boolean (`if amount then 0
  else 5`) as `type_mismatch` naming the condition.
- The same test asserts that the mask under `format: ess/36` is refused as
  `unsupported_format_version` naming `if` and `ess/37`.
- The same test asserts that the shared evaluator (`ess-primitives`) answers `Unknown` for the
  mask when `secret` is unknown, and the else arm's text for a `secret` of 12 characters.
- The same test asserts that `ess verify conform synthesize` on the mask's model writes the view
  expectation once for a stored `secret` of 11 characters (then arm) and once for one of 12
  characters (else arm), and that `ess verify conform run --target interpreted` passes every
  scenario.
- The same test runs the suite on a target answering the then arm for every row and one scenario
  fails, and runs `ess verify conform mutate` on the model and the suite kills the new `arm-swap`
  mutant.
- `cargo test -p ess-synth --locked --test expression_values` passes a case asserting that generated
  Rust and Go keep the mask's view an obligation whose text names `an expression value
  (`{expr: "if len(secret) < 12 then …"}`)`.
- `cargo test -p ess-entity-runtime --locked --test expression_values` passes a case asserting that
  Entity Runtime lowering refuses the mask's model with the existing expression-value reason
  (`an expression value (`{expr: …}`, ess/27)`).

## Scope

- cited: crates/specify/ess-primitives/src/expr.rs
- inferred: crates/specify/ess-primitives/src/predicate.rs
- cited: crates/specify/ess-domain/src/expression/value.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- cited: crates/verify/ess-conformance/src/mutate.rs
- inferred: crates/verify/ess-conformance/tests/conditional_value.rs (new)
- inferred: crates/generate/ess-synth/tests/expression_values.rs
- inferred: crates/generate/ess-entity-runtime/tests/expression_values.rs
- cited: docs/design/language-grids.md
- cited: website/docs/guides/specify/values-and-views.md

confidence: medium

Shared surfaces moved: the `ess/37` `FORMAT_HISTORY` sentence, `CHANGELOG.md`, and the regenerated
`spec-versions.md` and `ess.schema.json`.

## Fit review

1. **Need.** A value chosen by a condition over the facts it reads. Requester's syntax (theirs):
   `if(cond, a, b)`. Reproduction above.
2. **Class.** Gap: the value grammar refuses comparison (`expr.rs:1455`).
3. **Already expressible?** For a command value, two outcomes guarded by the condition, each with
   its own `sets:`: longer, but it multiplies outcomes per conditional and does not reach a view
   field at all, which has no branches. So no idiom for the view.
4. **Fit.** It is G1's decided conditional (`generic-language-features.md:94`, spelling
   `if c then a else b`), whose condition is the compact predicate grammar the guards already read
   (`generic-language-features.md` § Spelling), so the language gains no second comparison syntax.
   It opens the conditional row in every value position at once (`sets:`, payloads, view fields;
   measure inputs follow G3's expression measure rule). Synthesis cost is decided: one witness per
   arm.
5. **Second adopter.** A catalogue read that answers `availability` as
   `if quantity == 0 then "sold out" else "in stock"`; an invoice event whose `fee` is
   `if amount >= 1000 then 0 else 5`.
6. **Cost.** Source format `ess/37`, counted under G1 (policy 2); one keyword form; no new
   diagnostic code; one mutation class. IR: the parsed expression, as today.
7. **Alternatives.** (a) Change nothing: no view can state a value that depends on its row's facts
   by a case split. (b) The requester's `if(c, a, b)` call: refused, G1 decided the keyword
   spelling and a call would be a second spelling. (c) A declared redaction (`redacts: secret`, the
   requester's second proposal), which states only that the clear text is not answered: refused,
   because the exact mask is expressible with this story and the text functions, and a property
   "differs from the stored text" passes a target that answers any other text, so it checks less
   than the figure the reader sees (policy 1, `generic-language-features.md:588-593`).
   (d) G1's conditional: chosen.

## Decisions

accept, redesigned. The request's `if(cond, a, b)` is G1's `if c then a else b`. The declared
redaction alternative is not taken, for the reason in question 7 (c).

- Generated Rust and Go keep the command or view an obligation, and Entity Runtime refuses it:
  that is what both already do for every expression value (`ess-synth` `determined.rs`
  `ResolvedPayloadValue::Expression`, `ess-entity-runtime` `subset.rs` `EXPRESSION`), and this
  story adds no generated evaluation.

## Source

github:beyond10x/ess#756
