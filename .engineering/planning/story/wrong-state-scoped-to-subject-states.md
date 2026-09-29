---
format: aep.planning-md/3
id: story:wrong-state-scoped-to-subject-states
kind: story
status: implemented
title: A command cannot answer differently in different states its moves do not start from (wrong_state is one flag over all of them)
refs:
- provider: github
  reference: beyond10x/ess#201
relations:
- serves: vision:O2
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-28T17:59:48Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T01:53:24Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
# Story: A command cannot answer differently in different states its moves do not start from (wrong_state is one flag over all of them)

## Why

beyond10x/ess#201. Coordinator decision: `.engineering/waves/ess-0.41-decisions.md` row #201 (summarised in the wave page).

## Scope (story-scoper, 2026-09-28, on e9327819658583ae45953acafa3e41d3bbd5b7f4)

The defect reproduces on this tree (0.40.0 candidate, `e932781965`) and is not fixed. Every refusal in the issue comes from code that is still there. I read the code only; nothing was built or run.

**(1) Does it reproduce**
- cited: `crates/specify/ess-domain/src/command.rs:4644-4657` (`subject_authority`) refuses a `when_subject_state:` branch whose own `subject` is not a command-input subject. The issue's third attempt hits this: `error: Gone` branches carry no subject. The message is exactly "a subject-state guard requires an existing moves or updates subject and input identity".
- cited: `crates/specify/ess-domain/src/command.rs:4567-4572` (`outcome_condition`) refuses `when_subject_state` together with `wrong_state` on one branch.
- cited: `crates/specify/ess-domain/src/command/subject_state.rs:109-116` (`validate_shape`) refuses any `wrong_state` branch in a command that uses held-state guards. This is the first attempt's ESS-COMMAND-004.
- cited: `subject_state.rs:119-131` also refuses an input-selected branch whose `selection_subject` is not an input-identity subject. It is a second barrier behind the first one.
- cited: `wrong_state` is a plain bool that becomes `OutcomeCondition::WrongState` (`command.rs:4608`). No field scopes it to a subset of states.
- cited: `crates/specify/ess-compiler/src/ir.rs:1253-1265` (`ResolvedCommand::selection_subject`) only borrows a sibling's subject for an `Otherwise` branch with an `error:`. A `SubjectState` refusal gets none.
- cited: synthesis requires the branch's own subject in `crates/verify/ess-conformance/src/synthesize.rs:3484-3489` (`outcome.subject.ok_or(StrategyWithoutGuard)`) and `:3533-3545` (`observe_subject_state`).
- cited: precedent. The predicate form already lets a subject-less refusal read "the subject its siblings name", via `subject_fact::common_subject` / `reading_subject` (`crates/specify/ess-domain/src/command/subject_fact.rs:40-56, 86-110`).

**(2) Where the fix lands (option B: admit `when_subject_state:` on a refusal)**
- cited: `command.rs` `subject_authority`: let a held-state refusal with no subject through, deferring to a command-level check, as `SubjectPredicate` does.
- cited: `subject_state.rs` `validate_shape`: resolve the subject through a common-subject helper instead of `selection_subject` only, and refuse by name when no sibling names one.
- cited: `ir.rs` `selection_subject`: extend the sibling fallback to `SubjectState` (and possibly `StateChange`) refusals.
- cited: `synthesize.rs` around `:3484`, `:3533` and `:3656-3663` (`reach` → `reach_in_state`): use the selection subject, not `outcome.subject`, to arrange and observe the held state.
- inferred: interpreters that select branches by held state need the same subject resolution: `verify/ess-conformance/src/{interpret.rs,interpret/execute.rs,reference.rs,faulty.rs}` and `generate/ess-entity-runtime/src/lib.rs`.
- inferred: `ess-compiler/src/resolve.rs:4334` passes the condition through unchanged; it probably needs no change.
- inferred: schema and doc text for `when_subject_state`, plus a CHANGELOG / `changes/` entry.

**(3) Collisions**
- inferred, high: #209. It also concerns arranging an existing subject for refusal scenarios, in `synthesize.rs` `prepare` / `prepare_subject` / `reach`.
- inferred, medium: #198 and #199. Both concern arranging state through the creation search and the `admits_held_state` / partition logic in `synthesize.rs`. #199 may also touch `subject_state.rs` / `finite.rs`.
- inferred, low: #196. Same file (`synthesize.rs`) but a different region: `Map` witness candidates.
- inferred, none: #210 (`mutate.rs` scoring) and #195 (binding accessors, `ESS-BINDING-003`).

**(4) Design decisions for the implementor**
- inferred: option A (scoped `wrong_state: [states]`) or option B (`when_subject_state` on a refusal). B reuses the existing partition prover and the sibling-subject precedent, and its surface is smaller. A changes the type of `wrong_state` and the `refuses:` semantics.
- inferred: whether option B needs a new source-format version gate. `uses_subject_state` exists for format gating (`subject_state.rs:20-29`).
- inferred: whether the relaxation also covers `when_state_changes:` refusals.
- inferred: whether one branch may name several states (`when_subject_state: [Delivered, Cancelled]`). The issue's two-branch spelling works without that.
- inferred: how the unaddressed states (`Placed` here) are proved covered. The default branch `shipped` has to satisfy `validate_move` for every state the partition leaves to it.

Confidence: high that it reproduces (the literal message matches, cited). Medium on the full list of files to change (the interpreter and runtime sites are inferred).

Paths:
- crates/specify/ess-domain/src/command.rs
- crates/specify/ess-domain/src/command/subject_state.rs
- crates/specify/ess-domain/src/command/subject_fact.rs (precedent)
- crates/specify/ess-compiler/src/ir.rs
- crates/verify/ess-conformance/src/synthesize.rs
- crates/verify/ess-conformance/src/interpret.rs (inferred)
- crates/verify/ess-conformance/src/reference.rs (inferred)
- crates/generate/ess-entity-runtime/src/lib.rs (inferred)

Verdict: needs-design (small: pick option B plus the format-gate answer, then it is open).
