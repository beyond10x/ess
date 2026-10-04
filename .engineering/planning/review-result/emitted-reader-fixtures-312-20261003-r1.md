---
format: aep.planning-md/3
id: review-result:emitted-reader-fixtures-312-20261003-r1
kind: review-result
status: active
title: Emitted-reader fixture compatibility independent review
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

# Independent whole-unit review r1: story 312 emitted-reader fixture migration

Candidate `803677f9c9c7532b8b30d3e83a68b9604ad32dbb` over base `93ffe140c4ada191cc2ffcac5249968aea9bfba6` was reviewed in full. It changes only `generated_docs.rs`, `leaf_payloads_go.rs`, and `runtime_suite_admission.rs`; the exact patch SHA-256 is `648405d4cadc94180c1d22d09ae308b3e270f70394f9146d597a0b16443b0c7c`.

No actionable correctness, compatibility, or scope finding was found.

Current generated suites now remain exact `/34` and `/35` documents with typed Empty provenance. Historical all-major and leaf-vocabulary fixtures remove that current-only field and retain valid ordinary/coverage envelopes. The generated-doc probes still execute real Go and TypeScript readers for every registered major under both report-format environments. Leaf tests retain correct and faulty target modes, nonempty reports, exact Rust/Go scenario equality, and coverage input lineage. The all-major Go sweep retains future-version refusal precedence, and TypeScript registration coverage remains unchanged. All 11 tests remain, with no ignored test.

Reviewer execution was not performed: the fresh retained-binary preflight measured `11,978,960,896` bytes below the required `12,884,901,888` byte floor and refused before any binary or runtime process started. The reviewer hash-verified the exact retained binaries and audited the author's raw candidate evidence. That evidence records 5/5 + 4/4 + 2/2 passing with zero failed, ignored, measured, or filtered tests; actual Go and TypeScript paths ran without skips. Scoped strict Clippy and package formatting also passed.

```findings
[]
```
