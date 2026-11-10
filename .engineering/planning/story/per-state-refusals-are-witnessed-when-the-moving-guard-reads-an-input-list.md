---
format: aep.planning-md/3
id: story:per-state-refusals-are-witnessed-when-the-moving-guard-reads-an-input-list
kind: story
status: active
title: Per-state refusals are witnessed when the moving branch's guard reads an input list
summary: wrong_state witnesses are refused ESS-SYNTH-003 when the moving branch's when_subject reads an input list, though the outcome scenario solves it (beyond10x/ess#726)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#726
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/wrong_state_own_guard_over_input_list.rs
- confidence: cited
  path: docs/design/cross-record-and-stored-field-guards.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:45:31Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:45:32Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

A wrong-state row keeps its `<entity>/state/<S>/refuses/<command>` scenario only where synthesis finds
an input that selects the command's move on that row, so that the wrong state is the only reason the
command refuses (`docs/design/cross-record-and-stored-field-guards.md`, "Wrong state" and
"Conformance", lines 308-489). When the moving branch's `when_subject:` guard holds a quantifier over a
stored list whose body quantifies over an input list, synthesis never offers such an input: the
per-state scenario is refused ESS-SYNTH-003, although the same guard is solved for the branch's own
outcome scenario.

Minimal reproduction (format `ess/34`, one file):

```yaml
entities:
  - name: example.order.Order
    identity: {name: order_id, type: Uuid}
    fields:
      - {name: requires, type: List<String>}
    lifecycle:
      initial: Pending
      states: [Pending, Admitted]
      terminal: [Admitted]
      transitions:
        - {name: admit, from: [Pending], to: Admitted}
commands:
  - name: example.order.Place          # creates the order, sets requires: input.requires
  - name: example.order.Admit
    input:
      - {name: order_id, type: Uuid}
      - {name: granted, type: List<String>}
    outcomes:
      - name: admitted
        when_subject:
          predicate:
            forall:
              in: requires
              as: c
              that: {exists: {in: input.granted, as: g, that: g == c}}
        moves: example.order.Order.admit
        instance: order_id
      - {name: wrong-state, wrong_state: true, error: example.order.OrderStateConflict}
      - {name: not-covered, error: example.order.DecisionInvalid}
views:
  - {name: example.order.Orders, source: example.order.Order, consistency: read_your_writes, ...}
```

```console
$ ess verify conform synthesize --path c --out c.json
refused: refusal[ESS-SYNTH-003]: entity example.order.Order has no scenario `example.order.Order/state/Admitted/refuses/example.order.Admit`
  no candidate of the 1 tried satisfies `a `example.order.Order` row in this state holding forall c in requires: (exists g in input.granted: (g == c))`
```

The outcome scenario `example.order.Admit/outcome/admitted` in the same run is witnessed with
`granted: [granted.0-132, requires.0, requires.0-132, requires.0-133]` against a row holding
`requires: [requires.0, requires.0-132, requires.0-133]`.

Cause, from the code: the per-state input search (`refusal_input_from`,
`crates/verify/ess-conformance/src/synthesize/subject_fact.rs:3214`) grounds on the row, and retries
over input quantifiers (`grounded_over_input`, line 3412), only the guards the row must refute
(`halves` and `windowed`, lines 3256 and 3409). The moving branch's own stored guard, which the row
must hold (`own_stored`, line 3305), is checked against each candidate but never handed to the
search, so no candidate is drawn from the row's `requires` values. Plain grounding (`ground_leaf`,
line 1385) grounds a quantifier over the input only where `free` is given, so the inner
`exists g in input.granted` grounds to nothing. The outcome search (`inputs_for`, line 1892) grounds
every guard of the command both ways, which is why it succeeds.

The issue's own reduction (`input.missing.count == 0` alone) does not reproduce at 0.92.0 or 0.86.0;
the refusal needs the quantifier over the stored list.

## Acceptance

