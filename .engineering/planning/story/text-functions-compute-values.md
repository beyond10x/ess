---
format: aep.planning-md/3
id: story:text-functions-compute-values
kind: story
status: draft
title: 'Text functions compute values: len, prefix, suffix, repeat, lower, upper, trim, and documented concatenation'
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
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/expr.rs
- confidence: inferred
  path: crates/specify/ess-primitives/tests/text_functions.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: website/docs/guides/specify/values-and-views.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 5
---
## Need

A value computed from a stored text needs text functions: its length, its first or last `n`
characters, a text repeated `n` times, and concatenation. A read that answers a stored secret
masked (`"secret:"`, the first 4 characters, 12 `*`, the last 4) is one such value. The expression
language has no text function (`crates/specify/ess-primitives/src/expr.rs:162-202`: `abs`, `min`,
`max`, `coalesce`, `sha256`, `base64url`, `ascii`, and `round`), although the language design
lists `len`, `lower`, `upper`, `trim` and `s + t` in its text family
(`docs/design/generic-language-features.md:97`). Concatenation already works but is undocumented:
the guide's operator table lists `+` over Integer and Decimal only
(`website/docs/guides/specify/values-and-views.md:218`).

Minimal reproduction (0.92.0, `ess/36`): entity `example.keys.Key` with `secret: String`, a
command storing it, and a view field `{name: secret, type: String, expr: …}` over it.

```text
expr: '"secret:" + secret'        -> valid; synthesis expects "secret:secret", interpreted target passes
expr: 'len(secret)'               -> ESS-SPEC-012 `len` is no function of the expression language
expr: 'prefix(secret, 4)'         -> ESS-SPEC-012 `prefix` is no function ...
expr: 'substring(secret, 0, 4)'   -> ESS-SPEC-012 `substring` is no function ...
expr: 'repeat("*", 20)'           -> ESS-SPEC-012 `repeat` is no function ...
```

## Acceptance

- At the new source format, each of these validates as a view field, a `sets:` value and an event
  payload value: `len(secret)` (an Integer, characters), `utf8_bytes(secret)`, `prefix(secret, 4)`,
  `suffix(secret, 4)`, `repeat("*", 12)`, `lower(secret)`, `upper(secret)`, `trim(secret)`, and
  `"secret:" + prefix(secret, 4) + repeat("*", 12) + suffix(secret, 4)`. Each takes a String or a
  newtype of one and gives a String (`len` and `utf8_bytes` an Integer). A negative or non-Integer
  count, and a non-text operand, are refused as `type_mismatch` naming the function. Below the new
  format each function is refused as `unsupported_format_version` naming it.
- `prefix(s, n)` and `suffix(s, n)` with `n` past the length give the whole text; `repeat(s, n)`
  with `n` above 4096 is refused at validation where `n` is a literal and is `Unknown` at
  evaluation otherwise; `lower`, `upper` and `trim` map ASCII only (`generic-language-features.md`,
  "Text" rule, answer 6).
- The shared evaluator (`ess-primitives`) computes each; `ess verify conform synthesize` writes the
  computed figure on the row and the event, with a stored text at least 9 characters long whose
  first and last 4 characters differ, so a target answering `suffix` where `prefix` is declared
  fails; `ess verify conform run --target interpreted` passes.
- `website/docs/guides/specify/values-and-views.md` lists `s + t` over two Strings and every
  function above; `website/docs/reference/predicates.md` and grid O in
  `docs/design/language-grids.md` (row "text functions", new) say where each applies.
- Generated Rust and Go behaviours and queries keep such a command or view an obligation named by
  the expression, and Entity Runtime refuses it as `FunctionCallUnsupported`, as for every
  expression value today (`values-and-views.md`, after the operator table;
  `crates/generate/ess-entity-runtime/src/subset.rs:683`).
- In a guard (`when:`, a filter), these functions stay refused by name, the hint naming G1's guard
  wave.

## Scope

- cited: crates/specify/ess-primitives/src/expr.rs
- inferred: crates/specify/ess-domain/src/expression/value.rs
- inferred: crates/specify/ess-domain/src/system.rs
- cited: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/specify/ess-primitives/tests/text_functions.rs
- cited: docs/design/generic-language-features.md
- cited: docs/design/language-grids.md
- cited: website/docs/guides/specify/values-and-views.md
- inferred: website/docs/reference/predicates.md
- inferred: website/docs/reference/spec-versions.md
confidence: medium

## Fit review

1. **Need.** A read answers a value derived from a stored text by length, slicing, repetition and
   concatenation, under the stored field's own name. Requester's syntax (theirs): `length(s)`,
   `substring(s, start, end)` or `prefix(s, n)` / `suffix(s, n)`, `repeat(s, n)`, text
   concatenation. Reproduction above and `repro/run.txt`.
2. **Class.** Gap for `len`, `prefix`, `suffix`, `repeat` (`expr.rs:162-202`). Documentation defect
   for `+` over text: it is implemented (`expr.rs:227`, `expr.rs:1142`) and the guide says
   otherwise (`values-and-views.md:218`).
3. **Already expressible?** The field under the stored field's name is: the `concat` variant
   computes `secret` from `secret` (`run.txt`). The mask is not. A declared redaction (the
   requester's second proposal) is not either (`unknown field redacts`, `run.txt`).
4. **Fit.** Grid O row "text functions" (`language-grids.md:76` today refuses `lower`, `upper`,
   `trim`): this opens the row for String and its newtypes in every value position, which is G1
   wave 2, item 5 (`generic-language-features.md:662-663`). Guards are G1 wave 3 and stay refused
   by name. `prefix`/`suffix`/`repeat` join the text family under the same rules: characters for
   counts as `len`, ASCII-only case, no patterns (answer 4). Spelled as functions, like every
   function, not as new keys.
5. **Second adopter.** A support tool that shows a card number as `**** **** **** 1234`
   (`repeat("*", 4) + … + suffix(number, 4)`); a list read that answers a 140-character preview of
   a body (`prefix(body, 140)`).
6. **Cost.** The next open source format (one G1 decision, policy 2). New function names; no new
   diagnostic code; IR carries the parsed expression as today. `ess verify diff` compares the
   parsed tree, as for every expression. No generated-code change (obligation by name).
7. **Alternatives.** (a) Change nothing: the read stays the clear text in the specification.
   (b) `substring(s, start, end)`: refused, two offsets invite off-by-one disagreements and the
   need is the ends of a text. (c) A declared redaction, `redacts: secret`: refused here, see
   `story:a-value-expression-chooses-between-two-values-by-a-condition`, Decisions. (d) The G1
   text family plus `prefix`, `suffix`, `repeat`: chosen.

## Decisions

accept, redesigned. Taken from the request: `prefix`, `suffix`, `repeat` and text concatenation
(already implemented, now documented). Changed: `length` is G1's `len`; `substring` is not added.
The conditional is `story:a-value-expression-chooses-between-two-values-by-a-condition`.

## Source

github:beyond10x/ess#756
