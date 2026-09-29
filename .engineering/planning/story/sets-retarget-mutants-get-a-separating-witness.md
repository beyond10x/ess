---
format: aep.planning-md/3
id: story:sets-retarget-mutants-get-a-separating-witness
kind: story
status: implemented
title: 'sets-retarget: with three boolean inputs, the mutant''s suite still witnesses the swapped pair with one value (follow-up to #161)'
refs:
- provider: github
  reference: beyond10x/ess#202
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T01:53:24Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
# Story: sets-retarget: with three boolean inputs, the mutant's suite still witnesses the swapped pair with one value (follow-up to #161)

## Why

beyond10x/ess#202. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #202 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The defect is still there on this tree (`e932781965`, release 0.40.0). The fix is small: one function in synthesis, plus one design choice about which pairs to separate.

| # | Finding | Basis |
|---|---|---|
| 1 | The mutant generator pairs each `sets:` input with the **first** input of the same type (`.find(...)`). That gives `paid→gift`, `gift→paid` and `rush→paid`, which matches the issue. | cited: `crates/verify/ess-conformance/src/mutate.rs:636-653` |
| 2 | The #161 fix is `distinguished()`. It walks `outcome.sets` in order and, when a source and a sibling of the same type hold one value, moves the **sibling first**. | cited: `synthesize.rs:5363-5421` (loop `'moved: for moving in [sibling, field]` at :5395) |
| 3 | Root cause: `distinguished()` never checks whether a move undoes a pair it already separated. In the `gift` mutant the model's sets are `paid: input.paid, gift: input.paid, rush: input.rush`. So `input.gift` is only ever a sibling, and a later `rush` step that flips `paid` can put `paid == gift` back. With 3 inputs of a 2-valued type, no single witness separates every pair anyway. | cited: code path; inferred: the exact order of moves (I did not run it) |
| 4 | Only one witness is made per call. `freshened()` calls `distinguished()` at :5306, and `freshened()` has 4 callers: :2074, :2109, :4745, :7147. Nothing adds a second witness when separation fails. | cited |
| 5 | Nothing on the branch fixes this. The last related commit is `dda521887a` (#161). | cited: `git log` |
| 6 | Fix location: `distinguished()`, and `equals_a_sibling()` (:5347) if it has to respect pinned pairs. Possibly also the witness loop in the `freshened` callers, for a second scenario. | inferred |
| 7 | The adopter case with an **event** writing 3 booleans probably goes through the same `InputField` path, as an event-triggered command input. | inferred, not traced |

Design decision the implementor must make (I recommend option a):
- **a) Target-name priority.** In a mutated model, a `sets` entry whose target `T` names an input of the same type that is not its source (`gift: input.paid` while an input `gift` exists) marks the pair (source, `input.T`). Separate that pair first and pin both fields so later moves cannot collapse them. This needs no mutant provenance in synthesis. It relies on target and input sharing a name, which is true of every retarget the generator emits only when names line up.
- **b) Pass the mutated pair explicitly** from the `mutate.rs` mutant into synthesis, and separate exactly that pair. This is exact, but it adds an API seam through `--emit`.
- **c) Issue's second option.** Add a second witness per outcome whenever the number of same-typed `sets` sources exceeds the type's cardinality. It is general, costs more scenarios in every suite, and changes baseline counts (the `mutation_audit.rs` expectations).
- Whichever is chosen: pinned separations must never be undone. That is a guard inside the `'moved` loop.

Collisions with the other issues:
- **#196** (a Map input is always witnessed as `{}`, so the `sets:` assertion passes): same neighbourhood, the `sets` witness and `candidates`/`freshened`. Likely conflict. inferred
- **#203** (mutate: a suite that gains synthesis refusals is scored survived) and **#210** (`--collect` refuses to score when the baseline has skipped scenarios): mutate scoring and `mutation_audit.rs` expectations. Low to medium. inferred
- **#198, #199, #204, #209, #193**: same 9096-line `synthesize.rs` but other functions (when_subject, arrangement, view filters). Textual merge risk only. inferred
- **#195, #201, #205, #211**: likely other crates or the model layer. No overlap expected. inferred

Confidence: high on the code path and on it not being fixed (read, not executed); medium on the move order that produces the reported witness.

Paths:
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/mutate.rs (only if option b)
- crates/verify/ess-conformance/tests/connective_and_source_mutants.rs (new case, inferred)
- crates/verify/ess-conformance/tests/mutation_audit.rs (counts, inferred)

Verdict: open
