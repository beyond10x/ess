---
format: aep.planning-md/3
id: approval-record:openapi-import-open-objects-in-0700
kind: approval-record
status: approved
title: OpenAPI import admits open objects, nullable unions and empty schemas in 0.70.0
summary: 'Operator queue row 33, option A: one unit in 0.70.0 after 0.69.0 ships'
refs:
- provider: github
  reference: beyond10x/ess#647
relations:
- decides: story:openapi-import-admits-open-objects-as-json
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-10-23T13:57:49Z", actor: "human:timo", revision: 2, correlation: "ess-0690"}
---
## Decision

Operator morning queue row 33 (report rows 44 and 45), relayed by the orchestrator on 2026-10-23: "the OpenAPI import refusals become one unit in 0.70.0, after 0.69.0 ships: additionalProperties:true, nullable oneOf -> Optional, {} -> Json; an untagged object oneOf stays refused. Option A ... it does not delay 0.69.0."

## Effect

`story:openapi-import-admits-open-objects-as-json` is proposed for 0.70.0 and is not part of the 0.69.0 batch.

Decider: operator
