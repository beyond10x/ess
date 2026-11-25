---
format: aep.planning-md/3
id: decision-blocker:synthesis-parts-and-reuse
kind: decision-blocker
status: open
title: Does ESS split a synthesis into persisted parts that together hold every scenario of the system, merge them into the suite one synthesis writes, and reuse parts across runs by the digest of their inputs
summary: 'ST-D5, options A, B, C: do not build now, parts and a merge, or parts with reuse; architecture decision with the owner, no default'
tags:
- architecture
- gap-reports-2026-11-24
relations:
- blocks: story:a-synthesis-may-be-split-into-parts-that-are-kept-by-the-digest-of-their-inputs-and-merged-into-the-suite-one
revision: 5
---
Does ESS split a synthesis into persisted parts that together hold every scenario of the system, merge them into the suite one synthesis writes, and reuse parts across runs by the digest of their inputs: options A, B, C of ST-D5, an architecture decision with the owner. There is no default; nothing waiting on it takes one. The question has been with the owner since 2026-11-24; there is no answer yet.

## What the report asks

A split into parts that hold every scenario of the whole plus a merge equal to the single synthesis, and incremental reuse of unchanged parts. Both need a persisted part document and a merge that claims the bytes of the single synthesis (a determinism rule). The saving is unmeasured: the reporter's two largest parts took as long as the whole. By design since 0.97.3 the runs of all `--component` parts do not cover the whole suite, and nothing merges suites.

## Options

- A. Do not build it now. Close need 2 and need 3 with the committed-suite recipe (ST-D1), `--jobs` and ST-D4; record the measurement and the cause that decides whether parts are cheaper than the whole (`scoping_reproduces`, the actor-scope extension) as the condition for reopening. The story is closed, not built.
- B. Parts and a merge, no reuse: a remainder part for the scenarios that cross components, a part document (`ess-conformance-part/1`, working name), a merge command whose output is `cmp`-identical to `synthesize` over the corpus. A persisted format and a determinism claim.
- C. B plus reuse: parts kept under the digest of their inputs, so a change redoes the parts whose digest moved. B's cost and a stable unit under change: a change to one command can move scenarios of commands that arrange through it, and the memories are shared across commands.

The coordinator recommends A, because no saving is shown for the reporter's cut and B or C commit a format and a determinism rule before the cost of a part is known. The owner decides. The Acceptance of the blocked story is written for C.

## Decisions

- ST-D5: A, taken by the operator session on 2026-11-25 on the owner's word of 04:03 CET (he is away); reversible until a release carries it: no parts are built now (not built); the need is answered by `--jobs` and the committed suite.
