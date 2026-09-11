# Refused mis-parsed predicate disjunctions

A compact predicate is one expression, and the reader admits only the expressions it can read.
`to == "" or text == ""` is not one of them: the empty-string quotes close and reopen, so the whole
line reads as a single comparison of `to` against the literal text `" or text == "`. The compact
form has no infix `or`, and the reader used to accept that spelling. The branch then became
unreachable by any honest input, synthesis built a candidate sending that literal as the value, a
correct implementation accepted it, and the scenario reported the implementation as wrong.

## The rule

After a closed quoted operand nothing may follow. A remainder is refused while the document is
read, naming the structured form that expresses what the author meant:

> tokens after a quoted operand are unsupported; use structured any/all/not

This is a refusal, not a new language. No infix `or`, `and` or `not` is added to the compact form,
and the structured spellings — `any:`, `all:`, `not:` — already express every disjunction. An
author who wrote a disjunction gets a refusal that names the construct; an author who meant a
literal keeps the literal by closing the quote and writing nothing after it.

What stays admitted: quoted Boolean words, single and double quotes, escape bytes inside a quoted
operand, whitespace around operators, the supported prefix `not`, and every structured predicate.

## One reader in three lanes

The rule is the reader's, so every lane that admits a suite applies it and the three agree:
`quoted_predicate_format` on the Rust side, the same admission in the persisted Go runtime, and the
browser lane's `coverage-admission.js`. A predicate refused by one is refused by all three.

## Formats

A structured comparison operand whose text is not a literal — a fact path such as `other.0` — needs
the corrected reader, and an original reader would have taken it for literal text. Below suite/8 it
is therefore refused rather than silently re-read:

> normalized structured comparison operands require suite/8 or /9

Ordinary suites carry that reader at `/8`, coverage suites at `/9`. A suite pinned to an older
format keeps the meaning its own reader had; nothing rewrites a historical byte.
