---
format: aep.planning-md/3
id: review-result:conditional-aggregate-measures-363-20261003-r1
kind: review-result
status: active
title: Conditional aggregate measures independent design review round 1
relations:
- reviews: story:feature-request-363
- reviews: component-design:conditional-aggregate-measures-source22
revision: 1
---
needs-revision

# Conditional aggregate measures — independent design review round 1

Reviewed `docs/design/conditional-aggregate-measures.md` (SHA-256 `f1534edf4ee88cb2af30c848636db0e32aa0b9d06b1a7a42fc728c82ed3d3b8c`) as introduced at `17e02294edece7c65e81bdce9bb17ac2e50eb42e`, the governing component design and current story revision 4, the approved binding-causal-observation and expression-family designs, and the cited source seams. The inspected source paths are byte-identical to `74a67d7cd`. This was a static, read-only pass: no compiler, formatter, target, generator, AEP mutation, repository edit, or publication was performed.

The sibling `aggregate.where` grammar, group-before-independent-selection rule, ungrouped/grouped empty behavior, `skip_absent` ordering, Unknown-as-nonpassing rule, source22 and suite38/39 plus conditional40/41 allocations, and required native/generated/browser lanes are directionally coherent. Seven findings remain.

## Findings

### 1. Blocker — the source-to-resolved predicate boundary is not bound at the aggregate parser

The proposal says to add a typed predicate to the “raw/domain aggregate” and only says to reuse the lexical-to-resolved boundary “where applicable” (`docs/design/conditional-aggregate-measures.md:52`). The approved expression-family design is stricter: a separate `LexicalPredicate` is nonserializable and nonevaluable, while only the environment-resolved `Predicate` may enter IR, digests, or suites (`docs/design/expression-family-source22.md:47-67,88-92`). Current `RawAggregate` owns a custom serde reader (`crates/specify/ess-domain/src/view.rs:450-582`), and `TryFrom<RawViewSpec>` converts it to `Aggregate` before entity/type validation supplies the view environment (`view.rs:1638-1717`). “Where applicable” leaves an implementor able either to discard source22 lexical information before the source row and parameters are known, or to retain an unresolved operand in the serializable/domain object.

Resolution: specify the exact three representations and conversion points. `RawAggregate.where` must retain the nonserializable lexical source form through parsing; semantic assembly must resolve it once against the source entity/state/identity and declared parameters with the shared source22 checker; `Aggregate` and `ResolvedAggregate` must hold only the resolved `Predicate`. Canonical back-conversion must serialize only the resolved predicate form. Direct domain/compiler assembly must repeat the check, and compile-fail/API tests must prove lexical values cannot serialize, hash, enter IR, or reach a suite. Empty source forms must be refused before their normalization to `Predicate::Always`, while explicit `true` remains a decided spelling if it is intended to be legal.

### 2. Blocker — adding `Predicate` has an unresolved public Rust trait compatibility consequence

The proposed domain member is not mechanically compatible with the current public type. `Aggregate` derives `PartialOrd`, `Ord`, and `Hash` (`crates/specify/ess-domain/src/view.rs:354-365`), while resolved `Predicate` derives only `Default`, `PartialEq`, and `Eq` (`crates/specify/ess-primitives/src/predicate.rs:521-607`). Adding `Option<Predicate>` directly removes those existing trait implementations unless predicate ordering/hash semantics are introduced. The design promises unchanged older bytes and products (`conditional-aggregate-measures.md:52`) but does not decide this source/API compatibility.

Resolution: preserve the existing public trait surface with a reviewed canonical ordering/hash representation for resolved predicates, or explicitly classify and version the Rust API break and identify all consumers. Add a compile-time compatibility control for the chosen `Aggregate` trait surface. Do not derive ordering from lexical spellings, because source22 permits different spellings of one resolved predicate.

### 3. Blocker — measure parameters do not preserve the existing paging-parameter conflict rule

The proposal makes parameter usage the union of the outer filter, all measure predicates, and paging (`conditional-aggregate-measures.md:35`), but does not decide overlap. Existing paging explicitly refuses a page/size parameter read by the outer filter because it would both select and slice rows; an omitted page read is also defined to return the full filtered view (`crates/specify/ess-domain/src/view/paging.rs:12-20,78-84,162-176`). Current native execution correspondingly allows both paging parameters to be absent (`crates/verify/ess-conformance/src/interpret/views.rs:27-40,87-112`). If a measure may read `param.page` or `param.size`, the same request simultaneously selects measure membership and slices rows, and a valid parameterless full read makes that predicate Unknown.

