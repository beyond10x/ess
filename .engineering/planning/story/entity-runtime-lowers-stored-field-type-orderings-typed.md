---
format: aep.planning-md/3
id: story:entity-runtime-lowers-stored-field-type-orderings-typed
kind: story
status: draft
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
  path: crates/generate/ess-entity-runtime/tests/
revision: 5
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

## Acceptance

- Lowering component `notes` of the model above either emits the `since` rule as entity-core
  `after`/`before` (an instant comparison), or returns `Err` with a lowering code naming
  `demo.notes.Note.since` and the ordering; no lowered rule is a `compare` over a `Timestamp` or a
  text operand.
- If lowered: writing `{note_id: n1, since: "2026-01-01T00:00:00Z"}` through the lowered
  definition is admitted and `since: "1999-01-01T00:00:00Z"` is refused by the rule (not answered
  `Unknown`).
- The same holds for an entity `invariants:` ordering over a `Timestamp` field.
- A red case first shows today's `compare` on the Timestamp-ordered stored field.
- An ordering over an Integer or Decimal field still lowers to `compare` (control).

## Scope

- cited: crates/generate/ess-entity-runtime/src/lib.rs
- inferred: crates/generate/ess-entity-runtime/tests/ (a test beside `adversary_d343_u2_pass1.rs`)

confidence: high
