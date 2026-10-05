---
format: aep.planning-md/3
id: story:entity-runtime-cli-command
kind: story
status: draft
title: ess generate entity-runtime command
tags:
- follow-up
revision: 1
---
## Outcome

ess generate entity-runtime command.

## Origin

Follow-up from beyond10x/ess#231 (2026-10-04). The library entry point lower_component exists; wiring it into the ess CLI links entity-core, whose workspace turns on serde_json arbitrary_precision, changing the released ess binary and the feature-off lane. Decide between linking it into ess (and leaving the feature-off lane) or a separate clap binary in ess-entity-runtime.
