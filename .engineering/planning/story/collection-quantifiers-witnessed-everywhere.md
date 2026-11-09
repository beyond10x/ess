---
format: aep.planning-md/3
id: story:collection-quantifiers-witnessed-everywhere
kind: story
status: draft
title: Every collection quantifier is witnessed with several elements, or says why not
tags:
- defect
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Outcome

Quantifiers over command-input collections (`exists t in input.tags`) are witnessed with several
elements, as stored collections are since beyond10x/ess#240, and a stored-collection quantifier that
falls back to a one-entry row says so instead of passing silently.

## Problem

Input lists. `witness.rs` builds an input list a guard reads with one element, every further element
a copy of element 0, "so a quantifier is decided by element 0 alone"
(`crates/verify/ess-conformance/src/witness.rs:56-57`, `:2839-2841`). Reproduced on ess 0.92.0 with
`FileItem(item_id, label, tags: List<String>)` declaring, in order, `blocked`
(`{exists: {in: input.tags, as: t, that: 't == "blocked"'}}`, error), `unchecked`
(`{not: {forall: {in: input.tags, as: t, that: 't != ""'}}}`, error) and `filed` (creates the item):

```
$ ess verify conform synthesize --path tags.yaml --target ir --out suite.json
3 scenario(s) (0 authored), 0 refusal(s)
  FileItem/outcome/blocked    tags = ["blocked"]
  FileItem/outcome/filed      tags = []
  FileItem/outcome/unchecked  tags = [""]
```

Every list holds at most one element, so a target that tests only `tags[0]` (or only the last
element) passes all three scenarios.

Stored collections. `subject_fact::arranged_row` first searches for a row that witnesses every
elementwise quantifier element by element and, where none is found, falls back to the plain search
(`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:6190-6270`, `Found::Plain`);
`spread` returns nothing for a collection no input writes or a `Boolean`-keyed map
(`subject_fact.rs:1728-1745`). Neither records a note, so the suite reads as if the quantifier were
witnessed.

## Acceptance

- Over the model above, the synthesized suite holds a scenario where the `blocked` list has two or
  more elements with `"blocked"` neither first nor last, and one where `unchecked`'s failing `""`
  sits between non-empty elements; a target reading only the first or only the last element, or
  `forall` for `exists` (and the reverse), fails a scenario on it.
- A nested quantifier over a bound element of an input list is witnessed the same way.
- Where no several-entry row can be built for a stored-collection quantifier (Boolean-keyed map,
  ordered comparison, collection not written from input), `ess verify conform synthesize` prints a
  `note:` naming the guard; a test asserts it.
- The models whose committed suite bytes move are named.

## Scope

- cited: crates/verify/ess-conformance/src/witness.rs (`Elements`, list building, `member_literals`)
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (`arranged_row`, `spread`, `witnesses_elements`)
- inferred: crates/verify/ess-conformance/src/synthesize.rs (notes)
- inferred: crates/verify/ess-conformance/tests/

confidence: medium

## Origin

Correction 1 of #240 (0.43): witness.rs gives input-collection quantifiers one element; the
stored-collection fallback records nothing.
