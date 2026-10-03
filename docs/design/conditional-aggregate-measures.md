# Conditional aggregate measures

Status: revised after independent design review round1; final design review and implementation are pending. Inspected source: `74a67d7cd`, with the review's representation, trait, paging, diff and resource seams rechecked at `ef516eed0`. Source21 remains reserved for one-time responses; this addition belongs to source22.

## Need and existing authority

One aggregate row must report several independently selected measures, for example all cases, completed cases and the cost of escalated cases. A source-level view filter selects the same rows for every measure, so separate filtered views cannot express one coherent scorecard observation. Issue363 proposes nested `count: {where: ...}` and `sum: {field: ..., where: ...}` arguments. Those spellings are evidence of the need, not the selected syntax.

Current `view.rs::RawAggregate` holds one function plus sibling `skip_absent`; `Aggregate` and compiler `ResolvedAggregate` hold the function, input and absent policy. Native `interpret/views.rs::query` selects rows with a typed view filter and fails on Unknown; `project` then groups those rows and evaluates each measure. `aggregate.rs` defines exact numeric behavior and empty results. Reuse these authorities.

A retained pre-history combined CLI validates the repository's aggregate-views fixture with source-level filters (exit0), but refuses the requested nested count condition with `unknown field where` (exit1). This is a parser/admission probe, not target execution or proof about the current candidate binary. Its source and log hashes are recorded in the owning story. Current source inspection confirms the missing field. Existing separate filtered views remain the workaround; they are not a replacement for one observation containing all measures.

## Source syntax and typed environment

Add one optional sibling `where` to the aggregate map, applying uniformly to its existing six functions:

```yaml
fields:
  - {name: total, type: Integer, aggregate: {count: {}}}
  - name: completed
    type: Integer
    aggregate: {count: {}, where: state == Completed}
  - name: escalated_cost
    type: Integer
    aggregate: {sum: cents, where: escalated == true}
  - name: selected_mean
    type: Optional<Decimal>
    aggregate: {avg: cents, where: state == Completed}
```

Keep the existing function argument forms, exactly-one-function rule and sibling `skip_absent`. Do not add a second field-argument object or nested count grammar. Unknown keys, duplicate where keys, null/empty predicates and multiple functions refuse. `where` reuses the existing nonempty typed view predicate grammar, including its scalar, list and composite forms; it is not a free-form JSON expression. Add no aliases such as a measure-level `filter` or `having`.

Explicit Boolean `true` and `false` are legal and resolve to Always and Never respectively. Reject null, empty text, empty sequence and empty map at the source reader before normalization; they cannot become an accidental Always. An omitted condition remains None, distinct from explicitly authored true, including in canonical descriptions and structural diff.

The environment is one source row's observable fields, identity and lifecycle state plus declared `param.*`, just like the outer view filter. It never sees a group result, projected alias, another measure, command input, subject/related namespaces or host time. The expression family's exclusion of `now` from view filters applies here. Dotted paths and typed parameter operands compose only through the accepted expression-family implementation, with its ordinary type/refusal rules. Every predicate leaf must type-check even under a short-circuit branch.

View parameter usage is the union of the outer filter, all measure predicates and existing paging parameters. A parameter used solely by a measure is valid; an undeclared parameter and a declared-but-unused parameter remain refused. A predicate may read a source field not projected in the result. Optional absence follows the existing typed predicate rules; explicit definedness can turn a missing value into a known result. Unknown is never silently false. A well-typed statically false measure predicate is meaningful and allowed: a nonempty group still has a zero/absent measure and can catch a target that drops the predicate. An invalid state or type remains refused.

This declared/unused union does not relax paging conflicts. Pass the union of outer-filter and every measure-predicate parameter read as the selection set to `view/paging.rs` validation. A page or size parameter in that set refuses even when only a measure reads it. With neither paging parameter supplied, a view still returns the complete filtered/grouped result; a measure cannot introduce a required read of those omitted slicing parameters.

## Evaluation and empty results

For one immutable query observation:

1. Validate query parameters and apply the outer view filter to source rows.
2. Partition those rows by the existing typed group keys. Ungrouped views retain their one partition even when empty; grouped partitions exist only when at least one outer-selected row has that key.
3. Independently evaluate each measure's `where` over every member of that original partition. True admits that row to this measure; False excludes it. Unknown makes the complete view observation undetermined/non-passing through the target's existing error contract. Never return a partial scorecard or mutate another measure's membership.
4. On the selected values, apply the existing `skip_absent`, type equality, sum, extrema and rounding semantics. Ordering and paging apply to the completed result as today.

