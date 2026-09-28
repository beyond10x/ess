---
format: aep.planning-md/3
id: review-result:adversary-5w-retry-pass-2
kind: review-result
status: active
title: Adversary pass 2, retry (the-5-waves)
relations:
- reviews: story:bounded-retry-bindings
revision: 1
---
Adversary pass 2 against story:bounded-retry-bindings (#165), aep:adversary, 2026-09-28, the-5-waves wave 4.

verdict: NEEDS-CHANGE
cases: added 3, red 2 (executed 1840→1843)
origin: introduced 2, pre-existing 0, undecided 0

New cases in `crates/verify/ess-conformance/tests/adversary_retry_pass2_count.rs` and `crates/specify/ess-domain/tests/adversary_retry_pass2_schema.rs`. Red: an arrangement command that reaches the counted binding's event through another binding sets it off before the count begins; the generated schema states `attempts` minimum 0 and allows repeated `final` names while the reader refuses both. Judgement without a case: a very large `attempts` is accepted and cannot be met inside one eventual window; the coordinator kept no upper bound.

Coordinator routing: both to the implementor's final correction (transitive `sets_off`, schema minimum 2 and `uniqueItems`); both cases green afterwards.

```findings
[{"file":"crates/verify/ess-conformance/src/synthesize/bounded_retry.rs","line":170,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"clean_trigger checks only direct emits, so a setup command that reaches the event through another binding still invokes the counted binding before the count begins"},
 {"file":"schemas/generated/ess.schema.json","line":1,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"RetryBound.attempts has schema minimum 0 and final allows duplicates, while the reader refuses attempts below 2 and repeated final names"}]
```
