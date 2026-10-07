---
format: aep.planning-md/3
id: review-result:retained-replay-fixture-consistency-20261004-r1
kind: review-result
status: active
title: 'Retained replay fixture: independent review 1 approves'
relations:
- reviews: task:retained-replay-fixture-consistency
revision: 1
---
approve

```findings
[]
```

# Retained replay fixture consistency — independent whole-unit review 1

Reviewed candidate `8e7c21a0c5055635e771c2c5b2b4fa9181bc909e` against parent `3f5b7524633cfb20f178887e2c1a5f54d9eb313b` and canonical `task:retained-replay-fixture-consistency` revision 7.

The bot-authored candidate changes only `crates/verify/ess-conformance/tests/fixtures/retained-replay-runtime.go`, with 6 insertions and 3 deletions. Patch SHA-256 is `a1040c6ac78ed660f554232d53dc475f561130e5b1711a001a4449681eff9ba3`; fixture SHA-256 is `2fe25f862f9256caf4c0f529e3646456bf867ac483206f5c9bd5a1d8e9106699`. Current integration differs from the candidate parent only in planning evidence, so the reviewed source bytes merge unchanged.

The fixture now returns `actual-write` for both original and replay command results and requires that exact token at the base retained query. Direct, integer, exact-input, and adversary routes use that base method. Held and external-stale fixtures override command and query behavior and retain their distinct `observed-write` contract. No production runtime, assertion, fault mode, status expectation, callback count, or admission behavior changed.

The preserved counterfactuals passed after adding only the two result tokens, then passed again after adding the exact query-token guard: adversary 3/3 and full generated Go package 186/186 at each stage, with no failures or skips. This isolates the missing tokens as the cause and demonstrates exact-token delivery across the original and replay reads.

Final author evidence on the frozen commit passed:

- complete Rust target: 36 passed, 0 failed, 0 ignored, 0 filtered;
- actual nested Go: 189 test events passed, 0 failed, 0 skipped;
- strict affected-target Clippy, package format, repository format, and diff checks: exit 0.

The raw controls cover complete/incomplete projections, original/retry/both row faults, replay mutations, exact and neighboring Integers, two exact-input callbacks, held-state and external-stale faults, old/unknown envelope refusals, coverage, and mixed complete/legacy execution. This task changes a Go fixture and requires actual Go execution. It changes no TypeScript fixture or runtime and makes no TypeScript execution claim.

The author ran all compilers and runtimes. The reviewer independently inspected the complete change and routing, verified all hashes and manifests, parsed the retained results, and checked merge-source equivalence without rerunning the executable.

Approval covers this one-file unit. The combined full conformance package run remains a separate post-integration obligation.
