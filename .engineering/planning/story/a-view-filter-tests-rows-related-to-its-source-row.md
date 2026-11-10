---
format: aep.planning-md/3
id: story:a-view-filter-tests-rows-related-to-its-source-row
kind: story
status: active
title: A view filter tests rows related to its source row
tags:
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:navigation-reaches-any-depth-in-view-filters-and-guards
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: cited
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/system.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/row_set.rs
- confidence: inferred
  path: docs/design/language-grids.md
- confidence: cited
  path: docs/design/related-rows-quantifier.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:42:37Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:42:38Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A view shows a row only where no row of another entity is tied to it in a stated way: a window
counts only when it is not inside a maintenance window of its environment. The tie is not a
declared reference (a maintenance window names an environment and a period, not the outage), so
navigation cannot reach it. The related-rows quantifier `exists: {in: {entity: E}, as: b, that: P}`
(beyond10x/ess#551, `docs/design/related-rows-quantifier.md`) states exactly this test, but is
admitted only in a row selector's `where:`, not in a view `filter:`.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/d-row-set` of the triage of
beyond10x/ess#564):

```yaml
  - name: demo.ops.OutsideMaintenance
    source: demo.ops.Window
    filter:
      - not:
          exists:
            in: {entity: demo.ops.Maintenance}
            as: m
            that: {all: [m.environment == environment, m.starts_at <= started_at, m.ends_at > started_at]}
    fields:
      - {name: window_id, type: demo.ops.WindowId}
```

```text
$ ess specify validate --path d-row-set
error[ESS-VIEW-009]: `exists m in {entity: demo.ops.Maintenance}: (...)` walks the rows of `demo.ops.Maintenance`, which a quantifier does only in the `where:` of a `{related: …}` read and of a row-set `when_related:`; ...
exit 1
```

## Acceptance

At `ess/37`:

- `ess specify validate --path d-row-set` exits 0; the quantifier is admitted in a view `filter:`
  under the rules it has in a selector (tied to the source row by a top-level
  `<b>.<field> == <source field>`, `exists` only, one level; `forall` refused with its
  `not exists` rewrite), and refused by name in a measure `where:`;
- `ess verify conform synthesize` writes a scenario that arranges a window inside a maintenance
  window of its environment, one inside a maintenance window of another environment, and one
  outside every maintenance window, and expects only the last two; `ess verify conform run
  --target interpreted` passes, and a target that ignores the environment tie fails;
- the view's rows change when a maintenance window is planned after the window is written (the
  filter reads the related rows when the view is read), and the scenario reads the view again
  after that write;
- below that format the refusal stays, its hint naming the format; a model without it keeps its IR
  bytes;
- generated Rust and Go view queries keep such a view an obligation naming the quantifier; Entity
  Runtime refuses it by name.

## Scope

- cited: crates/specify/ess-domain/src/expression.rs (the `rows` environment check and refusal, around line 2975)
- inferred: crates/specify/ess-domain/src/view.rs (the filter's environment admitting related rows)
- inferred: crates/verify/ess-conformance/src/interpret/views.rs
- inferred: crates/verify/ess-conformance/src/synthesize/row_set.rs
- inferred: crates/verify/ess-conformance/src/synthesize.rs (view filter witnesses)
- inferred: crates/generate/ess-synth/src/view_query.rs (obligation naming the quantifier)
- inferred: crates/specify/ess-domain/src/system.rs
- cited: docs/design/related-rows-quantifier.md (§ "Where it stands", the view `filter:` row)
- inferred: docs/design/language-grids.md (grid R view filter × row set)
- inferred: website/docs/reference/predicates.md
- inferred: CHANGELOG.md
confidence: medium

## Fit review

1. **Need.** A view row shown only where no row of another entity, tied to it by a stated
   condition, exists when the view is read. Reproduction above (run.txt § D). Requester's need
   (the issue body): "it is not inside a maintenance window"; their proposed shape was a
   `source:` list, refused by name since `story:a-view-source-list-is-refused-by-name`.
2. **Class.** Gap: no view position reads a row set (`docs/design/language-grids.md` grid R, view
   `filter:` × row set "no").
3. **Already expressible?** Not in the view. Copying "inside maintenance" into the window at write
   time goes stale when maintenance is planned later; a consumer-side filter moves the figure out
   of conformance (policy 1).
4. **Fit.** Grid R's target opens the row-set column in "every view position (G2)"
   (`docs/design/language-grids.md` Target); the closing rule opens the view-filter row whole.
   The spelling reuses the related-rows quantifier unchanged, with its rules
   (`related-rows-quantifier.md` § Rules). That page refused the view filter because "generated
   targets … enumerate no rows by a selector" (`related-rows-quantifier.md:182`); since `ess/28`
   and `ess/36` the same targets keep a navigated view and an expression measure an obligation
   named, which meets "every target handles it or refuses it by name", so the reason no longer
   refuses the cell and the page's row is amended. The view's row count stays the source's
   (`generic-language-features.md:287-289`): a filter only removes rows, so it is no join.
   Measure `where:` stays refused: it is a different row of grid R, opened by its own decision.
5. **Second adopter.** Orders shown only where no open dispute names the same customer; shifts
   shown only where no leave request of the same agent covers the shift's start.
6. **Cost.** `ess/37`; no new key; no new diagnostic code; synthesis gains view
   witnesses with arranged related rows (the selector's arrangement, reused).
7. **Alternatives.** (a) Change nothing. (b) A `source:` list or join: refused by G2 (one source,
   navigation only). (c) A dedicated `excludes_related:` view key: a second spelling of the
   quantifier. (d) Chosen: the existing quantifier admitted in the view filter.

## Decisions

**Accept, redesigned.** The requester's `source:` list stays refused; the need is met by admitting
the related-rows quantifier in a view `filter:`, from `ess/37`, with
`docs/design/related-rows-quantifier.md` § "Where it stands" amended and grid R's view-filter row
closed.

## Source

github:beyond10x/ess#564

