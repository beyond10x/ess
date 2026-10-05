---
title: Adopt ESS at the depth you need
sidebar_position: 3
description: Four ways to adopt ESS, from a specification alone to generated behaviour, what each costs and checks, and why a repository may stop at any of them.
status: shipped
lede: Adopting ESS does not mean generating your code. A specification is useful on its own, and each further step is a choice you make per repository.
---

# Adopt ESS at the depth you need

The [Start here](../getting-started.md) path goes from a specification to an OpenAPI contract and a
synthesized TypeScript suite. That is one route, not the only one. A repository can use ESS for
review and diff alone, hold the code it already has to a conformance suite, generate contracts
beside that code, or generate behaviour from the specification.

The modes are independent. Each one works without the ones after it, a repository may stop at any
of them, and two repositories in one system may stop at different ones.

| mode | commands | what you commit | what it checks | what it does not | next step |
|---|---|---|---|---|---|
| specification only | `ess specify validate`, `ess verify diff`, `ess verify impact` | the specification and `ess-inputs.yaml`; see [what to commit](../guides/commit-generated-files.md) | the model is consistent, and what a change breaks and which scenarios and artifacts it owes again | anything about running code | [Write a specification](../guides/write-a-specification.md) |
| conformance against a hand-written implementation | `ess verify conform synthesize`, `ess verify conform run`, `ess verify conform check-history` | the synthesized runner and suite, beside your own target; see [what to commit](../guides/commit-generated-files.md) | that your implementation answers every scenario the specification obliges, and that a concurrent history has an order the model accepts | the runner is not a library you import, and no target is reached over a wire protocol ([the alternatives](./test-pyramid.md#commit-the-generated-runner)) | [Verify an implementation](../guides/verify-conformance.md) |
| generated contracts beside hand-written code | `ess generate --kind openapi`, `asyncapi` or `schema`; `ess generate types` | the projections your consumers read, with `.ess-output`; see [what to commit](../guides/commit-generated-files.md) | the contract is the specification's, and `ess generate --check` fails when the committed copy is stale | that your code honours the contract; a newtype collapses on the wire ([what a projection cannot carry](../guides/generate-artifacts.md#what-a-projection-can-quietly-destroy)) | [Generate contracts](../guides/generate-artifacts.md) |
| generated behaviour | `ess generate synthesize` | the synthesized code your realization links; see [what to commit](../guides/commit-generated-files.md) | every capability is generated, a named obligation, or refused with its reason | durable storage and production authentication stay ports you implement, and synthesis refuses modeled Binary64 on every target ([structural synthesis](./ess.md#structural-synthesis-ess-generate-synthesize)) | [Synthesize code](../guides/synthesize.md) |
| retrofitting, the entry point for an existing repository | the `ess:retrofitting` agent skill, then `ess specify validate` | the specification derived from what you ship; see [what to commit](../guides/commit-generated-files.md) | that shipped behaviour has a specification from its contract, cluster or code | it changes no code and generates nothing on its own | [Use ess with an agent](../start/use-with-an-agent.md) |

A mode further down the table costs more to adopt and checks more. Moving from one mode to the next
changes what you commit and leaves what you committed before in place.
