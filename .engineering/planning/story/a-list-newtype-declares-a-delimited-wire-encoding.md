---
format: aep.planning-md/3
id: story:a-list-newtype-declares-a-delimited-wire-encoding
kind: story
status: draft
title: A list newtype declares a delimited wire encoding
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#759
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-timestamp-newtype-declares-its-wire-encoding
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/types.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/list_encoding.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-diff/src/change.rs
- confidence: inferred
  path: docs/design/value-wire-encodings.md
- confidence: inferred
  path: schemas/generated/ess.schema.json
revision: 4
---
## Need

A list travels on the wire as one text value split on declared separators: `q=a,b c` is the list
`["a", "b", "c"]`. A `List<…>` view parameter is published and read as a repeated key
(`style: form`, `explode: true`;
`.engineering/planning/story/an-optional-list-view-parameter-is-read-as-published.md`, Need). No
declaration says that a list arrives as one delimited value.

Minimal reproduction (0.92.0, `ess/36`), the view of
`story:a-text-operator-reads-a-binder-or-a-field-as-its-operand` (`repro/run.txt`):

```text
- {name: q, type: List<String>, split: {on: [",", " "]}}
-> unknown field `split`
```

This story is the model half: the design, the key, validation, IR, diff, and the values synthesis
may send.

## Acceptance

- `docs/design/value-wire-encodings.md` (the page `story:a-timestamp-newtype-declares-its-wire-encoding`
  writes) gains "Delimited lists": a newtype of `List<String>` (or of a list of a String newtype)
  may declare `encoding: {delimited: [",", " "]}`. On the wire the value is one text, the elements
  joined by the first separator; reading splits on any separator and drops empty pieces; an element
  that is empty or holds a separator is not a value of the type.
- At the new source format, `example.calls.Tokens` (`kind: newtype, of: List<String>, encoding:
  {delimited: [",", " "]}`) as the type of `q` validates, and so does the reproduction filter over
  it. `delimited:` on a non-list or a list of non-text elements is `type_mismatch`. An empty
  separator list, an empty separator, or a separator listed twice is refused naming it. Below the
  new format the key is refused as `unsupported_format_version` naming it.
- `ess specify compile` writes the encoding on the resolved type, omitted when absent; a model
  without it keeps its IR bytes; `schemas/generated/ess.schema.json` is regenerated; `ess verify
  diff` reports a moved separator list as the type's encoding change, the same kind as the
  `Timestamp` encoding.
- `ess verify conform synthesize` sends values of the type whose elements are non-empty and hold no
  separator, and an authored literal element holding one is refused as `ESS-AUTHOR-015` naming the
  separator. A model without the key keeps its suite bytes.

## Scope

- inferred: docs/design/value-wire-encodings.md
- cited: crates/specify/ess-domain/src/types.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-diff/src/change.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/specify/ess-domain/tests/list_encoding.rs
- inferred: schemas/generated/ess.schema.json
confidence: medium

## Fit review

1. **Need.** One text value carries a list, split on declared separators. Requester's syntax
   (theirs): `split: {on: [",", " "], distinct: true}` on a `List<String>` parameter.
2. **Class.** Gap: a list parameter has one wire form, the repeated key.
3. **Already expressible?** No (`run.txt`, `split`). A `String` parameter keeps the wire and loses
   the quantifier over its tokens.
4. **Fit.** The wire form of a value is a fact of its type, as decided for `Timestamp`
   (`story:a-timestamp-newtype-declares-its-wire-encoding`). The same `encoding:` key on a newtype
   holds it in every position: a view parameter, a command input or a body member. It is not a key
   on one parameter. Grids R and O are unchanged: the value is a `List<String>` to every quantifier.
5. **Second adopter.** `ids=1,2,3` on a list read, the comma-delimited form OpenAPI calls `style:
   form`, `explode: false`, common in list APIs; a `scope` value of space-delimited tokens (OAuth 2.0
   `scope`, RFC 6749 § 3.3).
6. **Cost.** The next open source format; one value shape of `encoding:`. Synthesis bounds the
   elements it sends. No suite format: values stay lists.
7. **Alternatives.** (a) Change nothing. (b) The requester's key on a parameter: refused, one
   position of several. (c) `distinct: true` as part of the wire form: not taken. A filter that
   quantifies over the tokens answers the same with or without duplicates, and an exact count of
   distinct tokens is `distinct` in the predicate language. (d) `encoding: {delimited: […]}` on a
   list newtype: chosen.

The requester's third ask, a per-token sign (`-x` meaning "must not contain x"), is refused as out
of the language. It is a grammar inside a text value, the kind ESS declines: a caller-supplied
filter "would need a declared grammar" (`docs/design/view-paging.md`, "Declined: the caller-supplied
filter expression"), and patterns are out (`docs/design/generic-language-features.md`, answer 4).
There is no idiom that keeps it checked: two parameters, one for included and one for excluded
tokens, would change the wire. The specification states the included tokens, and a server that also
reads signs does not contradict it.

## Decisions

accept, redesigned. The split moves from a parameter key to a list newtype's `encoding:`, beside
the `Timestamp` encodings. `distinct` is not taken, and the per-token sign is refused as out of the
language.

## Source

github:beyond10x/ess#759