Resolution: include all measure-predicate parameter reads in the row-selection set passed to paging validation and preserve the existing refusal when either paging parameter is read by any measure. Keep the larger union only for declared/unused checking. Add a negative measure-only `page`/`size` collision case and a control showing an omitted paging pair still returns the whole view when no measure reads it.

### 4. Blocker — semantic diff and canonical descriptions can miss a changed condition

The proposal requires canonical IR/descriptor authority and generated OpenAPI/docs prose (`conditional-aggregate-measures.md:52-56,64`) but never binds ESS diff behavior. The approved expression-family design requires resolved predicate changes to be behavioral (`docs/design/expression-family-source22.md:610-624`). Current aggregate diff compares only `ResolvedAggregate::to_string()` (`crates/verify/ess-diff/src/diff.rs:1757-1793`), and the current display/description contains function, input, and `skip_absent` only (`crates/specify/ess-compiler/src/ir.rs:1516-1585`). A condition can therefore change, disappear, or be added while the existing diff sees identical computation text; docs/OpenAPI can also omit the new authority if only the field is added.

Resolution: bind one canonical resolved-condition rendering used by aggregate `Display`/`describe`, docs, and OpenAPI, or compare the predicate directly in `ess-diff` while rendering a stable condition separately. Adding, removing, narrowing, or otherwise changing `where` must emit `FieldAggregateChanged` (or a new explicitly classified view change). Add semantic-diff tests for add/remove/change and equivalent canonical forms, and add `crates/verify/ess-diff` to the story scope.

### 5. Blocker — admission of six functions is stronger than the required execution matrix

The design promises `where` uniformly on all six aggregate functions (`conditional-aggregate-measures.md:15,31`), but the witness requirement executes only “at least two differently filtered measures plus an unconditional total” (`:62`), and the named all-six control is admission-only (`:70`). The Rust and Go generated query emitters have separate count, distinct, sum, average, and extrema branches (`crates/generate/ess-synth/src/rust/behaviour/query.rs:271-353`; `crates/generate/ess-synth/src/go/behaviour/query.rs:386-449`), so count/sum success cannot prove that selection is applied before `count_distinct`, `min`, `max`, and `avg`. The promise that scalar, list, and composite predicate forms work at this new site (`conditional-aggregate-measures.md:31`) likewise has no site-specific execution control.

Resolution: require a conditional healthy case for each of count, count_distinct, sum, min, max, and avg in every required native/generated Rust/generated Go/browser lane and in the Go/TypeScript suite readers. Every case needs matching and nonmatching members in one surviving group. Add function-specific dropped/inverted-condition faults, plus at least one list/quantified and one composite condition at the measure site so forwarding only simple comparisons cannot pass. Retain the existing exactness and empty/Optional controls rather than replacing them.

### 6. Blocker — conditional predicate evaluation is not charged to the approved finite budgets

The new algorithm evaluates every measure predicate for every member of every original group (`conditional-aggregate-measures.md:39-44`), which multiplies work by rows, measures, and predicate nodes. The approved causal design caps contract/program metadata, live rows/groups, private state, predicate depth, and total evaluation work; crossing a bound must be named nonpassing and cannot produce a partial Complete result (`docs/design/binding-causal-observation.md:219-233`). The conditional design neither says that serialized predicates count toward the 4 MiB program bound nor defines how per-row/per-measure predicate work consumes the 16 million-unit budget.

Resolution: include every serialized measure predicate in the shared program-size bound and define deterministic work accounting for each visited measure/member/predicate node (including quantified elements). Budget exhaustion must abort the whole observation with the existing private, value-free nonpassing result before returning rows. Add boundary and one-over controls in native, generated Rust/Go, and browser acceptance.

### 7. Warning — governed dependencies and one required source surface remain prose-only

The story says implementation follows #361/#362 and applicable expression slices (`.engineering/planning/story/feature-request-363.md:44,62`; design line 78), but its relation graph contains only `decomposes` and `serves` (`story:feature-request-363.md:10-12`). Its current scope also omits `crates/verify/ess-diff` (`:13-39`), which finding 4 shows is required. This can make the story dispatchable without the causal/typed-expression prerequisites its own acceptance requires.

Resolution: record explicit governed predecessor edges to stories 361 and 362 and to the exact expression-family implementation artifact(s) whose lexical/resolved and 40/41 semantics this site consumes; add the diff path to scope. Keep conditional use of 40/41 data-driven at suite selection rather than treating numeric ordering as compatibility.

