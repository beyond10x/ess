---
format: aep.planning-md/1
id: design:typed-response-outcome-payloads
kind: design
status: draft
title: Typed response observations and complete emitted payloads
relations:
- informed_by: story:typed-response-outcome-payloads
revision: 1
---
# Typed response payloads: bounded implementation decision

Source inspected at base 1fd6ba62c497c1c0fd4354745e2f60e26859c235. Proposal for coordinator decision before implementation.

## Authored contract

Add optional command `response`, an ordered list of the existing typed Field declarations, with the same duplicate-name, type-reference and nested named-record rules as command input. This is a closed typed response record, not arbitrary JSON. No inferred input or response fields.

Use an explicit tagged payload source, preserving all existing string meanings:

```yaml
response:
  - name: item
    type: demo.WrapUp
  - name: message
    type: String
outcomes:
  - name: Cancelled
    emits: [demo.WrapUpCancelled]
    payload:
      demo.WrapUpCancelled:
        item: {response: item}
        message: {response: message}
        receipt_id: {generated: true}
```

The existing string `response.item` remains a literal, including under the new format. Object sources are closed and mutually exclusive: response field or generated ownership. Existing `input.foo` and literal strings remain unchanged. Response access is exactly one declared field; nested values travel whole, including typed records and lists. No expression/path engine. Response source in entity `sets` is refused in this unit rather than implicitly extending subject-state mutation.

## Completeness and typing

For the new authored version, every field of every event in each emitting outcome must have either an input/literal/response mapping or explicit generated ownership. The diagnostic identifies command, outcome, event and field. Generated ownership still requires declared shape/invariants at runtime; it is not an assertion bypass. Events with no emitting outcomes are outside this check. Legacy specifications retain sparse implementation-owned payload semantics.

Response field sources must resolve against the command's declared response and be assignable to the target field; retain the existing explicit conversion policy and refuse executable conversion without an implementation. Unknown/duplicate response fields, unknown target fields, contradictory sources and generated ownership in `sets` are refused. Response records are ordered declarations, serialized only when present.

## Actual execution and persistence

Carry response fields and resolved response-source/generated variants through typed IR. Add an optional actual returned response record to Rust SemanticCommandResult and Go CommandResult. Reuse existing typed Node value machinery internally, but admit against the declared closed response fields and named types before use; this is not an arbitrary source-schema bag.

Synthesize a typed response observation/assertion for the exact preceding invocation and its outcome. Validate returned response shape and compare mapped event payload values to that observed response, not to fixture values or a callback-modifiable expected request. Missing response/field, malformed response and mismatching mapped event must fail. Generated fields retain the ordinary event shape assertion. Later invocations must not reuse stale response values. Response-free legacy runs behave unchanged.

Native Rust and Go generation emits a command-specific typed response record and carries it in response-bearing outcome results; unrelated commands retain their generated API/bytes. Typed realization helpers map declared response fields to event fields without inventing values; generated fields remain supplied by implementation. Native callers and conformance adapters expose actual returned values. The response-bearing branch relation must be explicit in the generated result shape, with no requirement to fabricate a response for a refusal branch that has no response mapping.

## Exact proposed compatibility allocation (coordinator owns routing)

Propose authored `ess/4` because the global emitted-payload completeness meaning changes. Refuse new response/generated source vocabulary under ess/1–3, preserve existing sparse and literal meanings there, and apply completeness under ess/4. No ess-ir/2: additive typed IR fields are omitted for legacy commands; source provenance identifies the new contract, as existing feature additions do.

Propose `ess-conformance/8` ordinary and `/9` coverage-bearing for the new response assertion vocabulary, preserving /1–7 bytes and admission. Old/current readers must refuse a response step pinned to an older suite envelope before execution. Existing report/2 counts remain valid. Target-failure and semantic-diff vocabulary selectors need coordinator allocation only if new persisted variants require it; response attachment and mapping changes must remain visible to semantic diff.

## Focused evidence and scope

First-red domain cases: complete response mapping parses/validates; incomplete emitted payload is refused; explicit generated field and external zero-emitter event validate; legacy response-prefix literal remains literal. Then one runtime record witness with changed response versus unchanged emitted event, missing/malformed response and response-authority isolation, executed in Rust and generated Go. Prove generated native typed response APIs compile and execute. Add fresh/pinned suite and source version refusal checks. No full gate.

Own CommandSpec/RawCommandSpec/Outcome payload parsing and validation in domain command.rs, relevant type validation traversal, ResolvedCommand/ResolvedPayloadValue and response resolution in compiler ir.rs/resolve.rs, payload synthesis and scoped response module, SemanticCommandResult and runner response observation, corresponding Go CommandResult/response dispatch, native command item/result emission under ess-synth Rust/Go, semantic command-response/payload comparison in ess-diff, necessary exhaustive-match callers and focused tests. No ErrorSpec/ResolvedError edits; no Go predicate evaluation changes. Coordinator owns aggregate imports, format catalog/selectors, schemas, generated repository artifacts and common website docs. Send exact coordinated routing patches after implementation symbols exist.

Decision requested: approve the closed tagged spelling, command-level response fields with response-bearing branch results, new-format completeness plus generated escape, and /4 + /8–9 allocation (or supply shared unreleased allocation).

## Final decision and implemented boundary

The approved source format is ess/4, with ordinary suite/8 and coverage suite/9 and the existing report/2. New error naming and command response/payload deltas use ess-diff/4. Closed response source objects and explicit generated ownership preserve legacy string meanings and source1–3 sparse payload behavior. Runtime observations compare independently supplied actual responses and emitted payloads from the same invocation; missing, malformed, stale and mismatched data fail. Structs and tagged unions are closed; Optional absence/null and Map<String, T> records are supported within the existing observation limits. Unsupported declaration constraints retain explicit capability refusal.

Native Rust/Go typed response helpers were compiled and executed on valid and mutated values. The browser explicitly refuses response observation steps. Error-wire and predicate implementations are separate units sharing only coordinator-owned format routing. The initial design's pending allocation question is settled by this record; no release pin or downstream deployment is implied.

<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 85dca7a12be829d59192ad9cce719cb29598a30ef4445d7ba2ebcd7cdacdf3f4, retained as local-evidence:runtime-gaps/publication-replay/snapshots/85dca7a12be829d59192ad9cce719cb29598a30ef4445d7ba2ebcd7cdacdf3f4.md. Source creation recorded at 2026-09-11T05:03:11Z. Private labels and local paths are projected to descriptive aliases.
<!-- public-import-provenance:end -->
