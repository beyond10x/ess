---
format: aep.planning-md/3
id: story:feature-request-393
kind: story
status: active
title: Realize existing event payloads as standalone typed libraries
refs:
- provider: github
  reference: beyond10x/ess#393
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/model_types.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/model_types.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/generate/ess-gen/src/model_types.rs
- confidence: cited
  path: crates/generate/ess-gen/src/schema.rs
- confidence: cited
  path: crates/generate/ess-gen/tests/fixtures/model-event-types.yaml
- confidence: cited
  path: crates/generate/ess-gen/tests/model_types.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize.rs
- confidence: cited
  path: crates/generate/schema-contract/tests/event_payload_types.rs
- confidence: cited
  path: docs/design/event-payload-type-roots.md
- confidence: cited
  path: docs/design/types-only-realizations.md
- confidence: cited
  path: website/docs/guides/generate-artifacts.md
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/formats.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T22:51:43Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T22:51:59Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
---
# Issue 393: event payload roots for data realization

Prepared for root's accept-redesigned decision; no AEP or production changes. Source inspection at caller checkpoint 9050c95cf65be4d9e184d0abdf9fd53608c15265. All paths below are repository-relative. Read `.agents/skills/assessing-external-requests/SKILL.md`.

## Fit review

1. **Need independent of syntax.** A producer needs the already-declared event record as a standalone typed library without a second struct declaration whose fields can drift. Requester's alternatives are authored `payload: <struct>` or allowing event selection through `generate types`. Minimal retained `event.yaml` declares `audit.entries.Entry` with integer sequence and `audit.entries.EntryRecorded` with nested Entry plus a producer field wire-named producer_name. Existing cached CLI reports ess 0.51.0, SHA256 `0adc9b2a340adab8a205cd9834bde81f5f4cdf18f73b2b2f81aff959540afe89`; exact build commit is not established, so this is a retained-binary reproduction supported separately by current source inspection. `specify validate --path event.yaml` exits0. `generate types --path event.yaml --root audit.entries.EntryRecorded --target typescript --out event-output` exits1: `unknown_root: no resolved model type has this qualified name`, creates no output directory. The identical command selecting audit.entries.Entry exits0 and writes a types-report/3 library. Logs and exit files reside beside this review. No rebuild was needed.

2. **Class.** A projection/realization gap, not an authored-language gap: EventSpec already owns typed fields (`ess-domain/src/command.rs:3915–3938`), and JsonSchema emits event payload documents through the shared type mapper (`ess-gen/src/schema.rs:104–154`, `types.rs:920–930,1002`). Current `ModelTypes::select` deliberately searches only ir.types and emits unknown_root for events (`ess-gen/src/model_types.rs:35–59`); documentation promises named type roots (`docs/design/types-only-realizations.md:16–43`). No new authored surface is justified for this need.

3. **Existing expression/idiom.** The event is expressible and validated. `ess generate schema` already provides its record schema; AsyncAPI embeds that same record and reachable definitions (`ess-gen/src/asyncapi.rs:1062–1075`). The desired standalone library is unavailable from the event root: demonstrated failure above. Duplicating its fields into a named struct would generate a library but does not establish equality and is not a satisfactory idiom. No exact #393 AEP duplicate was found in canonical story/specification/decision-blocker search. Important correction to the initial intake: asyncapi.rs:85–90 claims names are unique per kind, but this comment is stale for current authored admission. The actual `same-name.yaml` type+event collision exits1 with duplicate_declaration, consistent with `ess-domain/src/system.rs:844–885`: a qualified name has one owner across kinds. Preserve that rule and test it; do not invent a synthetic event schema prefix merely to address an inadmissible source collision.

