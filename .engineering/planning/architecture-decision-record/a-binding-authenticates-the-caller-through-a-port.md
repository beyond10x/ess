---
format: aep.planning-md/3
id: architecture-decision-record:a-binding-authenticates-the-caller-through-a-port
kind: architecture-decision-record
status: proposed
title: A binding authenticates the caller through a port; the model keeps authorization
summary: 'Proposes an authentication scheme in ess-http/1 and an Authenticate port in generated servers; ess/N keeps may: and caller unchanged (row 38).'
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 3
---
Status: proposed (draft). Not accepted; nothing under it is dispatched before acceptance.

## Context

ESS states authorization and leaves authentication out:

- An actor's `may:` names which commands and views it may use, and since `ess/16` a command can read
  attributes of the caller (`{caller: account_id}`, `caller.agent_id` in a guard;
  `docs/design/caller-values.md`). Both assume the caller is already known.
- `ess-http/1` states a challenge status and header, "not how a caller proves who it is", and keeps
  `securitySchemes` out of the OpenAPI projection "until the model has an authentication construct"
  (`docs/design/http-presentation-binding.md:625`, `:655-661`; `crates/generate/ess-gen/src/openapi.rs:164-191`).

Adopter repro (beyond10x/a2a, tracker row 38, ESS 0.67.0): A2A's `GetExtendedAgentCard` is
answered only to an authenticated caller. The specification grants the command to an actor, but the
generated server has no step that establishes who the caller is, so the adopter checks the bearer
token by hand before dispatch (a2a `router.rs:612`). Nothing in the suite checks that an
unauthenticated call is refused.

## Options

| | what it is | cost |
|---|---|---|
| A | **Authentication in the model.** `ess/N` actors declare how they authenticate (`bearer`, `api_key`, `mutual_tls`, by name), and the projection emits `securitySchemes` and `security` per operation. | New format version; puts credential mechanics into the domain model, which ADR 02 (`architecture-decision-record:ess-evolution-02-semantics-before-bindings`) keeps transport-free; every binding has to agree with the model's scheme. |
| B | **Authentication in the binding, identity through a port.** `ess-http/1` (and later `ess-jsonrpc/1`, `ess-grpc/1`) declares a scheme per route or per binding; the OpenAPI projection emits it; generated servers call an `Authenticate` port that maps the request's credential to an actor and its caller attributes, or to a refusal with the binding's declared challenge. The model keeps `may:` and `caller` unchanged. | Binding format change (`ess-http/1` gains a key; versioned by the binding, not `ess/N`); a new port in generated servers; conformance gains a binding-level scenario: no credential or a wrong one is answered with the challenge status and no effect. |
| C | **Keep it out.** Document that the implementor authenticates before calling the generated entry. | Nothing to build; every adopter writes and leaves unchecked the same pre-dispatch code. |

## Recommendation

B. How a caller proves who it is depends on the transport; who may do what is domain semantics.
B keeps that split, gives the OpenAPI projection a scheme it can state truthfully, and lets
conformance witness the refusal of an unauthenticated call.

## Effect

- **Format:** `ess/N` unchanged. `ess-http/1` gains an authentication key (scheme name and kind,
  per binding with a per-route override); unknown keys keep being refused.
- **Synthesis:** for every route that declares a scheme, a scenario without a credential expects the
  declared challenge and no event; caller attributes in existing scenarios come from the port's
  test implementation.
- **Generated code:** OpenAPI `securitySchemes` and `security`; Rust servers call `Authenticate`
  before decoding the body, and the actor it returns is checked against `may:`; Go and TypeScript
  follow.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.

## Downstream report (2026-10-23)

- ess#681: an adopter needs `securitySchemes` and a per-operation `security` requirement in the projected OpenAPI. Where the scheme is declared is this decision (A: actors, B: binding), so the projection waits on it.

## Open question: a scheme taken from a composed contract (2026-10-23)

An adopter specifies RFC 6750 bearer usage as its own ESS specification: domain `rfc6750.bearer`, a component `resource-server` reached by network, six `Access*` commands, and refusals that carry `WWW-Authenticate: Bearer` with wire error `invalid_token`.

Under option B, may a binding's scheme (per binding or per route) name a composed authentication contract (a conformance of the component to that specification, the composition the model already has) instead of restating the scheme? The projected `securitySchemes`, the 401 challenge and the binding-level refusal scenario would then come from the contract, and a binding that restates a different challenge would be refused.

To settle before acceptance:

- whether composition can carry transport facts (header, challenge, scheme name) without breaking the transport-free model of ADR 02, or whether the contract supplies only the refusal and its wire error while the binding keeps the scheme;
- how a binding names the contract, and which side owns the challenge when both declare one;
- whether conformance runs the contract's scenarios against every route that names it.

A neutral reproduction of the adopter's specification is available on request.
