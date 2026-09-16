---
format: aep.planning-md/1
id: epic:compact-live-scenarios
kind: epic
status: active
title: Compile compact live scenarios and execute them through native conformance targets
revision: 3
---
## Outcome
An author writes typed fixture recipes and Given/When/Then steps, including asynchronous event barriers and bounded observation windows. Native ESS resolves pinned component contracts and produces the same scenario semantics for Rust and Go. A generated Go execution session runs without testing.T and emits admitted native evidence.

## Acceptance
Closed versioned authoring and suite vocabularies retain exact provenance and source maps. Existing formats remain compatible. The target reports observations; the evaluator owns matching and verdicts. Unsupported observation, partial setup, cancellation, gaps and artifact failures never establish successful conformance. Rust and Go agree on adversarial fixture outcomes.

## Scope
Cited: crates/verify/ess-conformance/src/{authored.rs,scenario.rs,target.rs,runner.rs,go/runtime.go,go/mod.rs}, crates/specify/ess-composition/src/lib.rs and crates/edge/ess-cli. Existing entities/types are model-owned; this extends scenario vocabulary rather than inventing product entities. No generic property bags, foreign assertion DSL or second persisted system IR.

## Dependencies and ownership
Independent adopter work, not an expansion of the existing evolution initiative or fake-backend scope. Related contracts and live qualification remain in their owning private repositories. Shared conformance files have one implementation owner; no concurrent edits there.

## Verification
Full task check including consumer checks, native Rust/Go cross-language tests, old-reader compatibility, deterministic regeneration, docs gates. Reports distinguish model validity, controlled target evidence and deployed behavior.