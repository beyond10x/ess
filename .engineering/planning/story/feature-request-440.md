---
format: aep.planning-md/3
id: story:feature-request-440
kind: story
status: active
title: Binding inputs read from stored state at consume time
tags:
- ess-0.54.0
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#440
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- depends_on: story:feature-request-445
scope:
- confidence: inferred
  path: crates/specify/ess-domain/tests/binding_stored_read_idiom.rs
- confidence: cited
  path: docs/design/filtered-related-reads.md
- confidence: cited
  path: website/docs/guides/specify/bindings-and-components.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-05T20:40:15Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":2}}}
- {from: "proposed", to: "active", at: "2026-10-05T20:40:16Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":2}}}
---
## Outcome
Resolve beyond10x/ess#440: Binding inputs read from stored state at consume time.

## Origin
beyond10x/ess#440, filed 2026-10-05; reported from an adopter's specification, where an event consumer computes a flag from the stored record the event names.

## Fit review
1. Need: when an event is consumed, the command it causes must use a value held in stored state at that moment (for example the current `bridged` flag of the call a `LegJoined` event names), not a value carried in the payload. The requester proposes a binding `mapping:` source that reads one field of a record the event selects, like `{related: …}` does for commands. That syntax is theirs. Minimal reproduction: `<fit-review scratch>/probe-440/`. A mapping entry `call_bridged: {related: {via: call_id, field: bridged}}` is refused: `components.yaml: unknown field 'related', expected 'selection' or 'path'` (installed ess 0.52.0; 0.53.0 head reads the same object shape, `crates/specify/ess-domain/src/binding.rs:833-879`).
2. Class: convenience. The fact can be stated today, on the invoked command. The binding guide says accessors do not "compute values, or read session context" (`website/docs/guides/specify/bindings-and-components.md:192`). This is a documented limit, not a defect.
3. Existing idiom: the binding maps the identity (`leg_id: event.leg_id`), and the invoked command reads the stored record. It can use `{related: {via, field}}` in `sets:`/`payload:` (`docs/design/value-expressions.md:224-266`, ess/16), `when_related`/row-set guards and `{related: {entity, where, field}}` (`CHANGELOG.md:42`, 0.53.0, ess/22), or a stored-subject guard. The probe validates (`probe v1 — 3 file(s), valid`). Synthesis writes 8 scenarios, and the `MarkLeg/marked` scenario asserts `expect_event LegMarked {"call_bridged": true}`, a value read from the referenced `Call` row. The 2 synthesis refusals are probe gaps (no `wrong_state`, `drop` policy), not about the read. The snapshot rule already exists: the store "immediately before branch selection", one snapshot for every guard and value of the command (`docs/design/filtered-related-reads.md:10-13`), and E8 reads "as it is when the outcome runs" (`value-expressions.md:226`).
4. Fit: a binding-side store read would add a second snapshot. The binding would read at consume time and the command would read again at its own branch selection, so the mapped value and the command's guards could disagree. It would also break redelivery: "A redelivery carries the context of the occurrence it repeats" (`bindings-and-components.md:302-303`), but a store read made again on redelivery may return a different value. A binding belongs to no component (`components.yaml`, above the domains), so it has no owned store to read from. Every target would need a store port in its binding adapters: the interpreter (`crates/verify/ess-conformance/src/interpret/bindings.rs:320-340`), synthesis (`synthesize/binding_effects.rs`), `ess-synth` (`plan.rs:1470-1490`), AsyncAPI and docs. With the command-side read, all of that already exists.
5. Second adopter: a `ParcelScanned{parcel_id}` event causes `RecordScan`, which must record the parcel's current destination region. That is the same need, and the same idiom answers it: `region: {related: {via: parcel_id, field: region}}` on `RecordScan`. Whether a related read may cross into another domain's entity: I don't know (not probed). If it may not, the remaining idioms are a field on the event, where the publisher holds the value, or a declared authority, which is the guide's own answer for a value the event does not carry (`bindings-and-components.md:192-194`).
6. Cost of the request as proposed: a new mapping source variant (`MappingSource`, `binding.rs:884`; `ResolvedMappingValue`, `crates/specify/ess-compiler/src/ir.rs:2116`), source format ess/23, a store port in every binding lane, a new snapshot rule, synthesis arrangement for binding reads, and new diff classifications. Cost of the decline: one guide section and one test. No format bump.
7. Alternatives: (a) change nothing: the need is met, but the refusal is a serde message that names neither the idiom nor the reason. (b) Proposed: a `{related:}` mapping source. Refused, for the double snapshot, redelivery and ownership reasons above. (c) Chosen: decline, with the idiom documented.

## Decisions
Decline, with the idiom. The story body is the decline record. A binding maps only the event payload, the delivery context, host context or read, and selections. A value read from stored state is read by the invoked command, under the command's one pre-branch snapshot (`{related:}`, `when_related`, stored-subject guards). Built: the section `## Read stored state in the command a binding invokes` in `website/docs/guides/specify/bindings-and-components.md`, after `## Read a field inside an event envelope`, with the probe's model. Reply on the issue. No format bump (ess/22 and ess-conformance/43 unchanged). Depends on #445 (edge recorded): both edit `bindings-and-components.md`, and this section lands after #445's literal-mapping text.

Noted, not in scope: the refusal of a `{related: …}` or `{subject: …}` mapping value is a serde message (`unknown field 'related', expected 'selection' or 'path'`, `binding.rs:833-879`) that names neither the idiom nor the reason. The issue did not ask for a diagnostic change.

## Acceptance
- binding_command_side_related_read_validates_and_asserts_value: the guide section's fenced model validates, and synthesis asserts the referenced row's value on the invoked command's event.
- binding_stored_read_guide_section_states_the_idiom: `website/docs/guides/specify/bindings-and-components.md` has the heading `## Read stored state in the command a binding invokes`. The section contains "`{related:`", "`when_related`", "one snapshot", "redelivery" and "no store" (a binding belongs to no component). Its fenced model is the one the case above validates. The case reads the page and fails on a missing heading, phrase or model. It lives in `crates/specify/ess-domain/tests/binding_stored_read_idiom.rs`.
- binding_mapping_related_shape_still_refused: `x: {related: {via, field}}` in `mapping:` is still refused, with today's message.

## Scope
- website/docs/guides/specify/bindings-and-components.md  cited — new section beside the accessor limit (192)
- docs/design/filtered-related-reads.md  cited — snapshot rule the guide cites
- crates/specify/ess-domain/tests/binding_stored_read_idiom.rs  inferred — idiom, guide-section and refusal-kept cases
