---
format: aep.planning-md/3
id: story:a-null-when-absent-binary64-field-realizes
kind: story
status: draft
title: A null_when_absent Binary64 field realizes in Rust
summary: An Optional<Binary64> field with presence null_when_absent gets its binary64 location at the anyOf node, so Rust realization refuses it.
tags:
- ess-0.70.0
revision: 2
---
## Finding

Adversary pass on G-U15 (ce0bd3b57, merged 53f7f7571) before 0.69.0, on integrate/ess-batch-0690 f1197bcd4; recorded for 0.70.0 under approval-record:ess-0690-cut-after-gu19.

- `crates/generate/ess-gen/src/model_types.rs:360`: `required()` points at the `anyOf`, not the number; refusal `model_binary64_layout: compiler-owned Binary64 location does not resolve to a numeric wire node`.
- Pre-existing for declared structs (`demo.lab.Reading`); G-U15 makes it reachable through response roots.
- Red tests: `a_null_when_absent_binary64_response_field_realizes`, `a_null_when_absent_binary64_struct_field_realizes` (schema-contract, adversary_gu15_body_roots.rs).
