---
format: aep.planning-md/3
id: review-result:adversary-5w-caller-pass-2
kind: review-result
status: active
title: Adversary pass 2, caller (the-5-waves)
relations:
- reviews: story:caller-value-source-and-guard
revision: 1
---
Adversary pass 2 against story:caller-value-source-and-guard (#168), aep:adversary, 2026-09-28, the-5-waves wave 4, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 8, red 3 (executed 2157→2165)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `adversary_caller_pass2.rs` (ess-conformance, ess-domain, ess-gen, ess-entity-runtime). Red: one attribute name at two actor types is sent at the first actor type; an identity-less excludes lets the swapped run assert an empty view; the conflicting-types refusal repeats a type. Green: input-instead-of-caller and refuse-every-caller mutants caught; 403 rule; Entity Runtime refusals.

Ledger against pass 1: carried 0, new 3, resolved 3. Coordinator routing: final correction round; the coordinator verifies the diff.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize/caller.rs", "line": 88, "category": "boundary", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "Callers::of values each attribute name once, so an actor declaring the same name at another type is sent a value of the first actor type."},
 {"file": "crates/verify/ess-conformance/src/synthesize/caller.rs", "line": 426, "category": "contract-drift", "severity": "blocker", "verdict": "NEEDS-CHANGE", "origin": "introduced", "message": "reads_every_row misses an identity-less excludes, so the swapped run asserts a caller-filtered view holds no rows."},
 {"file": "crates/specify/ess-domain/src/command/caller_value.rs", "line": 254, "category": "contract-drift", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "The conflicting-types refusal lists a type more than once."}]
```
