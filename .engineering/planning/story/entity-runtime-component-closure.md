---
format: aep.planning-md/3
id: story:entity-runtime-component-closure
kind: story
status: draft
title: Entity Runtime lowering drops a command whose subject is outside the component without a diagnostic
tags:
- follow-up
revision: 1
---
## Outcome

Entity Runtime lowering drops a command whose subject is outside the component without a diagnostic.

## Origin

Found while implementing beyond10x/ess#231 (2026-10-04): when a command's subject entity is outside the component's closure, lower_command drops the command and reports nothing. It should refuse it by name with the reason.
