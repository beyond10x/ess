---
format: aep.planning-md/3
id: story:a-view-parameter-carries-its-wire-name-on-every-http-reader
kind: story
status: implemented
title: A view parameter is named on the wire by its naming.wire in the projection, the generated servers and the runner
summary: Adopter found no wire-name override for a view query parameter (snake_case vs camelCase); naming.wire appears to cover it in OpenAPI, unverified elsewhere (a2a wave 2, H7, 2026-10-22).
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T04:42:46Z", actor: "human:timo", revision: 5, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T04:42:47Z", actor: "human:timo", revision: 6, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:39Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H7**, a2a wave-2 HTTP binding gaps list, line 14. View query
parameters go on the wire in snake_case (`context_id`); A2A 11.5 requires camelCase (`contextId`).
No refusal: "no wire-name override for a view parameter found".

## Where it lives

Possibly already expressible, not verified end to end:

- A view's `params` are `Vec<Field>` (`crates/specify/ess-domain/src/view.rs:969`), and a `Field`
  carries `naming` (`crates/specify/ess-domain/src/types.rs:484-486`) whose `wire` is "what this is
  called on the wire" (`crates/specify/ess-domain/src/name.rs:202-204`).
- The OpenAPI projection names a view's query parameter by `types::wire_name(param)`
  (`crates/generate/ess-gen/src/openapi.rs:2994-2999`, `crates/generate/ess-gen/src/types.rs:710-711`),
  which is `naming.wire` when given.

So `params: [{name: context_id, type: …, naming: {wire: contextId}}]` should project `contextId`.
Not checked: whether the generated servers and the conformance runner read the same wire name for a
view parameter. If they do, the outcome is an answer to the adopter plus documentation (the view
parameter section does not say `naming` applies); if not, it is a defect in those readers.

## Format

None if `naming.wire` already covers it. A per-binding override in `ess-http/1` is the alternative
only if the model-level wire name is wrong for another transport.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22), checked on 0.68.0.

- Verified on 0.68.0 for the generated Rust server: a view parameter declared `wire: ownerName` (the `naming` keys are flattened onto the field; `naming: {wire: …}` on a view field is refused as `unknown field \`naming\``) is read from the query string as `ownerName` by the generated route, and a view field declared the same way is written as `ownerName` in each row. So the authored form in the Where-it-lives section above should read `wire: contextId`, not `naming: {wire: contextId}`.
- Still not checked: the generated Go server and the conformance runner.
- The rest of the adopter's view-shape report (the `rows` envelope, cursor paging, row-shaping parameters, state wire values) is filed separately as `story:a-view-answer-takes-its-protocols-list-shape`.
