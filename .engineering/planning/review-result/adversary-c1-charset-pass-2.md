---
format: aep.planning-md/3
id: review-result:adversary-c1-charset-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave correctness-1 unit charset
relations:
- reviews: story:a-field-constrained-by-a-charset-publishes-that-charset
revision: 1
---
unit: story:a-field-constrained-by-a-charset-publishes-that-charset
verdict: green
cases: executed 934→937, red 0
origin: introduced 0, pre-existing 0, undecided 0
wrote-outside-worktree: ~/.cache/ess-wave-c1/charset/adv2/ (7 logs)
needs-coordinator: no

Adversary pass 2 (`aep:adversary`), 2026-09-28, head 5519a581c + `crates/specify/ess-domain/tests/adversary_charset_pass2.rs`. Pass-1 F1–F3 fixed.

| case | checks | now |
|---|---|---|
| `no_document_in_the_repository_the_parser_reads_is_newly_refused` | every .yaml/.yml and fenced yaml block in .md/.mdx (206 texts parsed) against the schema with and without the 10 new charsets | green |
| `the_comparison_sees_a_selection_mapping_refused_inside_any_of` | the comparison sees a refusal inside anyOf | green |
| `the_validator_and_the_schema_both_refuse_a_hyphenated_input_selector_and_path` | assemble+validate and the schema both refuse hyphenated SelectionInput.name / mapping selection / path[] | green |

`cargo test -p ess-domain --no-fail-fast`: 937 passed, 0 failed. `cargo test -p ess-xtask --no-fail-fast`: 297 passed. `cargo xtask schema --check`: current. ess-cli schema_bundle, schema_registry_identity(+adversary), empty_projection: 27 passed. clippy/fmt clean.

Attacked, not broken: Transition pattern equals validate_segment (name.rs:25-51) and TransitionRef; Field::PATTERN equals field_name (types.rs:848); periodic bindings with selections refused (binding.rs:1007); mapping selection/path checked unconditionally (binding.rs:958); no format-version gating; accounting doc matches the schema diff and the 67cf…→93c2… pin chain.

Not raised (outside the charset class): SelectionMapping.path is limited to 3 entries (MAX_SEGMENTS) by the validator; the schema publishes no maxItems.

```findings
[]
```
