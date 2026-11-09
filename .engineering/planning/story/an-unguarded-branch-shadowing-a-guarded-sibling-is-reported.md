---
format: aep.planning-md/3
id: story:an-unguarded-branch-shadowing-a-guarded-sibling-is-reported
kind: story
status: draft
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
- confidence: inferred
  path: crates/edge/ess-cli/tests/
- confidence: cited
  path: crates/specify/ess-domain/src/command/precedence.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/subject_state.rs
- confidence: inferred
  path: crates/specify/ess-domain/tests/
revision: 5
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

- `ess specify validate` on the reproduction above (`ess/35`) reports `dropped` as unreachable,
  naming `dropped` and `finished` and the held state `Open`: a refusal under the existing
  `UnreachableBranch` code, or a warning if the adopter replay shows a specification valid today
  would be refused (the D186 fallback that the input-guard twin used).
- With `dropped` declared before `finished`, validate reports nothing about either branch.
- A guarded branch that is first reader for at least one (held state, input) case of the
  partition is not reported.
- Below `ess/27` the reproduction keeps its ESS-COMMAND-004.

## Scope

- cited: crates/specify/ess-domain/src/command/subject_state.rs
- cited: crates/specify/ess-domain/src/command/precedence.rs
- inferred: crates/specify/ess-domain/tests/
- inferred: crates/edge/ess-cli/tests/

confidence: medium

## Decisions

None yet.
