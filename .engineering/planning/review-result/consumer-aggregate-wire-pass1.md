---
format: aep.planning-md/3
id: review-result:consumer-aggregate-wire-pass1
kind: review-result
status: active
title: Aggregate wire candidate requires native model and causal-cut contracts
relations:
- reviews: story:feature-request-361
- reviews: story:feature-request-362
revision: 1
---
needs-revision

Root source review of aggregate-result-observation-wire-candidate.md, SHA256 64d60937c33fe7f0af2cb56a40a8d8c19b3b7fefb823b9019d547a748f472c8d, against carrier d4dc8d3f57359e84b176f043ba6607613040b003. This source changed only nested-response files from the proposal's inspected 86b4a994; the compiler/executor/target files cited here are unchanged. Review executions: zero builds, tests or target probes. Root inspected the concrete source; earlier precondition and history baselines are separately recorded actual executions.

```findings
[
  {
    "file": "aggregate-result-observation-wire-candidate.md",
    "line": 99,
    "category": "native-semantic-model-adapter",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The carried Contract is a new normalized named model, but the reused command executor and accepted #292 context consume EssIr and its compiler-minted handles. EssIr has Serialize only; fields, EssIrParts and from_parts are compiler-private (ir.rs:2165,2233,2256), so a conformance reader cannot deserialize this Contract into that authority or pass it directly to the executor. Specify and scope the actual admitted reconstruction/compiler path or a shared model abstraction, including constraint/condition/conversion/presence fidelity and how it avoids a second native semantic implementation. Round-trip and mutation evidence must cover the source-to-contract-to-runtime path. Do not silently add an unchecked public EssIr constructor or expand #292's scope by assumption."
  },
  {
    "file": "aggregate-result-observation-wire-candidate.md",
    "line": 288,
    "category": "binding-cut-contract-incomplete",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "introduced",
    "message": "The proposal correctly establishes that ObservedInvocation lacks completion and causal authority, but delivery_cut is nevertheless an operation in the proposed executable grammar while its target capability, actual scenario-step trigger, admission/negotiation and adapter implementation remain choices. The only new step is PrepareAggregate. Source_completion has no identified current ESS source declaration and controlled_drain is not an existing callback. Bind one concrete independently checkable capability/step protocol and failure taxonomy, identify the owner of required transport changes, and prove a relevant-binding positive case, or explicitly keep this subfeature as an unresolved completion dependency without presenting delivery_cut as an implementable admitted operation. A pending exactness obligation must not become a blanket binding refusal or a successful partial inventory."
  }
]
```

The remaining direction is useful: exact finalized step references, one retained precondition prefix, immutable query cuts, source-owned values separated from implementation-owned captures, per-invocation event occurrence authority, dynamic group merge/split, all six exact aggregate functions, historical omission and real old-reader refusals. Existing aggregate arithmetic does use checked i128 intermediates; Integer source validation has a narrower primitive authority, so those must stay distinct as proposed.

Before implementation, also make terminal classifications concrete: an independently false target assertion is Failed; an observer arithmetic/budget/capability limit is a distinct non-passing capability/error result under existing report authority, never relabelled a target mismatch. Retain source-only synthesis refusal separately from runtime inability to derive an expected value. The finite budgets are candidate choices requiring actual boundary/performance evidence, not established supported scale.

All new runner hooks must preserve existing one-time disclosure policy, including windows/final scans and value-free diagnostics for malformed replies, target errors, observer failures and resource exhaustion. Do not serialize captured observations, data-derived paths, excerpts or hashes; observer state stays scenario-private. Source mapping cannot turn separately generated fields into a single origin. Treat these as implementation acceptance obligations, not an inferred new source exception.

Scope coordination: #292 is authorized for the shared value/executor context only, not this model reconstruction or aggregate program. The shared integrator owns synthesis sequencing. All ess-transports development remains the separately assigned third Claude session. No delivery DTO/client changes are authorized by this review. The source pair 34/35 remains held according to the acknowledged ownership record, but publication status must be checked again before a new wire capability is bound. No version allocation or implementation approval is made here.
