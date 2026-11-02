---
format: aep.planning-md/3
id: story:an-http-binding-projects-its-security-schemes
kind: story
status: draft
title: An HTTP binding projects its security schemes
summary: OpenAPI securitySchemes follow from the authentication port
refs:
- provider: github
  reference: beyond10x/ess#681
relations:
- decomposes: epic:adopter-gaps-wire-and-answers
- informed_by: architecture-decision-record:a-binding-authenticates-the-caller-through-a-port
- serves: vision:O2
revision: 2
---
## Need

beyond10x/ess#681, kept by the 2026-11-02 reconcile (store hygiene H0 of wave W1, section 2f). The
OpenAPI document projected for an `ess-http/1` binding should carry `securitySchemes` that follow
from the binding's authentication port
(`architecture-decision-record:a-binding-authenticates-the-caller-through-a-port`).

## Acceptance

- The OpenAPI projection of a binding whose routes authenticate through a port declares the matching
  `securitySchemes` and per-operation `security`; a binding without one is unchanged.

## Schedule

Roadmap W8 (0.91.0), unit HTTP2.
