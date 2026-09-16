---
format: aep.planning-md/1
id: epic:compact-live-scenarios
kind: epic
status: active
title: Compile compact live scenarios and execute them through native conformance targets
revision: 4
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

## Branch handoff for later adoption (2026-09-16)

The operator requested that this implementation and the finite consumer-classification work be committed and pushed on the feature branch, retained for later adoption and extension. No integration, release or tag is part of this handoff. Three downstream authored live pilot scenarios now have measured passing native conformance reports; that result does not discharge this repository's independent consumer-coverage obligations.

The latest targeted compact and recipe tests passed; site-build passed. The subsequent full task check was interrupted at the operator's halt and is not a passing final gate. Consumer-check still refuses at accounting with 71428 diagnostics and zero executed qualification cases. Preserve blocker:live-pilot-consumer-coverage and the finite claims in docs/design/live-consumer-qualification.md. Resume by producing evidence for the outstanding cells and validating the complete gate before integration; never refresh baseline eligibility merely to obtain green.
