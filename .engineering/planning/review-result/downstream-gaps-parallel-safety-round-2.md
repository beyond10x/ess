---
format: aep.planning-md/3
id: review-result:downstream-gaps-parallel-safety-round-2
kind: review-result
status: active
title: Parallel-safety critic, downstream gaps round 2
relations:
- reviews: story:feature-request-229
- reviews: story:feature-request-257
- reviews: story:feature-request-265
- reviews: story:feature-request-266
- reviews: story:feature-request-267
- reviews: story:feature-request-268
- reviews: story:feature-request-269
- reviews: story:feature-request-270
- reviews: story:feature-request-271
- reviews: story:feature-request-272
revision: 1
---
# Parallel-safety review — round 2, `epic:downstream-reported-gaps`

**approve**

## What I read

10 story bodies (`aep plan artifact show story:feature-request-{229,257,265,266,267,268,269,270,271,272}`), the full `aep plan artifact graph` (confirms zero `depends_on`/ordering edges among the 10 — only `serves`/`decomposes`), and the full `aep plan artifact waves` run, filtered to `collision:` lines where both sides are in this 10-item set (34 pairs, all `(inferred)` except one leg of 257×272). I cross-checked every one of those 34 pairs, symmetrically, against each story's own new "Would collide with" line and against round 1's record (`review-result:downstream-gaps-parallel-safety-round-1`).

All 5 round-1 findings are fixed and verified independently, not just trusted from the `outcomes` tag:

| Round-1 finding | File | Fixed how |
|---|---|---|
| 266 omitted subject_fact.rs vs 229/271 | `.../story/feature-request-266.md` | now lists `229 on subject_fact.rs` and `271 on subject_fact.rs` |
| 272 omitted subject_fact.rs vs 229/266/271 | `.../story/feature-request-272.md` | now lists `subject_fact.rs` for all three |
| 269 omitted 265 (plan.rs, synthesize.rs) | `.../story/feature-request-269.md` | now lists `265 on plan.rs, synthesize.rs` |
| 266 omitted 265 (synthesize.rs) | `.../story/feature-request-266.md` | now lists `265 on synthesize.rs` |
| 267 omitted 265 (synthesize.rs) | `.../story/feature-request-267.md` | now lists `265 on synthesize.rs` |

Beyond re-verifying those five, I re-derived the full 10×10 collision matrix from the live `aep plan artifact waves` output and confirmed every one of the 34 pairs is now named, with the correct shared file(s), on **both** sides' "Would collide with" notes — e.g. 268↔269 (9 shared files across `ess-gen`, `ess-synth`, `ess-compiler`, `ess-domain`, `ess-conformance`, one doc) is listed identically and completely on both 268's and 269's bodies. No pair in the set is missing, no pair names a shared file the other side omits, and no story's scope is empty (all 10 carry populated cited/inferred scope lists, so none is "unassessed").

## What I could not establish

None. All 10 items were placeable via cited or inferred scope (story:feature-request-265's 19-entry scope skews toward `cited`; story:feature-request-268/270 skew toward `inferred` only — both are honestly labelled in their own confidence lines, which is a design/scope concern, not a parallel-safety defect). Whether the epic-level ordering (sequencing these collisions across waves) is itself adequate is out of my lane; each collision is *named*, which is what this pass checks — the CLI's own wave computation already places all 10 in distinct, non-overlapping waves (1, 1, 2, 3, 4, 6, 9, 12, 14, 15, 16 respectively), consistent with the documented collisions.

```findings
[]
```

**Store**: `~/.local/state/worktree/trees/b10x/ess/ess-gaps-plan`. Round-1 record read: `review-result:downstream-gaps-parallel-safety-round-1`.
