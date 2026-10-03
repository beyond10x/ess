---
format: aep.planning-md/3
id: review-result:binding-arrangement-drop-design-20261003-r1
kind: review-result
status: active
title: Binding arrangement and drop design independent review
relations:
- reviews: story:feature-request-267
- reviews: story:feature-request-266
revision: 1
---
needs-revision

story:feature-request-267 — mandatory invocation observation for drop and flow conflicts with the current target contract, which makes `observe_invocations` mapping-only and promises an untraced target can still prove the failure policy — docs/design/binding-arrangement-and-drop.md:15

story:feature-request-267 — the design requires the exact destination identity to be arranged before the trigger but defines neither pre-trigger derivability nor a named refusal when that identity is mapped from an event field or accessor observable only after the trigger event — docs/design/binding-arrangement-and-drop.md:53

story:feature-request-267 — the drop proof depends on cumulative invocation snapshots across two sequential whole-window checks, but the design leaves “return every attempt” as adapter prose while the target API defines no cumulative-history or observation-baseline semantics — docs/design/binding-arrangement-and-drop.md:76

story:feature-request-267 — the unchanged-state step order does not explicitly require the existing pre-trigger query/snapshot and post-count query/comparison pair, so an implementation can emit an `ExpectSubjectUnchanged` claim without the snapshot authority it consumes — docs/design/binding-arrangement-and-drop.md:73

What I read: the coordinator contract and revised #266/#267 stories at the pinned integration commit, plus current binding synthesis and arrangement, binding gaps, mapping IR, scenario vocabulary, target trait, and native/Go/TypeScript runner implementations. Empty-input count and empty-selection every-invocation compose correctly in all three runners, and each instruction waits through its full eventual window.

What I could not establish: no normative cumulative per-correlation invocation-history promise, no pre-trigger rule for event-derived destination identities, and no reconciliation of drop/flow tracing with the target trait’s mapping-only obligation exist in the reviewed sources. This was a read-only design review; no execution evidence is claimed.

```findings
[
  {
    "file": "docs/design/binding-arrangement-and-drop.md",
    "line": 15,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "mandatory invocation observation for drop and flow conflicts with the current target contract, which makes `observe_invocations` mapping-only and promises an untraced target can still prove the failure policy"
  },
  {
    "file": "docs/design/binding-arrangement-and-drop.md",
    "line": 53,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the design requires the exact destination identity to be arranged before the trigger but defines neither pre-trigger derivability nor a named refusal when that identity is mapped from an event field or accessor observable only after the trigger event"
  },
  {
    "file": "docs/design/binding-arrangement-and-drop.md",
    "line": 76,
    "category": "design",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the drop proof depends on cumulative invocation snapshots across two sequential whole-window checks, but the design leaves “return every attempt” as adapter prose while the target API defines no cumulative-history or observation-baseline semantics"
  },
  {
    "file": "docs/design/binding-arrangement-and-drop.md",
    "line": 73,
    "category": "acceptance",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the unchanged-state step order does not explicitly require the existing pre-trigger query/snapshot and post-count query/comparison pair, so an implementation can emit an `ExpectSubjectUnchanged` claim without the snapshot authority it consumes"
  }
]
```
