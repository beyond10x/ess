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

Rust targets expose `SemanticCommandResult.response`; Go targets expose `CommandResult.Response`. These contain actual returned values, admitted against the closed declared response fields before comparison. The suite's `ExpectResponsePayload` step names the exact command, outcome and emitted event. It reads only the immediately preceding invocation result; a previous invocation or separately observed event cannot supply authority. Missing required response data, malformed values, undeclared fields and unequal mapped payloads fail; an undeclared field passes only in a record that declares `undeclared_fields: ignored` (below). Optional source absence or null is observable and requires absence or null on the optional event target.

Go snapshots response-bearing command results before subsequent target callbacks. Returned response and event maps do not share mutable assertion authority. Integer admission and comparison retain exact signed 64-bit values; they do not round an Integer through binary64. Other admitted primitive representations use the existing conformance value contract, including its existing Timestamp text validation boundary.

Response-mapped event fields are checked as whole typed values by the response assertion. Their old flattened payload leaves are omitted to avoid treating an absent optional record as an inaccessible required leaf. Generated and other fields keep their existing shape assertions. No legacy shape rule changes.

The observer uses the existing closed nominal declaration vocabulary and shared bounded value traversal. Its admitted subset includes ordinary newtypes, records, enums, tagged unions, optionals, lists and `Map<String, T>`. Limits are 256 response/target fields, 4096 reachable named declarations, depth 128, 64 entries per collection, 4096 bytes per text value and 1 MiB contract/value budgets. Recursive and Binary64 response types are explicitly refused at synthesis, and so are a record (`struct`) with invariants ("record invariant on a response type"), a type that reaches a `reading` ("reading-attached response type") and a newtype of anything but `String` that declares invariants. This does not claim general invariant validation.

### String-newtype constraints on returned values

A response type may be, or reach, a newtype of `String` that declares `alphabet:`, `prefix:` or `invariants:` (beyond10x/ess#499). The rules travel with the observation as the closed carrier the one-time profile already uses, `constraints`: a map from each constrained nominal type reachable from the response (or from a mapped event target) to `{alphabet, prefix, invariants}`. It appears on `expect_direct_response` and on `expect_response_payload`, and only when at least one reachable type is constrained; an observation without one keeps its bytes. A direct observation of an outcome marked `one_time_response:` carries none, because the one-time trace already checks the same rules on the same value.

Each runner checks the actual returned value at every reachable position of a constrained type — the field itself, a record field, a union variant, and an `Optional`, `List` or `Map` element — after the shape check: every character is in the alphabet, the text starts with the prefix, and every invariant is true over a lone `value` text fact (`value.count` is its Unicode scalar count). A value that breaks a rule fails the step `ESS-CF-PAYLOAD`. The native, Go and TypeScript runners use the check the one-time observer uses, so the three agree on every vector.

Which invariants are admitted is decided once, at synthesis: an invariant that reads anything but `value` or `value.count`, or that quantifies, is refused by name ("invariant `<predicate>` on response type `<name>` does not decide over its value alone"), and the outcome keeps its `ESS-SYNTH-001` refusal. No observer-side list of predicate operators exists.

### Records that ignore undeclared fields

A response is closed by default, and so is every struct it reaches. Some protocols require a reader to ignore members it does not recognise so that a producer can add extension members; a server following one fails a closed suite the first time it adds a member. From `ess/24` the author states the opening once, with `undeclared_fields: ignored` (beyond10x/ess#500), in exactly two places:

```yaml
format: ess/24
types:
  - name: catalog.keys.PublicKey
    kind: struct
    undeclared_fields: ignored     # this record, wherever it is reached
    fields:
      - {name: kid, type: String}
commands:
  - name: catalog.orders.PlaceOrder
    undeclared_fields: ignored     # the response object only, never the input
    response:
      - {name: order_ref, type: String}
```

The opening is opt-in and local. `refused` is the default; a document that does not write the key keeps its meaning, its IR bytes and its compiled digest. On a command the key governs the response root only: a struct the response reaches is open only where that struct says so, and its closed siblings still refuse undeclared keys. Declared fields keep their presence and type checks in every case, so a response that omits `order_ref`, or sends it as a number, still fails. Undeclared fields are never readable: guards, bindings, views and `{response: field}` payload sources still name declared fields only. Input stays closed, because the suite never sends undeclared input.

The key is refused by name, at its line, everywhere else: on a command without `response:` as `missing_declaration` (there is nothing for it to govern), and on a newtype, an enum, a union, an entity, an event, an error, a view or an actor as `unsupported_construct`. Event payloads are closed in their schema projections and not in the observers; that inconsistency is recorded and left out of this construct. Under `ess/23` and earlier the key is refused with `unsupported_format_version` naming `ess/24`.

The IR carries a command's key as `undeclared_fields: ignored` on the resolved command and the open structs as `undeclared_fields_ignored`, a set of type names beside `types`; both are left out when nothing is open, which is what keeps closed models' bytes. The set sits beside the types rather than inside the resolved struct body so that every exhaustive pattern over that body, in the generators and the conformance crates, keeps its shape; a reader asks `EssIr::undeclared_fields(<type>)`. Projections write `additionalProperties: true` explicitly at an open object rather than omitting the keyword, because a keyword is an assertion and an absent one reads as an oversight; the observers carry the flag per object, at the response root and per struct declaration.

## Persisted meaning

Response assertions require `ess-conformance/8`, or `/9` with coverage inventory. A response assertion pinned to an older envelope is refused before callbacks. Existing report/2 count semantics remain unchanged. A response-only command declaration change produces `ResponseChanged`; response/generated mapping changes produce `OutcomeResponsePayloadChanged`. These require `ess-diff/4`; ordinary legacy payload deltas retain their existing vocabulary/version.

A response observation carrying `constraints` requires `ess-conformance/46`, or `/47` with coverage inventory. The pair is selected only when some observation carries the member, and it is cumulative over every major below it, the counted event-claim pair `/44` and `/45` included. Every reader refuses the member under a lower major, and a reader that admits only through `/45` refuses a `/46` or `/47` suite by version, so no runner passes a constrained value it did not check.

The browser replay adapter currently refuses these newer suite envelopes. It must not silently execute a suite while ignoring response assertions. Consumer source migration and truthful adapter extraction of returned responses are separate adoption steps; native API generation is not evidence that an external adapter has migrated.
