---
format: aep.planning-md/3
id: review-result:adversary-5w-related-pass-1
kind: review-result
status: active
title: Adversary pass 1, related (the-5-waves)
relations:
- reviews: story:related-record-value-source
revision: 1
---
Adversary pass 1 against story:related-record-value-source (#166), aep:adversary, 2026-09-28, the-5-waves wave 3.

verdict: NEEDS-CHANGE
cases: added 4, red 3 (suite counts incomplete: build dir deleted mid-run for disk)
origin: introduced 3, pre-existing 0, undecided 0

New cases in `adversary_related_pass1.rs` (ess-conformance, ess-domain). Red: an implementation reading the first referenced row passes; an owns-arranged owner bound by an input via leaves the read unasserted; an owns relation carried by via does not settle an ambiguous identity type and the hint remedy is refused. Green: correct target passes both documents.

Coordinator routing: all three to the implementor.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":389,"category":"mutant","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"introduced","message":"The decoy is always arranged after the referenced row, so an implementation that reads the first row passes."},
 {"file":"crates/verify/ess-conformance/src/synthesize/related.rs","line":418,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"When the input via is already bound to an owns-arranged owner, point_at declines, so the read is unasserted."},
 {"file":"crates/specify/ess-domain/src/command/related_value.rs","line":75,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"An owns relation carried by via does not settle an ambiguous identity type, and the hint remedy is refused."}]
```
