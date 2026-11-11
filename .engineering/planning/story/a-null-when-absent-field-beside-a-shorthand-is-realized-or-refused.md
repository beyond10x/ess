---
format: aep.planning-md/3
id: story:a-null-when-absent-field-beside-a-shorthand-is-realized-or-refused
kind: story
status: draft
title: A null-when-absent field beside a shorthand is realized or refused
summary: A shorthand struct with a null_when_absent Optional field compiles but every target refuses it as model_shorthand (turn-3 adversary, backlog)
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-low
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

Found by the adversary pass on `story:a-shorthand-struct-binary64-location-resolves` (turn 3),
outside that story's acceptance. A shorthand struct whose other field is `Optional<…>` with
`presence: null_when_absent` compiles, but no target generates it:

```text
--target go: exit status: 1 error: /$defs/tracker.ops.Marked/x-ess-shorthand: model_shorthand: the record requires a field other than "label", which the short form leaves unfilled
```

TypeScript gives the same error. The cause is not Binary64: an `Optional<String>` field gives the
same refusal. The compiler checks only that the field is Optional
(`crates/specify/ess-domain/src/shorthand.rs:228-232`). ess-gen then publishes the field as required
(`crates/generate/ess-gen/src/types.rs:855`), and every target refuses it with `model_shorthand`
(`crates/generate/schema-contract/src/realize.rs:1651`).

The red case is kept, ignored with this story's id:
`crates/edge/ess-cli/tests/adversary_shorthand_binary64.rs::adversary_a_null_when_absent_binary64_field_in_a_shorthand_struct_realizes`.

## Acceptance

- One of the two holds, decided when the story is accepted:
  - `ess specify validate` refuses `null_when_absent` beside a shorthand, naming the field;
  - or the short form fills the field with null, and Rust, Go and TypeScript realize the struct.
- The ignored case runs and passes.
