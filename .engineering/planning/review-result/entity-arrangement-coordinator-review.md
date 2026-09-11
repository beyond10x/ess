---
format: aep.planning-md/1
id: review-result:entity-arrangement-coordinator-review
kind: review-result
status: active
title: 'Arrangement deciding review: null identity and stale query evidence'
relations:
- reviews: story:authored-entity-state-arrangement
revision: 1
---
unit: story:authored-entity-state-arrangement candidate259a00cf
verdict: CONFIRMED
cases: executed0→3, red3
origin: introduced2 / pre-existing0 / undecided0
wrote-outside-worktree: assigned arrangement/adversary/root-deciding.log
needs-coordinator: correct null-identity admission and stale query reuse
<!-- public-import-provenance:begin -->
Final-state public import from the privately retained integration store, not an event-equivalent historical replay. New CLI creation/status changes occur at import time; original evidence retains its observed timestamp. Superseded bodies/scopes and original transitions remain in private journal SHA256 3bb1ff4f0a4bd3b7e283ec98fd02ad4afdeaa90ca72d490eaa280eca899e99c1. Source artifact SHA256 a7564dd07fc58551146b4a065fd487e22213a5d1c32d81baa8d033f64ddb1df4, retained as local-evidence:runtime-gaps/publication-replay/snapshots/a7564dd07fc58551146b4a065fd487e22213a5d1c32d81baa8d033f64ddb1df4.md. Source creation recorded at 2026-09-11T03:34:13Z. Private labels and local paths are projected to descriptive aliases. This is a disclosed immutable-review publication projection, not a new review. Original review body SHA256 dd70e24c1e9c3da5b1f996a95d8518dc0e386142a44bfa6168ff2912f5566ba7, retained privately as local-evidence:runtime-gaps/publication-replay/snapshots/dd70e24c1e9c3da5b1f996a95d8518dc0e386142a44bfa6168ff2912f5566ba7-body.md. Verdict, finding identifiers, outcomes and measured counts are retained; label/path redaction may change their literal spelling.
<!-- public-import-provenance:end -->

The interrupted agent produced three tests but executed none because its new test used ViewExpectation::Count rather than the declared Counts. Root corrected that fixture spelling only, then executed the three deciding cases in cargo test -p ess-conformance --test authored adversary_entity_setup --offline --locked -- --nocapture. Exit101:0 passed,3 failed. Original full output is retained privately at local-evidence:ess-evolution-20260910/priority-wave/arrangement/adversary/root-deciding.log. No full gate or previous green package rerun.

- input.rs:146 validate_entity_setup accepts a null identity when the model declares Optional<Uuid>, while the compiled suite's admission rejects identity null. Actual source compiler reported complete and the resulting suite was not admissible. This is introduced by setup; reject null consistently at source/model validation rather than publish unusable setup.
- runner.rs:1312 and go/runtime.go:1972 acknowledge successful setup but retain the last queried view snapshot. Actual admitted sequence query(empty), establish(one row), assert(count0) passed in Rust and generated Go. The positive control setup, query, assert(count1) passes. Setup must invalidate cached query observations so a later assertion obtains evidence after the setup. Existing earlier assertions remain historical facts; no fabricated query or command is needed.

These cases reach the actual authored compiler and public admitted execution paths. Unlike the separate accessor helper-only diagnostic, they demonstrate product behavior. No implementation correction was applied before this report.

```findings
- file: crates/verify/ess-conformance/src/input.rs
  line: 146
  category: contract-drift
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Authored setup accepts null identity although its produced suite refuses that identity.
- file: crates/verify/ess-conformance/src/runner.rs
  line: 1312
  category: property
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: Rust and Go setup acknowledgments retain a pre-setup query snapshot that can satisfy a later assertion.
```