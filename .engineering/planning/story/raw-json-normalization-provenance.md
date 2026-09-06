---
format: aep.planning-md/1
id: story:raw-json-normalization-provenance
kind: story
status: draft
title: Preserve declared raw JSON token bytes during normalization
relations:
- derived_from: story:source-pinned-data-normalization
revision: 1
---
## Evidence

Source-driven settings normalization found a concrete lexical boundary: an
embedded JSON document is copied as raw token bytes and those bytes contribute
to a later cache identity. Whitespace, member order and numeric spellings remain
observable even when the parsed JSON values are equal. Absent raw data and an
explicit JSON null also differ.

The current normalization input/evaluation boundary produces serde_json::Value
and emits canonical JSON. The closed recipe operations do not capture original
token bytes. Copying a parsed value therefore does not preserve this source
contract. A Bytes model type can describe retained bytes but does not implement
their extraction from the enclosing input document.

## Outcome

Provide explicit, source-pinned lexical JSON preservation where the source
contract requires it, without treating arbitrary JSON objects as opaque by default
or silently changing the existing exact/binary64 numeric policies.

## Acceptance

- Bind capture scope, validation order, byte representation, source locations and
  format compatibility in the normalization design before implementation.
- Preserve exact selected token bytes, including whitespace inside the token,
  property order, escapes and numeric spelling; distinguish absence from null.
- Ordinary unselected fields retain the declared normalization/input semantics.
- Raw data is not interpreted as a replacement recipe or executable source.
- Reference and generated targets preserve the same bytes and fail consistently
  at unsupported boundaries. Generic fixtures demonstrate equal parsed values
  whose original byte identities differ.
- Keep complete source/provenance accounting and prevent partial successful
  adapters when a selected lexical boundary cannot be realized.

## Relation

This is a concrete input-fidelity requirement of source-pinned-data-normalization.
Adopter field names and immutable implementation citations remain in the adopter
specification. It is not permission to introduce a generic domain property bag or
to reinterpret all JSON objects as raw byte buffers.
