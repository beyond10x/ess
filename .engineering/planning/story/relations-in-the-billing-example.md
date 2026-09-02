---
format: aep.planning-md/1
id: story:relations-in-the-billing-example
kind: story
status: implemented
title: The billing example shows an ownership relation end to end
summary: examples/billing carries one owns relation, validated, compiled into the golden fixtures and explained in its README.
owner: ess
tags:
- examples
- relations
relations:
- decomposes: epic:entity-relations
- depends_on: story:relations-projected
revision: 4
---
# Story: The billing example shows an ownership relation end to end

## Outcome

Somebody learning ESS opens `examples/billing/` and sees one relation declared, validated and projected, and the `ess-schema` plugin skill can point at it as the pattern for *an Account owns many CommercialClients*.

## Context

`examples/billing/domains/invoice.yaml` declares `billing.invoice.Invoice` with an identity `invoice_id: billing.invoice.InvoiceId` and typed fields (`:62-82`) and no relation to any other entity. The adopter case that motivated the epic is an ownership relation; the example should carry one of the same shape.

## Acceptance

- One `owns`-kind relation is declared in the example (a customer or account entity owning invoices, or the equivalent the design page chooses).
- `ess validate examples/billing` exits 0 and `ess compile` output for the example is checked into the golden fixtures.
- The example README explains the relation in three sentences.

## Out of Scope

A second example repository.

## Ambiguities

- `inferable` — the example's structure and file names: `examples/billing/{system.yaml,domains/invoice.yaml,domains/email.yaml,components.yaml,topology.yaml}`.

## Open Questions

None.
