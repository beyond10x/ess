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
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/input_list_quantifiers.rs
revision: 9
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

The model is the reproduction above, kept as a fixture of
`crates/verify/ess-conformance/tests/input_list_quantifiers.rs`; every bullet is a case of
`cargo test -p ess-conformance --locked --test input_list_quantifiers`.

- The synthesized suite holds a `FileItem/outcome/blocked` scenario whose `tags` has at least three
  elements with `"blocked"` neither first nor last.
- The synthesized suite holds a `FileItem/outcome/unchecked` scenario whose `tags` has at least
  three elements with the one `""` neither first nor last.
- Run against that suite, each of three mutated targets fails at least one scenario: one reading
  only `tags[0]`, one reading only the last element, and one evaluating `forall` for `exists` and
  `exists` for `forall`.
- With `blocked` rewritten as a nested quantifier over a member of a bound element (input
  `lines: List<Line>`, `Line` holding `codes: List<String>`, guard
  `exists l in input.lines: exists c in l.codes: c == "blocked"`), the `blocked` scenario's
  deciding `codes` list has at least three elements with `"blocked"` neither first nor last.
- For a stored-collection quantifier over a `Boolean`-keyed map, `ess verify conform synthesize`
  prints a `note:` line naming the guard and the fallback to a one-entry row (a new `Note`
  variant).
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` passes, and each row
  this change re-pins names "input-list quantifiers witnessed with several elements" beside it.

## Scope

- cited: crates/verify/ess-conformance/src/witness.rs (`Elements`, list building, `member_literals`)
- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs (`arranged_row`, `spread`, `witnesses_elements`)
- cited: crates/verify/ess-conformance/src/synthesize.rs (`Note`, line 285)
- inferred: crates/verify/ess-conformance/tests/input_list_quantifiers.rs (new)

confidence: medium

Shared surfaces moved: re-pins the suite-bytes rows of models whose input-list quantifiers gain
elements; proposes its `CHANGELOG.md` sentence; no format row.

## Origin

Correction 1 of #240 (0.43): witness.rs gives input-collection quantifiers one element; the
stored-collection fallback records nothing.
