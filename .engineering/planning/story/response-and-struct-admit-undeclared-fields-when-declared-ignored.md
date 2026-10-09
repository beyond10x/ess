---
format: aep.planning-md/3
id: story:response-and-struct-admit-undeclared-fields-when-declared-ignored
kind: story
status: draft
title: 'A response or struct type can declare undeclared_fields: ignored for protocol extension members'
relations:
- serves: vision:O2
revision: 1
---
## Outcome

A specification can declare that a command's response, or a struct type, admits fields it does not declare, while every declared field stays required and typed. Several protocols require a client to ignore response members it does not recognise so a server can add extension members. Today ESS closes every response, both in the projected JSON Schema (`additionalProperties: false`) and in the conformance observers, so a server that follows such a protocol fails its own suite (https://github.com/beyond10x/ess/issues/500). After this story the author states it once with `undeclared_fields: ignored`. Schema, OpenAPI and AsyncAPI project `additionalProperties: true` at exactly that object. The Rust, Go and TypeScript response observers admit undeclared keys there and nowhere else. Every closed declaration keeps the bytes and refusals it has now.

## Fit review

1. **Need, apart from the syntax.** The domain fact: "this producer may add fields beyond the declared ones, and a conforming reader ignores them." ESS cannot state it. Reproduction (minimal, neutral nouns): `.engineering/repro/500/spec/` declares `catalog.orders.PlaceOrder` with `response: [{name: order_ref, type: String}]` and an outcome `placed` with `returns: true`.
   - `ess specify validate --path spec` prints `catalog v1 — 2 file(s), valid`.
   - `ess generate --path spec --kind schema` writes `schema/responses/catalog.orders.PlaceOrder.schema.json` containing `"additionalProperties": false`.
   - `ess verify conform synthesize --target go` plus a probe target (`.engineering/repro/500/gosuite/target_test.go`) passes with `{order_ref}` and fails when the target adds one member: `step 2: ESS-CF-PAYLOAD: response has undeclared field` (ess 0.56.0, `ESS_REPORT_FORMAT=2 PROBE_EXTENSION=1 go test`).
   - *Requester's proposed syntax:* `response_members: open` on the response. It keeps declared members required, admits undeclared ones in the observer, and projects `additionalProperties: true` or omits it.
2. **Class: gap.** This is not a defect. Closure is documented as intended:
   - `crates/specify/ess-domain/src/command.rs:2530` ("Closed fields of the response returned by this command");
   - `crates/specify/ess-compiler/src/ir.rs:1710`;
   - `docs/design/typed-response-outcome-payloads.md:3,33` ("A command may declare a closed response record"; "undeclared fields … fail");
   - `crates/generate/ess-gen/src/types.rs:110-115` (`additionalProperties: false` "on every object, in both directions … the intended cost").

   The fact itself cannot be stated at all. It is a gap, not a convenience.
3. **Can it already be expressed? No.**
   - Authored struct types have no open form. `crates/specify/ess-domain/src/types.rs` has no open-record body. `ess_extra` exists only for imported JSON Schema models (`crates/generate/schema-contract/src/realize/rust.rs:251-292`).
   - `Map<String, Json>` cannot hold top-level siblings of declared fields; it would be one nested member.
   - Composition `reader: true` (`docs/design/composition-type-conformance.md:81-112`) is a consumer-side assertion between two specifications. It changes no projection and no observer.
   - Observers refuse undeclared keys in `crates/verify/ess-conformance/src/direct_response.rs:136-142`, `src/response.rs:171-177`, nested structs in `src/selection.rs:689-697` (`ValueProfile::closed`, `:650`), Go `src/go/prerequisites.go:306-317`, and TS `src/ts/direct_response.ts:183,235`.
