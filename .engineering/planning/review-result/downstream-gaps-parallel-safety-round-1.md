---
format: aep.planning-md/3
id: review-result:downstream-gaps-parallel-safety-round-1
kind: review-result
status: active
title: Parallel-safety critic, downstream gaps round 1
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
# Parallel-safety review — round 1, `epic:downstream-reported-gaps`

**needs-revision**

story:feature-request-266 — its scope cites `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (line 21), which `aep plan artifact waves` reports colliding with story:feature-request-229 (inferred) and story:feature-request-271 (cited), but the "Would collide with" note (line 48) hedges only `related_guard.rs` — not even in 266's own scope — and never names `subject_fact.rs` — .engineering/planning/story/feature-request-266.md:21,48

story:feature-request-272 — its scope cites `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (line 21), which `aep plan artifact waves` reports colliding with 229, 266 and 271 on that same path, but the "Would collide with" note (line 55) names only `related_guard.rs` for 229/270/271 and omits `subject_fact.rs` and 266 entirely — .engineering/planning/story/feature-request-272.md:21,55

story:feature-request-269 — its scope cites both `crates/generate/ess-synth/src/plan.rs` (line 27) and `crates/verify/ess-conformance/src/synthesize.rs` (line 45), the same two files story:feature-request-265 cites (both sides fully cited — `aep plan artifact waves` shows no `(inferred)` tag on either), but the "Would collide with" note (line 79) names 268/267/266 and never 265 — .engineering/planning/story/feature-request-269.md:27,45,79

story:feature-request-266 — its scope cites `crates/verify/ess-conformance/src/synthesize.rs` (line 17), which story:feature-request-265 also cites; `aep plan artifact waves` reports the pair fully cited on both sides, but the "Would collide with" note (line 48) names 267/268/269 and never 265 — .engineering/planning/story/feature-request-266.md:17,48

story:feature-request-267 — its scope cites `crates/verify/ess-conformance/src/synthesize.rs` (line 19), which story:feature-request-265 also cites; `aep plan artifact waves` reports the pair fully cited on both sides, but the "Would collide with" note (line 56) names 266/268/269 and never 265 — .engineering/planning/story/feature-request-267.md:19,56

**What I read:** 10 story bodies (`aep plan artifact show story:feature-request-{229,257,265,266,267,268,269,270,271,272}`) plus the parent epic; the full artifact graph (`aep plan artifact graph --format json`), which confirms zero `depends_on`/ordering edges among the 10 — only `serves`/`decomposes`; the full `aep plan artifact waves --format text` run, filtered to `collision:` lines where both sides are in this set of 10 (34 such pairs found). Confidence: all 10 items placed by `waves` (none unassessed).

**What I could not establish:** story:feature-request-268 and story:feature-request-270 declare every scope path as `inferred` (no `cited` entries at all), so any collision resting solely on their surfaces (e.g. 268×269 across ~9 files) is weaker than stated confidence in their own "Would collide with" notes suggests — noted, not itself a finding since the confidence prefix is already honest. Collisions between an in-set item and an out-of-set story (hundreds of lines in the raw `waves` output) are outside my lane and not reported. Whether the epic's Order section ("the synthesis stories that share `synthesize/aggregate.rs` or the relation-guard arrangement run in sequence") is itself adequate is a scope/design question, not mine — I note only that it doesn't mention `synthesize/subject_fact.rs` or the top-level `synthesize.rs` binding-vs-grant overlap either, which is consistent with the per-story gaps above.

```findings
- file: .engineering/planning/story/feature-request-266.md
  line: 48
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: its scope cites crates/verify/ess-conformance/src/synthesize/subject_fact.rs (line 21), which `aep plan artifact waves` reports colliding with story:feature-request-229 (inferred) and story:feature-request-271 (cited), but the Would-collide-with note hedges only related_guard.rs and never names subject_fact.rs
- file: .engineering/planning/story/feature-request-272.md
  line: 55
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: its scope cites crates/verify/ess-conformance/src/synthesize/subject_fact.rs (line 21), which `aep plan artifact waves` reports colliding with 229, 266 and 271 on that same path, but the Would-collide-with note names only related_guard.rs for 229/270/271 and omits subject_fact.rs and 266 entirely
- file: .engineering/planning/story/feature-request-269.md
  line: 79
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: its scope cites both crates/generate/ess-synth/src/plan.rs (line 27) and crates/verify/ess-conformance/src/synthesize.rs (line 45), the same two files story:feature-request-265 cites (both fully cited per `aep plan artifact waves`), but the Would-collide-with note names 268/267/266 and never 265
- file: .engineering/planning/story/feature-request-266.md
  line: 48
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: its scope cites crates/verify/ess-conformance/src/synthesize.rs (line 17), the same file story:feature-request-265 cites (fully cited on both sides per `aep plan artifact waves`), but the Would-collide-with note names 267/268/269 and never 265
- file: .engineering/planning/story/feature-request-267.md
  line: 56
  category: parallel-safety
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: its scope cites crates/verify/ess-conformance/src/synthesize.rs (line 19), the same file story:feature-request-265 cites (fully cited on both sides per `aep plan artifact waves`), but the Would-collide-with note names 266/268/269 and never 265
```
