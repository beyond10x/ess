---
format: aep.planning-md/3
id: review-result:authored-aggregate-fixtures-312-20261004-r1
kind: review-result
status: active
title: Authored aggregate fixture migration independent review round 1
relations:
- reviews: story:feature-request-312
revision: 1
---
needs-revision

# Story 312 authored and aggregate fixture migration — independent whole-unit review round 1

Candidate `4f6af5ef7c2f5473eb56b76d338a8771e2b91729` over exact parent `b6d16b97ce381db7dfb1af3af446bac0701327e4` has one documentation finding. The six test behaviors, provenance controls, historical fixtures, target faults, and execution evidence are otherwise sound.

`crates/verify/ess-conformance/tests/aggregate_views_mutants.rs:8` says that TypeScript must refuse the suite before any callback. The same file's acceptance test explicitly admits the current suite, constructs one target, and makes that target throw. Update the module header to describe the actual TypeScript admission/target boundary. No behavioral change or rerun is required if that comment is the only correction.

The exact three-file patch SHA-256 is `1d443ee086805bee077abfc3759fa54adb517f9027e0785fefa7325a1b1f68ac`. Only the authorized Rust tests changed, all six tests remain present, and no ignored or filtered acceptance was introduced. Production source, shared helpers, generated runtime fixtures, and target implementations are unchanged.

The authored coverage cases assert current `/35`, typed Empty provenance, the direct defined-aggregate predicate control, one authored selected id, and exact full parent lineage. The eventual-view case retains its direct `used_by` and historical minimum-26 controls beside current `/34` and typed Empty. Aggregate historical ordinary 14 and coverage 11/15/17 fixtures remove current-only provenance through the existing legacy helper and reach their intended admission/refusal diagnostics. Native healthy and named-mutant assertions, actual Go healthy/filter-mutant execution, and actual TypeScript reader/target admission remain intact.

Reviewer execution was not performed. The reviewer hash-verified the exact retained scoped binaries and audited the author's raw evidence. That evidence records all six scoped tests passing with zero failed, ignored, or filtered tests, including actual Go, TypeScript compilation, and Node paths. Strict scoped Clippy and both owning format checks passed. The wider author command records 82 passed and the separately owned unchanged `upsert_by_existence_go` callback-parity failure; it is accurately retained and is not reported green.

```findings
[
  {
    "file": "crates/verify/ess-conformance/tests/aggregate_views_mutants.rs",
    "line": 8,
    "category": "documentation",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "pre-existing",
    "message": "the module contract still says the TypeScript lane must refuse the suite before any callback, while this file's acceptance test explicitly admits the current suite and proves target construction; update the header to describe the actual TypeScript admission/target boundary"
  }
]
```
