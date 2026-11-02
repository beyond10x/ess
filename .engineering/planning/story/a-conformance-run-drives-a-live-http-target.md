---
format: aep.planning-md/3
id: story:a-conformance-run-drives-a-live-http-target
kind: story
status: draft
title: A conformance run drives a live HTTP target
summary: 'Stage 3 of a-command-binds-to-a-protocol-or-deployment-http-route: a downstream adopter runs its suite against a served HTTP target'
relations:
- decomposes: epic:adopter-gaps-wire-and-answers
- serves: vision:O2
- supersedes: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 2
---
## Need

Stage 3 of `story:a-command-binds-to-a-protocol-or-deployment-http-route`, split out by the
2026-11-02 reconcile (store hygiene H0 of wave W1, sections 2d and 2f). Stage 1 shipped in 0.60.0
and stage 2 is implemented (the `http-*` stories); what remains is a conformance run against a
served HTTP target, so a downstream adopter can run its suite against its own deployed service.

## Acceptance

- A conformance run drives a target served over its `ess-http/1` binding and gives the same verdicts
  as the in-process run of the same suite.

## Schedule

Roadmap W9 (0.92.0), unit HTTP3.
