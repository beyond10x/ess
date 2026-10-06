---
format: aep.planning-md/3
id: review-result:ess-054-plan-scope-r2
kind: review-result
status: active
title: Scope critic, ESS 0.54.0 intake, round 2
tags:
- ess-0.54.0
relations:
- reviews: release-plan:ess-054
- reviews: story:feature-request-426
- reviews: story:feature-request-426a
- reviews: story:feature-request-426b
- reviews: story:feature-request-427
- reviews: story:feature-request-428
- reviews: story:feature-request-429
- reviews: story:feature-request-430
- reviews: story:feature-request-432
- reviews: story:feature-request-433
- reviews: story:feature-request-434
- reviews: story:feature-request-435
- reviews: story:feature-request-436
- reviews: story:feature-request-437
- reviews: story:feature-request-438
- reviews: story:feature-request-439
- reviews: story:feature-request-440
- reviews: story:feature-request-441
- reviews: story:feature-request-442
- reviews: story:feature-request-443
- reviews: story:feature-request-444
- reviews: story:feature-request-445
- reviews: story:feature-request-446
- reviews: story:feature-request-447
- reviews: story:feature-request-448
- reviews: story:feature-request-449
- reviews: story:feature-request-450
- reviews: story:feature-request-451
- reviews: story:feature-request-452
- reviews: story:feature-request-453
- reviews: story:feature-request-454
- reviews: story:feature-request-455
- reviews: story:feature-request-456
- reviews: story:feature-request-457
- reviews: story:feature-request-458
- reviews: story:feature-request-459
- reviews: story:ess-generate-check
revision: 1
---
needs-revision
release-plan:ess-054 — the Reconciliation says "One story per issue … #426 is split into `feature-request-426a` and `feature-request-426b`", but the set also carries `story:ess-generate-check`, a new public `ess generate --check` flag. The flag is claimed by the `delivers` edge only, and no sentence in the body names it. Add one sentence recording the #435 split and the flag it ships, so the approver sees it. — .engineering/planning/release-plan/ess-054.md:68

**What I read.** I read the parent, the 36 stories in `ids.txt`, the 34 issue JSONs and the r1 findings with `OUTCOMES.tsv`. I also ran `aep plan artifact graph`.

**Promises and tracing.** I extracted 40 promises from the parent and traced 38 to a story. The 40 are:
- 33 issue resolutions
- the 426a/426b split
- the Canon-side generator
- the shared `ess/23` bump
- the shared `/44`–`/45` suite pair
- a Fit review plus Decisions per story
- the #430 handoff
- "comment on and close every shipped issue"

The two untraced are the last two. Both are release-time steps the release-plan owns, so they are not gaps.

**Round-1 findings, each checked against the current text:**

| Story | Round-1 finding | Now |
|---|---|---|
| 426 | Duplicate note outcome with 426a | Fixed. Acceptance is "426a and 426b are implemented" and Scope is "none". |
| 435 | `--check` flag not asked for by the issue | Fixed. The flag moved to `story:ess-generate-check`, and 435 depends on it. This is the one the new finding builds on. |
| 440 | Diagnostic change in a decline story | Fixed. Reduced to a "Noted, not in scope" line. |
| 442 | Extra-key refusal in a decline story | Fixed. Reduced to a "Noted, not in scope" line. |

**Not findings, checked:**
- **Gaps:** none. All 33 issues have a story, and #426 is claimed by 426, 426a and 426b.
- **Duplicates:** none. 426 no longer claims the note, 438 and 441 split the `param * 1000` refusal and its documentation cleanly, and 449 and 451 each claim a different part of the stored-rules note.
- **Recorded narrowings, so not gaps:**
  - #434 declines the `UNMAPPED:` markers and open questions.
  - #438 does not build `default:`, `empty:` or `{in: param.L}`.
  - #450 builds enum attributes, not a reference-row noun.
  - #452 declines cross-domain cascade.
  - #459 leaves replace-a-set out.
  - #437 excludes row-set selectors.
- **Format versions:**
  - `ess/23` is introduced by 429 and extended by 450, 452, 458 and 459.
  - 442 declines without a bump.
  - The `/44`–`/45` pair sits with 427.
- **Reach, judged traceable:** 438's `in: query / explode: true` encoding (its Fit review Q6 settles the issue's `queues=a,b,c`), 439's range-selector synthesis, and 433's lane and docs work.

**Out of my lane, not counted toward the verdict:** the shared `read-api-view-idioms.*` files across 439, 441, 442, 443, 444, 446 and 447, and the `command.rs` and `set_effects.rs` ordering among 426b, 429, 445, 450, 452 and 459. These belong to the parallel-safety and design critics.

```findings
- file: .engineering/planning/release-plan/ess-054.md
  line: 68
  category: scope
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'the Reconciliation says "One story per issue" with only #426 split, but the set also carries story:ess-generate-check, a new public `ess generate --check` flag claimed only by the delivers edge; no sentence in the parent body names it, so add one recording the #435 split and the flag it ships'
```
