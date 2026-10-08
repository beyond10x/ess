---
format: aep.planning-md/3
id: story:negated-defined-optional-input-witnessed-beside-when-subject
kind: story
status: draft
title: The guard-negate mutant of a defined(<optional input>) guard beside when_subject is witnessed or scored
relations:
- serves: vision:O2
revision: 4
---
## Outcome

`ess verify conform mutate` reports the `guard-negate` mutant of a branch guarded by
`when: defined(<optional input>)` beside a `when_subject:` predicate that compares a stored field
with that same input as unwitnessed: synthesis finds no scenario for the negated guard
`not defined(<optional input>)` (`ESS-SYNTH-003`), so the mutant is neither killed nor shown
equivalent. After this story synthesis witnesses the negated guard (the input absent, the subject
arranged for the predicate) or the audit scores the mutant by a stated rule. Reported by an adopter
on 2026-10-08. Shape, renamed:

```yaml
- name: demo.order.Fulfil
  input:
  - {name: order_id, type: Uuid}
  - {name: receipt, type: String}
  - {name: expected_version, type: Optional<Integer>}
  outcomes:
  - name: stale-version
    when: defined(expected_version)
    when_subject:
      predicate:
        all:
        - state == Open
        - version != input.expected_version
    error: demo.order.OrderStateConflict
  - name: applied
    moves: demo.order.Order.fulfil
    instance: order_id
    emits: [demo.order.Fulfilled]
    payload:
      demo.order.Fulfilled: {order_id: input.order_id, receipt: input.receipt}
    sets: {receipt: input.receipt}
  - {name: wrong-state, wrong_state: true, error: demo.order.OrderStateConflict}
  - {name: not-found, unknown_instance: true, error: demo.order.OrderNotFound}
```

Unwitnessed: `guard-negate/demo.order.Fulfil/stale-version` (`defined` becomes `not defined`).
An open question for the design: with the input absent, `version != input.expected_version`
compares against an absent value; whether that predicate holds decides whether the mutant is
witnessable or equivalent.

## Fit review

1. **Need.** "A request that omits the expected revision is refused as stale" while the subject row
   is selected through the same optional field. The witness must send the input absent and still
   arrange the subject the other branches need.
2. **Class.** Synthesis coverage of a construct the language already accepts; no new syntax.
3. **Already expressible?** Yes in the specification; only the witness is missing.
4. **Fit.** The `defined` decision added to `mutate.rs` `satisfiable` for
   https://github.com/beyond10x/ess/issues/501 is the presence reasoning this needs; the witness
   lives in synthesis (`synthesize.rs` and `synthesize/subject_fact.rs`), which choose input values
   for `when_subject` rows. Inferred, to be confirmed by the reproduction.
5. **Second adopter.** An update command refusing `missing-etag` when the optional `if_match`
   input is absent, while `when_subject` selects the row by an input the same command carries.
6. **Cost.** Synthesis only; committed suites with such a guard gain scenarios. No format change.
7. **Alternatives.** (a) Change nothing: the branch stays unwitnessed. (b) This story.

## Decisions

- Accept. Starts after https://github.com/beyond10x/ess/issues/501 is on `main` (it shares
  `mutate.rs`); ships in the minor after 0.58.0's current order.

## Acceptance

- A minimal reproduction under `.engineering/repro/` (brand-free domain) shows `ESS-SYNTH-003`
  on the release this starts from, written first.
- After the change, synthesis writes a scenario that sends the optional input absent and observes
  the guarded refusal; `ess verify conform mutate` kills the branch's `guard` mutants.

## Scope

- `crates/verify/ess-conformance/src/synthesize.rs`, `src/synthesize/subject_fact.rs` (inferred).
- `crates/verify/ess-conformance/tests/` (new test).
- Held files: none at the time of writing.
