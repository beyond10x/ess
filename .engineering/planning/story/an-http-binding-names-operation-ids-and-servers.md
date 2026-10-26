---
format: aep.planning-md/3
id: story:an-http-binding-names-operation-ids-and-servers
kind: story
status: draft
title: An http binding names operation ids and servers
summary: 'Adopter report S5: HTTP binding has no operation_id and no servers (ess#653).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#653
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Problem

`ess-http/1` has no `operation_id` per route and no `servers`. The projected operationId is the qualified command name and the projected documents have no `servers`, so clients generated from them call the wrong URL and name operations differently from the served document. Adopter report S5, ess 0.68.0.

## Acceptance

- `a_route_operation_id_is_projected`: `routes[].operation_id: getNote` appears as that operation's `operationId`.
- `a_duplicate_operation_id_is_refused`: two routes with one `operation_id` are refused, naming both.
- `servers_are_projected`: a top-level `servers: [{url, description}]` appears as the document's `servers`.
- `a_binding_without_the_keys_is_unchanged`: projection without them is byte for byte as before.
