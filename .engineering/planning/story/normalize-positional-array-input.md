---
format: aep.planning-md/1
id: story:normalize-positional-array-input
kind: story
status: draft
title: Normalize declared positional arrays without guessing decoder policy
relations:
- derived_from: story:source-pinned-data-normalization
- serves: vision:O2
scope:
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/check.rs
- confidence: cited
  path: crates/generate/schema-contract/src/realize/normalize/recipe.rs
revision: 2
---
## Evidence

At ESS 9899faed9eb10eb2e4d9e6dc946f6d732e87c6b6,
crates/generate/schema-contract/src/realize/normalize/check.rs:198 refuses
Shape::Array with positional prefixes as unsupported_shape. recipe.rs declares
field reads and list operations but no explicit fixed-array input decoder.
The structural realization already retains positional schema arrays; the gap is
normalization admission/execution, not missing evidence that such schemas exist.

A source-contract audit found fixed-width decoded string arrays represented later
as named operand records. The downstream named record describes already-decoded
positions but does not implement source padding, truncation, null behavior or
position-specific element errors. Adopter identities and immutable source
citations remain in the adopter's private specification.

## Outcome

Bind and implement explicit positional-array normalization where the source
contract supplies arity and element policies. Do not silently flatten a tuple into
a homogeneous list or replace source decoding with stricter schema validation.

## Acceptance

- A binding design distinguishes checked tuple mapping from source decoder policy,
  including absent/null values, short/long inputs, invalid elements and error order.
- Input/output source identities and each positional read remain checked.
- A generic fixed-pair fixture demonstrates exact supported behavior in reference
  execution and generated targets; unsupported decoder policies refuse with source
  locations before adapter publication.
- Existing homogeneous-array and already-decoded-record recipes retain their bytes
  and semantics unless an explicitly versioned migration requires otherwise.

## Scope

- Primary: schema-contract normalize/check.rs and recipe.rs (cited).
- Execution and generated runtimes: normalize/eval.rs, input.rs, rust_runtime.rs.txt,
  go_input.go.txt and go_runtime.go.txt (inferred pending the binding design).
- Design and conformance corpus: docs/design/source-pinned-data-normalization.md,
  existing normalization Rust/Go/CLI suites (inferred).
- Confidence: medium (inferred); admission refusal is directly established, the
  supported decoder policy is a design decision.

## Status of the Finding

Filed during resumed source mapping. This is a concrete capability refusal and a
compatibility requirement, not a claim that every native decoder permissiveness
must be reproduced by a typed canonical contract. No implementation is claimed.
