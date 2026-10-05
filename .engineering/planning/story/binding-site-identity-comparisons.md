---
format: aep.planning-md/3
id: story:binding-site-identity-comparisons
kind: story
status: draft
title: Binding scenarios compare event identity fields with captured instances
tags:
- follow-up
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Binding scenarios (flow, delivery, mapping, on-failure, bounded retry) compare the identity fields of the events they expect against the captured instances, as command scenarios do since beyond10x/ess#273.

## Origin

Follow-up from the #273 merge (2026-10-04). On the integration branch the #273 adversary sweep found four binding scenarios of the binding-arrangement fixture (#266/#267) where `Kicked.job_id` copies an input instance and no step compares it; binding synthesis sites emit no `expect_event_values`. The sweep pins those four misses until this story lands.
