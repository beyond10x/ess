---
format: aep.planning-md/3
id: architecture-decision-record:changes-since-a-position-are-a-view-filter
kind: architecture-decision-record
status: accepted
title: Changes since a position are a view filter over a tracked change order
summary: 'Proposes changes: tracked on an entity and a changed_since view filter with a published position; history of field values stays a marker (row 21 group M).'
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:16Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: accepted 2026-10-23 by the operator, after reviewing all 15 proposed ADRs in full: "I accept all of them" (see this record's transitions).

## Context

A stored field holds its current value only. A guard compares an input with that value
(`when_subject:`), and a view reads current rows. Nothing in `ess/N` keeps earlier values or orders
changes, so two rules cannot be stated (no design page covers them; a search of `docs/design/` for
"watermark" finds only `periodic-binding-triggers.md`).

Adopter repro (beyond10x/connectors, 2 `ESS-LIMIT` markers of the 126 triaged in tracker row 21,
group M):

- `ess/domains/feed.yaml:14`: "a read with the last `next_watermark` returns every item created or
  changed since, and `next_watermark` never passes an undelivered change". Scenarios pass a stand-in
  watermark; the rule is left to the binding's own tests.
- `ess/domains/feed.yaml:22`: "a new revision the item has not carried before" is expressible only
  against the revision held now (`RevisionUnchanged`); a revision reused from an earlier change is
  not refused.

## Options

| | what it is | cost |
|---|---|---|
| A | **A change position per entity and a "changed since" view filter.** An entity may declare `changes: tracked`; every create and update advances a position the system mints in order; a view may filter `changed_since: input.<field>` and publish the position it reached. | New format version; the interpreter and generated stores keep a monotonic position per row; synthesis witnesses order (a change after the watermark is returned, one before is not) and that the published position never passes an undelivered change. Covers `feed.yaml:14`. |
| B | **Earlier values of one field.** A field may declare `history: kept`; a guard may test `input.x not in history(field)`. | New format version; storage grows with every write of the field; covers `feed.yaml:22` only; synthesis needs two earlier writes per witness. |
| C | **Keep current-value semantics.** Document both rules as binding obligations, as the adopter does now. | Nothing to build; both rules stay unchecked by any suite. |

## Recommendation

A. A change feed with a resume position is a common protocol shape, and A gives it one checked
construct. The position can reuse the minted-value machinery of
`story:a-minted-value-is-stored-and-published-by-generated-code` (not yet checked against that
story's design). B serves one marker at an unbounded storage cost; leave it as a marker until a
second adopter needs it.

## Effect

- **Format:** next open `ess/N`: `changes: tracked` on an entity, `changed_since:` in a view filter,
  and a published position field; older formats refuse each by name.
- **Synthesis:** scenarios that write rows on both sides of a captured position and expect only the
  later ones; a scenario that the published position does not pass a change that is still to come.
- **Generated code:** the ephemeral store and the interpreter keep a position per row; the storage
  port's `put` receives the minted position; Go and TypeScript follow Rust.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.
