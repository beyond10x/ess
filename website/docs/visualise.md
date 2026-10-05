---
title: Visualise a specification
sidebar_label: Visualise the billing model
hide_table_of_contents: true
description: The billing example played from its compiled IR, its model reference, and the conformance run it obliges.
---

import {EssPresentation} from '@beyond10x/ess-ui-templates';
import billingPresentation from '@site/data/billing.presentation.json';
import essSession from '@site/data/ess.terminal.json';

# Visualise a specification

This page draws the normative billing example, `examples/billing/`, from what the compiler makes of
it. Nothing on it is typed by hand: `cargo xtask site-data` runs the real `ess` binary on a copy of
the example, and `task check` fails when a committed recording differs from a fresh one.

## The billing model, played

The presentation below is written by `ess-ui data` from
[ess-ui-templates](https://github.com/beyond10x/ess-ui-templates) at the commit this site pins.
**Demo** plays the model: every entity's lifecycle on one canvas, with instance tokens moving through
their state machines while a run plays. Pick a run (the authored scenario, a synthesized scenario
or a seeded run), then step, scrub or play it. **Model** is the reference: lifecycles, entities and relations,
commands, events, views, and each declaration linked to its line in the example's source.

Every step it plays was executed at build time from the specification itself. The page only plays
the recorded steps back.

<EssPresentation data={billingPresentation} title="The billing example, presented by ess-ui" />

## The run it obliges

The same specification synthesizes its conformance suite, and the suite runs against the built-in
`billing` reference implementation:

<Terminal session={essSession} />

Every scenario name in that suite comes from a declaration in the model: an outcome, a lifecycle
transition, a state that refuses a command, an invariant, a binding. See
[Verify conformance](./guides/verify-conformance.md) for the commands and
[A specification and its contracts](./examples/specification-to-contracts.md) for the contracts
generated from the same files.
