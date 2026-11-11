---
format: aep.planning-md/3
id: story:an-aggregate-view-reads-through-a-reference
kind: story
status: active
title: An aggregate view reads through a reference
summary: Aggregate views refuse fields read through a reference; generated queries keep such reads as obligations (workbench report item 13, new on 0.92.0)
tags:
- backlog
- external-request
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:navigation-reaches-any-depth-in-view-filters-and-guards
- depends_on: story:aggregate-group-keys-are-expressions
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view/measure.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/navigation.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/view_reference.rs
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: inferred
  path: docs/design/language-grids.md
revision: 9
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:39Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:40Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A figure per group is computed over rows selected and grouped through a reference of the source:
downtime of windows whose incident is resolved, per cause of the incident, per customer of the
environment. One-hop navigation (`ess/28`) admits a filter path and a navigated field in a view
that projects rows, and refuses both in an aggregate view.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/e-aggregate` of the triage of
beyond10x/ess#564; also item 13 of the 2026-11-09 workbench report):

```yaml
  - name: demo.ops.DowntimeByEnvironment
    source: demo.ops.Window
    filter: [incident_id.state == Resolved]
    group_by: [environment]
    fields:
      - {name: environment, type: String}
      - {name: down_s, type: Integer, aggregate: {sum: duration_s}}
```

```text
$ ess specify validate --path e-aggregate
error[ESS-VIEW-009]: `demo.ops.DowntimeByEnvironment` reads through a reference of `demo.ops.Window`, and an aggregate view is not read through one in this cut
help: read the referenced row in a view of its own, or drop the navigation
exit 1
```

## Acceptance

At `ess/37`:

- `e-aggregate` validates (exit 0): an aggregate view's `filter:` reads through a reference of its
  source as a row view's does;
- a group key may be a navigated view field named in `group_by:`,
  `{name: cause_id, type: Optional<demo.ops.CauseId>, field: {related: {via: incident_id, field: cause_id}}}`,
  and a missing far row groups under the absent key, as an `Optional` group key does since
  `ess/15`;
- a measure expression may read a navigated path, `{sum: {expr: "duration_s * incident_id.weight"}}`
  (when `Incident` declares `weight: Integer`), with `skip_absent: true` required as for any value
  that may be absent;
- a measure `where:` reading through a reference stays refused by name (its own row of grid R);
- `ess verify conform synthesize` witnesses each with two referenced rows and a decoy, a far row
  that changes after the near rows are written (the group moves or the figure changes on the next
  read), and an absent reference; `ess verify conform run --target interpreted` passes, and an
  interpreted target reading the value at write time fails;
- below that format the refusal stays, its hint naming the format; a model without it keeps its IR
  bytes;
- generated Rust and Go view queries keep such a view an obligation naming the path; Entity
  Runtime refuses it by name.

## Scope

- cited: crates/specify/ess-domain/src/view/navigation.rs (the aggregate-view refusal, line 281)
- cited: crates/specify/ess-domain/src/view.rs (`validate_grouping`, measure expressions)
- inferred: crates/specify/ess-domain/src/view/measure.rs
- inferred: crates/specify/ess-compiler/src/ir.rs
- inferred: crates/verify/ess-conformance/src/aggregate.rs
- inferred: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- inferred: crates/verify/ess-conformance/src/synthesize/view_reference.rs
- inferred: crates/specify/ess-domain/src/system.rs
- cited: docs/design/generic-language-features.md (§ G2 "Aggregate, paged and `absent:` views refuse navigation by name", revised for aggregate views; § G3 measure input)
- inferred: docs/design/language-grids.md (grid R rows group key, measure input)
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** Figures over rows selected and grouped by a referenced row's current fields, checked
   by the suite. Reproduction above (run.txt § E). Requester's need (the issue body): "per-customer
   figures go through the customer's environments"; the workbench report's item 13 meets the same
   refusal.
2. **Class.** Gap: the first cut refused navigation in aggregate views by name
   (`docs/design/generic-language-features.md:252`).
3. **Already expressible?** Only by copying the referenced value into the source row at write
   time, which the issue rules out ("the incident changes after the window is written"); a
   consumer-side aggregation moves the figure out of conformance (policy 1).
4. **Fit.** Grid R's target opens the related-row column in every view position
   (`docs/design/language-grids.md` Target), and G3 lets a measure input read "the source row and
   its navigated references" (`generic-language-features.md:368`); `ess/36` kept navigation
   out of measures by name as "G2's" (`:416-417`). Spelling: the filter path and navigated field of
   `ess/28`, unchanged; a navigated group key is a view field named in `group_by:`, the rule
   `story:aggregate-group-keys-are-expressions` sets. The view's row count stays the source's, so
   aggregate scoping (`docs/design/aggregate-views.md` scoping) is unchanged. Paged and `absent:`
   views are not opened here; they keep the refusal by name. Targets: interpreter and synthesis;
   Rust and Go obligations, named; Entity Runtime refuses by name.
5. **Second adopter.** Revenue per customer segment through an order's customer; tickets per
   team through the ticket's queue.
6. **Cost.** `ess/37`; no new key; no new diagnostic code.
7. **Alternatives.** (a) Change nothing. (b) A join on arbitrary keys: refused by G2. (c) Chosen:
   the existing navigation shapes admitted in aggregate views, group keys by the group-key rule.

## Decisions

**Accept, redesigned.** The requester's per-customer figure is stated with `ess/28`'s navigation
shapes, admitted in an aggregate view's filter, group keys and measure expressions from G2's next
source format; measure `where:`, paged and `absent:` views keep the refusal by name. The workbench
report's second item, generated Rust queries applying a read through a reference instead of an
obligation, is a target change apart from this issue and is split out of this story. Retitle: "An
aggregate view reads through a reference".

## Source

github:beyond10x/ess#564; `.engineering/repro/workbench-2026-11-09/report.md` item 13

