---
format: aep.planning-md/3
id: review-result:consumer-related-values
kind: review-result
status: active
title: Native related values independent source review
relations:
- reviews: story:interpreted-related-values
revision: 1
---
approve

```findings
[]
```

Reviewed frozen patch 1bc0bccde4892209ef65ee8b76581910ababcba9acc187771590159966adae78 against da2dc12b5f0bb83f52782b0cdeedd1ac8c5bed40. Four files: interpret/execute.rs, interpret/execute/values.rs, tests/interpreted_related_values.rs, tests/related_values.rs.

The original Store is borrowed separately from Work.next. Related reads validate the source address and entity identity type, select the exact typed row, retain identity values held outside ordinary fields, and use the existing optional presence rules. Creation's Subject carrier exception uses only compiler-admitted unchanged input sources. selected_subject uses the existing compiler selection_subject accessor after selection; it does not modify guard selection. Error payloads share the original-store context. Existing atomic commit ordering remains intact.

Read all thirteen direct tests and the added SHIPPING execution test. Controls cover original reference/referent values, typed Integer and Json identities, creation carrier IR and execution, optional absence versus null, missing rows/required fields/references, early refusal, wrong-state and stored-guard errors, and failed-operation row/event preservation. Existing suite assertions are retained.

Reviewed implementor evidence: 53 unique focused tests, strict library and changed-test lint, scoped formatting; no full-gate claim. Reviewer executions: 0. This approves the bounded RelatedField unit, not unimplemented response/replay or other conformance features.
