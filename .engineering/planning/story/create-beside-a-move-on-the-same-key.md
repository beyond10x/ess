---
format: aep.planning-md/3
id: story:create-beside-a-move-on-the-same-key
kind: story
status: draft
title: One outcome creates a record and supersedes the current record on the same key
revision: 1
---
## Summary

An adopter keeps one current record per key by never editing a record: a change is a new record, and
the current record on the same key moves to a superseded state. The first record on a key supersedes
nothing. ESS 0.59.0 cannot state this in one outcome:

- `affects:` beside `creates:` is refused (ESS-COMMAND-009, `unsupported_construct`: a secondary
  effect is admitted beside `moves`, `updates` and `deletes` only);
- an `affects:` entry cannot create (its fields are `entity`, `where`, `each`, `instance`, `sets`,
  `moves`, `deletes`).

Each half validates alone. Split into two commands, the model either loses the atomicity (a window
with two current records, or none) or leaves the supersede transition without a cause
(ESS-ENTITY-005, `missing_causation`).

Wanted shape, as the adopter drafted it:

```yaml
outcomes:
  - name: accepted
    creates: demo.Ceiling
    instance: id
    affects:
      - entity: demo.Ceiling
        where: {all: [agent_id == input.agent_id, organization_id == input.organization_id]}
        moves: demo.Ceiling.supersede
```

No GitHub issue: this comes from an adopter's report.

## Spec first

The change is modelled in this repository's ESS specification and validated with the newest `ess`
before any code. A design section in `docs/design/set-effects-over-filtered-instances.md` comes first
and settles: whether the `where:` selection may match zero rows (it must, for the first record) and at
most one row, or any number; that the created record is excluded from the selection; the order of the
emitted events; and the format consequence (ess/25 if a document that was refused now reads).

## Acceptance

Each line names the failing test that is written first:

- `crates/specify/ess-domain/tests/create_beside_affects.rs::a_create_beside_a_move_on_the_same_key_validates`:
  the shape above validates, and the supersede transition counts as caused.
- `crates/verify/ess-conformance/tests/create_beside_affects.rs::the_interpreter_creates_and_supersedes_in_one_step`:
  with one current record on the key, one execution leaves the new record current and the old one
  superseded; with none, it creates only.
- `crates/verify/ess-conformance/tests/create_beside_affects.rs::synthesis_witnesses_the_first_and_a_later_record`:
  synthesis emits a scenario with no prior row and one with a prior row on the key.

## Out of scope

Creating several records from one outcome; an `affects:` entry that creates.
