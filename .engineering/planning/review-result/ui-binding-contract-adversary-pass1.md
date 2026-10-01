---
format: aep.planning-md/3
id: review-result:ui-binding-contract-adversary-pass1
kind: review-result
status: active
title: Adversary pass 1, story:ui-binding-contract (wave ui-live-apps-w1)
relations:
- reviews: story:ui-binding-contract
revision: 1
---
needs-change

Adversary pass 1 on story:ui-binding-contract at 34cb7435a6. Cases executed 118 to 130, red 4, all introduced. Test files: crates/ui/ess-ui/tests/adversary_binding_pass1.rs, crates/ui/ess-ui-check/tests/adversary_binding_pass1.rs.

- warning: a view whose model declares paging still gets a route; no code target serves it.
- warning: a menu from_view (export, channel view, references, completes) on a view with a required param reports no read_params error.
- warning: an explicit store: server/server_session is not refused when the state class is UNMAPPED.
- note: classify keeps a JSON null payload as Some(Null), which does not round-trip; no server emits it.

Held: all 16 recorded vectors match the live Rust and Go gatepass servers byte for byte; routes across two network components, a non-network component, contested commands and short vs qualified names; query wire names, Optional vs required, newtype scalars; classify for declared errors named like surface refusals, declared 404/403/502, 409 vs 422, 202 with error, empty and non-JSON bodies, unknown statuses.

```findings
[{"file":"crates/ui/ess-ui-check/src/model.rs","line":711,"category":"acceptance","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"read_refusals refuses only a document's server paging; a view whose model declares paging, which no code target serves, still gets a route"},
{"file":"crates/ui/ess-ui-check/src/model.rs","line":285,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"read_params runs only for reads that carry params; a menu's from_view (and export, channel view, references, completes) on a view with a required param reports nothing"},
{"file":"crates/ui/ess-ui-check/src/rules.rs","line":750,"category":"boundary","severity":"warning","verdict":"needs-revision","origin":"introduced","message":"placements skips an explicit store: server/server_session when the state class is UNMAPPED, so the server-state refusal never fires"},
{"file":"crates/ui/ess-ui/src/binding.rs","line":149,"category":"property","severity":"note","verdict":"approve","origin":"introduced","message":"classify keeps a JSON null payload as Some(Null), which serialises and reads back as None, so Answer does not round-trip; no server emits it"}]
```
