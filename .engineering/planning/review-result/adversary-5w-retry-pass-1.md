---
format: aep.planning-md/3
id: review-result:adversary-5w-retry-pass-1
kind: review-result
status: active
title: Adversary pass 1, retry (the-5-waves)
relations:
- reviews: story:bounded-retry-bindings
revision: 1
---
Adversary pass 1 against story:bounded-retry-bindings (#165), aep:adversary, 2026-09-28, the-5-waves wave 4.

verdict: NEEDS-CHANGE
cases: added 10, red 5 (executed 1830→1840)
origin: introduced 4, pre-existing 0, undecided 0

New cases in `adversary_retry_block.rs` (ess-domain) and `adversary_retry_count.rs` (ess-conformance). Red: a late extra attempt and a retried final refusal pass the exact count; an external success branch is forced as the failure; the arrangement sets off the counted binding; retry: null is admitted. Green: attempts 2, final by outcome or error, empty final list.

Coordinator decision on finding 1: the count must reach N and stay at N for the step deadline. Routing: all four to the implementor.

```findings
[{"file":"crates/verify/ess-conformance/src/runner/bounded_retry.rs","line":80,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the exact count passes as soon as N invocations are visible, so a late extra attempt passes"},
 {"file":"crates/verify/ess-conformance/src/synthesize/bounded_retry.rs","line":41,"category":"boundary","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"on-failure forces the first non-final external branch even when it carries no error"},
 {"file":"crates/verify/ess-conformance/src/synthesize/bounded_retry.rs","line":162,"category":"boundary","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the invocation count includes invocations set off by the arrangement"},
 {"file":"crates/specify/ess-domain/src/binding.rs","line":457,"category":"contract-drift","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"on_failure {retry: null} is now admitted as a bare retry"}]
```
