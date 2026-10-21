---
format: aep.planning-md/3
id: story:navigation-reaches-any-depth-in-view-filters-and-guards
kind: story
status: active
title: Navigation through declared references reaches any depth in view fields, view filters and guards
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#564
relations:
- decomposes: epic:generic-row-queries
- serves: vision:O2
- informed_by: story:row-queries-navigate-a-reference
- depends_on: story:a-set-effect-selection-reads-the-rows-state
scope:
- confidence: inferred
  path: crates/generate/ess-synth/src
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/view.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related.rs
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:07:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:07:59Z", actor: "human:timo", revision: 3}
---
## Outcome

From `format: ess/30`, navigation through declared references reaches any depth in a view filter,
a view field and a `when_related:` guard, as the binding design states
(`docs/design/generic-language-features.md` § G2, lines 236-239: "Depth is any chain of declared
references"; open question 8, lines 673-675: "a path is a finite sequence of reference steps
spelled in the specification … There is no transitive closure and no recursion"; wave 4 item 2,
line 623). The first cut (`ess/28`, lines 241-249) shipped one hop in views and kept the value
source's two-hop chain; "a second hop in a view or a guard … not in this cut and stay refused".
This story ships that second hop and every further one.

The spelling is the one the value source already has, `via: [a, b, …]`
(`{related: {via, field}}` with `via: [a, b]`, design table line 265): a list of reference steps
from the near row. A view filter writes the path `<a>.<b>.<field>`. A missing row at any step
reads as absent, by the first cut's rule (lines 244-247): a navigated view field is `Optional`, a
filter over an absent path shows no row, and a guard's `exists: false` answers a missing row at
any step.

Finding 10 of https://github.com/beyond10x/ess/issues/564 (downstream report, 2026-10-19): a guard
on a port reads the state of the board that owns the port's node. On the integration build
(e44d2b75c, sizing pass 2, 2026-10-21) a second hop is refused ESS-VIEW-003 in a view filter and
ESS-COMMAND-003 in a guard, and `via:` as a list in `when_related:` is a serde error rather than a
named refusal. Below `ess/30` those refusals stay, the guard's list form becoming a named refusal
whose hint names `ess/30`.

## Fit review

1. **Need.** A guard and a view read a row two references away: a port's board state; a window's
   incident's customer.
2. **Class.** Gap, decided by the binding design (open question 8); the one-hop limit is the first
   cut's, not a decision against depth.
3. **Already expressible?** Only by copying the far row's value at write time, which goes stale
   when the far row changes (the reason #564 reopened #447).
4. **Fit.** One navigation rule, the one the value source already applies to two hops; no new
   key, the existing `via:` takes a list in every position.
5. **Second adopter.** An order line showing its order's customer's current tier.
6. **Cost.** A format gate; synthesis arranges a chain with a decoy at each step and changes the
   far row after the near row is written (the `<view>/reference/<field>` scenario of the first
   cut, extended); generated Rust and Go view queries read the chain; Entity Runtime keeps
   `RelatedGuardUnsupported` for guards and refuses a navigated view by name.
7. **Alternatives.** (a) Change nothing. (b) A per-position depth limit: a second rule. (c) Any
   finite chain everywhere navigation is admitted (chosen, as designed).

## Decisions

- Accept, as designed: any finite `via:` chain in view fields, view filters and `when_related:`
  guards from `ess/30`; no transitive closure, no recursion (open question 8).
- Navigation inside a guard *predicate* (`E[input.a].x == E[input.b].x`) is not this story: it is
  outside the cut (lines 247-252) and stays deferred in
  `story:a-guard-compares-fields-of-two-named-rows`.
- Aggregate, paged and `absent:` views keep refusing navigation by name (line 252).

## Acceptance

Each line names the failing test written first, as a case in an existing test binary.

- `crates/specify/ess-domain/tests/row_queries_navigate.rs`:
  `from_ess_30_a_view_filter_navigates_two_hops`,
  `from_ess_30_a_view_field_navigates_two_hops`,
  `from_ess_30_a_guard_via_list_reads_the_far_rows_state` (the finding-10 shape validates), and
  `below_ess_30_a_guard_via_list_is_a_named_refusal_naming_ess_30` (no serde error).
- `crates/verify/ess-conformance/tests/row_queries_navigate_a_reference.rs`:
  `two_hop_view_filter_is_witnessed_with_a_decoy_at_each_step` and
  `two_hop_view_field_reads_the_far_rows_current_value` (the far row changes after the near row
  is written and the view answers the new value).
- `crates/verify/ess-conformance/tests/related_guard_state.rs`:
  `a_two_hop_guard_refuses_on_the_far_rows_state_and_on_a_missing_middle_row`.
- `crates/verify/ess-conformance/tests/view_reference_go.rs`:
  `generated_go_view_reads_a_two_hop_chain`.

## Scope

| unit | files | confidence |
|---|---|---|
| validation | `crates/specify/ess-domain/src/view.rs` (filter and field paths), `crates/specify/ess-domain/src/command/related_guard.rs` (`via:` as a list), `crates/specify/ess-domain/src/command.rs` (value-source chain, `:1162-1186`, `:1861-1863`, design citations) | inferred from design citations |
| interpreter, synthesis | `crates/verify/ess-conformance/src/interpret/`, `src/synthesize/related.rs` | inferred |
| generation | `crates/generate/ess-synth/src/` view queries (Rust, Go) | inferred |
| docs | `website/docs/reference/predicates.md`, `spec-versions.md` (`ess/30` row) | inferred |

Crates: ess-domain, ess-synth, ess-conformance (sizing: M).

## Order

Lane L4b, after `story:a-set-effect-selection-reads-the-rows-state`, before
`story:a-view-source-list-is-refused-by-name`. Serialized with the L4a lane on ess-domain
related-guard code.
