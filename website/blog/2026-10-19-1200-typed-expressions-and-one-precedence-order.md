---
title: "0.63 — Values computed by typed expressions, and one precedence order for guards"
description: >
  0.63.0 adds source format `ess/27`: a value may be computed by a typed expression, a guard may
  read a digest of its input, and every command's guards answer in one precedence order.
slug: typed-expressions-and-one-precedence-order
tags: [release, ess]
date: 2026-10-19T12:00:00+02:00
release_tag: "0.63.0"
---

0.63.0 is a feature release. It adds the source format `ess/27`, the suite formats
`ess-conformance/52` and `/53`, the delta format `ess-diff/20` and the suite comparison
`ess-suite-diff/1`. A model below `ess/27` keeps its meaning, its IR bytes and its compiled digest.
The [changelog](https://github.com/beyond10x/ess/blob/main/CHANGELOG.md) lists every change since
0.59.0.

{/* truncate */}

## Values computed by typed expressions

From `ess/27` a value in `sets:`, an event's `payload:` or an error's `payload:` may be computed by
an expression, `{expr: "<expression>"}`, and a view field may be computed from its source row, such
as `expr: "ended_at - started_at"`. Arithmetic over Integer and Decimal is exact, `/` is an
`Optional<Decimal>` that is absent at a zero divisor, `round` rounds a quotient once, and
`Timestamp - Timestamp` is a `Duration`. Synthesis runs the branch again on inputs worth checking
the figure on: a zero divisor and a rounding tie either way
([#560](https://github.com/beyond10x/ess/issues/560)).

A guard may read a fact through a digest or encoding function: `challenge !=
base64url(sha256(ascii(input.verifier)))` states a PKCE `S256` check as a held-row predicate
([#584](https://github.com/beyond10x/ess/issues/584)).

## One precedence order

Under `ess/27` a branch answers in the order of what it must read: the request alone, then whether
the addressed record exists, then what it holds, then the rows around it, then the accepting
branches ([#554](https://github.com/beyond10x/ess/issues/554)). Every guard pair once refused for
want of an order now validates, such as `when_related:` beside `when_subject:` or `wrong_state:`
([#282](https://github.com/beyond10x/ess/issues/282)).

## More in `ess/27`

- `shorthand: <field>` on a struct type lets a value be written as that one field
  ([#594](https://github.com/beyond10x/ess/issues/594)).
- An `each:` entry of `affects:` may move or delete the row each element names
  ([#596](https://github.com/beyond10x/ess/issues/596)).
- A binding condition may compare a `Boolean` leaf with `true` or `false`.

## Conformance

- `ess verify conform diff --from <suite> --to <suite>` says which scenarios two suites add,
  remove and change, and `--format json` writes an `ess-suite-diff/1` document
  ([#572](https://github.com/beyond10x/ess/issues/572)).
- `ess verify conform synthesize --check` compares what it would write with what is committed
  ([#571](https://github.com/beyond10x/ess/issues/571)).
- Suites `ess-conformance/52` and `/53` state the event catalog once at the root instead of in
  every scenario, check a value type's invariants inside a view's lists, maps, optionals and
  unions, and await a command the system issues itself
  ([#570](https://github.com/beyond10x/ess/issues/570),
  [#542](https://github.com/beyond10x/ess/issues/542)).
