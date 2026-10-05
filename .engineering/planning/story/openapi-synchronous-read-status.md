---
format: aep.planning-md/3
id: story:openapi-synchronous-read-status
kind: story
status: active
title: A synchronous read outcome is answered 200, not 202
refs:
- provider: github
  reference: beyond10x/ess#424
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T18:48:38Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T18:48:39Z", actor: "human:timo", revision: 3}
---
## Outcome

A synchronous read outcome is answered 200, not 202 (beyond10x/ess#424).

## Origin

Opened on GitHub as beyond10x/ess#424; added to the bundle on 2026-10-04 under the operator goal that every open issue is solved and merged through the integration branch. Issue text (first part):

> ## What happens
> 
> The OpenAPI projection answers every outcome without an `error` with `202 Accepted`
> (`crates/generate/ess-gen/src/openapi.rs`, convention table row: `| an outcome with no `error` | `202` | below |`).
> For a command that only reads and returns its answer in the same response, `202` tells an HTTP client the
> request was queued for later processing, which is not what happened.
> 
> ## Reproduction (ess 0.52.0)
> 
> The specification in #423: `catalogue.search.FindTitles`, one
> outcome `found` with `returns: true`, component `reached_by: network`. `ess generate --kind openapi` answers
> `found` as `'202'`.
> 
> ## Expected
> 
> A way for a specification to say a branch is answered synchronously with its result, projected as `200`. For
> example, an outcome with `returns: true` projects `200`, or a command-level declaration that it changes no
> state. Whichever construct ESS chooses, the synthesised HTTP server (`crates/generate/ess-gen/src/http.rs`) and
> the OpenAPI document must agree on the status.
