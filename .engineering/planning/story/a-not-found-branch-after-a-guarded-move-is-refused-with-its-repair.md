---
format: aep.planning-md/3
id: story:a-not-found-branch-after-a-guarded-move-is-refused-with-its-repair
kind: story
status: active
title: A not-found branch after a guarded move is refused with its repair, never as a defect in ess
summary: An external not-found declared after a when-guarded move gets one refusal naming before_row, not ESS-SYNTH-019 (adopter report 2026-11-04, ess 0.85.0)
tags:
- ess-w1-0870
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/precedence_order.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-11-04T08:47:27Z", actor: "human:timo", revision: 5, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
- {from: "proposed", to: "active", at: "2026-11-04T08:47:28Z", actor: "human:timo", revision: 6, executor: "agent:ess-w1-lead", correlation: "ess-0.87.0-w1"}
---
# Story: a not-found branch after a guarded move is refused with its repair, never as a defect in ess

## Outcome

An author whose `external:` not-found branch is declared after a `when:`-guarded accepting branch
gets one refusal that names the precedence step and the repair (declare the not-found first with
`before_row: true`), instead of an `ESS-SYNTH-019` whose help says the specification is fine and
ess is broken.

## Context

A downstream adopter's report of 2026-11-04 on ESS 0.85.0 (format `ess/34`): `ess verify conform
synthesize` gives four `ESS-SYNTH-019` refusals, one per command, on not-found refusal branches
whose outcome requires the store unchanged. `crates/verify/ess-conformance/src/synthesize.rs` is
unchanged between the 0.85.0 tag and `main` at 8d26c81f74 (`git diff --stat 0.85.0 origin/main`
empty), so `main` behaves the same. Not a regression: the reporter withdrew the claim that an older
release synthesized these.

## Minimal reproduction (written for this story, invented names)

`demo.shelf`: an entity `Bin` (identity `bin_id: BinId`, a Uuid newtype; lifecycle `Closed ->
Opened` by `open`), an enum `Move` with one variant `Open`, an error `BinNotFound` with one field
`bin_id: BinId`, and a command `HandleBin {bin_id, move}` with two outcomes in this order:

```yaml
- name: opened
  when: move == Open
  moves: demo.shelf.Bin.open
  instance: bin_id
  emits: [demo.shelf.BinOpened]
- name: not-found
  external: no bin carries input.bin_id
  error: demo.shelf.BinNotFound
```

On ess 0.85.0, synthesize refuses `not-found` twice: `ESS-SYNTH-003` ("no candidate of the 1 tried
satisfies ` outside opened (move == Open), the accepting branch declared first`") and
`ESS-SYNTH-019` ("a step requires `not-found` for {bin_id: …, move: Open}, which `opened` (move ==
Open) answers first under the precedence order"; help: "nothing to change in the specification;
this is a defect in ess to report"). Controls, from the reporter and re-run by the wave lead on
0.85.0:

- without `when: move == Open`: neither refusal, the `not-found` scenario is synthesized;
- without the error's `fields:`: no `ESS-SYNTH-019`, `ESS-SYNTH-003` stays;
- with `not-found` declared first and marked `before_row: true`: validate is clean and `not-found`
  is synthesized, no `ESS-SYNTH-003`, no `ESS-SYNTH-019` (wave lead, 2026-11-04);
- `before_row: true` left after `opened` is refused by validate (`ESS-COMMAND-004`, "declare
  `not-found` before `opened`").

## Problem

Under the precedence order (`docs/design/cross-record-and-stored-field-guards.md` § The precedence
order, step 6; `docs/design/input-guard-overlap-precedence.md`, beyond10x/ess#217) an accepting
branch declared before an `external:` one answers every input its guard claims. With one `Move`
variant, `opened` claims every input, so `not-found` is unreachable as declared: `ESS-SYNTH-003` is
the right answer, and its help does not name the repair. `ESS-SYNTH-019` is a self-check
(`synthesize.rs:17827-17850`, `RefusalCause::PrecedenceContradicted`) meant to catch a step
synthesis built wrongly; here a step is still built for the error's field `bin_id` (named like the
command input) and the self-check fires on it, reporting a specification-side cause as a defect in
ess.

## Acceptance

Named scenarios; each is a test function, and the story is done when each runs and passes.

- `crates/verify/ess-conformance/tests/precedence_order.rs` (or a new `notfound_after_guarded_move.rs`)
  - `a_not_found_after_a_guarded_move_is_refused_once_with_its_repair` — the reproduction gives no
    `ESS-SYNTH-019`, exactly one refusal on `not-found`, and its help names `before_row: true` and
    declaring the branch first.
  - `an_error_field_named_like_the_input_builds_no_step_the_precedence_refutes` — the same model with
    and without the error's `fields:` gives the same refusals.
  - `a_not_found_declared_first_before_the_row_is_synthesized` — the repaired model synthesizes the
    `not-found` scenario with the store unchanged, and no refusal names it.
  - `a_reachable_not_found_after_a_guarded_move_is_still_synthesized` — with a second `Move` variant
    the guard does not claim, `not-found` is synthesized as today (no change for models that work).
- `ESS-SYNTH-019` keeps firing on a step that genuinely contradicts the precedence order:
  every existing `precedence_order.rs` and `adversary_overlap_precedence_pass2.rs` case passes
  unchanged.

## Out of scope

- Changing the precedence order (an addressed row that does not exist taking an accepting branch out
  of step 6). Decision round question Q-A records it.
- A validate-time warning for an unreachable branch (`story:an-unguarded-branch-shadowing-a-guarded-sibling-is-reported`, a draft).

## Fit review

1. **Need.** An author is told ess is broken when the specification declares an unreachable branch.
   Requester's words: "4 ESS-SYNTH-019 refusals on not-found refusal branches whose outcome requires
   the store unchanged".
2. **Class.** Defect: `ESS-SYNTH-019`'s own text says it fires only for a defect in ess
   (`synthesize.rs:1767-1771`); here the cause is in the specification.
3. **Already expressible?** Yes: `before_row: true` on a not-found declared first (shown above). The
   answer to the author is that idiom, surfaced in the refusal.
4. **Fit.** No new surface. The refusal joins the existing `ESS-SYNTH-003` family; the help text
   names a construct that already exists (`ess/25`, `before_row`).
5. **Second adopter.** Any command that dispatches on an action enum (`op: Create|Update|Delete`) and
   declares its not-found last.
6. **Cost.** A changed help line and one fewer refusal on affected models; no format change, no
   suite byte change for models that synthesize today.
7. **Considered.** (a) change nothing: the author files a defect for a specification error. (b)
   change the precedence so an absent addressed row skips `opened`: contradicts step 3/6 of the
   documented order and every runtime that follows it (Q-A). (c) chosen: one refusal, with the
   repair.

## Decisions

Accept, redesigned (wave lead, 2026-11-04): the report asks that the refusals go away; the
specification as written makes `not-found` unreachable, so the fix is a correct refusal with the
repair, not a scenario. Decided 2026-11-04 by the coordinator, Q-A answer (a): the precedence order
stands; one refusal whose help names `before_row: true` and declaring the not-found first; no
ESS-SYNTH-019 (approval-record:ess-0870-wave).
