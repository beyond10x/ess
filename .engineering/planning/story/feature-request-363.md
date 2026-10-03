---
format: aep.planning-md/3
id: story:feature-request-363
kind: story
status: draft
title: Compose conditional aggregate measures in one view row
refs:
- provider: github
  reference: beyond10x/ess#363
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

Express several conditional aggregate measures in one view row, with a defined shared source-row and query-consistency contract.

## Fit review

1. Need: a scorecard row includes total items, completed items and costs under distinct conditions. Requester proposes count:{where:state == Done} and sum:{field:cents,where:...}. Brand-free Item scorecard validates ordinary count, but installed ESS0.51.0 rejects that conditional count with unknown field where. Probe363-conditional-count-closed SHA ef1e85133cc4fe921a13161844613e890a2f6a974e879ec64c46cef8cbcd1c93. No requester syntax is adopted by this record.
2. Class: separate conditional measures are already expressible, so fewer declarations alone is convenience. A single result row containing several differently filtered measures is a real composition gap, especially if one coherent query snapshot matters. Current view-level filter applies to every aggregate (docs/design/aggregate-views.md:34,152); it cannot select different row sets per measure. The issue asks for the single row but does not explicitly specify atomicity; design must state that contract rather than infer a stronger distributed guarantee.
3. Existing idiom measured: view filter state == Done with count:{} and sum:cents validates and synthesizes6 scenarios with0 refusals on0.51.0. Probe363-filtered-idiom-closed SHA ba304666f801b38f1c8cfbc440a6d0cae07750ed549049a0a15a898aac3e4c9d. This demonstrates source/synthesis support only, not an actual implementation run. Separate views remain a valid interim idiom but are not claimed equivalent to the requested combined row. RawFunction::Count(Empty) in ess-domain/src/view.rs:474 and its closed reader at:546 explain the rejected key.
4. Fit: retain one view-level source filter before grouping, then define a per-measure predicate using existing predicate vocabulary and consistent naming. Settle whether the existing filter key is reused inside aggregation rather than introducing where. Parameters, lifecycle/related copied fields, Optional presence, grouped/ungrouped empty results and every sibling aggregate function need coherent rules. Update native evaluation, generated Rust/Go/TypeScript/WASM surfaces, synthesis, diff, schema and docs together; every conformance runtime must execute the resulting authority. No target may silently ignore the condition.
5. Second adopter: a warehouse row with total/pending/committed reservations and total committed cost; a billing row with total/unpaid/overdue invoices. Both need different selections within one aggregate result, using the same existing view/predicate contracts.
6. Cost: new authored aggregate meaning and resolved IR require coordinated ESS versioning, schema projection, diff classification, generated API/code and all-target conformance support. Carry accepted semantics in the held unreleased ess21 bundle where compatible; do not allocate a bump per issue. Exact persisted scenario changes depend on the chosen witness design. No version is reserved here.
7. Alternatives: change nothing and use separate filtered views (valid measured idiom but no combined row contract); a bespoke count_if function for each condition (duplicates aggregate/predicate vocabulary and leaves sum/siblings inconsistent); one coherent optional predicate on the existing aggregate expression (preferred design direction). Final syntax and sibling/consistency rules require the binding design before source implementation.

## Decisions

Accept, redesigned for the single-row composition gap, not as a claim that conditional counts are otherwise impossible. Keep the working multi-view idiom recorded. The proposed where spelling is not accepted yet. Remains required full-backlog design/implementation work; no completion or permanent deferral is claimed.

## Acceptance

One admitted scorecard view returns different total/done/cost measures over independently populated matching and nonmatching rows. Healthy targets pass; apply-one-filter-to-all, ignore-measure-filter, filter-after-grouping and wrong-Optional-condition targets fail. Cover parameter changes, no matching rows, grouped rows, numerical exactness and sibling aggregates. Define consistency from existing view semantics without inventing cross-service atomicity. Run actual target parity and historical reader refusal; regenerate schema and classify semantic diff.

## Scope

Inferred cross-cutting source/domain aggregation, compiler IR, shared aggregate evaluator, projections and conformance synthesis/targets. Existing typed AggregateFunction/ViewSpec declarations provide the domain home. Exact scope must be established from the binding design and repository readers before dispatch; this draft authorizes no edits to shared source format or serial-owned synthesis.
