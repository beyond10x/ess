---
format: aep.planning-md/3
id: story:precedence-swap-of-disjoint-guards-is-equivalent
kind: story
status: implemented
title: A precedence swap of disjoint input guards is equivalent
summary: Optional inputs and undecidable conjuncts no longer leave a disjoint precedence swap unwitnessed (ess#703)
refs:
- provider: github
  reference: beyond10x/ess#703
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T07:28:22Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T07:28:22Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "active", to: "implemented", at: "2026-10-29T10:39:50Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"review_outcome":1}}, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
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

## Downstream shapes

The downstream report's inputs are all plain Boolean or plain enum (orchestrator, 2026-10-29). The
unit also covers the report's shapes, with neutral names, each a test in
`crates/verify/ess-conformance/tests/precedence_swap_disjoint_guards.rs`:

- `a_pair_disjoint_on_one_boolean_beside_other_pins_is_equivalent`: `enumE == A, enumC == H,
  boolP == false, boolQ == true` against `enumC == H, boolP == true, boolV == false`.
- `a_pair_disjoint_on_an_enum_beside_variant_attributes_is_equivalent`: `enumF == X` against
  `enumF == Y`, both with `opEnum.attr1 == true, opEnum.attr2 == false`.
- `a_pair_disjoint_through_variant_attributes_is_equivalent`: `opEnum.attrA == true, boolI ==
  true, boolH == false` against `opEnum.attrB == true, boolI == true, boolK == false`, where no
  variant has both attributes true; the check reads the enum's variant table.
- `a_pair_sharing_a_variant_with_both_attributes_is_not_equivalent`: the same pair where one
  variant has both attributes true.
- `a_pair_disjoint_through_a_disjunction_is_equivalent`: `opEnum.attr2 == true, boolS == true,
  boolK == false` against `any: [opEnum.attr2 == false, boolS == false], boolI == true,
  boolB == false`; every disjunct contradicts the other guard.

The decision extends accordingly: the overlap is unsatisfiable when its decidable part is, where
the decidable part includes enum-variant attributes (resolved over the variant table) and `any:`
disjunctions (unsatisfiable when every disjunct is). The reason stays `unsatisfiable_guard`.

## Downstream run on 0.76.0

On 0.76.0 the downstream specification still has 12 precedence-swap survivors, none with a
scenario that kills it. With neutral names, the shapes this issue covers are:

- S1: `k == X and c == true and f == false and ...` against `k == X and f == true and g == false`,
  disjoint on `f`, where `c` and `f` are read from the entity's stored state rather than from the
  command's input.
- S2: `op in [L] and op in [L] and e == Unavailable` against `op in [L] and op in [L] and
  e == Incomplete`, disjoint on the enum `e`, with a duplicated conjunct.
- S3: `p == R and q == S and s == true and ...` against `(p == C or q == M or s == false) and ...`,
  where each alternative of the disjunction contradicts one conjunct of the other guard.
- S4: `op in [about thirty values] and ...` against `op in [one value] and ...`, disjoint when the
  single value is outside the list.

The fix decides each of these as `equivalent` (`unsatisfiable_guard`) where the guards are provably
disjoint.

Out of scope here, a possible follow-up: swaps whose guards can both hold unless the model's
declared invariants exclude it (for example a field whose value depends on another). Deciding
equivalence under declared invariants is a separate change.

## Results per shape

Each downstream shape rebuilt as a neutral model, scored by `ess` 0.76.0 and by the batch 2 head.

| shape | 0.76.0 | cause on 0.76.0 | 0.77.0 |
|---|---|---|---|
| S1, pins contradicting beside trailing Boolean conjuncts | survived | 4 × 2⁵ = 128 combinations, over 64 | equivalent |
| S1, trailing `amount > 0` | survived | a conjunct the check could not decide | equivalent, naming the contradicting pins |
| S1, fields inside an input struct | survived | over 64 combinations | equivalent |
| S1, over stored fields | no precedence-swap site | a `when:` cannot read a stored field (ESS-COMMAND-003); `when_subject:` branches are not swap sites | unchanged |
| S2, duplicate `in` conjunct | unwitnessed | 35 × 3 = 105 combinations; the duplicate is not the cause | equivalent |
| S3, conjunction against a disjunction | survived | 6 × 6 × 2³ = 288 combinations | equivalent |
| S4, 30-value list against a value outside it | survived | 35 × 4 = 140 combinations | equivalent |
| S4 control, value inside the list | killed | — | killed |

Tests: `ess-conformance/tests/precedence_swap_disjoint_guards.rs`, the `s1_*` to `s4_*` cases.
Equivalence under declared invariants (S5–S7) stays out of scope; #703 names it as a follow-up.
