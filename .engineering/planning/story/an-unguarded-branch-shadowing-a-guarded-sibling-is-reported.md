---
format: aep.planning-md/3
id: story:an-unguarded-branch-shadowing-a-guarded-sibling-is-reported
kind: story
status: active
title: A guarded branch an unguarded sibling always shadows is reported
summary: Validate accepts a held state where an unguarded branch makes a guarded one unreachable
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/shadowed_guarded_branch.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:53Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:54Z", actor: "human:timo", revision: 10, decided_on: {"recorded":{"review_outcome":1}}}
---
## Problem

In one held state, a branch with no `when:` beside a guarded branch makes the guarded branch
unreachable (`selected_in_state` sees two selected), and `ess specify validate` accepts the command
anyway. Found by the wave D335 U2 implementor while reproducing an adopter finding on 0.77.0.

Reproduced on 0.92.0 (`ess/35`). `Item` has transitions `finish` and `drop`, both `Open → Done`:

```yaml
- name: gap.demo.FinishItem
  input:
  - {name: id, type: String}
  - {name: flag, type: Boolean}
  outcomes:
  - name: finished                 # no `when:`
    when_subject_state: Open
    moves: gap.demo.Item.finish
    # instance, emits, payload
  - name: dropped                  # shadowed: never answers
    when_subject_state: Open
    when: flag == true
    moves: gap.demo.Item.drop
    # instance, emits, payload
  - {name: unknown, unknown_instance: true, error: gap.demo.NoSuchItem}
  - {name: wrong-state, when_subject_state: Done, error: gap.demo.NoSuchItem}
```

- `ess specify validate` exits 0.
- `ess verify conform synthesize` exits 0 with ESS-SYNTH-003 "no candidate of the 2 tried
  satisfies `dropped selected in held state Open`", plus knock-on refusals for the `drop`
  transition and the `Done` state.
- The same model under `ess/26` is refused: `ESS-COMMAND-004 held state Open and input
  [flag = true] select 2 branches: finished, dropped`.

Cause: from `ess/27` (the one rule, `precedence::ONE_RULE`) `validate_partition` collapses the
selected set with `read_first` (`crates/specify/ess-domain/src/command/subject_state.rs`, "Under
the one rule the branch the plan reads first answers"), so an overlap is no longer an error, and
nothing checks that every guarded branch is still the first reader for some (state, input) case.
The input-guard twin of this check exists:
story:validate-refuses-a-branch-implied-by-an-earlier-input-refusal (implemented).

## Acceptance

All cases are in `crates/specify/ess-domain/tests/shadowed_guarded_branch.rs`, run with
`cargo test -p ess-domain --locked --test shadowed_guarded_branch`.

- On the reproduction above (`ess/35`), validation fails with one `UnreachableBranch` error at
  `outcomes.dropped` whose message names `dropped`, `finished` and the held state `Open`;
  `ess specify validate` on it exits 1.
- With `dropped` declared before `finished`, validation reports no error at either branch.
- A guarded branch that is first reader for at least one (held state, input) case of the
  partition (`finished` given `when: flag == false`) reports no error.
- The reproduction re-declared at `ess/26` keeps its single `ESS-COMMAND-004` error and gets no
  `UnreachableBranch`.
- No committed example or corpus model is newly refused: `cargo test -p ess-domain --locked` and
  the diagnostics pins stay green with no pin moved.

## Scope

- cited: crates/specify/ess-domain/src/command/subject_state.rs (`validate_partition`, line 358; `read_first`, line 449)
- cited: crates/specify/ess-domain/src/command/precedence.rs
- inferred: crates/specify/ess-domain/tests/shadowed_guarded_branch.rs (new)

confidence: medium

Shared surfaces moved: proposes its `CHANGELOG.md` sentence; no format row, no byte-pin row.

## Decisions

A refusal under the existing `UnreachableBranch` code, not a warning: the input-guard twin refused
too, and a model that holds the shadowed branch already fails at synthesis (ESS-SYNTH-003), so the
refusal moves an existing failure earlier rather than breaking a specification that works.
