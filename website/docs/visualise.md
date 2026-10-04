---
title: Visualise a specification
sidebar_label: Visualise the billing model
description: The billing example's invoicing domain, drawn from its compiled IR, and the conformance run it obliges.
---

import billingInvoice from '@site/data/billing-invoice.domain-graph.json';
import essSession from '@site/data/ess.terminal.json';

# Visualise a specification

This page draws the normative billing example, `examples/billing/`, from what the compiler makes of
it. Nothing on it is typed by hand: `cargo xtask site-data` runs the real `ess` binary on a copy of
the example, and `task check` fails when a committed recording differs from a fresh one.

{/* ESS-PRESENTATION-SLOT: the <EssPresentation> scenario player and model views replace the domain
graph below once the ess-ui-templates package is pinned. Until then this section shows the
docs-system DomainGraph of the billing example. */}

## The invoicing domain

The `billing.invoice` domain as `ess specify compile` resolves it: each entity with its identity and
fields, the lifecycle it moves through and the commands that move it, and the relations between
entities. Hover or select an entity to trace what it relates to.

<DomainGraph data={billingInvoice} title="billing.invoice" />

## The run it obliges

The same specification synthesizes its conformance suite, and the suite runs against the built-in
`billing` reference implementation:

<Terminal session={essSession} />

Every scenario name in that suite comes from a declaration in the model: an outcome, a lifecycle
transition, a state that refuses a command, an invariant, a binding. See
[Verify conformance](./guides/verify-conformance.md) for the commands and
[A specification and its contracts](./examples/specification-to-contracts.md) for the contracts
generated from the same files.
