---
format: aep.planning-md/3
id: story:a-refusal-reading-a-related-rows-state-may-answer-before-held-state-branches
kind: story
status: draft
title: A refusal that reads a related row's lifecycle state may answer before the held-state branches of its command
summary: Expressible and checked where the ordinary effect is a plain accepting branch; before a when_subject_state branch it conflicts with the precedence rule, RS-D1 with the owner, no default
tags:
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-related-row-may-answer-before-input-guards
- informed_by: story:refusal-above-held-state-branch-validates
- informed_by: story:every-guard-combination-validates-under-the-rule
- informed_by: story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view
revision: 1
---
## Summary

A command on one entity must answer differently while a related entity, named by a stored reference
of the addressed row, rests in one lifecycle state: it changes nothing then, instead of applying its
ordinary effect. The related row's state is readable today. What cannot be said is that this refusal
answers before the command's held-state branches. Where the command's ordinary effect is a plain
accepting branch the need is expressible and checked today; where the ordinary effect is selected by
the addressed row's own lifecycle state (`when_subject_state:`), the order the specification
states, held state at step 3 and related rows at step 4, is the one the refusal runs against.
Changing that order is the owner's: `decision-blocker:related-state-refusal-and-the-held-state-step`
(RS-D1, no default).

## Reproduction

Two entities, `Account` (lifecycle `Active -> Suspended`) and `Item` (lifecycle `Open -> Done`, a stored
field `owner` of the account's identity type), and `CompleteItem` on `Item`. On 0.98.0:

```yaml
    outcomes:
      - name: owner-missing
        when_related: {via: owner, exists: false}
        error: demo.work.OwnerMissing
      - name: owner-suspended
        when_related: {via: owner, predicate: state == Suspended}
        error: demo.work.OwnerSuspended
      - {name: wrong-state, wrong_state: true, error: demo.work.ItemStateConflict}
      - name: completed
        moves: demo.work.Item.complete
        instance: item_id
        emits: [demo.work.ItemCompleted]
        payload: {demo.work.ItemCompleted: {item_id: input.item_id}}
```

This validates and `ess verify conform run --target interpreted` passes 11 of 11, including the
scenario for `owner-suspended` (the suite suspends the owner through `SuspendAccount`, then sends the
command). With `when_subject_state: Open` added to `completed`, validation refuses it as
`ESS-COMMAND-004`: "`completed` answers at step 3 of the precedence order and `owner-missing`,
declared above it, at step 4; where both hold, the declaration order and the precedence order
disagree", help "declare `completed` before `owner-missing`", and declaring it first makes the
refusals unreachable on an `Open` item. A `when_related` and a `when_subject_state` on one branch is
refused `conflicting_declaration` ("keep the related guard and an optional `when:` over the input,
or the other key").

## Acceptance

Under the owner's answer to RS-D1; for A:

- `ESS-COMMAND-004` for a step-4 branch declared above a step-3 branch names the repair that works
  when the ordinary effect can be a plain accepting branch: drop `when_subject_state:` from it, keep
  `wrong_state:`, and declare the related-state refusal above it; a test holds the help text.
- The reference page for `when_related` gets that example, checked by the page's own test.

For B, C or D the acceptance is written when the owner picks one.

## Fit review

1. **The need, apart from the syntax.** A command's answer depends on the lifecycle state of a row
   its addressed row references, and that answer takes precedence over what the addressed row's own
   state selects. Minimal model above. The reporter's two proposals: a branch that reads a related
   entity's state and is decided before the held-state cuts; or a declared tie, an input true
   exactly when the related entity is in a state, checked in scenarios.
2. **Class.** Part convenience, part gap. The state read exists (`ess/20`,
   `website/docs/reference/predicates.md:1388`; `ess/28` for a value source,
   `spec-versions.md:71`). The pair "related-state refusal and held-state ordinary branch" is refused
   by the precedence rule, not by a missing read.
3. **Can it be expressed?** Yes for a plain accepting ordinary branch, shown above with a green
   suite (the figure stays checked). Through an input id: the reading is `{entity, where, forall}`
   with an explicit entity, which validates (`predicates.md:30`; a related-selector form notes a
   `references` relation to declare); not run to the scenario here. No for an ordinary effect selected by
   `when_subject_state:` (RS-D1).
4. **Fit.** The rule is `docs/design/generic-language-features.md` "The rule" and
   `docs/design/cross-record-and-stored-field-guards.md` (the step table, steps 3 and 4: a related
   row is "around the addressed record"). A request to place a related-state refusal before step 3
   conflicts with it. The sibling case, a related row ahead of input guards (step 1), was answered
   by keeping the order and narrowing (`story:a-related-row-may-answer-before-input-guards`), and
   an external refusal has `before_row: true`
   (`story:refusal-above-held-state-branch-validates`, implemented). Narrowing does not carry over:
   a branch has one condition, and `when_related` with `when_subject_state` is refused.
5. **A second adopter.** A task whose owner project is archived, a shipment whose carrier is
   suspended, any child command that must do nothing while its parent is in one state.
6. **Cost.** A: none to the formats. B: a new marker, a precedence step, a source format version,
   validation for every step pair, synthesis, the interpreter, generated Rust and Go, Entity Runtime
   lowering, docs and diff classification. C: a new kind of declaration. D: composition of two guard
   kinds in one branch.
7. **Considered.** A, B, C, D of RS-D1; A is the only one that adds no surface.

Findings covered: the related-state refusal beside held-state branches, the ambiguity of an input
read of a shared identity type (the explicit `entity` form), and the declared tie (RS-D1 C).
Borders: `story:a-related-row-may-answer-before-input-guards`,
`story:refusal-above-held-state-branch-validates`, `story:every-guard-combination-validates-under-the-rule`,
`story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected`,
`story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view` (an accepting branch that
changes nothing; the reporter's "changes nothing" may be that, not probed here).

## Decisions

- Defer: `decision-blocker:related-state-refusal-and-the-held-state-step`, RS-D1, no default: it
  changes the order of the one precedence rule.
- Nothing is built before the owner picks.

## Scope

- `crates/specify/ess-domain/src/command/related_guard.rs` and `command.rs` (the step pair
  refusal and its help), `website/docs/reference/predicates.md`; for B also `ess-compiler`
  `ir/precedence.rs`, `ess-conformance` `synthesize/related_guard.rs`, `interpret/execute.rs`,
  `ess-synth` and `ess-entity-runtime`.