- `ess verify conform synthesize` on the reproduction above writes
  `example.order.Order/state/Admitted/refuses/example.order.Admit` and reports no ESS-SYNTH-003 for it;
  its send to `Admit` carries a `granted` list holding every element the arranged row's `requires`
  holds, and expects `example.order.OrderStateConflict`.
- With a second move `refuse` from `Pending`, a sibling refusal `when: not distinct {in: bindings, …}`
  and a sibling `when: missing.count >= 1` beside the guard
  `input.missing.count == 0 and forall c in requires: (exists b in input.bindings: b.capability == c) and forall b in input.bindings: (exists c in requires: b.capability == c)`,
  each wrong state (`Admitted`, `Refused`) keeps its `state/<S>/refuses/<command>` scenario.
- `ess verify conform run --target interpreted` passes every scenario of both suites.
- A suite whose per-state scenarios were already witnessed keeps its bytes: the existing
  `crates/verify/ess-conformance` suite-digest tests pass unchanged.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- inferred: crates/verify/ess-conformance/tests/wrong_state_own_guard_over_input_list.rs
- cited: docs/design/cross-record-and-stored-field-guards.md
- inferred: CHANGELOG.md
confidence: high

## Fit review

1. **Need.** Synthesis gives up a per-state wrong-state scenario that the design obliges, for a moving
   guard that compares a stored list with an input list; the reproduction above is minimal and
   brand-free. The requester proposed no syntax: they ask that the per-state witness solve the guard
   as the outcome witness does.
2. **Class.** Defect. The design page says each wrong state keeps its scenario and "no state is
   dropped" (`docs/design/cross-record-and-stored-field-guards.md:483-489`); ESS-SYNTH-003 is for a
   guard no candidate can carry (`website/docs/reference/diagnostics.md:196`), and the outcome
   scenario in the same run shows a candidate exists (`repro/run.txt`).
3. **Already expressible?** Not without weakening the specification: rewriting the guard without the
   nested quantifier changes the domain fact, and an authored `ess-scenario/1` per state moves a
   check synthesis owes into hand-written files. No idiom keeps the checked figure.
4. **Design fit.** No authored surface changes. Grid R: a stored list read inside a quantifier over
   an input list is already readable in `when_subject:` (the reproduction validates). The fix reuses
   the existing over-input grounding (`grounded_over_input`, `subject_fact.rs:1283`, beyond10x/ess#516)
   for the guard the row must hold, as it is already used for guards the row must refute; it applies
   to every `when_subject:` move, sibling constructs included. No target, generator or diff class
   changes; only suites that refused gain scenarios.
5. **Second adopter.** `example.access.Grant` stores `scopes: List<String>`; `Approve` moves it from
   `Requested` to `Approved` when `forall s in scopes: exists a in input.approved: a == s`. Its
   `Approved` and `Revoked` rows lose their per-state refusals the same way.
6. **Cost.** No format version, keyword or diagnostic. Suites that previously refused these
   scenarios gain them (as for beyond10x/ess#278 and #516 in `CHANGELOG.md`); every other suite keeps
   its bytes because the extra grounding runs only on the retry after the first search failed.
7. **Alternatives.** (a) Change nothing: the adopter loses one wrong-state check per state and must
   author them. (b) Reuse the outcome witness's input on the per-state row: it was solved for the
   row the outcome arranged, not for the per-state row, whose values differ. (c) Chosen: hand the
   moving branch's own stored guard to the per-state search's grounding beside the guards the row
   refutes; it derives the input from the row actually arranged, inside the existing retry.

## Decisions

accept as proposed: the requested behaviour, the per-state witness solving the moving branch's
guard as the outcome witness does, is a defect fix with no authored surface and no format bump. The
story records the actual cause (the moving branch's own stored guard is not grounded over the input
in the per-state search), not the issue's `count == 0` reduction, which does not reproduce.

## Source

github:beyond10x/ess#726
