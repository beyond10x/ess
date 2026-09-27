---
format: aep.planning-md/2
id: story:subject-guard-input-and-case-folding
kind: story
status: draft
title: A subject guard compares with the input, and text compares without case
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 2
---
## Scope

- #157: `input.<field>` as an operand in a `when_subject: {predicate}` comparison.
- #140: `equals_ignore_case` / `in_ignore_case` over `String`, ASCII folding.

Design: `docs/design/value-expressions.md` § E6, E7 (accepted, not implemented).

## Acceptance

- A `when_subject` comparison of a stored field with `input.<field>` validates under the next
  source format and is refused below it; synthesis writes an equal and an unequal witness.
- The two text operators validate over `String` and its newtypes only; Rust, Go and TypeScript
  evaluators agree on ASCII folding; suites carrying them take a new suite format version; Entity
  Runtime lowering refuses them.
