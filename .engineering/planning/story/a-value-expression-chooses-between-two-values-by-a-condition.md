---
format: aep.planning-md/3
id: story:a-value-expression-chooses-between-two-values-by-a-condition
kind: story
status: draft
title: A value expression chooses between two values by a condition (if c then a else b)
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#756
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
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
revision: 4
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

## Acceptance

- At the new source format, a value expression takes `if <condition> then <a> else <b>`, where the
  condition is a predicate in the compact grammar over the same roots as the value (`==`, `!=`,
  `<`, `<=`, `>`, `>=`, `in`, `and`, `or`, `not`, `defined(x)`) and both arms are of one type; it
  validates as a view field, a `sets:` value and an event payload value. The mask
  `if len(secret) < 12 then "secret:" + repeat("*", 20) else "secret:" + prefix(secret, 4) +
  repeat("*", 12) + suffix(secret, 4)` validates once the text functions land.
- Arms of two types, and a condition that is not Boolean, are refused as `type_mismatch` naming
  both; below the new format `if` is refused as `unsupported_format_version` naming it.
- The shared evaluator computes it three-valued: an `Unknown` condition makes the value `Unknown`.
- `ess verify conform synthesize` witnesses each arm, as `{input, else}` already is
  (`generic-language-features.md`, synthesis cost table, row "presence, `coalesce`, `if`"): on the
  reproduction it writes the view expectation once for a stored text whose condition is true and
  once for one whose condition is false, on either side of the comparison's boundary (11 and 12
  characters for `len(secret) < 12`). A target answering one arm for every row fails one of the two.
  `ess verify conform run --target interpreted` passes.
- `ess verify conform mutate` gains the arm-swap class on a conditional value
  (`generic-language-features.md:193`), and the reproduction's suite kills it.
- Generated Rust and Go, and Entity Runtime, keep such a command or view an obligation, or refuse
  it, named by the expression, as for every expression value.

## Scope

- cited: crates/specify/ess-primitives/src/expr.rs
- inferred: crates/specify/ess-primitives/src/predicate.rs
- inferred: crates/specify/ess-domain/src/expression/value.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/mutate.rs
- inferred: crates/verify/ess-conformance/tests/conditional_value.rs
- cited: docs/design/language-grids.md
- inferred: website/docs/guides/specify/values-and-views.md
confidence: medium

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
6. **Cost.** The next open source format, counted under G1 (policy 2); one keyword form; no new
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

## Source

github:beyond10x/ess#756
