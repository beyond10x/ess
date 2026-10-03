---
format: aep.planning-md/3
id: review-result:consumer-292-design-pass1
kind: review-result
status: active
title: History design needs feasible generation and cross-row session barriers
relations:
- reviews: story:feature-request-292
revision: 1
---
needs-revision

Independent design review of `292-generated-history-executor-design.md`, SHA256 `ac79aaa8d0e6dc32ce41d6d7886ea9df8fc365d4425b67b7477c0ec26a090c8f`, against carrier `5c5aeaf795a46aacfd3709e04f630d83d8a6a837` (runtime `046db8a6805154aa5cafc63b0a4b741bc26e954d`). Reviewer test/build executions: 0. Source inspection only; the counterexamples below are required compiled regression fixtures, not claimed executed results.

```findings
[
  {
    "file": "292-generated-history-executor-design.md",
    "line": 81,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "A declared target type does not establish that a generated value satisfying its constraints exists. Source inhabitation checks only structural dependencies, while invariant admission checks paths/operands rather than joint satisfiability. Treating fresh generation as already valid can therefore produce a proven complete history explanation for an empty constrained domain. Require a nonempty-domain proof before generation becomes a proven transfer: a validated witness may certify existence but must never become the recorded value or drive predicates; inability to prove feasibility remains unresolved, and proven emptiness cannot produce a successor. Cover required contradictory constrained generation, a satisfiable constrained positive control, and Optional of an empty inner domain where absence remains possible. Apply this to required event/error payload generation as well as stored fields."
  },
  {
    "file": "292-generated-history-executor-design.md",
    "line": 89,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "Keeping the existing read-your-writes subject-key condition is unsound after cross-row effects become executable. judge currently requires completion of the client's earlier operation only when its subject_key equals the row under judgment or is empty. A command addressed to A that moves/deletes B via affects is omitted when judging B, so a read after that acknowledged write can still pass using a prefix before it. Bind read-your-writes coverage to the shared partition's acknowledged client operations (a conservative session barrier), or to sound per-explanation affected-row provenance; do not retain the subject-key filter for interacting partitions. Add a compiled subject-A/affected-B history whose lifecycle view shows stale B after the same client's command returned, plus other-client and no-subject controls."
  }
]
```

The shared executor context, explicit unknown presence, Kleene evaluation before refusal, original-store reads, staged rollback, conservative cross-row grouping, and unresolved-alternative propagation are sound directions. The proposal correctly rejects command-wide dependency refusals and does not borrow recorded outcomes for branch selection. Its scope names the necessary execution readers and primitive wrappers rather than hiding the work in the linearizer.

For the first finding, `crates/specify/ess-domain/src/system.rs:621` ignores constraints when determining structural inhabitation. `crates/specify/ess-domain/src/types.rs:1303` checks each invariant's expression, and `crates/specify/ess-domain/src/command/value_expression.rs:371` admits Generated without a satisfiability check at that expression seam. A required newtype over Integer with `value > 0` and `value < 0` is the minimal source-trace counterexample to compile. The current concrete `interpret/execute.rs:1453` obtains a witness or returns NoValue; the new abstract path must not remove that feasibility obligation. Feasibility proof is separate from selecting a representative as runtime authority. A failed bounded witness search alone is not proof that a domain is empty. Required struct constraints need joint feasibility, not just independently inhabited members; optional presence must not grant an impossible present branch.

For the second finding, `crates/verify/ess-conformance/src/linearize.rs:1429`–1440 constructs the client's mandatory completed operations using the exact subject-key filter. Global partitioning supplies the missing shared state but leaves that filter wrong for an `affects` write targeting another row. The proposed no-subject test will not catch this because the existing empty-key exception already includes that operation. The regression must use a nonempty A key and a distinct B row. The existing per-row snapshot limitation may remain; preserving it does not permit ignoring the client's acknowledged cross-row mutation.

The final design should retain the explicit uncertainty behavior for search and view reachability. A proven complete command ordering does not exhaust read explanations, and an unresolved alternative cannot be cached as dead. No additional finding was established for the proposed origin identities, absence/null distinction, early-return control flow, or whole-history grouping. Implementation still needs the direct-read audit, exact CLI 0/1/2/3 evidence, and independent source review described in the proposal. This review grants no production implementation or all-feature completion claim.
