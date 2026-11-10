---
format: aep.planning-md/3
id: story:entity-runtime-lowers-stored-field-type-orderings-typed
kind: story
status: active
title: Entity Runtime lowers a stored field type's ordering invariant typed, or refuses it by name
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/generate/ess-entity-runtime/src/lib.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/src/subset.rs
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/stored_field_orderings.rs
- confidence: inferred
  path: website/docs/reference/entity-runtime-lowering.md
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:41Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:43Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Finding

Measured by adversary pass 1 of wave D343 (`review-result:adversary-d343-u2-pass-1`, F4, out of
scope there, inferred from code): the Entity Runtime lowering of a stored field type's ordering
invariant goes through the untyped `lower_predicate` path, the path D343 U2 replaced for input
types. In `crates/generate/ess-entity-runtime/src/lib.rs`, `nominal_condition` (around line 1400)
returns `lower_predicate(predicate, rewrite)` whenever the placement is not an input, and
`lower_predicate` (around line 4605) is `lower_typed` with `Typing::untyped()`, so an ordering
becomes a plain `compare` (around lines 4644-4660). Entity invariants (`entity.invariants`, around
line 1019) take the same untyped path. An ordering over a `Timestamp` or text becomes a string
`compare`, which entity-core 0.28.0 answers `Unknown` when no scale is declared. U2's implementor
also noted that `==` on a `Timestamp` is still a string comparison, in guards as well.

Minimal model (valid on ess 0.92.0, `ess specify validate` exit 0): a creation stores a
`Timestamp` newtype bounded below.

```yaml
format: ess/22
system: demo
version: v1
domain: demo.notes
types:
  - {name: demo.notes.NoteId, kind: newtype, of: String}
  - name: demo.notes.NotBefore
    kind: newtype
    of: Timestamp
    invariants:
      - value >= "2000-01-01T00:00:00Z"
entities:
  - name: demo.notes.Note
    identity: {name: note_id, type: demo.notes.NoteId}
    fields:
      - {name: since, type: demo.notes.NotBefore}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.notes.Noted
    fields: [{name: note_id, type: demo.notes.NoteId}]
actors:
  - name: demo.notes.Writer
    may: [demo.notes.Write]
commands:
  - name: demo.notes.Write
    input:
      - {name: note_id, type: demo.notes.NoteId}
      - {name: since, type: demo.notes.NotBefore}
    outcomes:
      - name: noted
        creates: demo.notes.Note
        instance: note_id
        sets: {since: input.since}
        emits: [demo.notes.Noted]
        payload: {demo.notes.Noted: {note_id: input.note_id}}
components:
  - component: notes
    owns: {domains: [demo.notes]}
    accepts: {commands: [demo.notes.Write]}
    publishes: {events: [demo.notes.Noted]}
```

The `Note` definition's field rule for `since` lowers to a `compare` `Gte` over two strings; the
input is stored, so D343 U2 does not hold it as a precondition.

## Decisions

- A stored field's rule and an entity invariant are lowered typed, as a command input's rule is
  since D343 U2 (`nominal_condition`'s `input` branch, `Typing::over`): an ordering over a
  `Timestamp` becomes `after`/`before`, and an ordering over text is refused as
  `TextOrderingUnsupported`, the code a typed guard over text already carries. One typing for every
  placement removes the case where the same rule means two things, and entity-core has an instant
  comparison and no text ordering.

## Acceptance

Each case is one test in `cargo test -p ess-entity-runtime --locked --test stored_field_orderings`.

- `a_timestamp_ordering_on_a_stored_field_lowers_to_an_instant_comparison`: lowering component
  `notes` of the model above succeeds, and the `Note` definition's rule for `since` is entity-core
  `after`/`before`, with no `compare` in it. Failing before: the rule is a `compare` `Gte` over two
  strings.
- `a_lowered_stored_timestamp_rule_admits_and_refuses_as_ess_does`: through the lowered definition,
  `demo.notes.Write {note_id: n1, since: "2026-01-01T00:00:00Z"}` is admitted and
  `{note_id: n2, since: "1999-01-01T00:00:00Z"}` is refused by the rule (entity-core answers a
  refusal, not `Unknown`).
- `a_timestamp_ordering_entity_invariant_lowers_to_an_instant_comparison`: the model with
  `since: Timestamp` and an entity `invariants: [since >= "2000-01-01T00:00:00Z"]` in place of the
  newtype lowers to `after`/`before`, and the same two writes are admitted and refused.
- `a_text_ordering_on_a_stored_field_is_refused_by_name`: the model with `NotBefore` over `String`
  and `value >= "m"` returns `Err` with `LoweringCode::TextOrderingUnsupported` naming
  `demo.notes.Note.since`.
- `a_numeric_ordering_on_a_stored_field_still_lowers_to_compare`: the model with `NotBefore` over
  `Integer` and `value >= 2000` lowers the rule to `compare` `Gte` (control).
- `cargo test -p ess-entity-runtime --locked --test lowerable_subset` passes with the subset rows
  and `website/docs/reference/entity-runtime-lowering.md` naming the new tests as evidence.

## Scope

- cited: `crates/generate/ess-entity-runtime/src/lib.rs` (`nominal_condition`, `entity.invariants`)
- inferred: `crates/generate/ess-entity-runtime/src/subset.rs`
- inferred: `crates/generate/ess-entity-runtime/tests/stored_field_orderings.rs`
- inferred: `website/docs/reference/entity-runtime-lowering.md` (rendered from `subset.rs`)

confidence: high

Moves no shared surface.