A measure selecting no rows does not delete its group. Count and count_distinct are0; required-input sum is0; min/max/avg are absent. Preserve the distinct existing rule that sum with skip_absent over no present values is absent. Where selection happens before absent-input handling, so an excluded absent value cannot make an otherwise determined measure unknown. A row selected for a measure whose numeric field is required but unavailable still refuses normally. No aggregate arithmetic or overflow rule changes.

Two measures may overlap, be disjoint or be logically complementary; ESS does not assume they partition the group. Each result uses the same outer membership and query parameters. A measure predicate on state must observe the same lifecycle value that grouping and the outer filter observed.

### Finite execution

The causal-observation limits also govern these computations. All serialized measure predicates, including repeated predicates, count toward the shared 4 MiB canonical contract/program metadata limit. Retain the existing depth32, 4096 rows/groups and 32 MiB private-state bounds. No separate per-measure budget resets the shared 16,000,000-unit observation counter.

Charge one unit before visiting each measure/member pair, including an unconditional measure, and one before each predicate node actually visited. Charge one before each quantified collection-element visit, then charge every visited body node normally; nested quantifiers repeat these charges. Short-circuiting follows the shared predicate evaluator's deterministic declaration order and charges only visited nodes, while admission still validates every leaf. Existing grouping, aggregate arithmetic and observation-program charges remain additional; a condition cannot replace or erase them. Check the counter before each operation with checked arithmetic. Exactly the allowed budget is admissible; the next charge refuses. Program bytes are measured on the canonical closed program before target callbacks, not on a renderer's prose.

Exhaustion aborts the entire observation through the existing named, value-free nonpassing budget result. It returns no partial rows, no Complete status and no private input, value, hash or excerpt in diagnostics. Native, generated Rust, generated Go and browser controls execute at the exact work and byte boundaries and one over; predicate depth and nested quantified work also have boundary controls. Test fixtures use a bounded execution-policy seam to reach small work boundaries and additionally pin the production constants; they may not silently increase the production limits to admit a fixture.

## Representation, compatibility and readers

Use three exact representations. The input-only `RawAggregate.where` is `Option<LexicalPredicate>`, preserving authored operand distinctions. Parsing the aggregate map and grouping shape must not resolve it. The existing infallible `From<&RawAggregate> for Aggregate` cannot accept a conditioned input: replace that path with fallible semantic assembly supplied with source format, source entity observable fields/state/identity, declared view parameters and the binder environment. Resolve once using the shared source22 checker. `Aggregate.where` and compiler `ResolvedAggregate.where` are `Option<Predicate>` containing only resolved values. Restructure the current `TryFrom<RawViewSpec>` conversion so lexical aggregate input stays in the input-only view assembly until that environment is available; an environment-free convenience conversion can admit only unconditioned input and must refuse a conditioned one with a named missing-environment error. Never stash a lexical value in `ViewSpec`, a hidden side table or an executable enum.

The input-only lexical aggregate and enclosing input DTOs do not implement Serialize or Hash. Canonical source output uses a separate closed output representation built only from the resolved domain model; its aggregate where is rendered from resolved Predicate, including explicit fact mappings. Do not add a serializer to LexicalPredicate just to keep the current raw DTO serializer deriving. Schema generation describes the accepted input grammar through its owning custom JsonSchema implementation; it is not permission to serialize input DTOs. Resolve canonical output through the same typed admission path when reading it back. Direct domain assembly and compiler resolution repeat semantic checks against the view environment, even if a caller supplied Predicate directly. API/compile-fail controls cover lexical assignment to domain aggregate and IR, serialization, hashing/digest input and suite construction.

The source22 format fence is carried by `EssIr.format`; all newly admitted source syntax and its serialized resolved meaning require22. Through21, reject where at its authored location naming22. Unchanged earlier models retain their exact canonical source round-trip, IR bytes, digests and generated products.

### Public aggregate comparison and hashing

