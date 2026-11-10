---
format: aep.planning-md/3
id: story:navigation-reaches-any-depth-in-view-filters-and-guards
kind: story
status: active
title: Navigation reaches any depth in view fields, view filters and view expressions
tags:
- adopter-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
- informed_by: story:row-queries-navigate-a-reference
- depends_on: story:a-set-effect-selection-reads-the-rows-state
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:a-reference-scenario-reads-every-navigated-filter-conjunct
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src/view_query.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/expression.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: cited
  path: crates/specify/ess-domain/src/view/navigation.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/views.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/view_reference.rs
- confidence: cited
  path: docs/design/generic-language-features.md
- confidence: inferred
  path: docs/design/language-grids.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:07:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:07:59Z", actor: "human:timo", revision: 3}
---
## Need

A view reads a row more than one declared reference away, and a computed view field reads a
navigated value: a window counts only while its incident's cause counts (window → incident →
cause), and a window without an end ends with its incident (`coalesce(ended_at,
incident_id.ended_at)`). `ess/28` shipped one hop, in a view field (`field: {related: {via,
field}}`) and a filter path (`<via>.<field>`), and kept a second hop and navigation inside a view
expression refused.

This story is the view half: view fields, view filters and view field expressions. Guards are
`story:navigation-reaches-any-depth-in-guards`; aggregate views are
`story:an-aggregate-view-reads-through-a-reference`.

Minimal reproduction (ess 0.92.0, `format: ess/36`; `repro/b-two-hops`, `repro/b2-two-hop-filter`,
`repro/c-expr` of the triage of beyond10x/ess#564; `Window.incident_id` references `Incident`,
`Incident.cause_id` references `Cause { counts: Boolean }`):

```text
$ ess specify validate --path b2-two-hop-filter        # filter: [incident_id.cause_id.counts == true]
error[ESS-VIEW-003]: `incident_id.cause_id.counts` cannot select `counts` from `demo.ops.CauseId` (resolved prefix type `Uuid`); a view cannot select on something its source never observes
exit 1
$ ess specify validate --path b-two-hops               # field: {related: {via: [incident_id, cause_id], field: counts}}
system.yaml: invalid type: sequence, expected a string
exit 1
$ ess specify validate --path c-expr                   # expr: "coalesce(ended_at, incident_id.ended_at)"
error[ESS-VIEW-001]: `incident_id.ended_at` is no field of `demo.ops.Window`, the row `demo.ops.WindowEnds` computes `ends` from
exit 1
```

## Acceptance

At `ess/37` (`ess/37`, opened by
story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs):

- all three views above validate (`ess specify validate`, exit 0); a path through a reference the
  entity does not declare, or past a field that is no reference, is refused as ESS-VIEW-003 naming
  the step; a view field read through a chain is `Optional` (else `type_mismatch`);
- `ess verify conform synthesize` writes a `<view>/reference/<field>` scenario per chain that
  arranges a decoy at each step, a missing middle row, and a second read after the far row
  changes; `ess verify conform run --target interpreted` passes it, and an interpreted target
  reading the near row's field instead fails it;
- for `c-expr`, the scenario reads `ends` as the window's own end where it has one and as the
  incident's `ended_at` after the incident is resolved where it has none;
- below that format each keeps today's refusal, the serde error for `via: [..]` in a view field
  becoming a named `unsupported_format_version` naming the format; a model without a chain keeps
  its IR bytes;
- generated Rust and Go view queries keep a chained or expression-navigated view an obligation
  naming the path; Entity Runtime refuses it by name; `ess verify diff` reports a changed chain as
  the existing view field change.

## Scope

