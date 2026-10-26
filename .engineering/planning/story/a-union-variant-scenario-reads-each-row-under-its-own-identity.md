---
format: aep.planning-md/3
id: story:a-union-variant-scenario-reads-each-row-under-its-own-identity
kind: story
status: draft
title: A union-variant scenario reads each row under its own identity
summary: An observed identity resolves to the first event, not the latest, so the second variant row is read under the first id; reporter repro union-identity.
tags:
- adopter-report
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which quarantines 9 scenarios for gaps 1 and 2).

A scenario that covers each union variant in turn creates two rows and reads the second under the first event's observed identity: `{"kind":"observed"}` resolves to the first event, not the latest. Reporter repro `union-identity` (1 scenario, fails): the view check expects `box_id = …0001, kind = Plain` where the target correctly holds the second row under its own id.

Reproduced from the reporter's synthesized suite and run report; not re-run against a target here.

## Acceptance

- `a_second_variant_row_is_read_under_its_own_identity`: the neutral repro as a fixture; each variant's view expectation names the identity of the event that created that row, and the reference target passes.
- An observed-identity reference that is ambiguous between two events is resolved to the latest or refused by name; it never silently binds to the first.
