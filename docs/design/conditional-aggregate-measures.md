# Conditional aggregate measures

Status: proposed for the accepted #363 bundle scope; independent design review and implementation are pending. Inspected source: `74a67d7cd`. Source21 remains reserved for one-time responses; this addition belongs to source22.

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

The environment is one source row's observable fields, identity and lifecycle state plus declared `param.*`, just like the outer view filter. It never sees a group result, projected alias, another measure, command input, subject/related namespaces or host time. The expression family's exclusion of `now` from view filters applies here. Dotted paths and typed parameter operands compose only through the accepted expression-family implementation, with its ordinary type/refusal rules. Every predicate leaf must type-check even under a short-circuit branch.

View parameter usage is the union of the outer filter, all measure predicates and existing paging parameters. A parameter used solely by a measure is valid; an undeclared parameter and a declared-but-unused parameter remain refused. A predicate may read a source field not projected in the result. Optional absence follows the existing typed predicate rules; explicit definedness can turn a missing value into a known result. Unknown is never silently false. A well-typed statically false measure predicate is meaningful and allowed: a nonempty group still has a zero/absent measure and can catch a target that drops the predicate. An invalid state or type remains refused.

## Evaluation and empty results

For one immutable query observation:

1. Validate query parameters and apply the outer view filter to source rows.
2. Partition those rows by the existing typed group keys. Ungrouped views retain their one partition even when empty; grouped partitions exist only when at least one outer-selected row has that key.
3. Independently evaluate each measure's `where` over every member of that original partition. True admits that row to this measure; False excludes it. Unknown makes the complete view observation undetermined/non-passing through the target's existing error contract. Never return a partial scorecard or mutate another measure's membership.
4. On the selected values, apply the existing `skip_absent`, type equality, sum, extrema and rounding semantics. Ordering and paging apply to the completed result as today.

A measure selecting no rows does not delete its group. Count and count_distinct are0; required-input sum is0; min/max/avg are absent. Preserve the distinct existing rule that sum with skip_absent over no present values is absent. Where selection happens before absent-input handling, so an excluded absent value cannot make an otherwise determined measure unknown. A row selected for a measure whose numeric field is required but unavailable still refuses normally. No aggregate arithmetic or overflow rule changes.

Two measures may overlap, be disjoint or be logically complementary; ESS does not assume they partition the group. Each result uses the same outer membership and query parameters. A measure predicate on state must observe the same lifecycle value that grouping and the outer filter observed.

## Representation, compatibility and readers

Add an optional typed predicate member to the raw/domain aggregate and a resolved-only predicate member to `ResolvedAggregate`, omitted when absent. Reuse the expression-family lexical-to-resolved boundary where applicable; an unresolved operand may not serialize, enter an IR digest or reach execution. The source22 format fence is carried by `EssIr.format`; all newly admitted source syntax and its serialized resolved meaning require22. Through21, reject where at its authored location naming22. Unchanged earlier models retain their exact source round-trip, IR bytes, digests and generated products.

Source22 is already allocated to this bundle and not released separately. Do not create another source major for363. The aggregate observation program belongs to the already reserved ordinary38/coverage39 pair from `binding-causal-observation.md`; its closed measure descriptor must explicitly carry and validate the typed predicate. That reservation is not proof those readers are implemented. New readers reject this member in older envelopes, and genuine old readers refuse the new pair before callbacks. If a predicate uses separately versioned expression vocabulary, select the explicitly compatible expression40/41 contract, preserve all aggregate authority and prove the combined reader matrix; numeric version ordering alone is not compatibility evidence.

Rust contract admission reprojects source and compares the canonical IR, so a changed/removed measure condition cannot be smuggled into an unchanged contract. Emitted Go/TypeScript and any WASM/replay carrier must validate every admitted predicate and its format before target activity. Selected-suite/report carriers retain full parent provenance. No new report count, history event or manifest meaning is required by conditional measures; any implementation finding requiring one returns to the coordinator for explicit versioning.

## Synthesis and target obligations

Extend aggregate dependency extraction to every measure predicate's state/field/parameter reads. The shared aggregate authority for361/362 supplies exact row identity, immutable query snapshots and causal completion where bindings can alter those dependencies. Do not reuse a delta formula that assumes every member contributes to every measure. For a tested command, use the actual before/after rows and outcome to calculate each measure independently, including a state/field change that changes membership without adding or deleting a row.

Arrange witnesses with matching and nonmatching rows in the same group, a second group decoy and an outer-filter decoy. Observe at least two differently filtered measures plus an unconditional total in the same result. Require explicit witness coverage of entering/leaving a selected subset, all-false membership with a surviving group, empty ungrouped input and selected optional values. If the bounded arranger cannot produce a required contrast or causal cut, retain a named synthesis refusal; a constant target result is not sufficient evidence.

Native interpreter, semantic/history view observations, actual generated Rust and Go applications, served view endpoints and live browser/WASM products must implement the admitted profile or return their named unsupported-lowering result. Unsupported cannot satisfy this bundle's required healthy controls. Native, emitted Go and TypeScript conformance runners must execute the observation program and reject corrupt descriptors. OpenAPI and generated documentation describe each measure's predicate and empty behavior; JSON Schema is regenerated through its owner. UI projection consumes the existing typed result fields, not a new renderer-side filter.

No JavaScript or Go implementation is introduced as an independent tool: new committed executable checks are Rust; existing generated runtime resources remain under their owning generators. No production state injection or ad hoc test-only query endpoint is added.

## Named acceptance and decisive faults

- `conditional_measures_admit_only_from_source22`: all six function siblings, old-format location refusal, malformed/duplicate keys and unchanged unconditioned source/IR bytes.
- `conditional_measure_parameters_are_view_inputs`: parameter read only by a measure, undeclared/unused refusal, wrong parameter value and source-field-versus-result-name collision.
- `conditional_measures_keep_group_membership_independent`: two subsets plus total, same-group nonmatches, second-group and outer-filter decoys, a group whose conditional measures all select nothing.
- `conditional_measures_preserve_empty_and_optional_semantics`: empty ungrouped input, required empty sum0, skip_absent empty sum absent, min/max/avg absent, known absence and Unknown, numeric exactness and existing overflow refusal.
- `conditional_measures_follow_membership_changes`: a real state change and a real predicate-field update move rows into and out of a measure; actual binding-induced changes use the approved completion/cut authority.
- `conditional_measure_readers_reject_changed_authority`: source/IR/descriptor mismatch, forged old-major metadata, genuine old-reader refusal, ordinary/coverage/selected-parent agreement before callbacks.
- `conditional_measure_targets_detect_faults`: actual native/generated Rust/Go/browser healthy execution; native/Go/TypeScript runner agreement; faults that drop, invert or swap the predicate, apply it to the whole view, drop a zero-selected group, reuse a previous query's parameter, use a stale state, treat Unknown as false or return rounded large Integers must fail the named observation.

These are required tests, not observed results. Scoped lint, generator drift checks, affected suites and independent implementation review remain required. Integrate only after361/362 observation authority and relevant expression semantics have passed their own acceptance.