4. **Fit with what is there.**
   - *Vocabulary.* "Undeclared field" is ESS's existing word: the `UndeclaredField` shape error in `src/input.rs:1811`, and the observer message. "Member" and "open" appear nowhere in the authored language. So the key is `undeclared_fields: ignored`, not `response_members: open`.
   - *Generality.* The same question arises for a struct type reached from a response. Second example below: a key-set response whose nested record is extensible. A response-only key would leave that sibling unable to say it, which is a red flag. So the key is admitted on a `kind: struct` declaration and on a command, where it governs the command's `response:` only.
   - *Composition.* It does not interact with guards, bindings or views. Those read declared fields only, and undeclared ones are never readable. `{response: field}` payload sources still name declared fields only (`command.rs:1065-1071`).
   - *Every target handles it or refuses it by name:*
     - JSON Schema, OpenAPI and AsyncAPI emit `additionalProperties: true` at that object.
     - Rust, Go and TS observers skip the undeclared-key check at that object.
     - Native Rust and Go typed response records drop extras on decode. Inferred: ess-synth has no `deny_unknown_fields` (grep of `crates/generate/ess-synth/src` found none).
     - `ess verify diff` classifies the change, inferred to be through `ResponseChanged` and the existing type-change classification.
     - Generated docs state it.
     - On an event or error declaration, the key is refused by name. Events are already not closed by the observer (`src/scenario.rs:1776-1780`, `src/runner.rs:2120-2121`) but are closed in schema (`crates/generate/ess-gen/src/asyncapi.rs:115`). That is a separate inconsistency, not widened here.
     - On a command without `response:`, the key is refused (`missing_declaration`).
     - Input is out of scope, because the suite never sends undeclared input.
5. **A second, unrelated adopter.** A key-set service answers `ListKeys` with `response: [{name: keys, type: List<catalog.keys.PublicKey>}]`. Its key-record format says additional members may be present and readers that do not understand them must ignore them. From memory, this is RFC 7517 §4 for JSON Web Keys; not re-verified this session. So the openness sits on the nested struct, not on the response root. A second example: a catalog search response whose items carry vendor extension fields under a published extension policy.
6. **Cost.**
   - New source key on two constructs, so `ess/24` (`SUPPORTED_FORMATS` ends at 23, `crates/specify/ess-domain/src/system.rs:53-55`). Below `ess/24` it is refused naming `ess/24`.
   - The observation DTOs gain an omitted-when-closed member, so a new suite pair, `ess-conformance/46` and `/47`. The newest is 44/45 (`src/event_multiplicity.rs:41-44`). An older runner refuses the member as unknown rather than enforcing closure silently.
   - IR: additive field, omitted when closed. No `ess-ir/2`, following the precedent in `typed-response-outcome-payloads.md:21`.
   - Diff: no new format if `ResponseChanged` and the type change carry it (inferred; confirm in design).
   - No migration: closed is the default and existing bytes are unchanged.
   - Generated API: none for closed declarations.
7. **What else was considered.**
   - (a) **Change nothing.** The protocol fact stays inexpressible, and every conforming server fails its suite. Rejected: it is a gap, not a convenience.
   - (b) **The requester's `response_members: open`.** It introduces two new words for an existing concept, covers only the response root, and leaves nested records closed. Rejected as written; the need is kept.
   - (c) **Flip the default to open.** This reverses the documented projection policy (`types.rs:110-115`) for every adopter and weakens every closed contract. Rejected.
   - (d) **Chosen: `undeclared_fields: ignored`,** opt-in, on struct types and on a command's response, with a named refusal everywhere else.

## Decisions

**Accept, redesigned.**

- The need is real (gap). The proposed shape fails question 4: it brings new vocabulary ("members", "open") where ESS says "undeclared field", and it covers the response root only.
- Chosen design: one optional key, `undeclared_fields: ignored`, whose only other value is the default `refused`. It is admitted from `ess/24` on:
  - a `kind: struct` type declaration;
  - a command declaration, where it applies only to that command's `response:`, and only when `response:` is non-empty.
- It is refused by name on events, errors and any other declaration, and below `ess/24`.
- Declared fields keep their presence and type checks.
- Projections write `additionalProperties: true` explicitly at that object rather than omitting the keyword. A keyword is an assertion and an absent one reads as an oversight (`asyncapi.rs:105-110`).
- The observers carry the flag per object, at the response root and per struct declaration, in suite `ess-conformance/46` and `/47`.
- What changed from the request: the name, the location (struct types too), and the explicit `true`.
- What stays as the requester asked: declared fields stay required and typed, and the observer admits undeclared keys.
- The event-payload schema/observer inconsistency is noted and left out of scope.

