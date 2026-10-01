---
format: aep.planning-md/3
id: review-result:downstream-gaps-acceptance-round-2
kind: review-result
status: active
title: Acceptance critic, downstream gaps round 2
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
# Acceptance critic — round 2, epic:downstream-reported-gaps

approve

**What I read:** all 10 story artifacts in full (`aep plan artifact show story:feature-request-{229,257,265,266,267,268,269,270,271,272}`), the parent `epic:downstream-reported-gaps`, the round-1 record `review-result:downstream-gaps-acceptance-round-1` (its two blocker findings, verbatim), `aep plan artifact kinds` and `aep plan artifact lifecycle story`, `git status`/`git diff --stat` on `.engineering/planning/story/` to see which files actually changed since round 1, and `story:related-record-effects` (the story 229's acceptance now defers to). I `git grep`'d the tree for every diagnostic code and symbol the acceptances name (`unobservable_fact`, `ESS-SYNTH-003/004/008/010/017`, `ESS-AUTHOR-009`, `RawTrigger`, `RawFailure`, `granted_actors`) and confirmed each exists where cited.

Both round-1 blockers are resolved:

- **story:feature-request-229**: the ambiguous third bullet (deferring "effect on related records" to an unwritten specification or an uncriteria'd split) is gone. The Outcome is now narrowed to "the `state` case" only, and the split-out half moved to `story:related-record-effects`, which carries no `decomposes epic:downstream-reported-gaps` relation — it is not part of this set, and the epic's own Scope line already parenthesizes 229 as "(state inside when_related)", so the narrowing matches the parent's stated scope rather than creating a gap in it (a scope-critic question I note but do not decide).
- **story:feature-request-268**: the acceptance no longer offers a choice between an outcome-key design and an event-payload-filter design. It commits to `when: {command: C, outcome: O}` throughout, and the Origin section records the coordinator's design choice and its reason. All three bullets now read as checks against one design.

Rereading all 10 independently against the four defects, I found none. Every bullet names a before/after where the artifact is a transition (validator refusal → no refusal, ESS-SYNTH-0NN named → not named, pre-binding state → bound state), and every check is runnable (`ess specify validate`, a synthesis run inspecting scenario ids/coverage facts, a mutation that must fail). Several bullets join two clauses with "and" (229's accepting+refusing row, 265/268/269's Rust-and-Go dispatch, 271's mismatch+success), but in each case the two clauses are the single mechanism the story exists to fix (a two-sided witness, or one conformance suite run against both generated backends), not two independently-failable claims bundled for convenience — the same judgment round 1 already applied to this corpus's recurring style, which I re-verified rather than assumed.

**What I could not establish:**
- story:feature-request-268's Scope section still reads "the story's `on:` does not exist" while its Outcome/Acceptance use `when:` — a stale scope note from an earlier draft. It does not affect the acceptance's own checkability (self-consistent as written) and scope wording is not my lane, but I could not tell why it was left unupdated.
- I did not trace the ESS-SYNTH-003/ESS-SYNTH-004 cascade mechanism story:feature-request-271 names beyond confirming both codes exist in the tree; I did not verify the "cascading" relationship itself.
- Whether the "coverage fact" story:feature-request-265's second bullet names is an existing artifact shape or a new one to be introduced is outside what a grep for `coverage_build.rs` settled quickly; I did not chase it further since it does not change checkability.
- Out of my lane, not weighed into this verdict: the dense "Would collide with" cross-references (shared surfaces in `synthesize.rs`, `related_guard.rs`, `aggregate.rs`, `binding.rs` across 229/257/265–272) are `plan-critic-parallel-safety`'s and `plan-critic-design`'s question.

```findings
[]
```
