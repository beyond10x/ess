---
format: aep.planning-md/3
id: review-result:conditional-aggregate-measures-363-20261003-r2
kind: review-result
status: active
title: Conditional aggregate measures final independent design review
relations:
- reviews: story:feature-request-363
- reviews: component-design:conditional-aggregate-measures-source22
revision: 1
---
approve

# Conditional aggregate measures — independent whole-design review round 2 of 2

Reviewed the complete revised `docs/design/conditional-aggregate-measures.md` at commit `256b5cf863d632ffae1a525b14abb312854745d9`, SHA-256 `2fbceaadebf20e34cbbd9b38dd8690241adaed493154e46cd1e277e62cd2b085`, together with story 363 revision 6, component design revision 3, the immutable round-one review, current source seams, and the approved expression-family and binding-causal-observation contracts. The design is coherent and resolves all seven round-one findings. No remaining blocker, contradiction, stale mapping, or unbound compatibility consequence was found.

This is design approval only. The named implementation, generated-product, reader, old-byte, actual target, hard-limit, and independent implementation-review evidence remains required; none is inferred from the retained old CLI probes or this static review.

## Round-one findings

1. **Lexical/resolved boundary — resolved.** The revised contract defines an input-only `RawAggregate.where: Option<LexicalPredicate>`, delayed environment-aware semantic assembly, resolved-only `Aggregate.where` and `ResolvedAggregate.where`, a separate closed canonical output DTO, direct-domain/compiler rechecks, and compile-fail barriers against lexical persistence (`conditional-aggregate-measures.md:64-68`). This matches the approved expression-family rule that lexical predicates cannot serialize, evaluate, enter IR, or contribute to digests (`expression-family-source22.md:47-67,88-92`). Explicit true/false and empty-input refusal remain distinct (`conditional-aggregate-measures.md:31-33`).

2. **Aggregate traits — resolved.** The design preserves `PartialEq`, `Eq`, `PartialOrd`, `Ord`, and `Hash` with the old tuple first and an exhaustive tagged structural key for `Some(Predicate)` (`:70-76`). It binds `cmp == Equal` to structural equality and equal hashes, forbids lexical/display/serializer fallback, uses `Number::cmp` plus normalized `Number::exact_text`, and includes signed zero, equivalent decimals, and adjacent integers above 2^53. `None` preserves the old ordering and hash feed. The public struct/input-DTO source migrations are deliberately breaking Rust API changes assigned to the next 0.x minor release; persisted old bytes remain pinned separately.

3. **Paging conflicts — resolved.** Declared/unused accounting unions outer-filter, measure, and paging reads, while paging conflict validation receives outer-filter plus every measure-predicate read (`:37-39`). A measure-only page or size read refuses, and omitting both slicing arguments retains the complete view when separate required measure parameters are supplied (`:101-102`). This preserves `view/paging.rs`'s select-versus-slice rule and parameterless full-read behavior.

4. **Typed diff and rendered authority — resolved.** The design replaces display-only comparison with typed function/input/skip-absent/where comparison and classifies add/remove/change/narrow as `FieldAggregateChanged`; resolved-equivalent spellings do not change (`:82-84`). Explicit true remains distinct from omission. One canonical resolved description supplies display, docs, and OpenAPI while unconditioned descriptions remain byte-identical. Story scope now includes `crates/verify/ess-diff/src/diff.rs`.

5. **All-six execution — resolved.** Acceptance is the Cartesian product of count, count-distinct, sum, min, max, and average with native, generated Rust, generated Go, and live browser/WASM application lanes, plus native, emitted Go, and emitted TypeScript suite runners (`:88-94`). Every function has matching/nonmatching members, exact expected output, dropped and inverted predicate faults, and function-specific decisive values. List/quantified and composite predicates execute at the aggregate site in every lane. Existing empty, Optional, exactness, parameter, and membership controls remain additive (`:92,106,112`).

6. **Finite execution — resolved.** Repeated serialized predicates count against the shared 4 MiB closed-program bound; depth 32, 4096 rows/groups, 32 MiB state, and the single 16,000,000-unit observation counter remain shared (`:54-60`). The design assigns deterministic charges to measure/member visits, visited predicate nodes, and quantified element visits; preserves existing grouping/arithmetic/program charges; defines short-circuit accounting, checked pre-operation charging, exact-boundary success, and next-charge refusal. Exhaustion aborts the whole observation without partial rows, Complete status, or private values. Exact/one-over work, bytes, depth, and quantified controls run across native and generated targets.

