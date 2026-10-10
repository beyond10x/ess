# One derived precedence plan per command

Status: type and classification only (`story:selection-plan-design-and-type`, epic
`epic:one-selection-plan`, review finding 2 of beyond10x/ess#470). No consumer reads the plan yet;
the stories that follow move each one onto it. No source, IR or suite format changes.

## Why

The order in which a command's branches answer is stated once, in prose
([The precedence order](cross-record-and-stored-field-guards.md#the-precedence-order)), and
re-derived as control flow by validation, the model interpreter, synthesis, the Entity Runtime
lowering, the Rust and Go emitters and mutation. beyond10x/ess#486 was two of those derivations
disagreeing. This page gives the order one executable definition: a **precedence plan**, the
command's branches grouped into phases in the precedence order, each phase ordered.

## The phases

Eight phases. Six are the six steps of the precedence order; two sit outside them.

| phase | step | the branches it holds | order within the phase |
|---|---|---|---|
| `InputAbsent` | before 1 | `input_absent:` | — (at most one) |
| `RelatedRow` | 1 | `existing_instance:` on a command reading a related row or a row set; `exists: false` over a row the input names | `existing_instance:` first, then `exists: false` in declaration order |
| `InputRefusal` | 2 | `when:` + `error:` naming no subject and no `replays:`, its guard not trivially true; every other `when:` + `error:` is an **unguarded refusal** | declaration order |
| `Existence` | 3 | `existing_instance:` on any other command; `unknown_instance:`, except on a row-set upsert | `existing_instance:`, then `unknown_instance:` |
| `HeldState` | 4 | `when_subject_state:`, `when_state_changes:`, `when_subject:` (both shapes); `wrong_state:`; an unguarded refusal, except on a command reading a stored reference or a row set | the unguarded refusal first, then the held-state branches in declaration order, then `wrong_state:` |
| `PresentRelated` | 5 | where the composition below orders them: `when_related:` predicate refusals and row-set refusals; `exists: false` over a stored reference; an unguarded refusal on a command reading a stored reference or a row set; `unknown_instance:` on a row-set upsert | stored `exists: false` first, then the unguarded refusal, then the refusals in declaration order, then `unknown_instance:` |
| `Accepting` | 6 | `when:` without `error:`, `external:` (with or without `when:`), accepting `when_related:` and row-set branches, and every `when_related:` predicate refusal step 5 does not hold | declaration order |
| `Default` | after 6 | `otherwise:` | — (at most one) |

Each phase is the interpreter's (`crates/verify/ess-conformance/src/interpret/execute.rs`) reading
of its step:

| phase | where the interpreter reads it |
|---|---|
| `InputAbsent` | `without_input`, before any input field |
| `RelatedRow` | `existence::existing` for the phase's `existing_instance:`, then `related_absent` in `related::reads_in_order`, both from `responding_core`'s walk of the plan |
| `InputRefusal` | `refused_by_input`, over the phase's branches, at the phase's position in `responding_core`'s walk; `select` (the look-aheads) reads the ones it leaves |
| `Existence` | `existence::existing` for the phase's `existing_instance:`; `unknown_instance` from `held_rows`, gathered strictly at this phase after flattening the input; also from `addressed_row`, `selected_subject_refusal`, `stored_reference` and `take` |
| `HeldState` | `select`'s `HeldState` phase, the unguarded refusal first; `wrong_state` from `addressed_row`, `selected_subject_refusal` and `take` |
| `PresentRelated` | `stored_reference` for a stored `exists: false`; `select`'s `PresentRelated` phase for the unguarded refusal and the refusals; a look-ahead (`subject_refusals_before_present_related`, `stored_reference`) leaves the refusals out |
| `Accepting` | `select`'s `Accepting` phase |
| `Default` | `select`'s `Default` phase |

`responding_core` builds the plan once per decision and `select` reads its phases in order
(`story:interpreter-reads-selection-plan`). Until then the interpreter read `HeldState` and
`Accepting` in one declaration-order pass. Since beyond10x/ess#486, validation refuses a held-state
branch declared after an accepting or external branch whose input guard may hold with its own, so
on every command that validates the two phases read in turn give that pass's answer; the one
repository command declaring such a branch after a disjoint accepting one,
`ess-entity-runtime/tests/fixtures/held-state-after-disjoint-accepting.yaml` `demo.ticket.Close`,
answers alike either way.

`responding_core` walks every phase in the plan's order and answers each at its position: the
existence answers and `refused_by_input` where the plan places them, and each phase `select` reads
in place (`select_phases`), with the rows gathered so far; the first branch that answers ends the
walk. A phase order the `with_phase_order` seam exchanges moves each answer with its phase. The
fixed points:

- Input-named related rows are read only once `RelatedRow` has answered a missing one, and a
  stored reference's row only at `Existence`.
- `Existence` gathers the held rows strictly (`held_rows`): a row that cannot be read is the
  error it always was, and a held-state subject no row holds is answered `unknown_instance:`. A
  phase `select` reads before `Existence` sees only the rows that can be read, and a missing
  subject is answered at `Existence`.
- `addressed_row` on a row-set command answers at the end of `Existence`. The row-set tests and
  the step-5 look-ahead (`subject_refusals_before_present_related`) run once, at the first phase
  `select` reads.

### The markers answer; they are not read in order

`existing_instance:`, `unknown_instance:` and `wrong_state:` carry no guard. The plan places each in
the phase whose question it answers, and a consumer answers that question as the interpreter does.
"Looking ahead" below is the branch `select` would take from the phases after the marker's, with
the branches named left out:

- `existing_instance:` answers where the identity a creation would take is held; where the
  command's creations take different identities, the creation looking ahead selects
  (`existence::existing`).
- `unknown_instance:` answers where the addressed row is not held: at once on a command with a
  held-state branch (the held-subject loop), on a row-set command (`addressed_row`) and on a
  stored-reference command (`stored_reference`); on any other command only where the branch
  looking ahead selects addresses it (`selected_subject_refusal`, `take`), so a branch naming no
  row answers for a missing one. A row-set **upsert** — a row-set command with a creation taking,
  from the input, the identity its acting branches address (beyond10x/ess#462) — is the exception:
  `addressed_row` and `selected_subject_refusal` leave an absent row to the row sets
  (`creation_takes_absent`), and `unknown_instance:` answers only after them, where the branch
  then selected addresses the row (`take`). The plan closes `PresentRelated` with it there. With no
  `unknown_instance:` the same question is answered by the one `external:` refusal synthesis reads
  as not-found (`synthesize::declared_not_found`), else by `wrong_state:`.
- `wrong_state:` answers where the branch looking ahead selects moves from a state the row does not
  hold — with the present-related refusals left out where step 5 orders them
  (`subject_refusals_before_present_related`), and every `when_related:` branch left out on a
  stored-reference command, which looks ahead before reading the stored row (`stored_reference`).
  On a row-set command it also answers at once where every accepting branch acting on the row
  moves, and none from the state it holds (`addressed_row`): a branch that updates, preserves or
  deletes the row may act in every state, and then only looking ahead answers. An unguarded
  refusal is a `when:` branch, so looking ahead takes it wherever it holds, and then
  `wrong_state:` does not answer. Looking ahead comes before step 5, so `wrong_state:` stays in
  `HeldState` beside an accepting branch that updates: on
  `ess-compiler/tests/fixtures/precedence-row-set-updating-branch.yaml` it answers a closed row
  before the row-set refusal where the moving default is the branch selected, and not at all
  where the updating branch is (`ess-conformance/tests/precedence_plan_answers.rs`).

## The compositions that move a branch

A branch's phase is decided by its condition, its `error:`, `subject` and `replays:`, and by what
the rest of the command declares. These compositions move a branch between phases:

| branch | moves to | where the command | otherwise | interpreter |
|---|---|---|---|---|
| `existing_instance:` | `RelatedRow` | declares any `when_related:`, through the input or a stored field, or any row set | `Existence` | `responding_core`, the first `existence::existing` |
| `exists: false` | `PresentRelated`, first | reads the row through a stored field of the subject (`ess/22`, #304) | `RelatedRow` | `stored_reference` |
| `when_related:` predicate refusal | `PresentRelated` | is `ess/22` and declares `wrong_state:` (#282); is `ess/22` and reads several rows through its input (#283); reads its row through a stored field (#304) | `Accepting`, in declaration order | `select` reads the phase the plan places it in |
| row-set refusal | `PresentRelated` | always: a row-set refusal is the composition | — | `select`'s `PresentRelated` phase |
| `when:` + `error:` | the unguarded refusal: `HeldState`, first | gives the branch a trivially true guard, a subject or `replays:` (validation refuses the last two beside an `error:`) | `InputRefusal` | `refused_by_input` declines it; the head of `select` takes it |
| unguarded refusal | `PresentRelated`, after a stored `exists: false` and before the refusals | reads a stored reference or a row set | `HeldState`, first | `stored_reference` and `addressed_row` run before `select` |
| `unknown_instance:` | `PresentRelated`, last | guards on a row set and declares a creation that takes, from the input, the identity its acting branches address (an upsert, #462) | `Existence`, last | `addressed_row` and `selected_subject_refusal` leave the absent row to the row sets (`creation_takes_absent`); `take` answers it |

An unguarded refusal is read where `select` begins: after every answer the interpreter gives before
`select`, and before every branch `select` reads. On most commands that is the head of `HeldState`.
On a stored-reference command `stored_reference` answers the addressed row's existence and the
stored row's `exists: false` before `select`, and on a row-set command `addressed_row` answers the
addressed row's existence and held state; there the refusal answers after those, in
`PresentRelated` and before its refusals, which `select` reads after its head (beyond10x/ess#470
adversary pass 1, cases C1 and C2, `ess-conformance/tests/adversary_selection_plan_u1_pass1.rs`).
Validation admits no held-state branch on either command, so the refusal still answers before
every branch `select` reads.

An input refusal guarded by the held state as well (`when_subject:` beside `when:`) is a held-state
branch by its condition and sits in `HeldState`; no composition moves it.

### Where the plan's reading and the written ones part

The plan reproduces the interpreter. One place where the precedence page's wording does not decide
the case, for the page's owner to settle; no repository command exercises it. Step 1 names
`existing_instance` "on a command with a `when_related:` branch reading an input", and step 3 "on
commands without `when_related`". A command reading its related row through a stored field, or
guarded by a row set, is neither. The interpreter, and so the plan, answers it in step 1. It answers
differently from step 3 only on such a command that also declares an input refusal;
`filtered-related-reads.md` ("Subject borrowing and precedence") orders input refusals first for a
row set.

Validation reads one composition apart from the interpreter, for the story that moves validation
onto the plan: `related_guard::orders_present_refusals` does not order a stored reference's
refusals first where an accepting related branch moves the subject and no `wrong_state:` is
declared; the interpreter, and the plan, order them first on every stored reference.

## Decisions

### The classification lives in `ess-domain`

`ess_domain::command::precedence`: a data-free `ConditionShape` (one per `OutcomeCondition`
variant, with the facts a phase depends on and nothing else), a `BranchShape` adding `error`,
`subject` and `replays`, a `Composition` of the command-level facts above, and `place`, which
answers a branch's phase and its rank inside it.

Validation runs in `ess-domain`, below `ess-compiler`, and reads `OutcomeCondition`; every other
consumer reads `ResolvedCondition`. A plan built only from `ResolvedCommand` is out of validation's
reach, and a second classification in the compiler would be the re-derivation this work removes.
So each crate maps its own condition into the one shape — `ConditionShape::from(&OutcomeCondition)`
in `ess-domain`, `precedence::shape(&ResolvedCondition)` in `ess-compiler`, each a match with no
wildcard arm — and the rule exists once. This is the shape `TestStrategy` already has: decided in
the domain, carried by the IR.

### The type is `ess_compiler::ir::PrecedencePlan`

`ess_domain::selection::SelectionPlan` (binding first-occurrence selection) and
`ess_compiler::ir::ResolvedSelectionPlan` are taken, and they mean something else. "Precedence" is
the word the governing page uses for this order. `PrecedencePlan::new(&ResolvedCommand,
FormatVersion)` builds one; `ess_domain::command::precedence::order` builds the same grouping over
`BranchShape`s for a consumer holding a `CommandSpec`.

### Derived on demand, never serialised

The plan is a function of a command and its format. It is built where it is read, holds borrowed
branches, and implements no `Serialize`; no IR type gains a field. So
`EssIr::to_canonical_json`, `to_compact_json` and `source_digest` keep their bytes and no format
version is minted (AGENTS.md, "Determinism and formats"). It is not cached on the command either:
synthesis clones a command and drops outcomes (`synthesize/related_guard.rs`), and a cached plan
would then name branches the clone no longer has.

### One test seam

`with_phase_order(order, || …)` (`#[doc(hidden)]`) runs a closure with the eight phases in another
order, on the current thread, and restores the previous order afterwards, also on unwind. `order`,
and so `PrecedencePlan::new`, read it. Every consumer builds its plan inside its own entry point
(`responding_core`, the lowering, the emitters' writer, synthesis, `CommandSpec::validate`), so an
order the constructor honours is the one seam that reaches all of them; each consumer story's
"phases exchanged" test uses it and adds none of its own. `Phase` has no `Ord`: a consumer that
compared phases directly would bypass the seam.

## What the plan does not say

What a request carries: whether an Optional reference is absent, which of several rows is missing,
which external branch a provider takes, what a guard reads. Those stay the consumer's, read in the
plan's order.

## Pinned

`crates/specify/ess-compiler/tests/selection_precedence_table.rs` lists every command of every
repository model that compiles with its plan, and each model's `EssIr::to_canonical_json` digest,
in `tests/fixtures/selection-precedence-table.tsv`, which opens with a header stating how many
models it pins and how many command lines they hold. The digests were written at the story's base
before the plan existed. The test fails where a pinned plan or digest moves, a pinned model is
gone, the tree holds a model the table does not pin, or the table disagrees with its own header.
`SELECTION_PRECEDENCE_TABLE_WRITE=<path>` writes the table, header included, instead of comparing
it; after it the test passes with nothing else edited, so the change that adds a model re-pins in
its own commit.
