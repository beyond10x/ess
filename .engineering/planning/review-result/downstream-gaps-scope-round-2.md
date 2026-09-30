---
format: aep.planning-md/3
id: review-result:downstream-gaps-scope-round-2
kind: review-result
status: active
title: Scope critic, downstream gaps round 2
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
**approve**

**What I read:** `epic:downstream-reported-gaps` whole body first, written down as promises before touching any item; all 10 items whole body (`story:feature-request-229, -257, -265, -266, -267, -268, -269, -270, -271, -272`); `aep plan artifact graph` filtered for `decomposes: epic:downstream-reported-gaps` and for `story:related-record-effects`; `aep plan artifact kinds` and `aep plan artifact relations` (confirmed `decomposes` is the drafted-from edge); `story:related-record-effects` whole body (the artifact 229's related-records half moved to); round-1's four `review-result:downstream-gaps-*-round-1` records for continuity, since this is round 2 of the same set.

**Promises extracted from the parent, and traced: 18 of 18.**

- 5 category promises (Outcome's colon-list): *related-row guards and values* → 229, 270, 271, 272; *aggregate group keys* → 257, 272; *bindings that move state* → 266; *authorization* → 265; *binding language gaps it reported* → 267, 268, 269. All five traced, none duplicated — 257 and 272 both close ESS-SYNTH-017 but from distinct causes (257: group-key chain not recognizing a `RelatedField` set; 272: `reach()`'s related-guard refusal on the aggregate path), and 229/270/271/272 each close a distinct diagnostic (`unobservable_fact`; ESS-SYNTH-008; ESS-SYNTH-003/004; ESS-SYNTH-017).
- 10 issue-scope promises (Scope: #229, #257, #265–#272): each has exactly one story with a matching `refs github:beyond10x/ess#NNN`; the graph shows exactly these 10 `decomposes epic:downstream-reported-gaps` and no eleventh. Traced 10/10.
- 1 promise that each story names its conformance scenarios — every `## Acceptance` names a scenario id or diagnostic code. Traced.
- 1 exclusion — reporter's-scratch mini specs for #270–#272 "not copied into this repository" — none of 270/271/272's scope lists a copied scratch path; boundary held.
- 1 narrowing promise — the parent's own Scope line parenthesizes #229 as "(state inside when_related)". 229's Origin now names `story:related-record-effects` (confirmed to exist, `refs #229`, explicitly "the second half... split out... so that story is checkable") for the other half, outside this epic's `decomposes` edges. This is the honest-omission pattern the rubric names, and — new since round 1 — it is now backed by a real sibling artifact rather than only a promise inside 229's own acceptance text.

No gap, no reach beyond the parent (265's and 268's `## Decisions`/coordinator-chosen design notes pick *how* to implement an already-promised outcome, not new scope), no two items claiming the same outcome, no silent narrowing left unexplained.

**What I could not establish:** the exact GitHub issue bodies for #229/#257/#265–272 — I traced coverage from the epic's own Outcome/Scope text and the stories' `refs`, not from GitHub itself. Out of my lane, not part of this verdict: the acceptance and parallel-safety defects round 1 found and marked `fixed` (229's and 268's acceptance bullets; 266/267/269/272's "Would collide with" omissions) — I did not re-grade those, only confirmed the fixes did not move anything into or out of this epic's scope.

```findings
[]
```
