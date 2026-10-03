---
format: aep.planning-md/3
id: review-result:binding-arrangement-drop-design-20261003-r2
kind: review-result
status: active
title: Binding arrangement and drop final independent design review
relations:
- reviews: story:feature-request-266
- reviews: story:feature-request-267
revision: 1
---
approve

All four findings in `review-result:binding-arrangement-drop-design-20261003-r1` are closed. The design now reconciles the old mapping-only target documentation with the shipped exact-count and every-invocation claims while retaining Unsupported for adapters without tracing. It defines `observe_invocations` as a cumulative, non-consuming, correlation-scoped snapshot whose deadline does not reset history, including duplicate attempts and later-step reads. It bounds pre-trigger destination identity authority to literals, known trigger inputs and independently captured values, preserves conversion and Optional obligations, and assigns `BindingGap::DestinationIdentityUnavailable` to post-trigger-only/generated/unresolved identities. The drop sequence now explicitly queries and snapshots the actual subject before arming the refusal and trigger, runs mapped-input and exact-total-attempt observations through their full windows, then queries and compares the same complete visible row after the count window.

The complete pass also rechecked eventual binding settlement, direct-route preservation, wrong-state eligibility, unresolved/cyclic chain disposition, aggregate-completion exclusion, empty-selection every-invocation matching, empty-input exact count, cumulative late-retry detection, malformed-input retry detection, forced-refusal unchanged-state authority, and the named native/generated and native/Go/TypeScript controls. The two existing invocation instructions compose without changing their serialized meaning: every-invocation rejects a wrong mapped input among all correlated attempts, and exact count with empty input counts every attempt; cumulative history makes the sequential windows retain earlier attempts. `SnapshotSubject` records the actual complete returned row and `ExpectSubjectUnchanged` consumes that snapshot after the required second query.

Reviewed at integration HEAD `aec396fe64b8913c4b995aa09e3205f1e2703634`. SHA-256: design `878dd8fa21395fb6c244e9016c62c9232761b3acee3e0ef0364336a8fe4b422d`; story #266 `db28672430296bcef8e06ee844bd84fa080f68eefe396051f2c5cc3156632845`; story #267 `163bad151f2f7efa7cf171d6e1c5591acb32555c78eb59c18b7c7e1b3eced849`; prior review `3b05600709bfe05e2d69d21b25ed4d8385e46f2fd7e383fbefe2cf307d20211b`.

This was a bounded read-only design review. No repository/AEP files were changed and no build or executable test was run; implementation and execution evidence remain later gates.

```findings
[]
```
