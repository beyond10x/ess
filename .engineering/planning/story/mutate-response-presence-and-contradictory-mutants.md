---
format: aep.planning-md/3
id: story:mutate-response-presence-and-contradictory-mutants
kind: story
status: draft
title: mutate gains a response-presence class and scores contradictory mutants equivalent
tags:
- adopter-report
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-verdicts
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/direct_response.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_unkillable.rs
revision: 5
---
## Outcome

`ess verify conform mutate` gains a mutant class that drops or adds a response member's presence, so
an audit says whether the suite pins what a command returns. (The second half of the original
request, scoring a mutant whose guard can never hold as equivalent, already holds; see Decisions.)

## Problem

An adopter on ess 0.56.0 asked for both. On 0.92.0, `MutantClass`
(`crates/verify/ess-conformance/src/mutate.rs`) has fourteen classes and none touches a command's
`response:`; `--class` lists from-drop, transition-to, guard-boundary, sets-retarget, guard-negate,
guard-connective, error-swap, emit-drop, order-flip, sets-drop, precedence-swap, emit-swap,
grant-drop, grant-add.

Minimal reproduction (ess 0.92.0), a command returning a required member:

```yaml
format: ess/18
system: demo
version: v1
domain: demo.gate
types:
  - {name: demo.gate.GateId, kind: newtype, of: Uuid}
events:
  - {name: demo.gate.Passed, fields: [{name: gate_id, type: demo.gate.GateId}]}
commands:
  - name: demo.gate.Open
    input: [{name: gate_id, type: demo.gate.GateId}]
    response: [{name: ticket, type: String}]
    outcomes:
      - name: opened
        returns: true
        emits: [demo.gate.Passed]
        payload: {demo.gate.Passed: {gate_id: input.gate_id}}
```

`ess verify conform mutate --path model.yaml --target interpreted` derives emit-drop and emit-swap
sites for `Open/opened` and nothing for `ticket`, although the synthesized `Open/outcome/opened`
carries `expect_direct_response` with `fields: [{name: ticket, type: String}]`.

## Acceptance

- `ess verify conform mutate --help` lists a `response-presence` class (name to be fixed by the
  design), and `--class response-presence` over the model above derives a site on
  `demo.gate.Open/opened` member `ticket`.
- Its "drop" mutant (the member undeclared, or made `Optional`) is run against
  `--target interpreted` and scored `killed` by `demo.gate.Open/outcome/opened`, or, where no
  scenario can tell it apart, `equivalent`/`unwitnessed` with the reason in the text.
- Where a command declares an `Optional` member, the "add" mutant (the member made required) is
  derived and scored the same way.
- `--emit` writes the new class in the manifest and `--collect` scores it; the manifest/report
  format version moves only if the class name is persisted where an earlier reader refuses it.

## Fit review

1. Need: a mutation audit cannot say whether a suite pins the presence of what a command returns;
   the direct-response check exists (`expect_direct_response`, `src/direct_response.rs`) but no
   mutant exercises it. Requester's proposal: "a mutant class that drops or adds a response field's
   presence".
2. Class: gap. `mutate.rs` `MutantClass` has no response class (run above).
3. Expressible today: no; authored scenarios cannot kill a mutant that does not exist.
4. Fit: extends the closed `MutantClass` set the way `emit-drop`/`emit-swap` did; sites follow the
   `response:` members of `returns: true` outcomes; the interpreted, billing and oracle targets and
   `--emit/--collect` all score it.
5. Second adopter: any command returning a minted token, receipt or page cursor.
6. Cost: one class, its site walk, a manifest/report entry; possibly a manifest format version.
7. Considered: change nothing (presence stays unaudited); fold into `emit-drop` (wrong surface,
   events and responses differ). A new class wins.

## Decisions

- Response-presence class: accept, redesigned as a "drop"/"add" pair on declared `response:`
  members of `returns: true` outcomes, scored by the existing verdicts.
- Contradictory mutants: already delivered, removed from this story. A guard mutant that leaves its
  outcome's guard satisfiable by no input is scored `equivalent` (`ESS-MUTATE-005`) naming the guard
  when no scenario kills it (`Verdict::with_dead_guard`, `unsatisfiable_guard` in `mutate.rs`;
  commit 433b66d362; CHANGELOG 0.57.0 and 0.76.0; test
  `tests/mutation_unkillable.rs::a_mutant_whose_guard_no_input_satisfies_is_equivalent_and_names_the_guard`).
  A killed dead-guard mutant stays killed, which is right: the outcome it removed is observable.
  Orderings, text matches and quantifiers are not decided by design (`satisfiable`'s doc comment);
  extending that is a separate story if a case is reported.
- Adjacent, implemented: `story:mutate-covers-subject-related-guards-sets-and-authored`
  (beyond10x/ess#515).

## Scope

- cited: crates/verify/ess-conformance/src/mutate.rs
- inferred: crates/verify/ess-conformance/src/direct_response.rs
- inferred: crates/edge/ess-cli/src/main.rs
- inferred: crates/verify/ess-conformance/tests/mutation_unkillable.rs

confidence: medium
