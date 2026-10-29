---
format: aep.planning-md/3
id: story:a-composition-conformance-may-name-an-event-or-command
kind: story
status: implemented
title: A composition conformance may name an event or command
summary: 'Adopter report N10: compose cannot compare events or commands as types (ess#664).'
tags:
- ess-0.71.0
refs:
- provider: github
  reference: beyond10x/ess#664
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests/compose_conformance_v5.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/specify/ess-composition/src/conformance.rs
- confidence: cited
  path: crates/specify/ess-composition/src/lib.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/fixtures/payload-conformance
- confidence: cited
  path: crates/specify/ess-composition/tests/owner_types.rs
- confidence: inferred
  path: crates/specify/ess-composition/tests/payload_conformance.rs
- confidence: cited
  path: docs/design/composition-type-conformance.md
- confidence: cited
  path: docs/design/review-format-catalog.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/diagnostics.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 12
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T11:01:26Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-10-28T11:01:27Z", actor: "human:timo", revision: 11}
- {from: "active", to: "implemented", at: "2026-10-29T00:14:01Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Problem

A `conformances:` end must name a declared type; an event or command is refused as `unresolved_semantic_reference` (`crates/specify/ess-composition/src/lib.rs:1716-1744`). Adopters cannot prove a local event matches an owner's event. Adopter report N10, ess 0.68.0.

## Fit review

1. Need: a consumer cannot assert that its event, error or command input or response has the same payload as the owner's. Repros kept outside the repository (`/4`): `event.yaml` and `command.yaml` are refused as `unresolved_semantic_reference` ("does not resolve `type demo.gate.Admitted`"), and `event-key.yaml` fails to read with "unknown field `event`, expected `service` or `type`" (0.74.0). The requester proposed only "a conformance end may name an event (its payload) or a command (its input or response)", with no syntax.
2. Class: gap. A conformance end is `TypeBinding {service, type: DeclaredTypeRef}` (`lib.rs:316-341`). Payloads are field lists, not types (`ResolvedEvent.fields`, `ResolvedCommand.input` and `.response`; `ess-compiler/src/ir.rs:2265-2274`, `2027-2042`).
3. Expressible today: only the case where a local command answers a contract command's input, through `/4` `roles: implements` (`src/conformance.rs:95-145`, `docs/design/composition-contract-roles.md`). Events, errors, responses, and a local struct against a payload have no idiom. A copied struct plus a hand-written check moves the figure out of the specification (policy 1, `docs/design/generic-language-features.md:546`).
4. Fit: open the whole row of "declarations that carry a payload", which includes errors as well as events and commands. End keys follow the `EssSemanticRef` kinds: exactly one of `type`, `event`, `error` or `command`. A `command` end also carries a required `part: input | response`. A payload end is compared as an anonymous struct through the existing `Comparison::structs` (`conformance.rs:205-256`), so `reader: true` and the optional tolerance apply unchanged. A struct type end may meet a payload end; any other kind against a payload is drift ("a newtype where … is an event payload"). Admission stays `exports` (`lib.rs:980-999`): an event or command must be on the selected surface. No owned-domain widening for these kinds. Targets: compose is the only reader. `ess verify diff`, the generators, Entity Runtime and the suites never read a composition (`docs/design/composition-type-conformance.md:134-136`; no `composition` hit under `crates/verify/ess-diff` or `schemas/`).
5. A second adopter: a reporting service keeps a local `OrderShipped` event that is decoded from the owner's `orders.OrderShipped` stream. It needs `local: {service: reports, event: …}` and `conforms_to: {service: orders, event: …}`.
6. Cost:
   - Moves:
     - In `lib.rs`: a new `const` for `/5`; `SUPPORTED_COMPOSITION_FORMATS` becomes `[1, 2, 3, 4, 5]` (`:69`); the `validate_format` match and message (`:1621-1636`).
     - The three format-equality gates move to "/4 or later" for `roles` (`:514`, `:1641`) and "/3 or later" for `reader` (`:1660`). Otherwise `/5` refuses `roles` and `reader`. This is the change's safety fact.
     - The `TypeBinding` shape.
     - `ess-cli` help (`crates/edge/ess-cli/src/main.rs:176,182`) and the generated `website/docs/reference/cli.md:549,551`.
     - The `HISTORY` row `("ess-composition", 5, None)` and the doc comment in `crates/edge/ess-xtask/src/docs.rs:114-115,240-243`.
     - Rows in `website/docs/reference/formats.md:170-177`, `spec-versions.md:818-842` and `docs/design/review-format-catalog.md:43-48`; a design section; the CHANGELOG.
   - No JSON schema exists for compositions, so none moves. No diff classification. No new `CompositionCode`. The catalogue meaning of `type_conformance_drift` is widened beyond "field's name, presence or type".
   - The client plan is unchanged, because conformance ends are never added to the surface.
7. Alternatives: (a) change nothing, which leaves the adopter's hand-written check; (b) `semantic: {kind, name}` ends like `references:`, which has no slot for input or response and breaks the `{service, type}` bytes of `/2`-`/4`; (c) chosen: one key per kind plus `part`, where a `type` end serialises exactly as today. Command input against command input is admitted for uniformity. The design page says that `roles: implements` asserts a role and a conformance asserts that two copies agree. Changed from the request: errors added (sibling), `part` required, mixed type-and-payload ends admitted.

## Decisions

Accept, redesigned, at `ess-composition/5` (composition document format; no `ess/N`). It adds new authored keys to a conformance end, so AGENTS.md "Determinism and formats" requires a new version. `story:a-type-conformance-compares-value-constraints-and-variant-attributes` (#671) uses the same `/5`.

A `/1`-`/4` document keeps today's result:

- A `type` end serialises as `{service, type}`, so compiled bytes do not move. Already pinned by `tests/roles.rs:419-423` and `fixtures/contract-role/expected/composition-3.ir.json`.
- An `event`, `error`, `command` or `part` key below `/5` reads as the same unknown-field error as today, followed by a clause naming `/5`. This follows the `RolesBeforeFour` pattern (`lib.rs:494-509`).
- `reader` and `roles` stay admitted from `/3` and `/4` onward, `/5` included.

Note on `/6`: `story:role-input-drift-tells-a-caller-from-an-implementation` (#666) decided to keep its rule and needs no caller stance at `/5`. If its follow-up needs a new reader rule, it is at `ess-composition/6`.

The invariant and variant-attribute half (#671) is `story:a-type-conformance-compares-value-constraints-and-variant-attributes`.

## Acceptance

Command: `cargo test -p ess-composition --locked --test payload_conformance <fn>`.

- `an_event_conforms_to_an_owner_event`: `demo.gate.Admitted` against `wire.server.Subscribed` composes under `/5`.
- `a_command_input_conforms`, `a_command_response_conforms`, `an_error_payload_conforms`.
- `a_struct_type_conforms_to_an_event_payload`: a type end meets a payload end.
- `a_drifting_event_field_is_reported`: `demo.gate.Counted` (`channel: Integer`) is `type_conformance_drift` naming field `channel`.
- `a_reader_event_end_omits_a_field`: `reader: true` on a payload end.
- `a_command_end_requires_its_part` and `a_response_end_on_a_command_without_response_is_refused`.
- `an_event_end_outside_the_surface_is_refused`: `reference_outside_component`.
- `an_older_composition_format_refuses_the_kind`: under `/4`, `event:` is an unknown field whose message names `/5`.
- `v4_documents_keep_their_bytes_and_results`: the contract-role `/4` fixture's IR bytes are unchanged.
- `the_composition_format_admits_every_version` in `tests/owner_types.rs:172` now pins `[1, 2, 3, 4, 5]` (`cargo test -p ess-composition --locked --test owner_types the_composition_format_admits_every_version`).
- CLI: `compose_reads_an_event_conformance` in `crates/edge/ess-cli/tests/compose_conformance_v5.rs` (`cargo test -p ess-cli --locked --test compose_conformance_v5 compose_reads_an_event_conformance`).
- Docs: `task test-xtask` (`cargo xtask docs`, `cli-reference --check`, `diagnostics --check`).

## Scope

- `crates/specify/ess-composition/src/lib.rs`: format const, supported list, gates, `TypeBinding`, `validate_format`, catalogue meaning.
- `crates/specify/ess-composition/src/conformance.rs`: `resolve_conformances` (43-93) admits and compares payload ends.
- `crates/specify/ess-composition/tests/owner_types.rs`: format pin (172-178).
- `crates/edge/ess-cli/src/main.rs` (176, 182); `crates/edge/ess-xtask/src/docs.rs` (114-115, 240-243).
- `website/docs/reference/{formats,spec-versions,cli,diagnostics}.md`, `docs/design/review-format-catalog.md`, `docs/design/composition-type-conformance.md`, `CHANGELOG.md`.
- Inferred: `crates/specify/ess-composition/tests/payload_conformance.rs` and fixture dir `tests/fixtures/payload-conformance/` (needs a command with `response:` and an error); `crates/edge/ess-cli/tests/compose_conformance_v5.rs` (new).
- Shared with `story:a-type-conformance-compares-value-constraints-and-variant-attributes` (`lib.rs`, the CLI test, the docs) and `story:reference-outside-component-names-its-remedy` (`lib.rs` catalogue, `diagnostics.md`). Lands in this order: #664, then #671, with #665 in the same unit.

## Batch 1 integration

- Batch 1 runs U1, U2, U3 and U4 at once (`approval-record:ess-issues-batch-1`). This unit does not edit `CHANGELOG.md`, `changes/*.yaml` or `WHATS-CHANGED.md`: the implementor returns its changelog entry in its report, and the integration branch `integration/ess-issues-1` writes every entry once, after the last unit merges, so four entries under the empty `[Unreleased]` heading do not collide.