Preserve Aggregate's existing `PartialEq`, `Eq`, `PartialOrd`, `Ord` and `Hash` traits. Keep the old comparison tuple `(function, input, skip_absent)` first, then compare the optional resolved predicate using a private structural key. None sorts before Some. The key has an explicit tag per resolved Predicate/Operand variant and recursively compares its fields in declared order; path segments, binder names, operators, literals and ordered child/value vectors remain represented. It must cover every new resolved expression variant exhaustively, with no wildcard or lexical spelling fallback. Logical equivalence is not structural equality: do not sort, deduplicate or Boolean-normalize children in this key.

FactValue ordering uses its existing typed Ord; numeric comparison uses Number::cmp, preserving exact large Integers. Hash the same tagged structure, including collection lengths and variant tags, with numeric payload from Number::exact_text (facts.rs:260), which normalizes equal decimal values and signed zero. Never use the current JSON/binary64 number serializer, display text for a whole predicate, or fallible serialization with a fallback hash. This private key neither adds global traits to Predicate nor changes persisted bytes or digest algorithms. For None preserve the prior Aggregate hash feed as well as its ordering; for Some append a presence discriminator and the structural predicate key. Hash collisions between unequal values remain ordinary allowed collisions, not an equality mechanism.

Compile-time controls require all five traits on Aggregate. Property/table controls require `cmp == Equal` iff structural Eq and equal values yielding equal hashes, covering every resolved variant, nested composites/quantifiers, separately resolved equivalent spellings, signed zero, equivalent decimals and adjacent Integers above 2^53. Pin unconditioned ordering/hash feed against the prior tuple. Adding the optional public field still requires updating Rust struct literals; list that source migration in release notes and use the next 0.x minor release, never a patch-only release. This is a Rust API consequence separate from source22/persisted format allocation.

Source22 is already allocated to this bundle and not released separately. Do not create another source major for363. The aggregate observation program belongs to the already reserved ordinary38/coverage39 pair from `binding-causal-observation.md`; its closed measure descriptor must explicitly carry and validate the typed predicate. That reservation is not proof those readers are implemented. New readers reject this member in older envelopes, and genuine old readers refuse the new pair before callbacks. If a predicate uses separately versioned expression vocabulary, select the explicitly compatible expression40/41 contract, preserve all aggregate authority and prove the combined reader matrix; numeric version ordering alone is not compatibility evidence.

Rust contract admission reprojects source and compares the canonical IR, so a changed/removed measure condition cannot be smuggled into an unchanged contract. Emitted Go/TypeScript and any WASM/replay carrier must validate every admitted predicate and its format before target activity. Selected-suite/report carriers retain full parent provenance. No new report count, history event or manifest meaning is required by conditional measures; any implementation finding requiring one returns to the coordinator for explicit versioning.

### Semantic diff and rendered authority

Replace ess-diff's display-only aggregate comparison with typed function/input/skip_absent/where comparison. Adding, removing or changing a resolved condition, including narrowing it, emits the existing behavioral `FieldAggregateChanged` classification; source spellings resolving to identical Predicate do not produce a change. Explicit true versus omitted where remains a structural change, as with the IR member, and is not silently erased. Extend aggregate Display/describe with one shared canonical resolved condition suffix only when present; unconditioned text remains byte-identical. Generated docs and OpenAPI use that shared description so neither can omit the condition. Test add/remove/change/narrow, identical resolved compact/map spellings, and unchanged older descriptions. Regenerate affected docs/schema/projections through their owners.

## Synthesis and target obligations

Extend aggregate dependency extraction to every measure predicate's state/field/parameter reads. The shared aggregate authority for361/362 supplies exact row identity, immutable query snapshots and causal completion where bindings can alter those dependencies. Do not reuse a delta formula that assumes every member contributes to every measure. For a tested command, use the actual before/after rows and outcome to calculate each measure independently, including a state/field change that changes membership without adding or deleting a row.

Arrange witnesses with matching and nonmatching rows in the same group, a second group decoy and an outer-filter decoy. Observe at least two differently filtered measures plus an unconditional total in the same result. Require explicit witness coverage of entering/leaving a selected subset, all-false membership with a surviving group, empty ungrouped input and selected optional values. If the bounded arranger cannot produce a required contrast or causal cut, retain a named synthesis refusal; a constant target result is not sufficient evidence.

