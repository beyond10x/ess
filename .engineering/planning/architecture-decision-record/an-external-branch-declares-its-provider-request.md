---
format: aep.planning-md/3
id: architecture-decision-record:an-external-branch-declares-its-provider-request
kind: architecture-decision-record
status: accepted
title: An external branch declares the request it sends to a provider, and an invariant may forbid what it carries
summary: 'Proposes a typed request: on an external branch plus a never_sends: rule checked statically; ess/31 or later and a suite-format bump (row 42).'
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 2
transitions:
- {from: "proposed", to: "accepted", at: "2026-10-23T19:36:15Z", actor: "human:timo", revision: 2, decided_on: {"recorded":{"approval":1}}, correlation: "ess-0690"}
---
Status: proposed (draft). Not accepted; nothing under it is dispatched before acceptance.

## Context

ESS models an outside decision but not what is sent to get it:

- `external:` on an outcome is a free-text cause: "What outside the input decides this branch"
  (`crates/specify/ess-domain/src/command.rs:6956-6958`). It resolves to `External`,
  `ExternalWhen` or `ExternalSubject` (`command.rs:632-668`), and tests inject the branch rather
  than reach it.
- The generated external port gets the typed command input and answers yes or no
  (`docs/design/external-request-binding.md:20`, `story:external-request-binding`). That design is
  "a generated API change, not new authored syntax, an IR or suite format change" (`:8-10`). Its fit
  review rejected an untyped JSON envelope.
- Invariants quantify over the fields of one stored instance (`entity.rs:613-622`) or one value of a
  type (`types.rs:1152-1160`). Events have no invariants (`command.rs:5860-5868`), and nothing
  quantifies over an outbound message.
- `story:field-sensitivity-class` (draft) would label a field `pii.phone`, but it states no rule
  about where a labelled value may flow.

Adopter repro (adopter B, tracker row 42, ESS 0.68.0): a demo `demo.calls.Place` command looks up a
route with a third-party provider before it creates a call. The input carries `caller_number`.
The adopter's rule is data minimisation: "no field of the provider request carries
`input.caller_number`". The specification cannot say what the provider request contains, so it
cannot state the rule. The implementation builds the request by hand, and no scenario fails if the
number leaks into it.

## Options

| | what it is | cost |
|---|---|---|
| A | **A typed provider request on the external branch.** `ess/N` lets an `external:` branch name a request type and map its fields, `request: {type: demo.calls.RouteQuery, fields: {region: input.region, …}}`, from the sources an event payload uses (`PayloadSource`). A command or system invariant may forbid a source from reaching it, `never_sends: [input.caller_number]`. Validation checks that rule statically over the mapping, including through `{expr: …}` and copies. | New `ess/N` (two keys); the generated port receives the request value that generated code built from the mapping, not the raw input, so an implementation cannot add a field; conformance gains a recorded-request assertion, which needs a suite-format bump. |
| B | **The provider call as an emitted event.** Model the outbound call as an event with a payload and use `field-sensitivity-class` with a flow rule: "a field of class `pii.phone` reaches no event delivered to `<provider>`". | Reuses the event payload machinery, but an event is a fact the system publishes, not a question it asks and waits on. It ties to an unaccepted sensitivity vocabulary, and it confuses the event projections (publishers, AsyncAPI) with a request/response call. |
| C | **Keep it out.** Document that outbound payloads belong to the implementation and that data minimisation is checked in code review. | Nothing to build; the rule stays prose, and nothing checks it. |

## Recommendation

A. The request is part of the branch's meaning, since the provider decides on what it is sent. A
typed mapping is declarative data that validation can check without running anything. That makes
"never sends X" a decidable static rule over the declaration, not a runtime hope. It extends the
typed-port direction of `story:external-request-binding` rather than reopening the rejected untyped
envelope. Field sensitivity can later name sources by class (`never_sends: [class: pii.phone]`),
but the rule does not wait for it.

Boundaries: one request per external branch, no response mapping (the answer stays the branch
selected), and no transport or URL. Those belong to a binding (ADR 02,
`architecture-decision-record:ess-evolution-02-semantics-before-bindings`).

## Effect

- **Format:** next `ess/N` (`ess/31` if this is accepted before the release carrying `ess/31` ships,
  since that format is still open): `request:` on an `external:` branch and `never_sends:` on a
  command and on the system. Below it both are refused naming `ess/N`. A model without them keeps
  its IR bytes. A mapping that reads a forbidden source, directly or through an expression or a
  copied field, is refused as `conflicting_declaration`, naming the path.
- **Synthesis:** each scenario that takes an external branch with a `request:` asserts the request
  the test port recorded equals the mapped value. For each `never_sends:` source, one scenario sends
  a distinctive value there and asserts that no recorded request field contains it, so an
  implementation that copies it anyway fails. The suite format gains the recorded-request
  observation.
- **Generated code:** Rust and Go generate the request type and build it from the mapping before
  calling the external port, whose signature takes the request; the OpenAPI projection is unchanged
  (the call is outbound); Entity Runtime refuses `request:` by name until it carries ports.

## Acceptance evidence

None yet. Acceptance is the operator's move of this record to `accepted`.
