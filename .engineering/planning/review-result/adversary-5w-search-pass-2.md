---
format: aep.planning-md/3
id: review-result:adversary-5w-search-pass-2
kind: review-result
status: active
title: Adversary pass 2, search (the-5-waves)
relations:
- reviews: story:witness-search-beyond-64-candidates
revision: 1
---
Adversary pass 2 against story:witness-search-beyond-64-candidates, aep:adversary, 2026-09-27, the-5-waves wave 2, after correction round 1.

verdict: APPROVE
cases: added 11, red 2 (executed 886→897)
origin: introduced 1, pre-existing 1, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_search_pass2.rs`. Red (both on constructed guards nothing is shown to reach): eight contradicting disjunctions past the 64-breakdown budget; a strict chain of four compared fields. Green: four/five disjunctions, mixed types, presence-policy pairs, quantifiers, negation; pass-1 corrections hold.

Ledger against pass 1: carried 0, new 2, resolved 3. Coordinator routing: final correction states the limits (docs) and rewrites the two cases to assert todays refusal naming the story.

```findings
[{"file": "crates/verify/ess-conformance/src/witness.rs", "line": 725, "category": "boundary", "severity": "note", "verdict": "INFEASIBLE", "origin": "introduced", "message": "the 64-breakdown budget per goal refuses an eight-disjunction guard whose only satisfying breakdown is the 256th, while predicates.md says such guards synthesize"},
 {"file": "crates/verify/ess-conformance/src/witness.rs", "line": 1956, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "a field compared only with other fields gets just 0 and -1 as alternatives, so a strict chain of four compared struct members is never witnessed"}]
```
