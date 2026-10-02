---
format: aep.planning-md/3
id: review-result:consumer-trust-example-pass1
kind: review-result
status: active
title: Independent review of explicit example facts
relations:
- reviews: story:interpreted-trust-gate
revision: 1
---
needs-revision

unit: explicit example fact sources; Go tree base 69befeea2cfa0bd8b229689a3a64ca541fa37270 plus frozen trust-fixture.patch SHA256 718cadb3659605b1817c111839da932d0e127cfe74c7f8ff34070e61f481ebe4
verdict: NEEDS-CHANGE
cases: own executions 1 existing targeted test; 0 passed / 1 failed / 0 ignored / 3 filtered; red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: assigned synthesis Cargo cache and task-specific external TMPDIR; report and raw log in assigned TS tree scratch
needs-coordinator: caller migration followed by integrated Interpreter parity and preserved fault matrix

Review source diff: none. The implementation tree's 107-path candidate is unchanged by this review; all 107 source hashes and both supplied report/patch hashes matched. No AEP changes, commits or remote writes. No new test was needed: the repository already contains a decisive assertion for the missed caller. I selected that one test before running any broader suite; no broader suite was run.

| File and line | Category | Severity | Verdict | Origin | Finding |
| --- | --- | --- | --- | --- | --- |
| crates/verify/ess-conformance/tests/interpreted_command_execution_adversary.rs:377 | contract-drift | blocker | NEEDS-CHANGE | undecided | The required issued_at migration leaves this existing IssueInvoice caller at line 375 unchanged, so its asserted successful sequential history now returns no outcome; migrate all remaining callers with explicit timestamps while preserving their assertions and fault semantics. |

Measured counterexample: cargo test -p ess-conformance --test interpreted_command_execution_adversary adversary_a_sequential_history_of_the_unfaulted_reference_is_one_the_step_allows -- --exact, executed against the frozen candidate using the assigned synthesis target, jobs 2, incremental disabled, dev/test debug 0. Exit 101. Exact behavioral output:

```text
running 1 test
test adversary_a_sequential_history_of_the_unfaulted_reference_is_one_the_step_allows ... FAILED

---- adversary_a_sequential_history_of_the_unfaulted_reference_is_one_the_step_allows stdout ----
assertion `left == right` failed: the reference issued the second invoice
  left: None
 right: Some("billing.invoice.IssueInvoice/issued")

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 3 filtered out; finished in 0.00s
```

Reachability is direct: the existing test creates two real Billing invoices, supplies only invoice_id to the real IssueInvoice handler, then requires issued. The new reference.rs:569 required-input check returns SemanticCommandResult::undeclared for this unchanged caller. This is not the already-reported generated-Go profile-v2 expectation failure. Exact-base execution was not performed, hence the conservative undecided origin; source comparison directly identifies the new required-input branch and the unchanged caller.

Additional sites in the same caller migration, inspected but not executed here: interpreted_command_execution.rs:294 passes invoice_input containing only invoice_id to the now-mapped command; view_consistency.rs:119 and view_consistency_adversary.rs:105 and view_consistency_adversary_pass2.rs:105 construct IssueInvoice with empty input before subject substitution. Those callers need an audit even where a test might otherwise remain green after its intended successful issue became an undeclared result. Do not weaken their assertions or classify the new missing input as an intrinsic interpreter limitation.

Other reviewed scope, with no additional concrete finding: Oracle order.yaml:102 stores contact from the actual input, and the new regression has a distinct alternate-contact decoy. Billing invoice.yaml:273/280 declares the Timestamp input and assignment rather than inventing a target clock. Reference and realization copy supplied text and compare parsed RFC3339 instants; the regression's offset-bearing earlier instant sorts opposite lexical order and invocation order. Generated Rust/Go behavior copies the required timestamp, Web encoding/decoding and catalog preserve it, and JSON Schema/OpenAPI mark it required. The actual browser SCRIPT at _run.ts:325 now supplies it, the source panel carries the matching source declaration, and bridge bytes compare equal. The example README explicitly records the command API consequence and does not claim an ESS format migration.

The frozen suite diff adds explicit timestamp inputs and mapped-field assertions without deleting scenarios or assertions. Retained before/after ID files compare equal for both 33 Billing and 34 Oracle cases. The 20 existing fault assertions are unchanged; only two faulty.rs workload inputs were migrated. Author logs record final 12 execution +20 faults +2 new source regressions passing, realization 12 unit +3 conformance passing, and projection-check current. Those are the author's executions, not mine, and they did not select the failing caller above. Full Interpreter 33/34 comparison, its fault matrix, browser execution/site checks and combined packages remain integration obligations.

Raw execution log: ess-backlog-ts-parity-20261002/target/backlog-input/trust-review-existing-caller.log. Report: ess-backlog-ts-parity-20261002/target/backlog-input/trust-fixture-adversary-review.md. Build artifacts: ../ess-backlog-synthesis-20261002/target. Temporary compiler files: task-specific external cache. No implementation or test source was written.

```findings
[
  {
    "file": "crates/verify/ess-conformance/tests/interpreted_command_execution_adversary.rs",
    "line": 377,
    "category": "contract-drift",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "undecided",
    "message": "The required issued_at migration leaves the existing IssueInvoice caller at line 375 unchanged, so the exact sequential-history regression fails with outcome None instead of issued; migrate all remaining callers with explicit timestamps while preserving their assertions and fault semantics."
  }
]
```
