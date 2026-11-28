---
format: aep.planning-md/3
id: story:a-state-only-an-each-move-reaches-is-arranged-for-the-refusals-of-the-entitys-other-commands
kind: story
status: draft
title: 'A state only an each: move of an affects: entry reaches is arranged for the scenarios of the entity''s other commands'
summary: 'An adopter reports that an affects: entry with each: over a list input leaves two outcomes with no scenario (ESS-SYNTH-001); the report does not reproduce, a neighbour does (ESS-SYNTH-004)'
tags:
- adopter-report
relations:
- serves: vision:O2
revision: 3
---
## Outcome

An `affects:` entry with `each:` over a command's list input does not stop its outcome from being witnessed, and a
state that only such an entry reaches is arranged for the scenarios of the entity's other commands. An empty list is a
valid witness of the outcome.

## Origin

A downstream product specification gave three commands a list input and wrote each command's outcome as `affects:` with
`each:` over that list. After that change two outcomes got no scenario (`ESS-SYNTH-001`) and, through them, 84
scenarios of the commands that need those outcomes' state. The adopter's ask: such an `affects ... each:` must not stop
witnessing the outcome, and an empty list is a valid witness. The adopter's specification is not in the store.

## Reproduction

Run with the released `ess` 0.99.8 on models written for this story, neutral nouns (`catalog.shelf`), kept in
`.engineering/repro/a-state-only-an-each-move-reaches-is-arranged-for-the-refusals-of-the-entitys-other-commands/`.

