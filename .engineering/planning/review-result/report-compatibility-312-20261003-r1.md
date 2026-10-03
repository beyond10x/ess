---
format: aep.planning-md/3
id: review-result:report-compatibility-312-20261003-r1
kind: review-result
status: active
title: Independent whole review of current-suite mutation report compatibility
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

# #312 report compatibility slice — independent whole review 1

Candidate `c5682d541bb95bf050b14c748a9acaf56e74ca32` changes only three Rust test files. No findings.

The migration binds every fresh suite/34 report to its exact admitted bytes through `CountReport::from_run`, then validates each deliberate collector stand-in again with `CountReport::from_json`. The stand-ins are explicitly labelled test setup. Exact suite digest/version, specification identity, scenario membership, category counts, and execution/conformance statuses remain enforced; the added digest, count, and status corruptions are refused both directly and through mutation collection.

The legacy compatibility test does more than relabel a current report. It removes only the new initial-state provenance before admitting the unchanged scenario bodies as suite/4, confirms the specification digest is preserved and suite-byte digest changes, executes that admitted suite against actual Billing, requires every scenario to pass, writes report/1 from the actual run, and proves mixed report/1 + report/2 collection equals the all-current result. The unsupported unchanged-scenario case still excludes exactly one shared scenario, and the original manifest/1, manifest/2, and manifest/report/3 controls and mutation-verdict assertions remain intact.

Independent execution used the three coordinator-supplied, SHA-256-verified candidate binaries without invoking Cargo:

- `adversary_mutate_dead_guard_pass1-c610d412deeb4f2b`: 5 passed, exit 0;
- `adversary_mutate_pass1-833209c184157646`: 4 passed, exit 0;
- `adversary_mutate_pass2-eaf800e3ce5fd4bd`: 10 passed, exit 0.

Total: 19 passed, 0 failed, 0 ignored, 0 measured, 0 filtered. The source checkout used by the binaries was clean at the exact candidate and each compiled test source matched the candidate blob. Author evidence separately records the same 19/19, strict focused lint exit 0, and formatter exit 0. This approval covers only the bounded report-compatibility slice and does not claim the unrelated 31-target baseline is green.

```findings
[]
```
