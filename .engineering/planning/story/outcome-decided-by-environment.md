---
format: aep.planning-md/1
id: story:outcome-decided-by-environment
kind: story
status: draft
title: 'An outcome a caller cannot see: refusals decided by neither input nor entity state'
summary: 'Design first: how a command states a refusal that depends on environmental fact (caller network, deployment allow list) so it is neither a nondeterministic branch nor UNMAPPED'
owner: timo
tags:
- consumer-integration
- design-first
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
revision: 1
---
## Context
`ESS-COMMAND-004` refuses two unconditional outcomes and `ESS-COMMAND-012` refuses `wrong_state` on a command that moves no entity, so a command's branches must be decided by input (`when`) or by the lifecycle state of the entity it moves. `specs/services/pusher` (the consumer repository, 2026-09-10) met a refusal that is neither: the pusher's HTTP publish endpoint answers 403 when the caller's source address is outside a configured CIDR list (src/HttpEndpoint.js:74-79). The address is not an input of `Publish` and no entity carries it; the 403 is now UNMAPPED prose in interfaces.md, and the model claims `Publish` has one outcome. Deployment allow lists, rate limits and feature flags are the same shape.

## Acceptance
A design page under docs/design decides one of: (a) an explicit `precondition:` outcome kind that names an environmental fact the implementation evaluates, is rendered in docs/OpenAPI as a refusal, and yields a conformance obligation the target answers rather than a scenario the suite synthesizes; (b) the fact is modelled as input the binding must map from a declared source; (c) the refusal stays outside the executable subset by rule, with the rule written where authors will find it. The page cites the pusher case and states what each option costs in determinism, synthesis and conformance. No code lands until the page is reviewed.

## Scope
Cited: crates/specify/ess-domain (outcome validation for COMMAND-004/012), docs/design/ess-closed-loop-execution-conformance-design-v0.1.md (G14 and the determinism argument), docs/design/ess-review-v0.1.md.

## Boundaries
This story delivers the design page and a decision, not the construct. It must not weaken COMMAND-004 or COMMAND-012 for the existing cases. It must not introduce a generic "any condition" escape hatch.

## Verification
The design page exists, names the three options with the pusher example, and records the decision and its reviewer; `task check` unaffected.
