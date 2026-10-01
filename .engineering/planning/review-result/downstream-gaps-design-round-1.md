---
format: aep.planning-md/3
id: review-result:downstream-gaps-design-round-1
kind: review-result
status: active
title: Design critic, downstream gaps round 1
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
approve

What I read: 10 story artifacts (`feature-request-{229,257,265,266,267,268,269,270,271,272}`) and `epic:downstream-reported-gaps`, whole body each, via `aep plan artifact show`; `aep plan artifact relations` (vocabulary); `aep plan artifact graph --format json` (16.8k lines, searched for every occurrence of the ten ids as either node or edge target — none appear as an edge target anywhere in the store, and none of the ten declares an edge to another besides `serves vision:O2` / `decomposes epic:downstream-reported-gaps`); `aep plan artifact validate` (no output names any of the ten); `aep plan artifact waves` (full store run, filtered to pairs where both sides are in this set).

Findings: none.

What I found and why it doesn't change the verdict:

- **No cycle, no serialising chain.** Zero declared edges exist among the ten — no `depends_on`, no `blocks`, nothing. `waves` reports no cycle for this set and does not exit 2.
- **Heavy file collision, correctly out of my lane.** `waves` reports 34 pairwise collisions among the ten (dominated by `synthesize.rs`, `related_guard.rs`, and the 268/269 pair sharing 10 files across domain/compiler/synth/gen). Every story's own `Scope` section already names its collisions under "Would collide with," and the epic's own `## Order` section states scheduling is delegated to `aep plan artifact waves` and that "the synthesis stories that share `synthesize/aggregate.rs` or the relation-guard arrangement run in sequence." That is exactly the rubric's carve-out: an edge (or, here, wave ordering) recording a shared file is not a serialising-chain defect, and the drafter has already named the trade-off rather than hidden it. This is parallel-safety's lane, not mine, and it is already handled at the epic level.
- **No split abstraction.** I checked the two highest-collision pairs closely (229/270/271/272 on `related_guard.rs`; 268/269 on the binding declaration surface). Each of the four related-guard stories names its own diagnostic code and its own independently synthesizable scenario (229: `unobservable_fact` admission of `state`; 270: ESS-SYNTH-008; 271: ESS-SYNTH-003/004; 272: ESS-SYNTH-017 on the aggregate path) — four separate bugs in one region, each demonstrable alone. 268 (per-outcome binding trigger) and 269 (per-refusal failure policy) extend different fields of the same binding declaration (`RawTrigger` vs `RawFailure`) toward different, independently acceptance-testable outcomes; neither's Outcome or Acceptance text requires the other's mechanism to be described.
- **No horizontal slice.** Every story's acceptance ends in a demonstrable, runnable scenario or diagnostic change (a synthesized scenario, a validate refusal removed, a generated-server check) — none is a layer-only slice waiting on siblings to be end-to-end.

What I could not establish: whether `story:feature-request-229`'s acceptance line — "The effect-on-related-records half is specified here and implemented in its own story if it does not fit one unit" — names a story that exists anywhere in the store; I did not find one among the ten or in the wider graph. That is a scope-coverage question (does the set cover what 229's own outcome promises), not a design-shape one, so I'm naming it out of my lane rather than as a finding.

```findings
[]
```
