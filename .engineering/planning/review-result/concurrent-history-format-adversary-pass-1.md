---
format: aep.planning-md/3
id: review-result:concurrent-history-format-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:concurrent-history-format
relations:
- reviews: story:concurrent-history-format
revision: 1
---
# Adversary pass 1 — story:concurrent-history-format

Dispatched as `aep:adversary` against `ess-chc-format` (uncommitted working tree on
`impl/concurrent-history-format`, base `472d35fbe`), 2026-09-27. Report as returned.

unit: story:concurrent-history-format, uncommitted working tree on impl/concurrent-history-format (base 472d35fbe)
verdict: NEEDS-CHANGE
cases: executed 1860→1868, red 4
origin: introduced 5 / pre-existing 0 / undecided 0
wrote-outside-worktree: 4 paths plus per-pid suite dirs (part 6)
needs-coordinator: whether a `Returned` operation must carry an `outcome` is a design call (finding 3)

Cases added (tests/history_format_adversary.rs, ess-domain tests/ess_history_schema_adversary.rs):
`an_operation_written_as_a_json_array_is_refused` RED; `one_uuid_spelled_in_two_cases_is_one_identity` RED;
`a_returned_operation_with_no_outcome_is_refused` RED; `the_schema_refuses_every_number_form_the_reader_refuses` RED;
`a_history_written_as_a_json_array_is_refused`, `number_forms_other_than_plain_u64_are_malformed`,
`a_repeated_key_is_malformed_at_either_level`, `the_schema_admits_the_base_document` green.

Suite runs: `cargo test -p ess-conformance --locked --no-fail-fast` exit 101, 807 passed 3 failed;
`cargo test -p ess-xtask --locked --no-fail-fast` exit 0, 303 passed;
`cargo test -p ess-domain --locked --no-fail-fast` exit 101, 754 passed 1 failed.

Findings: (1) array-spelled operation admitted, history.rs:144; (2) duplicate check compares UUID
strings, history.rs:372; (3) Returned without outcome admitted, history.rs:407-412; (4) schema admits
number forms the reader refuses, schemas/ess-history.schema.json:14-15; (5) drift scan ignores serde
alias and rename_all, history_model.rs:151-170 — mutants rename_all camelCase, alias "result",
returned_at: Option<String> each pass 6/6 drift tests.

Attacked and not broken: digest mismatch refused before operation checks; Returned with absent/null
return instant; return before invoke; return equal to invoke (accepted); Indeterminate with return
instant or outcome; client out of range; clients 0; unknown fields at both levels; duplicate keys;
trailing bytes; format absent/wrong/non-string; key-order independence; schema enum and field-set
drift; schema if/then; rename(deserialize=…) mutant caught.

Coordinator routing: all five `introduced` → back to the implementor (correction round 1). Finding 3
decided by the coordinator: a `Returned` operation requires an outcome.

```findings
- file: crates/verify/ess-conformance/src/history.rs
  line: 144
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: an operation written as a positional JSON array is admitted, contrary to the schema's type object and the module's "exactly this document" promise
- file: crates/verify/ess-conformance/src/history.rs
  line: 372
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: duplicate detection compares UUID strings, so one identity in upper and lower case passes as two operations
- file: crates/verify/ess-conformance/src/history.rs
  line: 407
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: a Returned operation with no outcome is admitted, leaving the checker nothing to compare against the specification
- file: schemas/ess-history.schema.json
  line: 14
  category: contract-drift
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the schema admits 7.0, 1e1 and integers above u64::MAX that the reader refuses as malformed, so a schema-valid writer can be refused
- file: crates/edge/ess-xtask/tests/history_model.rs
  line: 151
  category: mutant
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: the drift scan ignores serde alias and rename_all, so a type carrying wire names the model does not declare passes it
```
