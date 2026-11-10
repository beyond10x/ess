---
format: aep.planning-md/3
id: story:text-shape-operators-ignore-case
kind: story
status: draft
title: contains, starts_with and ends_with have ignore-case forms
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#759
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: cited
  path: crates/generate/ess-synth/src/determined.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/predicate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/predicate.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/predicate.ts
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: docs/design/language-grids.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 7
---
## Need

A row matches when a field holds a text as a substring regardless of ASCII case (SQL
`ilike '%t%'`). Equality and membership have ignore-case forms (`equals_ignore_case`,
`in_ignore_case`, `ess/15`; `crates/specify/ess-primitives/src/predicate.rs:500-523`). The three
text-shape operators (`starts_with`, `ends_with`, `contains`) have none.

Minimal reproduction (0.92.0, `ess/36`), the view of `story:a-text-operator-reads-a-binder-or-a-field-as-its-operand`
with `contains` replaced (`repro/run.txt`):

```text
from_number: {contains_ignore_case: t}
-> ESS-SPEC-012 unknown operator "contains_ignore_case"; expected one of eq, ne, lt, lte, gt, gte, any_of, none_of, exists, truthy, starts_with, ends_with, contains, equals_ignore_case, in_ignore_case, trimmed
from_number: {starts_with_ignore_case: t}
-> ESS-SPEC-012 unknown operator "starts_with_ignore_case" ...
```

## Acceptance

- At `ess/37`, `contains_ignore_case`, `starts_with_ignore_case` and `ends_with_ignore_case`
  validate (`ess specify validate`, exit 0) in every position the three operators validate in (a
  command guard, `when_subject:`, a `when_related:` predicate, a row-set `where:`, a view `filter:`),
  with the operands those take: a literal, `{param: …}` and `{input: …}`; the `{fact: …}` operand on them is
  story:a-text-operator-reads-a-binder-or-a-field-as-its-operand's, which lands after this one. `trimmed:` wraps them as it
  wraps the case-sensitive three. Folding is ASCII only, as `equals_ignore_case`
  (`docs/design/generic-language-features.md`, "Text"; answer 6).
- Below `ess/37` each is refused as `unsupported_format_version` naming `ess/37`.
- Synthesis witnesses each with the existing fold refutations
  (`crates/verify/ess-conformance/src/witness.rs:3602-3640`): the selected side holds the operand
  in its other case, and the other side holds it with one character changed.
  `ess verify conform run --target interpreted` passes; a target that compares case-sensitively
  fails the first scenario, and a target that ignores the operator fails the second.
- `ess verify conform mutate` swaps each with its case-sensitive form, and the witness kills it.
- The Go and TypeScript predicate runtimes evaluate them, and the suite passes against each.
- Generated Rust and Go behaviours and view queries keep a guard or filter carrying one an
  obligation naming the predicate, as `equals_ignore_case` is kept (`determined.rs:1331`).
- Entity Runtime lowering refuses them as `CaseFoldUnsupported`, naming the operator, where it
  refuses `equals_ignore_case`.
- `website/docs/reference/predicates.md` lists them beside `equals_ignore_case`, and grid O's
  "ignore-case" row in `docs/design/language-grids.md` covers the five operators.

## Scope

- cited: crates/specify/ess-primitives/src/predicate.rs
- inferred: crates/specify/ess-domain/src/expression.rs
- cited: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/src/mutate.rs
- cited: crates/verify/ess-conformance/src/go/predicate.go
- cited: crates/verify/ess-conformance/src/ts/predicate.ts
- cited: crates/generate/ess-synth/src/determined.rs
- cited: crates/generate/ess-entity-runtime/src/lib.rs
- cited: crates/generate/ess-entity-runtime/src/subset.rs
- cited: website/docs/reference/predicates.md
- cited: docs/design/language-grids.md

confidence: medium

Moves shared surfaces: proposes its sentence for the `ess/37` `FORMAT_HISTORY` row and for
`CHANGELOG.md`, and changes the regenerated `ess.schema.json` and spec-versions page.

## Fit review

1. **Need.** A case-insensitive substring, prefix or suffix test. Requester's syntax (theirs):
   `contains_ignore_case`, `starts_with_ignore_case`, `ends_with_ignore_case`.
2. **Class.** Gap: no form exists (`run.txt`), and the expression language has no `lower()` in a
   guard yet (`docs/design/language-grids.md:76`, "`lower` `upper` `trim`: no").
3. **Already expressible?** Only by listing every case variant of the token, which a token from a
   parameter cannot do.
4. **Fit.** Grid O has two rows: "starts_with ends_with contains" and "ignore-case equality, in"
   (`language-grids.md:73-74`). This closes the ignore-case column across all five text operators.
   It does not add a sixth spelling of case folding. The names follow the existing
   `equals_ignore_case` pattern, and the `trimmed:` wrapper composes as it does with the others
   (`predicates.md`, "From `ess/25`: surrounding whitespace"). G1's `lower(s)` in guards
   (`generic-language-features.md:97`, wave 3) stays the general form; these operators lower to
   `contains(lower(a), lower(b))` when it lands, as `equals_ignore_case` will.
5. **Second adopter.** A customer search over `email` that a server matches with `ilike`; a product
   filter on `sku` prefixes regardless of case.
6. **Cost.** `ess/37`; three operator keywords; no new diagnostic code; one
   mutation pairing. No suite format beyond what a view expectation carrying a fold operator already
   selects (`ess-conformance/20`, `predicates.md` after the ignore-case examples).
7. **Alternatives.** (a) Change nothing: the search is unstated. (b) G1's `lower()` in guards
   first: the general form, but it waits for wave 3's grammar in guards and view filters, and these
   operators would lower to it unchanged. (c) The ignore-case column of the existing operator
   family: chosen. It is the requester's spelling.

## Decisions

accept as proposed. The three operators are taken as named, with the existing ASCII-only folding,
operands and `trimmed:` composition.

The generated Rust and Go behaviours keep the three an obligation rather than evaluating them,
because they keep `equals_ignore_case` and `in_ignore_case` one today (`determined.rs:1331`, no
`FoldMatch` arm in `rust/behaviour.rs` or `go/behaviour.rs`).

## Source

github:beyond10x/ess#759
