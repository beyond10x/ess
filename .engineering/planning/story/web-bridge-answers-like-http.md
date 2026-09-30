---
format: aep.planning-md/3
id: story:web-bridge-answers-like-http
kind: story
status: draft
title: The web bridge answers a command with the HTTP surface's shape
relations:
- serves: vision:O2
revision: 1
---
## Outcome

The web bridge answers a command with the same shape as the HTTP surface: `outcome`, flat `error`/`payload` on a refusal, and `published`.

## Acceptance

- The bridge's refusal is no longer nested under `refusal:`; the generic page and `bridge.js` read the flat shape; the browser lab tests pass.
- One test compares a bridge answer with the HTTP answer for the same command and input.

## Origin

Aligning every surface on `published` (story generated-server-publishes-and-reads-headers) left the bridge's refusal nested, as it was before.
