---
format: aep.planning-md/3
id: story:precedence-swap-of-disjoint-guards-is-equivalent
kind: story
status: active
title: A precedence swap of disjoint input guards is equivalent
summary: Optional inputs and undecidable conjuncts no longer leave a disjoint precedence swap unwitnessed (ess#703)
refs:
- provider: github
  reference: beyond10x/ess#703
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T07:28:22Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T07:28:22Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Problem

`ess verify conform mutate --class precedence-swap` scores a swap of two input-guarded `when:`
branches `unwitnessed` (ESS-MUTATE-004) when no input satisfies both guards but the checker cannot
decide the overlap: an `Optional<Boolean>` or `Optional<enum>` input compared with two different
literals, or two different pinned values beside a conjunct it cannot decide (`amount > 0`,
`starts_with`). No scenario can kill such a swap, so the class never reaches exit 0. A plain
Boolean or enum input pinned to different values is already `equivalent` (`unsatisfiable_guard`,
`crates/verify/ess-conformance/src/mutate.rs:2384`). Reported by a downstream consumer
(beyond10x/ess#703), reproduced on 0.76.0 with a toy specification.

## Fit review

1. Need: a precedence-swap mutant that no input can tell apart is scored `equivalent`, so a
   specification with disjoint input guards can reach exit 0 on the class.
2. Class: defect in the equivalence check. `decide` treats one undecidable conjunct as undecided for
   the whole guard (`mutate.rs:2801`), and an absent `Optional` input compared with a literal counts
   as undecided (`mutate.rs:2898`, `:2901`), although such an input selects neither branch.
3. Not expressible otherwise: the author cannot annotate a mutant.
4. Fit: no authored surface, no format. The mutation report keeps its format; only the verdict of
   such mutants moves from `unwitnessed` to `equivalent`.
5. Second case: any command routing on an optional mode flag.
6. Cost: one check in `mutate.rs`; mutation reports of affected specifications change verdicts.
7. Considered: a new reason code (the downstream ask). Declined: the existing reason
   `unsatisfiable_guard` already means "no input satisfies the overlap", which is what holds here.

## Decisions

Accept, redesigned. The precedence-swap equivalence check proves the overlap
`all: [guard(first), guard(second)]` unsatisfiable when any subset of its decidable conjuncts is
unsatisfiable, and treats an overlap whose value is unknown because an `Optional` input is absent as
not taken (neither branch selects it). The verdict reuses the reason `unsatisfiable_guard` and
names the contradictory conjuncts. No new reason code, no format change. Swaps whose guards can
both hold stay `survived` or `unwitnessed` as today.

## Acceptance

- Optional Boolean and Optional enum swaps are `equivalent` with reason `unsatisfiable_guard`:
  `crates/verify/ess-conformance/tests/precedence_swap_disjoint_guards.rs`,
  `an_optional_input_pinned_to_two_values_is_equivalent`.
- Pinned values beside an undecidable conjunct are `equivalent`: same file,
  `a_contradiction_beside_an_undecidable_conjunct_is_equivalent`.
- Overlapping guards stay scored: same file, `guards_that_can_both_hold_are_not_equivalent`.
- The class reaches exit 0 on the reproduction once every swap is equivalent or killed: same file,
  `the_precedence_swap_class_reaches_exit_zero`.
- Run: `cargo test -p ess-conformance --locked --test precedence_swap_disjoint_guards`; existing
  mutate test targets of the crate that touch `precedence-swap` stay green.
