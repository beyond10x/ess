---
format: aep.planning-md/3
id: story:generated-rust-types-may-fold-null-and-drop-unknown-members
kind: story
status: active
title: Generated rust types may fold null and drop unknown members
summary: 'Adopter report N13: No option to fold null and missing in generated Rust (ess#667).'
refs:
- provider: github
  reference: beyond10x/ess#667
- provider: github
  reference: beyond10x/ess#668
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T12:51:25Z", actor: "human:timo", revision: 3, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T12:51:25Z", actor: "human:timo", revision: 4, correlation: "ess-0690"}
---
## Problem

Generated Rust keeps an explicit `null` as `Present(Null)` and keeps unknown members of an open object in `ess_extra`, refusing to encode a colliding key (`crates/generate/schema-contract/src/realize/rust.rs:316`, `:330-344`). Some consumers want neither and have no option. Adopter reports N13 and N14, ess 0.68.0. Same options surface as the derive story.

## Acceptance

- `fold_null_decodes_null_as_absent`: with the option on, `{"x": null}` and `{}` decode to the same value for an `Optional` field; encoding omits the field.
- `drop_unknown_members_has_no_extra_field`: with the option on, an open object's type has no `ess_extra`, decodes `{"y": 1}` and encodes without it.
- `the_report_records_each_option`: the generation report lists each option as an obligation the consumer accepted.
- `defaults_are_unchanged`: without the options, generation is byte for byte as before.
