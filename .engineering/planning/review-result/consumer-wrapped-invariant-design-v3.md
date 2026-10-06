---
format: aep.planning-md/3
id: review-result:consumer-wrapped-invariant-design-v3
kind: review-result
status: active
title: 'Wrapped invariant design v3: bounded review approval'
relations:
- reviews: story:optional-value-invariant-observation
revision: 1
---
approve

Reviewed immutable candidate `wrapped-value-invariant-design-v3.md`, SHA256 `7d68859b7b16edc3ee0e0448229353e45ee06c59e8b79b25e6e65fbbfebddf39`, against the three findings in the prior caller v2 review and the cited source seams. Reviewer execution count: **0**. This is design review only; no compiler, tests, browser, target execution, source changes or AEP writes.

```findings
[]
```

The three prior blockers are closed in the design:

- **WVI-D2-1, semantic numeric cost:** v3 lines350–378 specifies value-based coefficient/exponent normalization, zero handling, ordinary decimal rendering and checked length before allocation. Equivalent admitted spellings share a cost; historical serialization and raw document-byte admission remain separate. This is implementable from native `Number::exact_text` (`crates/specify/ess-primitives/src/facts.rs:269`) without reconstructing a lost transport token. The Go `json.Number` and TS `JsonNumber.raw` distinction is now explicitly handled, rather than mistaken for a common spelling. The design does not expand the accepted numeric value domain or authorize rounding to make a debit.
- **WVI-D2-2, deterministic work schedule:** v3 lines380–458 defines logical S/O/F/P events, order, multiplicity and scalar debits independently of fused or repeated physical traversals. S is once per row; O terminals trigger F once, reused across the invariant list; each invariant receives a full P preflight including all branches and quantifier iterations. Duplicate fact records collapse by typed path and kind, while distinct logical value locations do not. The finite graph, declaration/index/UTF-8 map order and explicit record-kind order provide a portable traversal. Native lexical `FactSource` rebinding already supplies the needed context behavior (`crates/specify/ess-primitives/src/predicate.rs:655–709`); generated typed evaluators remain explicit implementation scope. Exact cross-runtime event traces and N−1/N/N+1 tests are required at lines668–672. This closes the design ambiguity without claiming those traces are already implemented or measured.
- **WVI-D2-3, marker erasure:** v3 lines466–512 requires `ConformanceSuite::Serialize` itself to refuse incomplete state before fields, preserves Clone and checked transformations, audits reconstruction and guards borrowed-field serializers. That addresses the existing derived Serialize/Deserialize escape (`crates/verify/ess-conformance/src/scenario.rs:101–117`), including `to_value`. The explicit audit recognizes `coverage::Document` bypasses the suite serializer and keeps its admission checks (`coverage.rs:889–925`), and requires completeness before `coverage_build::finish_inventory` extracts/merges fields (`coverage_build.rs:145–179`). Rechecking constructors/copies after integration prevents treating the listed paths as an exhaustive future guarantee. Historical complete bytes and standalone assertion authority are preserved.

The browser agreement also now matches the actual product. Lines134–140 and615–637 distinguish full original-byte/lineage admission and provenance comparison from per-DTO authentication against IR. The shared Runner handles the new expectation without a second JavaScript evaluator or new ABI operation. The generic typed presenter must retain the new fields and keep its own resource limits. Actual acceptance uses the existing `browser_response_conformance.rs` packaging helpers and concrete independent target fixture, both ordinary and coverage routes, exact emitted host and real Firefox. Library-only WASM does not complete browser acceptance. No browser production edit is assumed necessary when the shared admission/serialization changes suffice.

No remaining concrete contradiction was found in this bounded re-review. Root still owns binding the final design, version registration, dependency/scope decisions and implementation dispatch. This approval does not establish runtime parity, measured limits, old-reader refusal, source arrangement completeness or all-feature browser completion; the specified negative/boundary matrices remain required implementation evidence.
