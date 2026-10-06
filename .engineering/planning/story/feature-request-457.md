---
format: aep.planning-md/3
id: story:feature-request-457
kind: story
status: active
title: A view field derived from the lifecycle state
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#457
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/predicate_reference_page.rs
- confidence: cited
  path: crates/specify/ess-domain/src/entity.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/derived_from_state_invariant.rs
- confidence: inferred
  path: website/docs/guides/specify/values-and-views.md
- confidence: cited
  path: website/docs/reference/predicates.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T13:26:30Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T13:26:30Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#457: A view field derived from the lifecycle state.

## Origin
beyond10x/ess#457, filed 2026-10-05; a downstream status read answers four flags computed from the record's lifecycle state, and its view declared none of them.

## Fit review
1. Need: a read model whose fields are a function of the held lifecycle state, with two states that may map to the same value. The specification should say which value each state yields and have that checked. Minimal reproduction, written fresh: a `Ticket` with states `New, Triaged, Active, Waiting, Closed` and a status view answering `live: Boolean`, true in every state except `Closed` (`<fit-review scratch>/probe-457/`). The requester proposed no syntax, only "a view field derived from the lifecycle state by a declared mapping (state to value), checked against every state".
2. Class: convenience. It can be expressed today, at greater length. A view field must name a field of its source entity: `ess 0.52.0` refuses a bare `live: Boolean` on the view with "`probe.ticket.Ticket` has no field `live`, so `probe.ticket.Tickets` promises an observation nothing produces" (`probe-457/refused/`; rule in `crates/specify/ess-domain/src/view.rs:32`). However, an entity invariant may read the lifecycle as `state` (`crates/specify/ess-domain/src/entity.rs:886-891`), so the mapping can be stated and checked on a field the view projects. One documentation defect turned up: `website/docs/reference/predicates.md:29` says an entity invariant reads "the entity's own fields" and does not mention `state`, which the code and the probe both admit.
3. Existing idiom: declare the value as an entity field, write it with `sets:` on every branch that enters a state with a different value, state the mapping as invariants over `state`, and project the field in the view:
   ```yaml
   fields: [{name: live, type: Boolean}]
   invariants:
     - {any: [state != Closed, live == false]}
     - {any: [state == Closed, live == true]}
   ```
   On the installed `ess 0.52.0` (not the 0.53.0 binary), `ess specify validate --path inv-stored` printed `probe v1 — 1 file(s), valid`. Synthesis wrote one `probe.ticket.Ticket/invariant/after/<command>/<outcome>` scenario for each of the 6 outcomes, so the mapping is checked in every state some branch enters. A drifted copy whose `close` branch forgets `sets: {live: false}` also validates (`inv-drift/`). It is caught at run time, not by validate: the interpreter's `BrokenInvariant` ("the model's own outcome leaves an instance violating one of its declared invariants", `crates/verify/ess-conformance/src/interpret/execute.rs:288-293`, `:2321-2330`) is the intended check. On 0.52.0 that scenario reported `unsupported` because that interpreter does not read views. I don't know whether 0.53.0 reports it as broken.
4. Fit: the idiom reuses `sets:`, invariants and view projection unchanged, so it adds no surface. A new mapping key (sketch: `by_state: {New: true, …}` beside `aggregate:` on `RawViewField`, `view.rs:753-772`) would be one more place that reads `state`, after guards, filters and invariants. It would also need a decision for every sibling: `shape:` views, `filter`/`order_by`/`group_by` reading a derived field, aggregates, event payloads, Entity Runtime lowering, and Rust, Go and TypeScript view queries.
5. Second adopter: an order read answering `cancellable: Boolean` (true in `Placed` and `Paid`, false in `Shipped`, `Delivered` and `Cancelled`) and `phase: Open | Done`. The domain fact is general, and the idiom covers it with one stored field and invariants per flag.
6. Cost: the idiom costs a guide section and the one-line reference correction, with no format change. A `by_state:` construct would cost `ess/23`; an optional IR member with an old-reader test (AGENTS.md "Determinism and formats"); a new `ess verify diff` change kind (a changed mapping value breaks readers, likely `ess-diff/15`); generated view-query code on three targets; a synthesis witness per declared state, including states no command reaches (explicit seeds, beyond10x/ess#413); and an Entity Runtime refusal by name.
7. Alternatives: (a) change nothing: rejected, because the idiom is undocumented and the reference page contradicts it; (b) document the idiom and fix `predicates.md:29`: chosen; (c) accept, redesigned, as a total `by_state:` mapping on a view field: real but larger, and it overlaps ground the operator archived on 2026-09-15 (`story:a-field-may-be-derived-rather-than-stored`, retired with the fake-backend epic, `ESS-EVOLUTION.md:16-17` in the workspace root); (d) a Boolean view field defined by a predicate (`holds: state in [...]`), rejected because it covers flags only, not an enum-valued phase. The requester's mapping is answered by the invariant form, which states the same mapping and is checked after every branch.

## Decisions
Decline, with the idiom. The story body is the decline record, with the probe commands and output above. The idiom is a stored field written by `sets:`, with invariants over `state` stating the state-to-value mapping and the view projecting the field. Reply to the requester. In the same change, correct `predicates.md:29` to say an entity invariant also reads the lifecycle `state`. Add the guide section `## A view field derived from the lifecycle state` to `website/docs/guides/specify/values-and-views.md`, after `## A view can be paged`. No format bump. If the operator wants the shorter mapping form, reopen as accept, redesigned (`by_state:`, on the `ess/23` that #429 introduces) after confirming the 2026-09-15 exclusion does not cover it.

## Acceptance
- invariant_reads_state_example: `predicates.md` gains an `ess-check="invariants"` example reading `state` (for example `{any: [state != Closed, quantity > 0]}` over the page's `shop.order.Order`, states `Open, Closed`), run by `crates/edge/ess-cli/tests/predicate_reference_page.rs` with `ess-expect="synthesizes"`.
- invariant_over_state_catches_a_forgotten_set: in `crates/verify/ess-conformance/tests/derived_from_state_invariant.rs`, a model whose branch enters a state without writing the mapped value makes the interpreter report `BrokenInvariant` (`crates/verify/ess-conformance/src/interpret/execute.rs:289`) naming the invariant over `state`, and the faithful model passes. The story closes when this case passes. If the interpreter does not report it, making it report it is part of this story.
- lifecycle_derived_field_guide_section_states_the_idiom: `website/docs/guides/specify/values-and-views.md` has the heading `## A view field derived from the lifecycle state`. The section contains "stored field", "`sets:`", "invariants over `state`", "projects the field" and "every state some branch enters". Its fenced model validates. A case in `crates/verify/ess-conformance/tests/derived_from_state_invariant.rs` reads the page and fails on a missing heading, phrase or invalid model.

## Scope
- website/docs/reference/predicates.md  cited — line 29 omits `state` from what an entity invariant reads; add the tested example
- website/docs/guides/specify/values-and-views.md  inferred — new section after "A view can be paged" (:333)
- crates/edge/ess-cli/tests/predicate_reference_page.rs  cited — runs the new example
- crates/specify/ess-domain/src/entity.rs  cited — `EntitySpec::STATE`, the invariant pseudo-field, unchanged
- crates/verify/ess-conformance/src/interpret/execute.rs  cited — `BrokenInvariant` (289), the check the idiom relies on; edited only if it does not report the forgotten-`sets:` case
- crates/verify/ess-conformance/tests/derived_from_state_invariant.rs  inferred — new test for the forgotten-`sets:` case and the guide section
