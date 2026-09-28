---
format: aep.planning-md/3
id: review-result:adversary-5w-w1fix-pass-1
kind: review-result
status: active
title: Adversary pass 1, w1fix (the-5-waves)
relations:
- reviews: story:defined-over-optional-aggregates
- reviews: story:literal-fallback-after-else
revision: 1
---
Adversary pass 1 against the w1fix follow-ups (story:defined-over-optional-aggregates, story:literal-fallback-after-else), aep:adversary, 2026-09-27, the-5-waves.

verdict: CONFIRMED (pre-existing only)
cases: added 14, red 2 (ess-conformance 884→894, 4 ess-cli targets)
origin: introduced 0, pre-existing 2, undecided 1

New cases in `adversary_w1fix_{defined_aggregate_format,fallback_in_struct}.rs` (ess-conformance) and `adversary_w1fix_{authored_defined_format,predicate_page_filters}.rs` (ess-cli). Red: an optional input with an else: literal also copied plainly is never left out; authored satisfies refuses missing() over an Optional struct (ESS-AUTHOR-026). Note: the full ess-cli run was invalidated because the coordinator pruned test binaries in the build dir while the run was going.

Coordinator routing: findings 1 and 2 are gaps in this releases constructs and go to the implementor; finding 3 in the same round.

```findings
[{"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4280, "category": "boundary", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "An optional input with an else: literal that is also read plainly into an Optional field is never left out, so the literal fallback is asserted nowhere."},
 {"file": "crates/verify/ess-conformance/src/authored.rs", "line": 2248, "category": "contract-drift", "severity": "warning", "verdict": "CONFIRMED", "origin": "pre-existing", "message": "Authored satisfies refuses defined/missing over an Optional aggregate (ESS-AUTHOR-026) though synthesis admits it and formats.md documents it."},
 {"file": "crates/edge/ess-cli/src/main.rs", "line": 3516, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "undecided", "message": "ess conform author --format json with a refused scenario exits non-zero with an empty suite document and no refusal text."}]
```
