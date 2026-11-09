---
format: aep.planning-md/3
id: story:diff-decides-an-added-subject-guard
kind: story
status: draft
title: ess verify diff decides an added when_subject refusal
summary: An added when_subject refusal that refuses a subject state and input the earlier revision accepted is classified RefusalAdded instead of left undecided.
tags:
- adopter-report
- defect
- ess-0.70.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Outcome

`ess verify diff` classifies an added `when_subject:` refusal as `RefusalAdded` (breaking for callers) where some
stored subject state and input that the earlier revision accepted are refused by the later revision, instead of
leaving it undecided.

## Evidence

`Compatibility::narrowing` (`crates/verify/ess-diff/src/compatibility.rs:955-978`) decides an added or changed outcome
through `refuses_more`, which leaves every outcome that is not a plain `when:` or `otherwise` branch undecided,
`when_subject:` included (doc at :939-954). `story:guard-tests-list-membership` asked for an added membership guard to
be called breaking; that unit (0.69.0) left the line out.

Reproduction on 0.92.0, `ess/29`: a `Feed` entity (`label: String`, lifecycle `Open -> Cancelled`), `Rename`
(`updates: Feed`, `sets: {label: input.label}`, no guard). The later revision adds, before `renamed`:

```yaml
- name: locked
  when_subject: {predicate: label == "locked"}
  error: cascade.feed.WrongState
```

```text
$ ess verify diff --from before.yaml --to after-label-refusal.yaml --fail-on breaking
compatibility: 0 breaking, 1 unknown, 0 compatible
  changes  command cascade.feed.Rename: outcome `locked` added
           command/cascade.feed.Rename/outcome-added/locked
           unknown for callers, readers
gate --fail-on breaking: passed (0 acknowledged)
exit=0
```

A feed renamed to `locked` was accepted by the earlier revision and is refused by the later one, so the change is
breaking for callers, and `--fail-on breaking` passes it. Control: adding
`already-cancelled` (`when_subject: {predicate: state == Cancelled}`) to `Cancel`, whose `wrong-state` branch already
refuses every `Cancelled` feed, also reports `unknown for callers, readers`; it should stay undecided or compatible,
never breaking.

## Acceptance

Tests in `crates/verify/ess-diff/tests/narrowed_acceptance.rs` over the two pairs above:

- `an_added_subject_guard_refusal_is_breaking_for_callers`: `after-label-refusal` reports
  `command/cascade.feed.Rename/outcome-added/locked` as `refusal-added`, `breaking` for callers, and
  `--fail-on breaking` exits 4.
- `a_subject_guard_refusing_only_already_refused_states_is_not_breaking`: `after-already-refused` reports no
  `refusal-added` and `--fail-on breaking` exits 0.
- A `when_subject:` refusal whose stored state no outcome of the earlier revision can produce (no creator or update
  writes the field it reads to a refused value) stays unclassified, as "Unknown is an answer"
  (`compatibility.rs:34-62`).
- Existing cases in `narrowed_acceptance.rs` and `subject_fact_complete_refusal.rs` keep their verdicts.

## Scope

- cited: crates/verify/ess-diff/src/compatibility.rs
- cited: crates/verify/ess-diff/tests/narrowed_acceptance.rs
- inferred: crates/verify/ess-diff/tests/subject_fact_complete_refusal.rs
- inferred: crates/verify/ess-conformance/src/mutate.rs (`satisfiable_by_declared_values`, if the stored-state search lives there)

confidence: medium