7. **Governed dependencies and scope — resolved.** Story 363 records exact `depends_on` edges to 361, 362, 233, and 200, and includes the paging and diff seams. The design explains each dependency and requires its implementation acceptance before dispatch (`:114`). Suite selection is capability-driven rather than inferred from numeric major order.

## Whole-contract review

The sibling `where` syntax preserves every existing function argument and `skip_absent`, refuses unknown/duplicate/empty shapes, and reuses the closed typed view predicate grammar (`:15-35`). Its environment is restricted to one immutable source row's observable fields, identity, lifecycle state, and declared parameters. It cannot see group results, aliases, other measures, input/subject/related namespaces, or host time. Every leaf type-checks even when short-circuited.

Evaluation order is unambiguous: validate parameters and outer-filter rows, form existing groups, independently select each measure's members from the original partition, then apply existing absent and arithmetic rules (`:43-52`). Unknown makes the whole observation nonpassing, never false or partial. Zero-selected measures retain their group and preserve the existing count, count-distinct, required-sum, skip-absent-sum, extrema, and average empty results. Grouped and ungrouped empty behavior, Optional absence, overflow, exact arithmetic, and shared lifecycle snapshots remain unchanged.

Compatibility is closed at every layer (`:62-80`). Source22 gates the new source member while unchanged older source, canonical source, IR, digest, and generated bytes remain exact. Ordinary/coverage aggregate observations use 38/39; predicates requiring Family F vocabulary use an explicitly compatible 40/41 reader contract that retains all aggregate authority. Genuine old readers refuse before callbacks, new readers reject the member in older envelopes, contract admission reprojects and compares canonical IR, foreign readers validate predicates and formats, and selected suites/reports retain parent provenance. The design does not add report, history, or manifest meaning silently.

Synthesis uses real before/after rows and outcomes, includes same-group matches/nonmatches, a second-group decoy, an outer-filter decoy, entering/leaving membership, all-false membership, empty ungrouped input, and Optional values (`:86-94`). It depends on the approved causal cut rather than injecting target state or accepting a constant result. Unsupported lowering cannot satisfy required healthy controls. OpenAPI, docs, schema, and generated artifacts remain owned projections with drift checks.

The named acceptance set (`:98-114`) covers source/version admission, measure-only parameters and paging collisions, lexical persistence barriers, trait laws, behavioral diff, all-six application and runner execution, finite boundaries, independent grouping, empty/Optional arithmetic, real membership changes, reader authority, and decisive target faults. The faults explicitly cover dropped, inverted, or swapped predicates; whole-view filtering; dropped zero-selected groups; stale parameter/state reuse; Unknown-as-false; and rounded large integers.

## Seven-question fit review

1. **Need:** supported. Separate filtered views cannot express one coherent scorecard observation.
2. **Class:** supported. This is a bounded aggregate-view expressibility addition, with no ratio, having, time-bucket, join, free expression, or state-injection expansion.
3. **Existing idiom and evidence:** supported. The retained CLI results are correctly limited to old admission evidence, and current source shows the absent member.
4. **Fit:** supported. One sibling `where` reuses the accepted typed predicate environment and preserves grouping, paging, Unknown, and arithmetic authority.
5. **Second adopter:** supported. Warehouse total, available, and reserved-value measures exercise the same independent-membership contract.
6. **Cost and compatibility:** supported. The design binds lexical/output DTO restructuring, the Rust API/minor-release consequence, old bytes, typed diff, suite readers, generated products, finite budgets, and the full target matrix.
7. **Alternatives:** supported. Separate views fail the one-observation need; nested per-function objects duplicate established grammar; the sibling form uniformly extends all six functions.

Classification counts: `missing` 0; `contradicts` 0; `stale mapping` 0; `spec-only` 0; `unclear` 0. All design sections, all round-one findings, all seven fit questions, and every named acceptance family were reached.

The review was static and read-only. No compiler, target, generated program, formatter, AEP mutation, repository edit, or execution claim was made. The hardening defect-plant step is inapplicable to an unimplemented design and was outside the authorized read-only scope.

```findings
[]
```