## Seven-question fit review

1. **Need:** supported. One response with independently selected measures is not represented by separate outer-filtered views (`story:feature-request-363.md:48`; design line 7).
2. **Class:** supported. This is a view/aggregate expressibility gap, not a transport, state-injection, or free-expression feature (`story:feature-request-363.md:49`).
3. **Existing idiom/evidence:** supported. The retained CLI results are correctly labeled admission-only, and current source confirms `RawAggregate` has no `where` (`story:feature-request-363.md:50`; `view.rs:450-468`).
4. **Fit:** needs the representation and paging revisions above. Sibling `where` is coherent; unresolved source22 operands and paging aliases cannot be left implicit.
5. **Second adopter:** supported. Inventory totals/availability/reserved value exercise the same one-observation need (`story:feature-request-363.md:52`).
6. **Cost/compatibility:** incomplete. Source22 and 38/39→40/41 are identified, but Rust trait compatibility, semantic diff, complete all-six target proof, and finite work accounting are missing.
7. **Alternatives:** supported. Separate views do not express one scorecard; nested per-function argument objects duplicate the established map grammar; ratios/having/time buckets/joins remain out of scope (`story:feature-request-363.md:54`).

Classification counts: `missing` 7; `contradicts` 0; `stale mapping` 0; `spec-only` 0; `unclear` 0. All proposal sections and all seven fit questions were reached.

No implementation or execution status is inferred from the old CLI logs or this review. The hardening defect-plant step was not applicable to this unimplemented design and was prohibited by the read-only/no-execution brief; the review therefore makes no green-behavior claim.

```findings
[
  {
    "file": "docs/design/conditional-aggregate-measures.md",
    "line": 52,
    "category": "architecture",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the aggregate.where source representation is described only as a typed raw/domain member with lexical resolution reused where applicable, but the approved source22 contract requires a separate nonserializable LexicalPredicate resolved against the view environment before any Predicate can serialize, hash, enter IR or reach a suite; bind the exact raw, domain and resolved types and conversion/recheck points, including empty-form refusal and canonical back-conversion"
  },
  {
    "file": "docs/design/conditional-aggregate-measures.md",
    "line": 52,
    "category": "compatibility",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "adding Option<Predicate> directly to public Aggregate would remove its current PartialOrd, Ord and Hash implementations because Predicate lacks those traits; preserve a reviewed canonical trait surface or explicitly version and classify the Rust API break, and prove lexical spellings cannot determine ordering or hashing"
  },
  {
    "file": "docs/design/conditional-aggregate-measures.md",
    "line": 35,
    "category": "correctness",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the union-of-parameter-reads rule does not preserve the existing refusal when page or size also selects rows; include all measure predicate reads in paging conflict validation and test a measure-only paging collision plus the parameterless whole-read control"
  },
  {
    "file": "docs/design/conditional-aggregate-measures.md",
    "line": 54,
    "category": "compatibility",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the proposal does not require ess-diff or the shared aggregate description to include where, while current aggregate diff compares only Display text that contains function, input and skip_absent; bind add/remove/change as a behavioral view diff, stable docs/OpenAPI rendering, and equivalent-canonical controls"
  },
  {
    "file": "docs/design/conditional-aggregate-measures.md",
    "line": 62,
    "category": "acceptance",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the required execution uses only two filtered measures plus a total and all-six coverage is admission-only, so separate count_distinct/min/max/avg generator branches or list/composite predicates can ignore where and still pass; execute every one of the six functions with matching/nonmatching rows in every required lane, add function-specific condition faults, and exercise list/quantified plus composite predicates at this site"
  },
  {
    "file": "docs/design/conditional-aggregate-measures.md",
    "line": 43,
    "category": "limits",
    "severity": "blocker",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "per-measure predicates multiply evaluation work but are not assigned to the approved 4 MiB program and 16 million work-unit limits; define deterministic predicate-byte and row/measure/node accounting, whole-observation nonpassing exhaustion, and boundary/one-over target controls"
  },
  {
    "file": ".engineering/planning/story/feature-request-363.md",
    "line": 10,
    "category": "scope",
    "severity": "warning",
    "verdict": "needs-revision",
    "origin": "introduced",
    "message": "the design requires 361/362 and applicable expression implementation first, but the governed graph records no predecessor edges and story scope omits the required ess-diff surface; add exact dependencies and crates/verify/ess-diff to scope"
  }
]
```
