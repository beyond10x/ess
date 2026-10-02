---
format: aep.planning-md/3
id: review-result:consumer-native-interpreter
kind: review-result
status: active
title: Independent review of native interpreter execution and projections
relations:
- reviews: story:interpreted-trust-gate
- reviews: story:interpreted-eventual-views
- reviews: story:interpreted-bindings-and-unmet-obligations
revision: 1
---
approve

```findings
[]
```

Bounded read-only review of the native tree's frozen native-interpreter-frozen.patch SHA256 ac0800023e84f2be71177d1c86a1c57cc8578cd5714047d57ff37c39f727ead4, with native-interpreter-hashes.txt SHA256 acd830d52b6d9340b9dad9ba731e1c1b0390688dbac09c84e044f77e62cbcd63. All twelve manifest hashes match the source. Own test/build executions: 0. No source, test or planning changes. No concrete defect found in the ten-file scope below.

Authorship exclusion: I authored interpret/facts.rs and interpreted_scenario_facts.rs, and supplied their parent lifecycle/delegate integration sketch. This review does not independently approve those two files or those integration hunks; the coordinator reviews them separately. The remaining ten reviewed paths are interpret.rs (other changes), interpret/bindings.rs, interpret/execute.rs, interpret/protected.rs, interpret/views.rs, interpreted_not_found.rs, interpreted_responses.rs, interpreted_runtime_parity.rs, interpreted_views.rs and tests/support_interpreted/mod.rs.

Typed values: execute.rs only broadens implementation-owned generated event values through the existing typed witness mechanism; it does not populate arbitrary unassigned entity fields. Optional generated values remain absent. Ordinary returns now use the same declared-field witness construction previously limited to protected models, and the response regression validates the actual return against an independently constructed typed Observation containing String, List<Integer> and a declared record. Unsupported witness construction still propagates a named refusal before the command store is committed. The private issuance checks, capture bounds and checks against prior plaintext remain in the existing response helper.

Bindings: dispatch uses a bounded iterative queue instead of recursive event execution. It records actual attempted invocation input, invokes the target command, and propagates undetermined commands rather than reclassifying them as business failures. Final retry outcomes, bounded attempts, retry/drop/escalate branches and generated escalation publication remain distinct. Context-bearing bindings require their exact authority; event-only bindings do not acquire a context implicitly. Mapped values undergo target-type validation. I specifically checked nominal conversions: flat event mappings retain the source contract's historical whole-value semantics and validate the resulting target value, without deriving a transformed value from the conversion reason. Accessor Observation requires identity-first assignment, and selection helpers refuse opaque input conversions. This does not certify arbitrary host transformations; it preserves Billing's admitted same-String nominal crossing.

Views: projection reads derive from stored command states, with deterministic eventual visibility and immediate current state for declared read-your-writes or token-bearing requests. Row and parameter facts remain separate typed namespaces; timestamp predicates and ranking compare parsed instants while preserving returned spelling. Required missing projected/grouping fields refuse instead of being invented. Optional absence is represented explicitly, paging handles paired page/size with checked offset arithmetic, total is computed before slicing, and aggregate functions operate on the filtered stored rows. Tests cover actual aggregate and paging scenarios, offset-bearing timestamp order and filtering, omitted Optional parameter, and a required grouping key intentionally never assigned by the source. Refusal results now carry a consistency token so unchanged-state checks need not weaken their requested consistency.

Trust evidence: the actual model and committed suite digest must agree in healthy parity tests. Each comparison checks every scenario ID and exact status against the handwritten target; fault comparisons retain all Billing/Oracle faults and assert the inventory count. DropBinding and WrongMapping mutate actual compiled source bindings, while other faults use the existing Faulty target wrapper, so the proof is not a fabricated invocation log. Retry-system cases are explicitly outside this Billing/Oracle matrix. The not-found regression strengthens the old Passed-or-Unsupported allowance to require Passed. The coordinator reports final focused 99/0, actual 33 Billing and 34 Oracle scenarios plus sixteen fault comparisons, and strict lint success; these are author/coordinator executions, not mine. Existing reference execution tests provide the healthy target checks; this review adds no independent execution count.

Approval is limited to this frozen implementation scope and its current source-derived controls. It does not imply every possible model is interpretable: unsupported host transformations, unavailable typed witnesses, undeclared absolute time and periodic host facts retain explicit limits. It does not replace the coordinator's independent review of my facts contribution, combined package run, browser/runtime checks or release gates. Report written only to the assigned TS tree's ignored target/backlog-input/native-interpreter-adversary-review.md; no build cache was used.
