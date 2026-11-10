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
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/direct_response.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_response_presence.rs
- confidence: inferred
  path: website/docs/reference/formats.md
revision: 9
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

Every bullet is a case in `cargo test -p ess-conformance --locked --test mutation_response_presence`
over the model above, unless it names another command:

- `ess verify conform mutate --help` lists `response-presence`, and `--class response-presence`
  over the model derives exactly one site, `demo.gate.Open/opened` member `ticket`.
- Its drop mutant removes `ticket` from `demo.gate.Open`'s `response:`; run against
  `--target interpreted`, it is scored `killed` by `demo.gate.Open/outcome/opened`.
- With `ticket` declared `Optional<String>`, the class derives the add mutant (`ticket` made
  `String`) and no drop mutant; run against `--target interpreted`, the add mutant is scored
  `killed` by `demo.gate.Open/outcome/opened`.
- `--emit` over the model writes a manifest whose `format` is `ess-mutation-manifest/8` and which
  lists the `response-presence` mutant; `--collect` over it writes `ess-mutation-report/9` scoring
  it `killed`. The same commands with `--class emit-drop` write the same manifest and report
  `format` values as 0.92.0 writes for them.

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
6. Cost: one class, its site walk, a manifest/report entry, and one manifest and one report format
   version (see Decisions).
7. Considered: change nothing (presence stays unaudited); fold into `emit-drop` (wrong surface,
   events and responses differ). A new class wins.

## Decisions

- Response-presence class: accept, redesigned as a "drop"/"add" pair on declared `response:`
  members of `returns: true` outcomes, scored by the existing verdicts.
- The class is named `response-presence`, after the surface it mutates, as `emit-drop` and
  `grant-drop` are.
- The drop mutant removes the member rather than making it `Optional`, because the interpreter
  answers an `Optional` member with its absent witness (`witness.rs:1154`), so only removal or a
  required member changes what the closed response schema admits.
- Both mutants are `killed` on the interpreted target: the drop mutant's suite closes the schema
  against the `ticket` the target returns, and the add mutant's suite requires the `ticket` the
  target leaves absent.
- A manifest or report holding a `response-presence` mutant is `ess-mutation-manifest/8` and
  `ess-mutation-report/9`, because the class name is persisted and an earlier reader refuses an
  unknown class, as the grant classes moved `/7` and `/8` (`MutantClass::manifest_format`,
  `report_format`); a document holding none keeps its format.
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

- cited: crates/verify/ess-conformance/src/mutate.rs (`MutantClass` :106, `manifest_format` :189,
  `report_format` :5887)
- cited: crates/edge/ess-cli/src/main.rs (the `--class` help text beside `grant-add`, :931)
- inferred: crates/verify/ess-conformance/src/direct_response.rs
- inferred: crates/verify/ess-conformance/tests/mutation_response_presence.rs (new)
- inferred: website/docs/reference/formats.md (the two new mutation document versions)

confidence: medium

Shared surfaces: proposes one `CHANGELOG.md` sentence; `website/docs/reference/spec-versions.md` is
regenerated by the coordinator.
