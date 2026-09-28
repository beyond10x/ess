---
format: aep.planning-md/3
id: review-result:adversary-5w-related-pass-2
kind: review-result
status: active
title: Adversary pass 2, related (the-5-waves)
relations:
- reviews: story:related-record-value-source
revision: 1
---
Adversary pass 2 against story:related-record-value-source (#166), aep:adversary, 2026-09-28, the-5-waves wave 3, after correction round 1.

verdict: NEEDS-CHANGE
cases: added 6, red 4 (executed 1745→1751)
origin: introduced 4, pre-existing 0, undecided 0

New cases in `adversary_related_pass2.rs` (ess-domain, ess-conformance). Red: a three-variant enum decoy repeats the referenced value; `related` became a source keyword in every format, breaking ess/14-15 documents with a struct field named related; the creates-branch hint names a refused remedy; a value copied at creation passes. Pass-1 corrections hold. Disk reached 4K free during clippy.

Ledger against pass 1: carried 0, new 4, resolved 3. Coordinator routing: final correction round (held for disk); the coordinator verifies the diff.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":150,"category":"mutant","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"Decoys at first+1 and first+3 give a three-variant enum the referenced value in the second decoy."},
 {"file":"crates/specify/ess-domain/src/command.rs","line":1137,"category":"contract-drift","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"related is a source keyword in every format, so ess/14-15 documents with a struct field named related fail to parse."},
 {"file":"crates/specify/ess-domain/src/command/value_expression.rs","line":274,"category":"contract-drift","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"On a creates branch the ambiguous-input hint names a refused remedy."},
 {"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":101,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"Nothing changes the referenced field between creation and the branch, so a value copied at creation passes."}]
```
