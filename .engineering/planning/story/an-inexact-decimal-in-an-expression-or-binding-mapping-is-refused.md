---
format: aep.planning-md/3
id: story:an-inexact-decimal-in-an-expression-or-binding-mapping-is-refused
kind: story
status: draft
title: An inexact decimal in an expression or a binding mapping is refused
summary: 'Follow-up of story:an-inexact-authored-decimal-is-refused: a literal inside an {expr: ...} value and a binding mapping: Decimal constant are still read through binary64 without inexact_decimal_literal (adversary pass 3, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:an-inexact-authored-decimal-is-refused` refuses an authored `Decimal` literal that binary64
cannot hold (`inexact_decimal_literal`) where it is written as a number: a `sets:` or `payload:`
value of an outcome or an `affects:` entry, and every guard or filter position. Two authored
positions are still read through binary64 without a diagnostic:

- a literal inside an `{expr: …}` value (`sets: {amount: {expr: "input.amount + 0.1000000000000000000001"}}`,
  ess/27, beyond10x/ess#560): the expression text is parsed by `Expr::parse`, outside the
  collection the check uses;
- a binding `mapping:` constant (`mapping: {slot: 0.1000000000000000000001}` over a `Decimal`
  input, beyond10x/ess#445): `authored` reads only command outcomes.

Found by adversary pass 3 of that story; the two cases are in
`crates/specify/ess-domain/tests/adversary_inexact_decimal_pass3.rs`, ignored with this story's id.

## Acceptance

1. Both cases above are refused once with `inexact_decimal_literal`, at the field the `{expr:}`
   sets and at `binding.<id>.mapping.<target>`; the two ignored cases run and pass.
2. `0.5` in either position validates as before.
3. The changelog entry of the parent story no longer lists the two positions as not checked.
