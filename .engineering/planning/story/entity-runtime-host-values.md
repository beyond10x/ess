---
format: aep.planning-md/3
id: story:entity-runtime-host-values
kind: story
status: draft
title: Entity Runtime lowers now, related values and existence through host values
tags:
- follow-up
revision: 1
---
## Outcome

Entity Runtime lowers now, related values and existence through host values.

## Origin

Follow-up from beyond10x/ess#231 (2026-10-04). The lowerable-subset table marks now, {related:} values, unknown_instance and existence selection as needing a host value. Lower each through a host-supplied value instead of refusing it, one construct per slice, with the table row moving from refused to lowered.