- **The adopter's failure does not reproduce.** `each-over-list-witnessed.yaml`: `Publish {catalog_id, items:
  List<ItemRef>}` with one outcome that moves the catalog and moves each listed `Item` by `affects:` + `each:`. `ess
  verify conform synthesize --path each-over-list-witnessed.yaml --out <file>` writes 12 scenarios, 0 refusals. The
  outcome has its own scenario, sent once with an empty list and once with listed rows, and the commands that need the
  published catalog have theirs. Two more shapes were run (not committed) and also write 0 refusals: an entry with
  `sets:` that creates or updates the row each element names (11 scenarios), and that entry beside a `distinct:`
  refusal branch (12 scenarios). The list element must be a struct (`instance: i.item_id`); a bare id is refused by validation
  (`ESS-COMMAND-001`), so the adopter's lists are lists of structs or the failure is a shape not tried here.
- **A neighbour reproduces.** `state-only-each-reaches.yaml`: the same entry on `Archive`, moving `Item` from `Listed`
  to `Removed`, the state no other outcome reaches. The same command gives 16 scenarios and 1 refusal:
  `ESS-SYNTH-004`, "entity catalog.shelf.Item has no scenario `catalog.shelf.Item/state/Removed/refuses/catalog.shelf.ListOne`:
  it needs an instance of `catalog.shelf.Item` resting in `Removed`, and no declared move reaches it from `Hidden`".
  A move that an `each:` entry makes is not a move the arrangement uses, and `Item/transition/remove/by/Archive/archived`
  gets no scenario. This is the shape of the adopter's cascade (a refused outcome or state, then every scenario that
  needs it), one code over.

## Fit review

Consumer: a downstream product specification.

1. **Need.** A state, and an outcome, that only an `each:` entry of `affects:` takes must be reachable by synthesis, so
   the scenarios that need them are written. The adopter's spelling of the ask is "such an entry must not stop
   witnessing the outcome"; the need is the arrangement and the witness, not a new key.
2. **Class.** The adopter's exact failure: not determined, it does not reproduce (above). The neighbour: a defect
   against the design page, which says an empty list is an accepted answer (`docs/design/set-effects-over-filtered-instances.md:227`)
   and is sent as a witness where the guard admits one (`:268`). The page uses an entry's own command to move a row for
   that command's scenario (`:352-356`); it says nothing of a state another command's scenario needs, which is the gap.
3. **Already expressible.** Yes for the outcome's own witness (the first model). For the state, an idiom exists: one
   command per row whose outcome `moves:` the row (the shape of story:each-entry-moves-the-rows-its-elements-name,
   question 3 (a)). It moves a checked figure out of the batch call, so it is not an answer.
4. **Fit.** No new key, no new diagnostic code. The arrangement that walks declared moves to a state learns the moves
   an `each:` entry makes, with an element naming the row (inferred from the refusal text; the code was not read).
   Siblings: whether the filtered `affects:` entry (`instances:`) has the same gap was not run; the change checks it
   and opens the whole row of entries or none.
5. **Second adopter.** A bulk import marks each line `accepted` or `rejected` in one call, and a line in `rejected` is
   refused by the next command.
6. **Cost.** No format bump expected. Synthesis output changes only for models with a state reached through an entry:
   extra scenarios, no moved byte elsewhere (the suite-bytes pin targets pass with no row moved).
7. **Alternatives.** (a) Change nothing: the adopter keeps the per-row command. (b) Ask the adopter for a reduction
   first and fix nothing until it reproduces. (c) Fix the neighbour now and ask for the reduction at the same time.

## Decisions

EA-D1, how the report is handled (2026-11-27; recommended: C; no answer yet).

- A: change nothing; document the per-row command idiom.
- B: ask the adopter for a reduction (the two outcomes' source with names replaced, and the refusal text of `ess verify
  conform synthesize`) and fix nothing before it reproduces. Fails closed: no change ships for a failure nobody has seen.
- C (recommended): fix the neighbour, and ask the adopter for the reduction at the same time. The neighbour has a
  committed reproduction and a design-page sentence it contradicts; the reduction decides whether the adopter's failure
  is the same one.

## Acceptance

- `an_each_entry_over_a_list_input_keeps_its_outcome_witnessed`, in `ess-conformance`, on
  `each-over-list-witnessed.yaml`: 12 scenarios and 0 refusals, the outcome's scenario sends an empty list, and another
  sends listed rows.
- `a_state_only_an_each_move_reaches_is_arranged`, in `ess-conformance`, on `state-only-each-reaches.yaml`: no
  `ESS-SYNTH-004`, 0 refusals, a scenario `Item/state/Removed/refuses/ListOne` that arranges an `Item` resting in
  `Removed` through `Archive`, and the interpreted target passes the suite. A mutant that drops the entry's move is
  killed by it.
- The test runs on a model that synthesizes once for the whole target (under 60 s), and the suite-bytes pin targets
  pass with no row moved.
- Before the story moves, the adopter's reduction is read: either it reproduces on the released binary and a test of its
  own joins the acceptance, or the story says it did not.

## Adopter reduction, 2026-11-27 (0.99.9): the original report had another cause, and an each-move refuses its outcome beside a row-set guard

The downstream adopter whose report opened this story reduced its two refused outcomes to a model of 157 lines on 0.99.9
(its specification is not in the store). The reduction answers the question this story's Reproduction left open.

**The original report had a different cause.** The two outcomes the adopter named (a list input and an `affects:` entry
with `each:`, refused `ESS-SYNTH-001`) are refused because of a row-set guard in the same command, whose stored-row
selector holds a membership over the list with a body of several tests; the list input and the each-move are not the
cause. Deleting the guard gives the outcome a scenario; deleting the each-move, or the list input with its `distinct`
guard, leaves the message unchanged. That cause is recorded on
`story:a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed` (the membership counts only when its body
is the one equality), with the neutral model `C1` and its controls under
`.engineering/repro/a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed/`. The reproduction models of
this story had no row-set guard, which is why the adopter's failure did not reproduce here.

**Behind that cause there is one that belongs here.** Where the guard's equality is counted (the body is the one
equality), the success outcome that moves the entity and writes the guard's rows with `affects:` and `each:` is still
refused `ESS-SYNTH-001`:

> a step of the scenario changes rows of the selector's entity in a way synthesis does not follow (a set effect, a
> binding or a direct setup)

Neutral models, from the same directory (ess 0.99.9; one entity of entries that the staging command writes through
`each:` over its list input, one row-set guard over those entries):

| model | guard body | `affects:` each | scenarios | refusals |
|---|---|---|---|---|
| C2 | `e.name == name` | kept | 5 | 2 |
| C3 | `e.name == name` | deleted | 6 | 1 |

C2 refuses the guard's own outcome (no arrangement takes that branch) and the success outcome with the text above; C3
refuses the guard's own outcome only (`Entry.name` is read by a filter and no declared command writes it, because the
each-move was the one writer), and the success outcome and its transition have scenarios. So the each-move that writes
the rows a guard of the same command reads is what refuses the success outcome. It is not the state-only each-move of
the neighbour above (`ESS-SYNTH-004` on a state no other move reaches): the code differs and the shape does not need
another command.

Adopter's impact: in its whole specification on 0.99.9 (493 scenarios, 673 refusals, 512 of them `ESS-SYNTH-001`), 171
refusals carry this text, 133 of them naming the preview outcome as the step. The preview
outcome itself and the release outcome of a second command carry the "no equality" refusal first (the other story), so
this refusal is what follows once that one is cleared.

A third refusal sits behind this one: a list guard that excludes some element values leaves the each-entry with no four
elements to send, `story:an-each-entry-beside-a-list-guard-that-refuses-some-element-values-is-witnessed`.

Fit, in the words of `.agents/skills/assessing-external-requests/SKILL.md`:

1. Need: an outcome whose `affects:`/`each:` entry writes the rows a guard of its own command selects is witnessed, as
   the first section of this story asks (an each-entry does not stop its outcome from being witnessed).
2. Class: the same defect as this story's Outcome, in a shape its first reproduction did not hold.
3. Already expressible: partly. Dropping the guard or the each-move makes the outcome witnessed and drops the rule.
4. Fit: no new key. The arrangement follows an each-entry's effect on rows a guard reads, as it follows a set effect
   on them. Whether the entity's other row-set refusals of the command can be witnessed beside it is part of the work.
5. Second adopter: any command that registers each listed line and refuses a line already registered.
6. Cost: no format bump; suites change only for models with this pair.

Acceptance added by this report:

- C2 writes a scenario for the success outcome, so its 5 scenarios and 2 refusals become 6 or more scenarios and at most
  1 refusal (the guard's own outcome stays refused, as in C3, until a command writes the guard's field). Test: `C2` with
  its success outcome and transition witnessed and the interpreted target passing the suite.
- A target that ignores the each-entry fails a scenario of that suite.
- The acceptance above (the state-only each-move, the original report) is unchanged.

## Decision EA-D1 answered in part, 2026-11-27

EA-D1 option B asked the adopter for a reduction. The adopter sent one on 0.99.9 (section above): the original report is the
row-set guard of `story:a-row-set-selected-by-a-literal-identity-and-an-input-list-is-witnessed`, and the each-move refusal of the
success outcome is the part that belongs to this story. The neighbour above (the state-only each-move, `ESS-SYNTH-004`) is
unchanged and still reproduces only on its own model.
