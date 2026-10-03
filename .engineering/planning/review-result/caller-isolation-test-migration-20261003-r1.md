---
format: aep.planning-md/3
id: review-result:caller-isolation-test-migration-20261003-r1
kind: review-result
status: active
title: Scenario-isolation test migration independent review
relations:
- reviews: story:feature-request-275
revision: 1
---
approve

# #312/#275 scenario-isolation test migration — independent review pass 1

Candidate `ba4591de292f8b0db3b337ad359c04b8c7fea37a` over base `72167e08` changes only
`crates/verify/ess-conformance/tests/adversary_275_pass2.rs` (83 insertions, 17 deletions).

No findings. The removed global uniqueness rule conflicts with suite 34/35's accepted empty logical
namespace before each scenario. Story #275 requires freshness within one synthesized scenario, and
the migration retains that exact boundary: each sending scenario is either split at a later creating
call under a different caller or has an `UnswappedCallers` note; every split scenario compares the
second arrangement's numeric identity against its own first arrangement; at least one split and one
note must exist. A new mutation control executes all bounded-model scenarios successfully, then
reuses the first arrangement identity in the second arrangement of Amend and requires `Failed` plus
`ESS-CF-OUTCOME`. This catches the stale within-scenario creation bug rather than weakening it away.

```findings
[]
```

Author evidence shows the former assertion failed only because Amend and Rename each independently
drew `2.0`, while both used `1.0` then `2.0` within their own scenario and passed. Author compilation
and strict lint were green. Separately, the reviewer hashed the retained compiled binary as
`744caf3723697945f8dcb3ffe85f0d5907f881dca1dcc6922d5ea0809f374b99` and executed it without invoking
a compiler: 8 passed, 0 failed, 0 ignored, 0 measured, 0 filtered; exit 0. `git diff --check` exited 0.

This approval covers only the stated test migration.
