---
format: aep.planning-md/3
id: review-result:read-your-writes-parity-312-20261004-r2
kind: review-result
status: active
title: Generated read-your-writes parity final independent review round 2
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

# Story 312 generated missing-token read-your-writes parity — final independent whole-unit review round 2

Candidate `bcd8685fff937fcde4465ef387c5eef4526772f2` over original base `0b98f1bb60fd28b9d9e1a75e8fa8b8f157a0a8de` is approved for this bounded seven-path unit. The complete canonical patch SHA-256 is `6681f143ae931f6e58661fe432d3ef57979531295a9f7100616fc0dc667d04a0`.

The round-one lifecycle finding is fixed. Successful entity setup now clears generated Go and TypeScript unreadable view/command markers alongside the prior observation; TypeScript also clears `lastTotal`. A new admitted scenario performs a tokenless command, suppresses its read-your-writes query, successfully establishes an entity, then immediately expects the same view. Native, generated Go, and generated TypeScript all classify it as suite Error and make no query callback. Generated immediate expectations also reject an absent or differently named preceding read as Error without issuing a fallback Current query. Eventual reads retain their separate Current retry behavior.

All seven scenarios execute under current ordinary `/34` with typed Empty provenance and admitted historical ordinary `/32`: missing-token expectation Failed with no read, missing-token snapshot Error with no read, exact token propagation, no-prior-write Current, eventual Current, stale-view suppression, and setup-reset Error. The Go replay compares native and generated verdicts and callback transcripts. TypeScript records exact consistency requests. Dropped-token and changed-token mutants remain decisive.

The reviewer performed no test execution, runtime invocation, build, or compilation. The reviewer inspected the complete source delta, native and generated transition logic, admission boundary, helper bridges, and all raw author evidence, then independently hashed the six retained live binaries. Author evidence records the decisive pre-fix 3/5 red, corrected 5/5, neighbors 12/12 + 20/20 + 23/23 + 28/28 + 1/1, and after the lint-only helper extraction a fresh 5/5 + TypeScript 28/28. Strict six-target Clippy, package formatting, and repository formatting passed. Every reported test result has zero ignored, measured, or filtered tests.

The source is clean, bot-authored and bot-committed, and changes only the governed seven paths. No full refreshed package result is inferred from these scoped runs. Refreshed full-package, browser, and final combined acceptance remain separate story obligations after integration.

```findings
[]
```
