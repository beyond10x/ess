---
format: aep.planning-md/1
id: obligation:review-execution-recovery-implementation
kind: obligation
status: open
title: Implement finite execution recovery after the typed design
relations:
- decomposes: epic:review-boundary-remediation
- depends_on: story:review-execution-recovery-design
revision: 2
---
## Outstanding outcome

F11's partial execution/retry weakness remains open after the design story. Evidence source: `docs/reviews/2026-09-05-architecture-review.md:410` and `crates/edge/ess-cli/src/main.rs:1613`.

## Discharge condition

The complete recovery matrix from the binding design passes against the recorded integrated implementation commit.

## Procedure and matrix coverage

After the binding recovery design has validated any newly introduced typed model, create implementation stories through AEP for the exact edge modules and run their fake-executor failure/retry matrix. Record results against the integrated implementation commit; only those results can meet this obligation. Every failure point, interrupted evidence write, manual drift, equal-desired retry and explicit removal case must produce the designed observable outcome.

After the binding recovery design has validated any newly introduced typed model, create implementation stories through AEP for the exact edge modules and run their fake-executor failure/retry matrix. Record results against the integrated implementation commit; only those results can meet this obligation. Every failure point, interrupted evidence write, manual drift, equal-desired retry and explicit removal case must produce the designed observable outcome.

## Why it is not decomposed now

The review establishes no receipt identity, ownership, cardinality or authority model. Those are design inputs, not facts to guess into new entities. The design-first work is scheduled; implementation remains visibly owed. Expected ESS landing surface is the CLI executor, inferred, to be re-scoped after the contract is known. No live controller or real deployment is requested.