## Acceptance

- `ess specify validate` accepts `undeclared_fields: ignored` on a command with `response:` and on a `kind: struct` under `ess/24`. It refuses the key below `ess/24` naming `ess/24`, on a command without `response:`, on an event, and on an error, each with a located diagnostic. Checked by new cases in `crates/edge/ess-cli/tests/` beside the existing validate cases.
- `ess generate --kind schema` and `--kind openapi` write `"additionalProperties": true` at the response object and at a struct declared `ignored`. A closed declaration's bytes are unchanged, checked by a projection test plus `task projection-check` on `examples/`.
- On the issue's reproduction rewritten with the key, the synthesized Go, TypeScript and Rust runners pass a target that returns `{order_ref, extension}`. They still fail a target that omits `order_ref` or sends it as a number, and still fail an undeclared key on a closed response. Checked by observer tests in `crates/verify/ess-conformance` (`direct_response`, `response`, `selection`) and the Go/TS runtime tests (`src/ts/direct_response*.test.ts`, the Go runtime test).
- A nested struct declared `ignored` inside a response admits extras at that struct only; its closed sibling still refuses, in a `selection` observer test.
- A suite carrying the flag is written as `ess-conformance/46` (or `/47` with inventory). A `/45` reader refuses it before callbacks, in an admission test.
- `ess verify diff` reports a closed-to-ignored change on a response as `ResponseChanged` (or the decided classification), in an `ess-diff` test.
- `ess specify formats` lists `ess/24` with this row, and `cargo xtask schema` regenerates `schemas/generated/ess.schema.json`, checked by `projection-check`.

## Scope

- `crates/specify/ess-domain/src/command.rs`: `RawCommand` and `Command` gain `undeclared_fields` (cited: `response` at `:2532`, `:5440`). **Needs `command.rs`: yes.** `command/**`: inferred no.
- `crates/specify/ess-domain/src/types.rs`: `RawTypeBody::Struct` and `TypeBody::Struct` gain the key, plus its hand-written JSON schema (cited `:1092`, `:2136-2160`).
- `crates/specify/ess-domain/src/system.rs`: `FormatVersion::V24`, `SUPPORTED_FORMATS`, `FORMAT_HISTORY` (cited `:53-55`).
- `crates/specify/ess-compiler/src/ir.rs`: `ResolvedCommand` gains the field, omitted when closed (cited `:1712`), and the resolved struct body. **Needs `ir.rs`: yes.** `ir/**`: inferred possibly, for the resolved type body.
- `crates/generate/ess-gen/src/types.rs`, `openapi.rs`, `asyncapi.rs`: `additionalProperties` at a declared-open object (cited `types.rs:42,110`; `openapi.rs` response schemas inferred).
- `crates/verify/ess-conformance/src/direct_response.rs` (`:136-142`), `src/response.rs` (`:171-177`), `src/selection.rs` (`ValueProfile`, `validate_struct_value` `:650,689-697`; `Declaration::Struct` `:26`), `src/scenario.rs` and `src/admission.rs` for the suite version floor (cited `admission.rs:537`).
- `src/go/prerequisites.go` (`:306-317`), `src/go/response.go`, `src/ts/direct_response.ts` (`:183,235`), and `src/ts/response.ts` (`:375`). All cited.
- `crates/verify/ess-conformance/src/synthesize.rs`: inferred **no**. `Observation::of(ir, command, …)` (`direct_response.rs:57`) reads the flag from the resolved command and the declarations itself; the call at `synthesize.rs:3131` is unchanged. `src/synthesize/**`: inferred no.
- `src/interpret/execute.rs` and `interpret/execute/{related,existence}.rs`: **no** (inferred). The interpreter never reads undeclared response keys.
- `crates/verify/ess-diff`: classification (inferred).
- Generated docs page for a command and a struct (inferred).
- `CHANGELOG.md`; `website/docs/reference/spec-versions.md` via `cargo xtask format-history`; `docs/design/typed-response-outcome-payloads.md` gains a section. All cited as the repository's practice.
