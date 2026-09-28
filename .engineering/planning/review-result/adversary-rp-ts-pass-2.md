---
format: aep.planning-md/3
id: review-result:adversary-rp-ts-pass-2
kind: review-result
status: active
title: Adversary pass 2, wave runtime-parity unit ts
relations:
- reviews: story:generated-runtimes-run-every-emitted-suite-version
revision: 1
---
unit: story:generated-runtimes-run-every-emitted-suite-version (ts)
verdict: red
cases: executed 37→40, red 3
origin: introduced 2, pre-existing 0, undecided 2
wrote-outside-worktree: ~/.cache/ess-wave-rp/ts/adv2/{cases.rs,run1.log,run2.log,suite.log}; test packages adv2-*
needs-coordinator: no

Adversary pass 2 (`aep:adversary`), 2026-09-28, head cde14a7bb + `crates/verify/ess-conformance/tests/typescript_adversary_rp2.rs`. Pass-1 cases (rp1) 4/4 green.

| case | target answers | Rust | TS |
|---|---|---|---|
| `adversary2_typescript_class_instance_answer_is_read_as_json` | `class Row { id; note; }` vs `contains {note: null}` | failed | passed |
| `adversary2_typescript_class_answer_holding_plain_rows_is_read_as_json` | `new Answer([{id:"a", note: undefined}])` | failed | passed |
| `adversary2_typescript_date_in_a_row_is_read_as_json` | row `{at: new Date(...)}` vs `contains {at: "2026-01-01T00:00:00.000Z"}` | passed | failed |

Suite: typescript_runtime 4/4, typescript_suite_versions 29/29, rp1 4/4, rp2 0/3; exit 101.

Not broken: private fields, getters, async methods through the proxy; cyclic answer errors both sides; 200k rows ~110 ms/answer; exactNumbers spelling differences unreachable (suite re-emitted by Rust); changed_by admission matches defect for 1e-7, 1e21, 1e40; guard reads executors only.

```findings
[{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":2804,"category":"boundary","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"asJSON passes class instances through, so an own property holding undefined in a class row or class answer is read as null by matches and TS passes a row that Rust fails as not carrying the field"},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":2798,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"asJSON ignores toJSON, so a Date in a row stays an object in TS while Rust sees its ISO string, and a contains on that string passes in Rust and fails in TS"},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":725,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"exactNumbers now sends targets JsonNumber objects (no toJSON) for literal input, caller, identity and fields values past 2^53, and neither the request types nor the generated README say so"},{"file":"crates/verify/ess-conformance/src/ts/runtime.ts","line":2800,"category":"boundary","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"asJSON keeps sparse-array holes, BigInt and Map where JSON gives null, an error and {}; the states were built in a probe and no target was found producing them"}]
```

Trend: pass 1 → 5, pass 2 → 4 (carried 0 by signature; the class-instance finding reopens pass-1 finding 1 through a path the fix did not cover). Coordinator routing: correction 2 (last) — asJSON follows JSON.stringify semantics fully (toJSON, own enumerable keys, holes → null, Map → {}), JsonNumber documented in the request types and README; coordinator verifies by diff read.
