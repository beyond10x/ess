# Typed command responses and emitted payload ownership

A command may declare a closed response record. An outcome can state that an emitted event carries a field of the actual returned response. This records the relationship the adapter must expose; the suite never invents an expected response from the command input.

```yaml
format: ess/4
# Within a command:
response:
  - {name: item, type: Optional<example.api.Item>}
  - {name: message, type: String}
outcomes:
  - name: completed
    emits: [example.api.Completed]
    payload:
      example.api.Completed:
        item: {response: item}
        message: {response: message}
        receipt_id: {generated: true}
```

The response uses the same ordered typed field declarations as input. Source and event target types must be assignable or have a declared conversion. The executable response observer refuses conversions without executable transformation semantics. Response access reads exactly one declared field; a record, optional value, list or typed map moves whole. It adds no arbitrary path interpreter.

The object sources are closed and mutually exclusive. `{generated: true}` means the implementation supplies the value; event shape checks still apply. False, empty or contradictory source objects are refused. Existing string sources preserve their meanings in every source version: `input.field` reads input, and `response.field` remains a literal. Response and generated sources are not admitted in entity `sets`.

Under `ess/4`, each emitting outcome must account for every field of each emitted event using a source or explicit generated ownership. A diagnostic identifies every missing field. Events with no emitting outcomes are outside that check. Source versions 1–3 preserve sparse implementation-owned payloads and refuse the new declarations and object sources. No `ess-ir/2` is introduced: response fields and resolved source variants are additive and omitted for legacy commands, with source provenance carrying the new version. Compiler dependency edges include all named response types.

## Realization and observation

Native Rust and Go emit a command-specific typed response record. Only outcomes with response mappings carry a response field. Generated outcome helpers `response_payload_matches` and `ResponsePayloadMatches` compare the actual response with the independently supplied emitted fields, including nested records and maps. They do not fabricate business values or perform opaque conversions. Implementations still own the command behavior and return both its response and emitted facts. Response names participate in native symbol admission and collision handling.

JSON Schema emits the response under `schema/responses/<command>.schema.json`, with the same path validation, referenced-type closure and provenance rules as other schema artifacts. Legacy commands add no response artifact.

Rust targets expose `SemanticCommandResult.response`; Go targets expose `CommandResult.Response`. These contain actual returned values, admitted against the closed declared response fields before comparison. The suite's `ExpectResponsePayload` step names the exact command, outcome and emitted event. It reads only the immediately preceding invocation result; a previous invocation or separately observed event cannot supply authority. Missing required response data, malformed values, undeclared fields and unequal mapped payloads fail. Optional source absence or null is observable and requires absence or null on the optional event target.

Go snapshots response-bearing command results before subsequent target callbacks. Returned response and event maps do not share mutable assertion authority. Integer admission and comparison retain exact signed 64-bit values; they do not round an Integer through binary64. Other admitted primitive representations use the existing conformance value contract, including its existing Timestamp text validation boundary.

Response-mapped event fields are checked as whole typed values by the response assertion. Their old flattened payload leaves are omitted to avoid treating an absent optional record as an inaccessible required leaf. Generated and other fields keep their existing shape assertions. No legacy shape rule changes.

The observer uses the existing closed nominal declaration vocabulary and shared bounded value traversal. Its admitted subset includes ordinary newtypes, records, enums, tagged unions, optionals, lists and `Map<String, T>`. Limits are 256 response/target fields, 4096 reachable named declarations, depth 128, 64 entries per collection, 4096 bytes per text value and 1 MiB contract/value budgets. Recursive, Binary64, invariant-constrained and clock-attached response types are explicitly refused at synthesis. This does not claim general invariant validation. The measured response declarations motivating this unit contain typed maps and no declared invariants or reading attachments.

## Persisted meaning

Response assertions require `ess-conformance/8`, or `/9` with coverage inventory. A response assertion pinned to an older envelope is refused before callbacks. Existing report/2 count semantics remain unchanged. A response-only command declaration change produces `ResponseChanged`; response/generated mapping changes produce `OutcomeResponsePayloadChanged`. These require `ess-diff/4`; ordinary legacy payload deltas retain their existing vocabulary/version.

The browser replay adapter currently refuses these newer suite envelopes. It must not silently execute a suite while ignoring response assertions. Consumer source migration and truthful adapter extraction of returned responses are separate adoption steps; native API generation is not evidence that an external adapter has migrated.
