---
format: aep.planning-md/3
id: story:a-component-not-reached-over-the-network-has-no-http-route
kind: story
status: draft
title: A component not reached over the network has no http route
summary: 'Adopter report S2: A component not reached over the network still gets HTTP routes (ess#651).'
refs:
- provider: github
  reference: beyond10x/ess#651
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

`reached_by: network` gates grant projection (`crates/generate/ess-gen/src/http.rs:112`, `crates/generate/ess-gen/src/openapi.rs:558`), but every component's commands and views are still projected as HTTP routes. An adopter's OpenAPI diff carries 24 operations the service does not serve. Adopter report S2, ess 0.68.0.

## Acceptance

- `an_in_process_component_projects_no_routes`: a component without `reached_by: network` contributes no operation to `ess generate --kind openapi`, and the generation report names it as not reached.
- `a_network_component_keeps_its_routes`: a component with `reached_by: network` projects byte for byte as before.
- `an_http_binding_on_an_in_process_component_is_refused`: an `ess-http/1` binding routing a command of a component not reached over the network is refused by name.
