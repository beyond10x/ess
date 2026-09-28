---
format: aep.planning-md/3
id: review-result:adversary-rp-ts-pass-1
kind: review-result
status: active
title: Adversary pass 1, wave runtime-parity unit ts
relations:
- reviews: story:generated-runtimes-run-every-emitted-suite-version
revision: 1
---
unit: story:generated-runtimes-run-every-emitted-suite-version (ts), worktree ess-rp-ts HEAD 810ba9ae3 + one untracked test file
verdict: red
cases: executed 33→37, red 4
origin: introduced 2, pre-existing 0, undecided 2
wrote-outside-worktree: ~/.cache/ess-wave-rp/ts/adv1/{harness.rs,cases.rs,run1.log,suite.log,tscheck.log}
needs-coordinator: no

Adversary pass 1 (`aep:adversary`), 2026-09-28. New file `crates/verify/ess-conformance/tests/typescript_adversary_rp1.rs` (harness copied from typescript_suite_versions.rs).

| case | target does | Rust | TS |
|---|---|---|---|
| `adversary_typescript_presence_undefined_leaf_gets_the_rust_verdict` (/24) | `payload.partner_ref = undefined` on a null_when_absent leaf | failed | passed |
| `adversary_typescript_undefined_row_field_is_not_null` (/26) | row `{id:"a", note: undefined}` vs `contains {note: null}` | failed | passed |
| `adversary_typescript_changed_by_a_tiny_decimal` (/26) | `changed_by {total: 0.0000001}` | admitted | whole run refused: "the change in `total` is not a number" |
| `adversary_typescript_exact_integer_literal_in_contains` (/26) | `new JsonNumber("9007199254740993")` vs literal 9007199254740993 | passed | failed |

`cargo test -p ess-conformance --test typescript_runtime --test typescript_suite_versions --test typescript_adversary_rp1 --no-fail-fast`: exit 101 (adversary 0/4; typescript_runtime 4/4; typescript_suite_versions 29/29). Without ESS_TYPES_NODE both typecheck cases print "skipped: no @types/node" and report ok.

Not broken: dotted-leaf Absent/Blocked/null rules; JsonNumber vs number equality; bounded-retry decoding; unicode string operators; tsc strict and prettier (with @types/node). Not attacked: now_offset rounding, caller, page beyond unit modes, aggregate min/max/avg over empty sets.

```findings
[{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":4525,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"presence check reads an own property holding undefined as present-and-null, so a null_when_absent leaf a JS target leaves undefined passes in TS and fails in Rust (and the reverse for omitted_when_absent)"},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":4328,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"exactDecimal parses String(n), so 1e-7 or 1e21 is not a number; a changed_by amount that Rust admits makes the TS runtime refuse the whole suite (runtime.ts:5775)"},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":4252,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"matches treats a row field holding undefined as equal to a wanted null on an undotted key, where Rust sees the field absent and fails"},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":5282,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"literal values go through plainNumbers, so an integer literal above 2^53 is rounded before comparison and an exact JsonNumber target fails in TS while Rust passes"},{"file":"crates/verify/ess-conformance/tests/typescript_suite_versions.rs","line":664,"category":"judgement","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"the vocabulary guard counts a case label in the admission or decode switch as executed, so removing an executor case (e.g. changed_by at runtime.ts:3590) keeps it green"}]
```

Coordinator routing: all five to the implementor (correction 1). The two undecided are decided by the coordinator as in scope: target results are read as JSON reads them (undefined = absent), and literals keep exact integers.
