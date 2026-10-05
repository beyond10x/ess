---
format: aep.planning-md/3
id: review-result:ess-054-plan-design-r2
kind: review-result
status: active
title: Design critic, ESS 0.54.0 intake, round 2
tags:
- ess-0.54.0
relations:
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
story:feature-request-441 — still "documents" the refusal "and its hint" that #438 builds, so the note cannot be finished before 438 lands and no edge says so. Round 1 moved the refusal to 438, but this line did not change. Either name only today's refusal of `param.limit_s * 1000`, or record `depends_on story:feature-request-438` — .engineering/planning/story/feature-request-441.md:64
story:feature-request-458 — names #427's `ess-conformance/44`/`/45` pair as its fallback with no edge, although 438 got `depends_on 427` for the same case in round 1. Record `depends_on story:feature-request-427`, or say no suite change is possible — .engineering/planning/story/feature-request-458.md:66
story:feature-request-455 — says #454 owns "the rule that each pair the precedence order decides gets a primary witness outside the overlap and one overlap send". 454 owns only "the overlap-witness rule that existence and wrong-state scenarios resend a claimed input", so the owner's body does not state the rule 455 builds on. Either 454 states the general rule, or 455 cites `input-guard-overlap-precedence.md` rules 1–2 as its own base — .engineering/planning/story/feature-request-455.md:50

What I read: all 36 stories (Decisions, Acceptance and Scope of each; the Fit review where a seam depended on it), plus `aep plan artifact relations`, `aep plan artifact graph` and `aep plan artifact validate`.

Round-1 fixes that hold (checked against body and graph):

| Fix | Edges in the graph |
|---|---|
| 429 owns `ess/23` | 450, 452, 458, 459 → 429; 459 → 452 |
| 427 owns `/44`/`/45` | 438 → 427 |
| 441 creates the shared idioms note and test | 439, 442, 443, 444, 446, 447 → 441 |
| 438 before 439 | 439 → 438 |
| 454 owns the precedence section and witness | 455 → 454 |
| `story:ess-generate-check` split out of 435 | 435 → ess-generate-check; 436 → 435 |
| Other round-1 edges | 430→428, 437→434, 440→445, 445→426b, 450→426b, 449→451 |

- **426:** now a decision record with no Scope, so 426a and 426b do not share a surface with it.
- **Single owners:** one story owns each of these surfaces: `ess/23`, `ess-conformance/44`/`45`, `read-api-view-idioms.*`, `## Which branch answers`, `stored-rules-boundary.md`, and the `ess generate --check` verb.

What I could not establish:
- **Edges walked:** 22 `depends_on` edges between stories in the set, plus every `decomposes`, `serves` and `delivers` edge touching the set. I also walked outside the set, to `epic:downstream-reported-gaps` and `vision:O2`. There are no cycles. The longest `depends_on` chain is three stories (439→438→427, 440→445→426b, 459→452→429, 436→435→ess-generate-check), so the set does not serialise.
- **Validator:** `aep plan artifact validate` printed `valid`. It also printed unrelated stale `review-result` notes, which are not findings.
- **Out of lane (parallel-safety), not set as verdict:**
  - `spec-versions.md` (the `ess/23` row) is edited by 450, 452, 458 and 459, and `value_expression.rs` by 458 and 459, with no ordering between them.
  - 442, 443, 444, 446 and 447 each add a section to the same `read-api-view-idioms.md`, example and test.
  - `guards-and-predicates.md` is edited by 426a, 451, 454, 455 and 456, and `generate-artifacts.md` by 433 and 435.
  - 445 and 450 both edit `literal_representation` in `command.rs`.
  - 454 and 458 both change the wrong-state scenario.

```findings
- file: .engineering/planning/story/feature-request-441.md
  line: 64
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'still documents the refusal "and its hint" that #438 builds, so the note cannot be finished before 438 lands and no edge says so (round 1 moved the refusal to 438 but this line did not change); either name only today''s refusal of param.limit_s * 1000, or record depends_on story:feature-request-438'
- file: .engineering/planning/story/feature-request-458.md
  line: 66
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'names #427''s ess-conformance/44 and /45 pair as its fallback with no edge, although 438 got depends_on 427 for the same case in round 1; record depends_on story:feature-request-427, or say no suite change is possible'
- file: .engineering/planning/story/feature-request-455.md
  line: 50
  category: design
  severity: warning
  verdict: needs-revision
  origin: introduced
  message: 'says #454 owns the rule that each pair the precedence order decides gets a primary witness outside the overlap and one overlap send, but 454 owns only the rule that existence and wrong-state scenarios resend a claimed input, so the owner''s body does not state the rule 455 builds on; either 454 states the general rule or 455 cites input-guard-overlap-precedence.md rules 1-2 as its own base'
```
