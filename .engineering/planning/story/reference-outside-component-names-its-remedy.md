---
format: aep.planning-md/3
id: story:reference-outside-component-names-its-remedy
kind: story
status: draft
title: Reference outside component names its remedy
summary: 'Adopter report N11: reference_outside_component has no hint for an unowned domain (ess#665).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#665
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

`reference_outside_component` gives a hint only for the owner-declared case; a reference into a domain no selected component owns gets none. Adopter report N11, ess 0.68.0.

## Acceptance

- `the_refusal_names_domain_and_service`: the refusal names the referenced domain and the service.
- `the_refusal_suggests_an_owner`: where another component of the service owns the domain, the hint names it; otherwise it says to add the domain to the selected component's `owns`.
