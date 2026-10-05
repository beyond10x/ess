---
format: aep.planning-md/3
id: review-result:conditional-binding-design-20261003-r1
kind: review-result
status: active
title: Conditional binding and refusal policy design independent review
relations:
- reviews: story:feature-request-268
- reviews: story:feature-request-269
revision: 1
---
needs-revision

story:feature-request-269 — a bounded retry selected as the explicit fallback can receive a mapping/host failure before the command is invoked, but its bound advances only by total command invocation count; repeated pre-invocation failures therefore never consume the stated attempts and can retry forever — docs/design/conditional-binding-failure-policies.md:129

story:feature-request-269 — an escalation selected as the explicit fallback is required to emit its event even when mapping fails before a command input exists, but the contract supplies only the event handle and no payload-construction authority for that path; the current generated escalation obligation requires the completed failed command input — docs/design/conditional-binding-failure-policies.md:138

story:feature-request-268 — the story Outcome still promises reaction to one source-command outcome, although the accepted payload-only predicate explicitly cannot distinguish source outcomes that publish identical payloads; acceptance could therefore be judged against two incompatible product claims — .engineering/planning/story/feature-request-268.md:63

What I read: the complete coordinator design and the complete current bodies of stories #268, #194 and #269 at integration HEAD `8606b103c5ca5ac1809eafecea6c9b65ac909d9f`, plus the current predicate AST/evaluator, binding reader and resolver shapes, retry alias expansion, resolved failure API, generated Rust/Go dispatch, synthesis refusal for bounded retry, conformance target/runner invocation observation, suite format admission and diff format admission. Reviewed artifact SHA-256 values were `6a324b3b2cb6a46dcab50e085f85914bf60869dc1d12979cff5a285e558b0e15` (design), `4a6866d2b54bd64b147a41889dd3847dfc051727af87692888af9b39355cd0bf` (#268), `8e61347b9454a0d2df8d01cf3424cc2652b46c11cff4aa10348752f26aa52537` (#194), and `a1ad0b413ac3873d9dc7e1c0b934d891a910a58522a47e5d870a21bdbf9dba99` (#269).

What is established: the closed event-predicate subset is implementable over the existing AST; its True/False presence facts are conservative under the current Kleene evaluator; Optional-parent and separate Optional-child refinement is correctly distinguished; `ExpectNoInvocation` can use the existing deadline-scoped cumulative observation pattern and must fail any correlated invocation; suite/36–37, source/22 and diff/14 are the next unallocated versions; refusal/error alias expansion, one explicit complement, disjoint/exhaustive coverage, final-subset validation and total-count reselection after actual invocations are coherent.

What remains unresolved: source `attempts` means total invocations including the first (`binding/retry.rs:47`), while the new fallback also handles a failure that prevents the first invocation. The current generated transport deliberately propagates mapping obligations before recording an invocation (`rust/system.rs:1348–1354`) and constructs ordinary escalation from `&input` (`rust/system.rs:1366–1373`). The design must either exclude pre-invocation failures from this selected fallback, define and observe a separate delivery-attempt budget that advances on them, or constrain their legal fallback descriptors; and it must name the payload authority for escalation when no command input exists. The #268 Outcome should state the finite event-payload condition that acceptance actually implements. This was a bounded read-only review; no source, AEP artifact, build, or executable test was changed or run.

```findings
[
  {
    "file": "docs/design/conditional-binding-failure-policies.md",
    "line": 129,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "a bounded retry selected as the explicit fallback can receive a mapping/host failure before the command is invoked, but its bound advances only by total command invocation count; repeated pre-invocation failures therefore never consume the stated attempts and can retry forever"
  },
  {
    "file": "docs/design/conditional-binding-failure-policies.md",
    "line": 138,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "an escalation selected as the explicit fallback is required to emit its event even when mapping fails before a command input exists, but the contract supplies only the event handle and no payload-construction authority for that path; the current generated escalation obligation requires the completed failed command input"
  },
  {
    "file": ".engineering/planning/story/feature-request-268.md",
    "line": 63,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the story Outcome still promises reaction to one source-command outcome, although the accepted payload-only predicate explicitly cannot distinguish source outcomes that publish identical payloads; acceptance could therefore be judged against two incompatible product claims"
  }
]
```
