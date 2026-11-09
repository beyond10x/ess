---
format: aep.planning-md/3
id: story:a-not-defined-branch-narrows-an-optional-in-later-branches
kind: story
status: draft
title: A `not defined` branch narrows an Optional for the branches after it
summary: A payload after an absent-optional refusal still sees Optional<String> (workbench report item 19)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

A command refuses when a stored `Optional` field of its subject is absent
(`when_subject: {predicate: not defined(note)}` with `error:`), and its default branch (no guard)
then reads `{subject: note}` into a `String` payload field. The default is only ever taken with
`note` present, yet validation still reads it as `Optional<String>`:

```
error[ESS-COMMAND-002]: the subject's `note` has type `Optional<String>`, and `note` requires `String`; no conversion is declared
```

Same shape as #556 (closed). Item 19 of the 2026-11-09 workbench report; example `19`.

Reproduced on 0.92.0 (`ess/35`). Minimal shape, from the workbench base example:

```yaml
- name: gap.demo.NoteItem
  input:
  - {name: id, type: String}
  outcomes:
  - name: no-note
    when_subject: {predicate: not defined(note)}   # Item.note: Optional<String>
    error: gap.demo.NoSuchItem
    payload: {gap.demo.NoSuchItem: {id: input.id}}
  - name: noted                                     # default branch
    updates: gap.demo.Item
    instance: id
    sets: {n: 1}
    emits: [gap.demo.ItemNoted]
    payload:
      gap.demo.ItemNoted: {id: input.id, note: {subject: note}}   # ItemNoted.note: String
```

`ess specify validate` exits 1 with the diagnostic above. Two neighbouring shapes already validate
(exit 0) on 0.92.0, so the gap is the stored-field side only:

- The input side: `when: not defined(note)` refusal beside a default reading `input.note`
  (ess/16, `docs/design/optional-input-narrowing.md`; `refuses_absence_of` in
  `crates/specify/ess-domain/src/command/narrowing.rs:127-137`).
- The branch's own guard: `noted` with `when_subject: {predicate: defined(note)}`
  (ess/27, `narrows_subject_field`, `narrowing.rs:83-89`).

The stored-field rule has no counterpart of `refuses_absence_of`: `narrows_subject_field` looks
only at the outcome's own condition.

## Acceptance

Narrowing is order-independent, as for inputs: it applies to the default branch, not to "every
later branch".

- `ess specify validate` on the reproduction above exits 0: the default branch reads the subject's
  `note` at `String` in its payload, `sets:` and guards when a sibling refusal is guarded by exactly
  `when_subject: {predicate: not defined(note)}` (or `missing(note)`).
- A guarded non-default branch beside that refusal still reads `Optional<String>` and draws
  ESS-COMMAND-002; so does the default when the refusal's guard is weaker (`not defined(note)` in
  an `any:`, or with another conjunct).
- Below the format version that admits it, the reproduction keeps its ESS-COMMAND-002 and the hint
  names the version that admits the narrowing.
- The interpreted target and synthesis agree: `ess verify conform synthesize` then
  `ess verify conform run --target interpreted` on the reproduction passes, with a scenario for the
  refusal (note absent) and one for the default (note present).

## Scope

- cited: crates/specify/ess-domain/src/command/narrowing.rs
- cited: crates/specify/ess-domain/src/command/value_expression.rs
- inferred: crates/specify/ess-domain/tests/
- inferred: docs/design/optional-input-narrowing.md
- inferred: crates/verify/ess-conformance/tests/

confidence: medium

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.

## Decisions

None yet. Open: whether the relaxation needs a new format version (see
story:format-rule-for-relaxations); the ess/16 and ess/27 precedents each gated theirs.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, with its examples under `examples/`.
