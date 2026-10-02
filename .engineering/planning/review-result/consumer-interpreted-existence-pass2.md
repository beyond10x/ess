---
format: aep.planning-md/3
id: review-result:consumer-interpreted-existence-pass2
kind: review-result
status: active
title: Supplied identity and existence corrected independent review
relations:
- reviews: task:consumer-backlog-20261002
revision: 1
---
approve

```json
[]
```

Independent source review of exact frozen patch 2153840447cc64cddc4afce1caf882268e08b27c4b04efd447754071382e4fb3, three staged files against fc676ff13. Staged patch bytes matched the retained frozen patch. Reviewer executions: 0.

Pass 1's false-duplicate blocker is fixed. Shared entity/input addresses retain the pre-guard lookup required by existence precedence. Differing addresses on commands without related guards use the existing typed input/caller/external selection and read only the selected creation's identity. The new direct regression exercises equal textual IDs across distinct Item and Slot entities in both creation orders, then verifies duplicate refusal and unchanged state for each selected entity. Root reports an actual red reproduction with the previous all-slot implementation and green correction; I inspected the regression and resulting selection code without rerunning it.

Unknown-instance creation now executes the selected creation's effects, while direct and optional input identities follow their actual payload source and generated fallback uses the existing bounded allocator. Stored and emitted identity remain one value. The controls include exact supplied IDs, updates, reuse after deletion, singleton IDs, multiple uniform guarded creations, ordinary input-refusal precedence and duplicate precedence over related-row absence/input refusal.

The differing-address plus related-guard combination remains an explicit Unsupported result, because the own-address lookup precedes related-row selection under the binding contract. Differing addresses with unresolved external alternatives also remain Unsupported. Those are retained capability limitations, not completed feature coverage, and must remain visible in the release/backlog accounting. The current patch does not invent an address or mutate state in either case.

Validation is producer-owned: reported 9 focused tests and combined 74 passing tests across 9 binaries, strict library/changed-test Clippy and formatting. No full runtime/backlog completion or release readiness is inferred from this bounded approval.
