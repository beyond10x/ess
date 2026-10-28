---
format: aep.planning-md/3
id: approval-record:d135-a-breaking-unit-leaves-the-batch
kind: approval-record
status: approved
title: A unit that breaks consumers finds a non-breaking form or leaves the batch (D135)
summary: 'U6 follows the 2026-10-23 approval: nullable anyOf/oneOf imports as Optional'
relations:
- decides: story:openapi-import-admits-open-objects-as-json
- decides: story:an-imported-integer-format-realizes-at-its-native-width
- decides: story:generated-fields-keep-declaration-order-and-box-only-recursion
- informed_by: approval-record:openapi-import-open-objects-in-0700
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-10-28T10:43:07Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-1"}
---
## Decision

D135, taken by the orchestrator under the operator's grant on 2026-10-28: "any unit whose change
breaks generated-code consumers or a documented contract (e.g. #648 native int width, U7 Rust key
order on the wire, U6's departure from the 2026-10-23 approval): first look for a non-breaking form
(new format version, opt-in flag); if none, file a decision-blocker with `Decider: operator` and
leave that unit out of the batch. For U6, read the 2026-10-23 approval: if it records the
operator's acceptance, follow it (Optional<X>); otherwise send me the conflict with options."

## Applied

- U6, #647: `approval-record:openapi-import-open-objects-in-0700` (approved 2026-10-23, actor the
  operator) records the operator's words "nullable oneOf -> Optional". The scoper's departure (import
  the schema with a null gap) is dropped: a nullable `anyOf`/`oneOf` imports as `Optional<X>`, as
  approved.
- U6, #648 (bundle `int32`/`int64` realizing at native width) and U7, #661 (Rust field order on the
  wire): each unit first looks for a non-breaking form (a new import-envelope version, an opt-in
  flag). A unit with none leaves the batch and gets a `decision-blocker` with `Decider: operator`.
- The same test holds for every other unit, including U1's moved `generated/` files (D138).

Decider: operator
