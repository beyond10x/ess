---
format: aep.planning-md/3
id: story:the-openapi-says-published-only-for-events-the-component-publishes
kind: story
status: active
title: The OpenAPI document says published only for events the component publishes
summary: Generated OpenAPI text names an emitted event as published to consumers only when the component publishes it (adopter report 2026-11-04, ess 0.85.0)
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-generated-code
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/fixtures/openapi-http
- confidence: inferred
  path: crates/generate/ess-gen/tests/openapi_publishes.rs
- confidence: cited
  path: generated/go/gatepass/server/pass-service.openapi.json
- confidence: cited
  path: generated/openapi
- confidence: cited
  path: generated/rust/gatepass/crates/gatepass-server/src/pass-service.openapi.json
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T08:47:29Z", actor: "human:timo", revision: 8, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
- {from: "proposed", to: "active", at: "2026-11-04T08:47:29Z", actor: "human:timo", revision: 9, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
---
# Story: the OpenAPI document says published only for events the component publishes

## Outcome

A client author reading a component's generated OpenAPI document is told an event reaches
consumers only when the component declares it under `publishes:`; an event a branch emits that the
component does not publish is described as emitted and kept inside the component, so nobody
subscribes to an event that never arrives.

## Context

A downstream adopter's report of 2026-11-04 on ESS 0.85.0: "published to consumers" is written after
every emitting outcome, also for events absent from the component's `publishes`. The ask: the text
follows `publishes`. `crates/generate/ess-gen/src/openapi.rs` is unchanged between the 0.85.0 tag
and `main` at 8d26c81f74.

## Problem

Three places in `crates/generate/ess-gen/src/openapi.rs` state publication without reading
`component.publishes` (`crates/specify/ess-compiler/src/ir.rs:3116`):

- `:551-558`, the document description: "Events emitted by a branch are published to consumers
  through the event transport, and the `published` property of every response body lists them too";
- `:2131-2139`, each accepting status code's description: "Events this branch emits are published to
  consumers and listed under `published`";
- `:2683-2696`, each operation's outcome text: "Emits {events}, published to consumers and listed
  under `published`."

The AsyncAPI projection already reads `component.publishes` (`asyncapi.rs:896-923`), so the two
documents of one component disagree about the same event.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.

- `crates/generate/ess-gen/tests/openapi_publishes.rs` (new), on a minimal component (invented
  names) whose command has one outcome emitting `Opened` (published) and `Audited` (not published):
  - `an_outcome_text_says_published_only_for_published_events` — the operation text names `Opened`
    as published to consumers and `Audited` as emitted and not published.
  - `an_outcome_emitting_only_unpublished_events_says_nothing_reaches_consumers` — no "published to
    consumers" in that operation's text or status descriptions.
  - `the_document_description_follows_publishes` — the description's publication sentence names the
    component's published events, or says none are, when `publishes` is empty.
  - `the_published_property_description_matches_what_the_runtime_lists` — the `published` response
    property's description says exactly what the generated server puts there (the implementor reads
    the server template and pins it; if the server lists unpublished events, the description says so).
- The committed example trees regenerate (`generated/openapi/*.yaml`, the gatepass
  `pass-service.openapi.json` copies, `crates/generate/ess-gen/tests/fixtures/openapi-http/*`), and
  `task projection-check` and `ess-synth`'s `feasibility`, `adversary_related_guard_behaviour_pass1`
  and `http_binding_rust` pass on them.

## Out of scope

- What the generated server puts in `published` at run time.
- AsyncAPI wording.

## Fit review

1. **Need.** The generated contract tells consumers an event is published when the specification
   says it is not. Requester's ask (theirs): "the text follows `publishes`".
2. **Class.** Defect: the component's declared `publishes` is the authority
   (`ess-compiler/src/ir.rs:3116`), and the sibling AsyncAPI projection follows it.
3. **Already expressible?** The author declared it; the projection ignores it.
4. **Fit.** No new surface: the projection reads a field it already has.
5. **Second adopter.** Any component that emits an internal audit or bookkeeping event it does not
   publish.
6. **Cost.** OpenAPI description text changes in every generated document whose component emits an
   unpublished event (byte change in committed example trees); no schema or format change.
7. **Considered.** (a) change nothing. (b) drop the publication sentence entirely: loses true
   information for published events. (c) chosen: the sentence names what is published.

## Decisions

Accept as proposed (wave lead, 2026-11-04).
