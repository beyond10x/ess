---
format: aep.planning-md/3
id: dependency-blocker:aggregate-binding-cut-authority
kind: dependency-blocker
status: open
title: Aggregate binding effects lack a verified complete query boundary
relations:
- blocks: story:feature-request-361
- blocks: story:feature-request-362
withholds: test_result
revision: 2
---
## Missing dependency

Exact aggregate observations affected by asynchronous bindings need independently verifiable completion and causal-cut authority. Current target.rs::ObservedInvocation holds only binding, command and input; InvocationObservationRequest's deadline is a wait bound, not proof that all relevant binding effects/retries have completed. The reviewed aggregate v2 correctly removes the unimplemented delivery_cut/frontier sketch. No replacement capability is yet bound.

## What clears this

A jointly bound source/runtime contract and exact transport-owner request/response interface identifying the relevant causal occurrence closure, completion/retry ordering and actual execution/value authority, aligned with the aggregate read boundary; original-byte admission/disclosure rules; and actual independent healthy/fault adapter tests proving the claim. Interface agreement alone does not clear this evidence dependency. Preserve unrelated-binding positive controls; this blocks complete binding-affected #361/#362 acceptance, not every development step.

## Ownership and boundary

The sole held-bundle integrator now coordinates the required completion work under the operator's accepted implementation/release plan and “dont care, just integrate” response to the handoff question. Read-only discovery found no pending executable transport candidate or active ESS implementation by the historically assigned Claude session. Literal transport390 is already merged. The root will bind and independently test the missing completion capability; no synthetic drain or guessed occurrence is authorized. Keep this blocker open until its actual adapter proof exists. The full bundle stays on batch/ui-live-apps-complete-20261003, with one PR and no partial release.

## Evidence

Current target.rs::observe_invocations at line308 and ObservedInvocation at line974 were inspected at runtime c2c4f01c6cfe99c6a16db5670669fb9774ab6bb9. The design boundary and proposed scope are component-design:aggregate-observation-integration-boundary. Native shared context also depends on #292 freeze and a separately reviewed adapter; that dependency is represented by story edges, not conflated with this transport completion capability.
