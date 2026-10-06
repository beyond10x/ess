---
format: aep.planning-md/3
id: story:ts-prerequisite-runtime-race
kind: story
status: draft
title: TypeScript parity tests build their shared runtime package in a directory no other process writes
tags:
- ci
- ess-0.54.0
revision: 1
---
# TypeScript parity tests build their shared runtime package in a directory no other process writes

## Acceptance

`runtime_parity_typescript_28_35.rs` builds the `ts-prerequisite-runtime` and `ts-one-time-explorer`
packages in a directory owned by one test process (or under a file lock held across emit and
`tsc`), so two nextest processes running tests of that binary at once cannot read each other's
half-written sources; a nextest run of the whole binary with `--test-threads` above 1 passes
three times in a row.

## Evidence

- `main` `CI` run 37331001433 at 0.53.0 (`a81a8729d`), job `Test 12/20` (check run
  111838645746): `malformed_typescript_prerequisite_documents_refuse_before_all_callbacks`
  failed with `SyntaxError: The requested module './predicate.js' does not provide an export
  named 'Operand'` at `target/backlog-input/ts-prerequisite-runtime/essconform/dist/one_time_response.js:4`.
- The package directory is the fixed path at
  `crates/verify/ess-conformance/tests/runtime_parity_typescript_28_35.rs:280-282` (and `:1870`
  for `ts-one-time-explorer`), guarded by a `OnceLock`, which is per process. CI runs tests
  through nextest, one process per test.
- The same test passed in the local 0.54.0 gate (`cargo test`, one process per binary).

The race is inferred from the code and the error text; it has not been reproduced.

## Milestone

`release-plan:ess-054`: the 0.54.0 pull request runs the same shards, and `main` stays red until
this lands.
