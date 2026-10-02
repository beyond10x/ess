---
format: aep.planning-md/3
id: review-result:consumer-one-time-go-observer-pass1
kind: review-result
status: active
title: Go disclosure observer independent checkpoint review
relations:
- reviews: story:feature-request-389
revision: 1
---
needs-revision

Independent read-only review of the Go observer protocol checkpoint, base49bef4eac, patch SHA2569b2c9fc888ae5b6bfb5b4e5fb4d363e25ab5ac547d0f82ee08ca53dc80db4091. Own test/build executions: 0. All four production paths still match go-observer-protocol-checkpoint-sha256.txt. The two test/support paths have since gained additive fixtures; the author confirmed those changes and unchanged production bytes. This review assesses the frozen production and checkpoint patch, not a final suite35 or complete release claim.

```findings
[
  {
    "file": "crates/verify/ess-conformance/src/go/runtime.go",
    "line": 1770,
    "category": "boundary",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Protected runs sanitize only an Identity callback error. A successful target-supplied identity is retained and countDocument at line5122 publishes Name plus Version unchanged. A target whose successful identity contains the later captured plaintext therefore discloses that plaintext in the report despite a healthy command/view/event trace. Preserve the actual callback but replace both successful and failed protected identity metadata with the agreed static identity; retain unmarked behavior. Add a shared live control returning the issued value in successful Name and Version and assert its absence from report and diagnostics. This is a source-proven dataflow counterexample; own executions0. Root and the Go/native owners accepted it and are preparing the decisive shared fixture."
  }
]
```

The observer otherwise has explicit private scenario-local captures, complete selected-origin shape/constraint checks, fresh-value comparison before new capture, same-invocation sibling/key checks, independent publication-log scans before payload deduplication, completion checks, and aggregate row/event budgets. Resource validation precedes secret comparison and counts compact JSON escaping, number spelling, object entries/array elements and root-relative depth. Window code requires elapsed completion and a final independent scan; it accounts for the shared native clock across multiple windows at one operation. Protected scenario diagnostics discard target-supplied formatting arguments, and error/unsupported status joins preserve the existing Failed > Error > Unsupported ordering. These are source observations, not additional executed evidence.

An adapter-only boundary question was sent separately to the Go owner: one_time_execution.go scans ErrorPayload only when Error is nonempty. Native represents fields inside an optional declared error, but the additive Go DTO can hold an orphan payload. Scanning any non-nil supplied map or refusing that pairing would prevent an unchecked observation. No execution or accepted native equivalent is claimed for that question, and it is not a second blocking finding in this checkpoint verdict.

Author-reported18/18 and scoped lint are retained as author evidence only. Additional numeric, multi-field, actual coverage35 and successful-identity controls, final hashes, and integrated package checks remain outside this checkpoint approval. No Go source, shared source, or planning artifact was edited by this reviewer.