- cited: crates/specify/ess-domain/src/view/navigation.rs (path resolution, the one-hop limit)
- cited: crates/specify/ess-domain/src/view.rs (view field `field: {related}` and `expr:` reads)
- inferred: crates/specify/ess-domain/src/expression.rs (a navigated path as an operand of a view expression)
- cited: crates/specify/ess-compiler/src/ir.rs (`ResolvedViewReference { via: String }`, line 2431)
- inferred: crates/specify/ess-compiler/src/resolve.rs
- inferred: crates/verify/ess-conformance/src/interpret/views.rs
- cited: crates/verify/ess-conformance/src/synthesize/view_reference.rs
- inferred: crates/generate/ess-synth/src/view_query.rs (obligation naming the path)
- cited: docs/design/generic-language-features.md (§ G2, a decided subsection for depth and view expressions, with the IR shape, before code)
- inferred: docs/design/language-grids.md (grid R rows view field and view filter)
- inferred: website/docs/reference/predicates.md

confidence: medium

Moves shared surfaces: proposes its sentence for the `ess/37` `FORMAT_HISTORY` row and for
`CHANGELOG.md`, and changes the regenerated `ess.schema.json` and spec-versions page.

## Fit review

1. **Need.** A view that reads a value two references away, and a view value computed from the
   row and a navigated value. Reproduction above (run.txt § B, § C). Requester's shapes (theirs,
   the issue body): "its cause counts", "a window without an end ends with its incident", with a
   one-hop `field: {related: {via, field}}` example.
2. **Class.** Gap: the design decides depth as "any chain of declared references"
   (`docs/design/generic-language-features.md:238-240`, answer 8 at `:725`) and the first cut
   kept the second hop refused (`:248-252`).
3. **Already expressible?** Only by copying the far value into the near row at write time, which
   goes stale when the far row changes: the reason the issue reopened beyond10x/ess#447
   (copy-at-write "does not serve: the incident changes after the window is written"). No idiom
   keeps the figure checked.
4. **Fit.** No new key: `via:` takes a list as the value source already does (`via: [a, b]`,
   `generic-language-features.md:265`) and a filter path gains steps. Grid R: the view field and
   view filter rows close whole for the related-row column (`docs/design/language-grids.md` Target,
   "related row and row set: every … view position"); the view field row "takes any expression
   over its row (G3)" (same page), so a navigated path inside `expr:` is the same decision.
   IR: per § "Migration and format consequences" (`generic-language-features.md:617-620`) a
   chain is an optional list beside `ResolvedViewReference.via`, omitted for one hop, so one-hop
   models keep their bytes and no `ess-ir` version moves (AGENTS.md "Boundaries": no `ess-ir/2`).
   This answers the coordinator's 2026-10-22 hold ("a design page settles the IR shape"): the G2
   subsection records it. Targets: interpreter and synthesis read the chain; Rust and Go keep an
   obligation, named; Entity Runtime refuses by name; diff.
5. **Second adopter.** An order line showing its order's customer's current tier; a ticket view
   showing its desk's site's time zone.
6. **Cost.** One source format (G2's next), no new key, no new diagnostic code; the `<view>/
   reference/` scenario family extends to chains, no suite format unless the story finds a new
   expectation shape, which it then names.
7. **Alternatives.** (a) Change nothing. (b) A per-position depth limit: a second rule. (c) A
   join on arbitrary keys: refused by G2 ("a view reads other entities only by navigation from its
   source row", `generic-language-features.md:287-289`). (d) Chosen: any finite chain in every
   view read position, and inside view expressions, as designed.

## Decisions

**Accept, redesigned** (the requester's one-hop shape shipped in `ess/28`; depth and expressions
follow the binding design). Any finite `via:` chain in view fields and filters, and navigated
paths inside a view field's `expr:`, from `ess/37`; no transitive closure, no
recursion (answer 8). Guards move to `story:navigation-reaches-any-depth-in-guards`; aggregate
views to `story:an-aggregate-view-reads-through-a-reference`. Retitle: "Navigation reaches any
depth in view fields, view filters and view expressions".

## Source

github:beyond10x/ess#564

