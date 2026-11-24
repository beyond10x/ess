---
format: aep.planning-md/3
id: story:an-awaited-system-step-of-a-narrowed-external-branch-states-the-input-its-guards-read
kind: story
status: draft
title: The synthesized await of a system-issued command's narrowed external branch states the input its guards read
summary: 'A defect, reproduced on 0.98.0 and unchanged on main: the await of an external branch beside when_subject states no input, so a correct target answers the sibling and the scenario fails; hotfix candidate'
tags:
- defect
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-system-issued-awaited-step-honours-its-outcomes-input-guard
- informed_by: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes
- informed_by: story:external-branch-narrowed-by-subject-guard
revision: 1
---
## Summary

Hotfix candidate under `AGENTS.md` § Hotfix: reproduced on 0.98.0 with a generic model, and the code
that decides it is unchanged on `main` at 0.99.0.

A command a system actor issues has an `external:` branch narrowed to the rows a stored-field guard
selects (`external:` beside `when_subject:`, `ess/29`), with an input guard beside it: a `when:`, or an
`input.<field>` comparison inside the `when_subject:` predicate. The synthesized scenario for that
branch awaits the command and states no input. A correct target is held to the branch while the
system's own choice of input selects a sibling, so the scenario fails (`ESS-CF-OUTCOME`) on the
interpreted target and on every target that honours the specification. An external branch with an
input guard and no stored-field guard gets the input stated.

## Reproduction

A ledger entity with a `Proposed -> Validated -> Committed | Stale` lifecycle and an integer field
`counter`, a system actor `Scheduler` (`system: true`) that may issue `Commit`, and a boolean input
`recheck`. Everything but the command is the usual ledger model of
`crates/verify/ess-conformance/tests/external_narrowed_by_subject.rs` (`LEDGER`).

```yaml
  - name: demo.ledger.Commit
    input:
      - {name: transaction_id, type: demo.ledger.TransactionId}
      - {name: recheck, type: Boolean}
    outcomes:
      - name: stale
        external: the head moved after validation
        when: recheck == false
        when_subject:
          predicate: {all: [state == Validated, counter < 3]}
        moves: demo.ledger.Transaction.outdate
        instance: transaction_id
        emits: [demo.ledger.Outdated]
        payload:
          demo.ledger.Outdated: {transaction_id: input.transaction_id}
      - name: committed
        moves: demo.ledger.Transaction.commit
        instance: transaction_id
        emits: [demo.ledger.Committed]
        payload:
          demo.ledger.Committed: {transaction_id: input.transaction_id}
      - name: not-validated
        wrong_state: true
        error: demo.ledger.NotValidated
```

On 0.98.0 `ess verify conform run --target interpreted --path <model> --report-format 2` reports 15
scenarios, 11 passed, 4 failed: `demo.ledger.Commit/outcome/stale` and the three scenarios that reach
`stale`. The synthesized `stale` scenario forces the cause with `configure_external_outcome` and then
has an `await_outcome` step for `Commit` that carries only `shape`, no `input`. The same model with
`when: recheck == true` passes, which reads as the interpreter's own choice sending `true` (inferred). The
second form, `input.recheck == false` inside the predicate and no `when:`, fails the same scenario. A
third form, the branch declared after the sibling that takes every other input, goes through the same
function. The same models with a human actor issuing `Commit` pass: the send states its input.

## Cause (read, and probed)

`crates/verify/ess-conformance/src/synthesize/system_issued.rs`: `guard` (line 365) returns the input
guard of `When`, `ExternalWhen`, `SubjectPredicate`, `Related` and `RelatedSet` and `None` for
`ExternalSubject { predicate, input }`, so `guarded_input` (line 322, called at line 201) collects no
guard and states no input. `claim_of` (line 267) returns `Claim::Reached` for every external branch
before it looks at the condition, so the `SubjectPredicate` rule that classifies an `input.` read
inside the predicate (lines 281-286) is never applied to the narrowed external branch.

## Acceptance

- For the model above, `ess verify conform synthesize` writes an `await_outcome` step for the `stale`
  scenario that states `recheck` at the value synthesis sent, which `when:` admits, and `ess verify
  conform run --target interpreted` passes 15 of 15.
- The same holds for the form with `input.recheck` inside the `when_subject:` predicate (SX-D1) and
  for the branch declared after its sibling.
- An external branch narrowed by a stored-field guard with an input the send leaves absent is
  refused `ESS-SYNTH-006` for the scenario, as an external branch with a `when:` is today.
- A branch with no input guard states nothing and the bytes of every suite of a model without such a
  branch do not move.
- A regression test in `tests/system_issued_synthesis.rs` holds the three forms, red first.

## Fit review

1. **The need.** A scenario ESS wrote for a branch of a system-issued command must hold a correct
   target to that branch, so the await states the input that selects it. Minimal model above. The
   reporter's proposal (pin every input the branch's guard reads, also with a subject guard) is the
   need.
2. **Class.** Defect: `docs/design/system-issued-commands.md` (Which branches a scenario can claim)
   and `story:a-system-issued-awaited-step-honours-its-outcomes-input-guard` (implemented) state
   the await states the input a guard selecting the claimed branch reads; the narrowed external
   branch of `ess/29` was added after that story and was not given a `guard` arm.
3. **Expressible today.** Only by dropping the input from the guard (the reporter's workaround, plus
   an unmapped declaration), which moves a checked figure out of the specification.
4. **Fit.** No authored surface, no format version. Siblings: `story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes`
   did the same for `SubjectPredicate`; this is the external half of it.
5. **A second adopter.** Any scheduled or system-issued command that an outside check may refuse
   only on a given input and a given row state.
6. **Cost.** Suite bytes of a model that has such a branch change (the await gains an `input`); none
   for any other model.
7. **Considered.** Change nothing (a failing suite). Refuse the scenario for an `input.` read inside
   the predicate as `SubjectPredicate` is refused (SX-D1 B). Stating the input (SX-D1 A) wins: the
   scenario is the one failing today and the sent value already meets the guard.

## Decisions

- SX-D1, default A: for an `input.<field>` read inside the `when_subject:` predicate of a narrowed
  external branch, A states the input at the sent value; B refuses the scenario `ESS-SYNTH-024` as a
  `SubjectPredicate` of a system step is refused. Recommended A. Not architecture and not a security
  boundary; taken on default if unanswered.
- Accept as reported: the defect, with no new syntax.

## Scope

- `crates/verify/ess-conformance/src/synthesize/system_issued.rs` (`guard`, `claim_of`,
  `guarded_input`), `crates/verify/ess-conformance/tests/system_issued_synthesis.rs`,
  `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` and
  `synthesis-sources.digest` if a committed model has such a branch, `CHANGELOG.md`.
