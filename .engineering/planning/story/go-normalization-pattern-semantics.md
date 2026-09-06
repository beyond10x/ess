---
format: aep.planning-md/1
id: story:go-normalization-pattern-semantics
kind: story
status: draft
title: Qualify bounded ECMA-262 patterns in Go normalization
relations:
- derived_from: story:source-pinned-data-normalization
revision: 1
---
## Evidence

The standalone Go normalization target uses jsonschema/v6 v6.0.2, whose default
pattern engine is Go RE2. The reference uses jsonschema 0.52.1 with ECMA-262
translation and a bounded backtracking engine. An accepted source pattern such
as ^(?=a)a$ is not supported by RE2, and overlapping syntax alone does not prove
equivalent Unicode, anchors or character classes. Unbounded backtracking or
wall-clock-dependent success is not an acceptable deterministic replacement.

## Outcome

Qualify bounded ECMA-262 schema-pattern validation for Go normalization against
the pinned reference, or retain explicit source-located refusals for semantics
that cannot be preserved. Do not silently fall back to RE2 or skip patterns.

## Acceptance

- A binding design declares supported syntax, Unicode/anchor semantics and a
  deterministic resource-limit policy before implementation.
- Generated Go executes a shared corpus covering ordinary expressions,
  lookaround, backreferences, Unicode, anchors, empty matches and pathological
  backtracking, with reference success/refusal evidence.
- Unsupported patterns refuse before artifact publication at their bundle and
  schema pointer, including referenced definitions. Unselected schemas do not
  broaden refusals.
- Removing go_schema_pattern is supported by this evidence, not by a matcher
  merely accepting the same source text.

## Current Boundary

The initial Go target refuses every selected pattern obligation with
go_schema_pattern. Its generation test covers a referenced lookahead pattern
and confirms a plain root in the same retained bundle remains usable.
The full source-pinned normalization story stays active; this story records its
newly identified matcher gap, not a reduction of three-target acceptance.