The execution matrix is the Cartesian product of all six functions (count, count_distinct, sum, min, max, avg) and all required application lanes (native interpreter, generated Rust, generated Go, live browser/WASM). Execute each six-function contract through native, emitted Go and emitted TypeScript suite runners as well; merely admitting six descriptors does not cover the matrix. Each function sees matching and nonmatching members in a surviving group, and its exact healthy output must distinguish both dropping and inverting its own condition. Run function-specific dropped/inverted-condition targets and require the named observation to fail, rather than another unrelated assertion. Distinct fixtures include duplicate selected values; extrema and average fixtures make excluded values decisive. Include a list/quantified condition and a composite condition at this aggregate site in every application/runner lane, with faults that fail to forward those predicate forms. Keep all existing empty, Optional, exactness, parameter and membership-change controls alongside this matrix.

Native interpreter, semantic/history view observations, actual generated Rust and Go applications, served view endpoints and live browser/WASM products must implement the admitted profile or return their named unsupported-lowering result. Unsupported cannot satisfy this bundle's required healthy controls. Native, emitted Go and TypeScript conformance runners must execute the observation program and reject corrupt descriptors. OpenAPI and generated documentation describe each measure's predicate and empty behavior; JSON Schema is regenerated through its owner. UI projection consumes the existing typed result fields, not a new renderer-side filter.

No JavaScript or Go implementation is introduced as an independent tool: new committed executable checks are Rust; existing generated runtime resources remain under their owning generators. No production state injection or ad hoc test-only query endpoint is added.

## Named acceptance and decisive faults

- `conditional_measures_admit_only_from_source22`: all six function siblings, old-format location refusal, malformed/duplicate keys and unchanged unconditioned source/IR bytes.
- `conditional_measure_parameters_are_view_inputs`: parameter read only by a measure, undeclared/unused refusal, wrong parameter value and source-field-versus-result-name collision.
- `conditional_measure_paging_inputs_do_not_select`: page and size read only by a measure each refuse; omission of both paging arguments retains a complete healthy read with separate measure parameters supplied.
- `conditional_measure_predicates_are_resolved_before_persistence`: raw lexical retention, environment-aware assembly and direct-domain/compiler rechecks; lexical serialization/hash/IR/suite compile-fail controls; explicit true/false and empty-form distinction.
- `conditional_aggregate_traits_preserve_structural_identity`: compile-time traits, exhaustive resolved structural comparison/hash controls, exact numbers and unconditioned tuple compatibility.
- `conditional_measure_changes_are_behavioral_diffs`: added/removed/changed/narrowed conditions, equivalent resolved spellings, and shared canonical docs/OpenAPI descriptions.
- `conditional_measure_all_six_functions_execute`: complete six-function application/runner matrix with decisive per-function drop/invert faults, list/quantified and composite conditions.
- `conditional_measure_budgets_abort_the_whole_observation`: exact and one-over program bytes/work/depth, quantified charges, value-free refusal and no partial/Complete result across native/generated Rust/Go/browser.
- `conditional_measures_keep_group_membership_independent`: two subsets plus total, same-group nonmatches, second-group and outer-filter decoys, a group whose conditional measures all select nothing.
- `conditional_measures_preserve_empty_and_optional_semantics`: empty ungrouped input, required empty sum0, skip_absent empty sum absent, min/max/avg absent, known absence and Unknown, numeric exactness and existing overflow refusal.
- `conditional_measures_follow_membership_changes`: a real state change and a real predicate-field update move rows into and out of a measure; actual binding-induced changes use the approved completion/cut authority.
- `conditional_measure_readers_reject_changed_authority`: source/IR/descriptor mismatch, forged old-major metadata, genuine old-reader refusal, ordinary/coverage/selected-parent agreement before callbacks.
- `conditional_measure_targets_detect_faults`: actual native/generated Rust/Go/browser healthy execution; native/Go/TypeScript runner agreement; faults that drop, invert or swap the predicate, apply it to the whole view, drop a zero-selected group, reuse a previous query's parameter, use a stale state, treat Unknown as false or return rounded large Integers must fail the named observation.

These are required tests, not observed results. Scoped lint, generator drift checks, affected suites and independent implementation review remain required. Governed predecessors are story:feature-request-361 and story:feature-request-362 for observation authority, story:feature-request-233 for the shared lexical/resolved source22 implementation and expression40/41 descriptors, and story:feature-request-200 for typed parameter operands at this site. Those exact implementation artifacts must pass their own acceptance before dispatch of this unit; a reviewed design alone does not discharge a dependency. The suite selector remains capability-driven, never based on numeric version ordering.
