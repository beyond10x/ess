---
format: aep.planning-md/3
id: review-result:downstream-gaps-acceptance-round-1
kind: review-result
status: active
title: Acceptance critic, downstream gaps round 1
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
needs-revision

story:feature-request-229 — the acceptance's third bullet defers "the effect-on-related-records half" to being "specified here" or "implemented in its own story if it does not fit one unit," but no specification for that half exists anywhere in the story body (only the Outcome names it) and "does not fit one unit" names no checkable criterion, so nobody can tell whether that half of the story's own Outcome is done — .engineering/planning/story/feature-request-229.md:48

story:feature-request-268 — the acceptance's first bullet lets a binding declare either an outcome key (`on: {command: C, outcome: O}`) or an event payload filter without choosing between them, and bullets 2–3 ("runs after `O` and not after a sibling outcome," "dispatch honour the filter") only read as a single check under the outcome-key design, so the bullet's own claim cannot be confirmed by one check as written — .engineering/planning/story/feature-request-268.md:52

What I read: all 10 story artifacts in full (`aep plan artifact show story:feature-request-{229,257,265,266,267,268,269,270,271,272}`), the parent `epic:downstream-reported-gaps`, `aep plan artifact kinds`, `aep plan artifact lifecycle story`, and the two `.engineering/planning/story/*.md` source files for line citations; `git grep` against the ess tree (present in this worktree) for `unobservable_fact`, `ESS-SYNTH-017`, `ESS-SYNTH-010`, and `binding.rs`'s `when:`/`RawTrigger` to confirm the symbols the acceptances name are real.

What I could not establish: whether ESS-SYNTH-003/004/008 (271) and the `docs/design/binding-delivery-guarantees.md` "inherent limit" clause (267 bullet 2, "or the refusal states it is an inherent limit") name a real disjunction or an unresolved alternative — I did not exhaustively grep every diagnostic code, and did not treat 267's "or" as a finding since, unlike 268, it does not fork the underlying design. Also out of my lane: the dense "Would collide with" cross-references naming shared surfaces (`synthesize.rs`, `related_guard.rs`, `aggregate.rs`) across 229/257/266/267/268/269/270/271/272 are `plan-critic-parallel-safety`'s and `plan-critic-design`'s question, not acceptance's, and did not set this verdict.

```findings
- file: .engineering/planning/story/feature-request-229.md
  line: 48
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance's third bullet defers "the effect-on-related-records half" to being "specified here" or split into its own story "if it does not fit one unit," but no specification for that half exists in the story and "does not fit one unit" names no checkable criterion, so nobody can tell whether that half of the Outcome is done
- file: .engineering/planning/story/feature-request-268.md
  line: 52
  category: acceptance
  severity: blocker
  verdict: needs-revision
  origin: introduced
  message: the acceptance's first bullet lets a binding declare either an outcome key or an event payload filter without choosing, and bullets 2-3 only read as one check under the outcome-key design, so no single check can confirm the bullet's own claim
```