4. **Design fit and siblings.** Add explicit repeatable `--event QUALIFIED_EVENT` beside existing repeatable `--root QUALIFIED_TYPE`. `--root` remains type-only, including its existing failure for an event name; `--all-types` remains named-types-only and exclusive with both explicit selectors. Explicit --root and --event may be combined, selecting one shared transitive closure. In clap, make the required selection group allow multiple explicit args, while all_types conflicts with root/event. This fits the existing root-selection workflow without guessing kind or changing old invocations (`ess-cli/src/model_types.rs:8–35`). All existing targets—Go, Rust, TypeScript—consume the same sealed Plan, so implementation belongs before target emission, not in separate serializers (`schema-contract/src/realize.rs:207–235`). Events use Message::of_event and message, preserving the same closed object, optional presence, wire keys, nested types and annotations used by JSON Schema/AsyncAPI; no authored event/binding/outcome behavior changes. This issue does not introduce broker or batch-envelope authority (#390–392). Code generation must include actual Go/Rust codec execution and TypeScript compilation/runtime-obligation accounting, not just rendered names. Keep conformance behavior unchanged because the source semantics do not change; validate unchanged model/suite bytes for an existing fixture. Event model provenance must use EventRef seeds and reachable types; it cannot disguise the root as DeclaredTypeRef (`ess-compiler/src/refs.rs:120,408`; `ess-gen/src/provenance.rs:232,309`).

5. **Second adopter.** A sensor exporter and a separate archive reader share a ReadingRecorded event library: nested sensor identity, optional calibrated measurement and exact integer sequence must have one wire record. This is independent of the broker-usage example and needs no new syntax.

6. **Cost and compatibility.** No authored ESS, IR, suite, OpenAPI or AsyncAPI format change. Preserve existing type-only and bundle realization outputs byte-for-byte, including types-report/3. The published report/3 roots mean model types today (serialize-only, no admission reader: website/docs/reference/formats.md:219), so new event root identity should not silently reinterpret /3. Proposed conditional **ess-types-report/4** for selections containing an event, with a `model_roots` field listing typed `{kind: type|event, name: qualified-name}` roots. Existing `roots` remain exact schema-definition keys, and `declarations` remains the host-name mapping; type-only/bundle outputs omit model_roots and retain /3. Use a concrete Rust enum for typed root metadata, not property bags. The current report writer is realize.rs:403; report/3 was released in0.19.0 (ess-xtask/src/docs.rs:277), so this is not an unpublished version that can be repurposed. Search of current carrier code/docs found no existing /4 allocation. Register /4 as unreleased in format metadata/docs; root must coordinate if another unit allocates it. No format may claim successful event realization by merely widening a string's undocumented interpretation. New Rust API is additive: retain ModelTypes::select for old type callers; add typed-root selection. Generated event host declarations are new APIs, while old type names and schema keys remain unchanged.

7. **Alternatives.** (a) Change nothing and duplicate structs: smallest implementation, but drift remains uncheckable. (b) Add event `payload: NamedStruct`: permanent authored syntax, validation/diff/synthesis/runtime impact, and still not necessary to realize existing field declarations. (c) Make --root search both kinds: technically unique after current admission, but changes the documented selector contract and leaves root kind implicit; rejected for compatibility and evidence clarity. (d) Explicit --event through shared ModelTypes selection: preferred, no language change, straightforward provenance and closure. A generic --kind flag was considered but makes mixed selections awkward and repeats kind outside each root; a new typed URI root grammar is unnecessary CLI surface. Choose (d).

## Decision proposal

Accept, redesigned: select existing event payloads with `ess generate types --event`, keep existing --root behavior, and use the established schema/realization mapping. Do not add authored payload syntax, publisher connection behavior or envelope semantics in this unit. The root must record the decision and scope in AEP before implementation. No issue comment was sent by this worker.

## Narrow implementation design

- Introduce public ModelRoot enum Type/Event carrying existing QualifiedName identity. `ModelTypes::select_roots` accepts an ordered set; existing select converts supplied strings to Type roots while preserving diagnostics/bytes. Keep the existing schema key equal to qualified name: authored admission already rejects cross-kind duplicates. Defensive projection code must reject collisions before insertion if an IR violates expected ownership; never overwrite. Serialize new typed roots only in event-bearing report/4.
- Resolve selected events from ir.events; each event contributes `types::Message::of_event` -> `types::message` at its qualified-name definition, and `types::field_leaves` -> `types::reachable` contributes the named-type closure. Type roots retain current closure logic. No command, related event, sibling event or unused type is implicitly selected.
- Before serializing either event or reachable struct fields, use the current wire_name collision check (model_types.rs:64–82), including renamed collisions. Preserve input declaration order where schema mapping already preserves it. Existing Plan host-name normalization/collision refusal (realize.rs:251–283,875) remains authority: distinct model names that normalize to one host name must refuse before output, not gain traversal-order suffixes.
- Event-root direct Binary64/Optional/List/Map fields need numeric_locations entries at their actual event definition's properties, in addition to existing type-body traversal (model_types.rs:159–208); preserve exact original-token metadata and obligations across all native targets. Reuse field-level traversal rather than read x-ess annotations as authority. Newtypes reached through event fields retain current nominal metadata.
- Provenance seeds must include typed selected EventRefs and selected type references; complete reachable type authority is maintained by existing closure. Verify event shape/wire-name edits change selection identity, unrelated events/types do not, source digest still identifies whole source, and projection digest covers emitted schema.
- Retain output preflight/ownership/source-containment behavior (ess-cli/src/model_types.rs:44–64). A missing event, a name that is only a type supplied through --event, an invalid mixed selection, a wire collision or unsupported target shape must leave previous output intact. No partial successful library or report.

## Exact proposed typed source scope

Binding design first: `docs/design/event-payload-type-roots.md` (new), with a short cross-reference/update in `docs/design/types-only-realizations.md`.

Production: `crates/generate/ess-gen/src/model_types.rs`; `crates/generate/ess-gen/src/schema.rs` (public typed-root export); `crates/generate/schema-contract/src/realize.rs` (new typed report metadata/conditional format, no unrelated target changes); `crates/edge/ess-cli/src/model_types.rs`; `crates/edge/ess-xtask/src/docs.rs` (format registration). If a field-level Binary64 helper is extracted it stays in model_types.rs; no rewrite of shared types.rs should be necessary.

Tests: `crates/generate/ess-gen/tests/model_types.rs`; `crates/generate/ess-gen/tests/fixtures/model-event-types.yaml` (new); `crates/generate/schema-contract/tests/event_payload_types.rs` (new, real Go/Rust/TypeScript execution using existing test support); `crates/edge/ess-cli/tests/model_types.rs`. Reuse existing support without widening their unrelated profiles. Add any strictly required support file only after naming it in the scope; do not casually change all native codec fixtures.

Documentation: `website/docs/guides/generate-artifacts.md`; `website/docs/reference/cli.md` (generated help update via established generator); `website/docs/reference/formats.md`; `website/docs/reference/spec-versions.md`. Root-owned CHANGELOG entry at integration. No raw-source-schema regeneration is expected because RawSpecFile does not change.

Integration collision: external #394 edits schema-contract/src/realize.rs for native integer widths. Coordinate exact frozen commits; event metadata and integer width changes must both survive. This scope has no production conformance-runtime edits.

## Acceptance and verification before release

Measured red: event-only selection fails while named-type positive control succeeds on valid minimal source. Implement a new CLI red for --event once accepted, then prove all three targets realize event roots with nested closure and exact wire/presence behavior. Explicit type+event roots, repeated/deduplicated selection, all-types exclusivity, missing/wrong-kind roots, wire collisions, declaration collisions, output preservation, event provenance sensitivity, direct/nested Binary64 metadata and unchanged legacy selection bytes are required cases. Rust and Go must actually compile and round-trip emitted records, preserving absent/null and exact integers; TypeScript must actually type-check with strict/exactOptionalPropertyTypes and retain precision/validation obligations. Check emitted schemas against the existing event-schema projection. Scoped fmt/Clippy/affected-package tests plus final root ci-lint/docs/format checks apply. This preparation ran only the documented CLI probes and source reads; no proposed implementation or test is claimed complete.

## Decisions

Accept, redesigned by coordinator under the full consumer-backlog authority, 2026-10-02T22:54Z. The decision adopts the explicit --event selector, shared event schema and reachable closure, additive ModelRoot API, and conditional ess-types-report/4 for event-bearing selections described above. Existing type-only/bundle report3 bytes and --root behavior remain unchanged. No authored source construct, broker authority or new conformance vocabulary is introduced. The initial intake's cross-kind-name claim is superseded by the actual duplicate_declaration control recorded above. External394 remains independently owned; integrate frozen commits carefully at realize.rs instead of overwriting its native width work.

## Acceptance

Named regression cases event_root_selects_existing_payload, mixed_roots_share_reachable_closure, legacy_type_selection_bytes_unchanged, event_wire_and_host_collisions_refuse_before_output, typed_event_provenance_changes_only_for_selected_contract, event_binary64_metadata_preserves_authority, and generated_event_codecs_round_trip_on_all_targets must exercise the actual CLI/projection/compiled generated libraries. These are tooling projection/realization cases over existing ESS source, not a new product entity or newly authored conformance contract. Preserve source and synthesized suite bytes for a control model. Negative tests prove refusal leaves outputs intact. Owner executes scoped package tests/lint, independent adversary reviews frozen source, coordinator integrates with the existing runtime batch for one remote gate.
