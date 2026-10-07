---
format: aep.planning-md/3
id: story:fast-line-topology-central-writer-read-replicas
kind: story
status: draft
title: Topology cannot declare a central writer with per-environment read replicas
tags:
- fast-line
revision: 1
---
## Outcome

A deployment topology can declare one central write path with a read replica in each environment, or ESS documents the supported way to express it.

## Origin

Fast-line intake, 2026-10-04. On 2026-09-23 an agent run against a downstream identity service specification reported that `topology.yaml` cannot express one central write path with a read replica per environment; a later run on 2026-09-25 reported the service has no deployment file yet. Unconfirmed: the claim came from agent output, not from a validated specification.

## Open question

- Confirm against the current deployment/topology schema whether the shape is refused, then fit-review.
