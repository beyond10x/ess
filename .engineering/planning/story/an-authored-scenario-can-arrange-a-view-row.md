---
format: aep.planning-md/2
id: story:an-authored-scenario-can-arrange-a-view-row
kind: story
status: archived
title: An authored scenario can arrange an entity instance, so a read no command populates is checkable
summary: 'arrange: carries no fields, so an entity created by no command has no reachable view and every assertion over it is vacuous. Measured: five entities, nineteen views, no backend-sourced read checkable at all.'
relations:
- informed_by: task:ess-gaps-measured-in-a-consumer-specification
revision: 3
---
**Folded into `task:ess-gaps-measured-in-a-consumer-specification` on 2026-09-11**, at the operator's
request that all eight measured gaps arrive as one task for the maintainer to decompose. The body below is
kept because it carries the full argument, the measurements and the candidate resolutions; the task
summarises it. Archived rather than deleted so the record of what was drafted survives.

---

## What is missing

An authored scenario's `arrange:` names instances and their entity; it carries no fields
(`Arrangement` in `crates/verify/ess-conformance/src/authored.rs`). So the only way a row reaches a view is a
command in the timeline that creates it.

An entity that **no command creates** therefore has no reachable view, and every assertion over that view is
vacuous: an implementation that returns nothing passes.

Measured in an internal specification repository, 2026-09-11, while porting an acceptance
inventory. Five entities are created by no command — a call record, a transfer target, an SMS message, a
campaign, and every entity of the domain that models the upstream service. Between them they are the source
of the call-history view, the transfer-target view, the SMS-thread view, the active-campaign view and
fourteen more. **A conformance suite for that system cannot check a single one of its backend-sourced
reads.** The rows that would have covered them were recorded as unexpressible rather than written vacuously.

This is not only that repository's shape. Any system whose reads are populated by an upstream it does not
own has the same hole: the reads are precisely what a consumer wants checked, and precisely what cannot be
arranged.

## What would settle it

`arrange:` that can state a row, checked against the entity's declared fields:

```yaml
arrange:
  - instance: earlier
    entity: billing.invoice.Invoice
    fields: {amount: {amount: 120, currency: EUR}, issued_at: 2026-01-05T09:00:00Z}
```

Properties worth keeping: every field resolves against the entity's declaration, so a typo is refused at
`conform author` rather than at the first run; identity fields are required; and a target is asked to
establish the row through whatever seam it has, so the arrangement is a demand on the target and not a
fiction the runner maintains.

## Acceptance

- [ ] An authored scenario can arrange an entity instance with declared field values, refused when a field
      or a value does not resolve.
- [ ] The conformance target is asked to establish it, and a target that cannot answers `ErrUnsupported`
      rather than passing silently.
- [ ] A view over an entity no command creates is checkable, and an implementation returning nothing fails.

## Out of scope

Arranging a row that violates the entity's own invariants; arranging lifecycle history rather than state.

## Provenance

Filed 2026-09-11 from an internal application repository wave 3, `story:read-side-entities-need-a-creating-cause`
and `review-result:adversary-authored-scenarios-pass-1`.
