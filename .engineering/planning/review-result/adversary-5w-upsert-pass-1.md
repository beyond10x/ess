---
format: aep.planning-md/3
id: review-result:adversary-5w-upsert-pass-1
kind: review-result
status: active
title: Adversary pass 1, upsert (the-5-waves)
relations:
- reviews: story:upsert-outcome-by-existence
revision: 1
---
Adversary pass 1 against story:upsert-outcome-by-existence (#164), aep:adversary, 2026-09-28, the-5-waves wave 4.

verdict: NEEDS-CHANGE
cases: added 13, red 6 (executed 2085→2098)
origin: introduced 4, pre-existing 0, undecided 0

New cases in `adversary_upsert_{domain,witness,projections}.rs`. Red: the page and the suite disagree on which answer comes first beside an input refusal; a deletes: sibling is admitted as the update half; the one-row check is skipped when every view is filtered; create-or-refuse collides on a shared target. Disk was at 2.5G free.

Coordinator decision on finding 1: an input-guarded refusal is answered before existence selection (the #178 precedence); cases 3 and 4 assume existence-first and are rewritten to the decision. Routing: all to the implementor.

```findings
[{"file":"crates/generate/ess-gen/src/docs.rs","line":1680,"category":"contract-drift","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The page says the creating unknown_instance branch answers before any other branch, while the suite sends an input-guarded refusal first."},
 {"file":"crates/specify/ess-domain/src/command/outcome_shapes.rs","line":410,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"names_existing counts deletes, so a creating unknown_instance paired only with a deletes: sibling is admitted."},
 {"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":158,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"The one-row check is silently skipped when every view on the entity is filtered or parameterised."},
 {"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":77,"category":"concurrency","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"Only creates_unknown branches get a fresh identity, so create-or-refuse scenarios collide on a shared target."}]
```
