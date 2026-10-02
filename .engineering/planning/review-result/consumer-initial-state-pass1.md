---
format: aep.planning-md/3
id: review-result:consumer-initial-state-pass1
kind: review-result
status: active
title: Scenario isolation and cross-caller first independent review
relations:
- reviews: story:feature-request-312
revision: 1
---
needs-revision

```json
[
  {
    "file": "crates/verify/ess-conformance/src/synthesize/grant.rs",
    "line": 155,
    "category": "missing-same-row-caller-witness",
    "severity": "blocker",
    "verdict": "NEEDS-CHANGE",
    "origin": "independent-source-review",
    "message": "Arrangement is defined by a different command name, so a scenario that creates and updates/refuses the same identity through one command has an empty arranging set and is silently skipped. The attributed mixed-assignment path likewise assigns by command name, keeping both invocations under one caller. Two eligible principals are expressible, but a credential-partitioned implementation can still pass without a same-row cross-caller witness or a coverage explanation."
  }
]
```

Review of the current 32-file #312 unit against its canonical AEP story and binding design, before the correction/final freeze. Reviewer executions: 0. The earlier 39af snapshot was superseded by the coverage suite-reference/35 allowance, and the owner is now correcting this finding; this review does not approve either moving snapshot.

Concrete source: compiler fixture upsert-by-existence.yaml PutItem first creates on unknown_instance and then updates the same input identity. Give two attribute-free declared actors equal may grants for PutItem. Its updated scenario invokes PutItem for both arrangement and action. grant::cross_caller computes commands={PutItem}, removes PutItem from arranging, and skips. The existing same-command create-or-refuse InstallSwitch/already-installed family provides the analogous duplicate witness. For attributed actors whose credentials do not affect the branch, synthesize/caller.rs assigns Who::Second to every invocation of the command and crosses_callers remains false. The design requires the exact row to cross callers where expressible; comparing separately created rows does not satisfy it.

Repair must distinguish invocation role rather than merely command name. Attribute-free and caller-insensitive cases can preserve outcomes while changing the acting principal. Caller-sensitive cases must be planned against their actual mixed authority or report a precise remaining composition limitation; do not mutate expected authorization by rewriting attributes after synthesis.

Other inspected surfaces: typed provenance is mandatory exactly for /34–35 and forbidden on legacy /1–33; fresh suites select /34, coverage /35, and selected children preserve the original parent bytes. Native, Go and TypeScript readers close the provenance vocabulary. Human diagnostics state a logical namespace precondition rather than claiming a physical reset. Browser replay admits /35 only through its existing supported-step checks and visibly displays the precondition. Neither CountReport/2 nor source syntax gains an invented authority. These surfaces yielded no additional concrete production finding in this source review.

Validation limits: producer reports actual native/Go/TypeScript/WASM runs; this reviewer did not execute them. The Go/TS parity test compares exact counts/outcomes but checks diagnostic codes by presence in logs, so it establishes code inclusion rather than exact code-set equality; WASM compares the exact diagnostic set. Malformed provenance tests inspected here exercise native admission; a source review of matching generated readers is not a claim that every malformed case ran in every runtime. Root owns pending generated-fixture/version expectation migration after import. Those pending integration checks are not silently treated as completed by this review.
