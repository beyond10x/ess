---
format: aep.planning-md/3
id: story:a-forced-external-guard-relating-an-input-list-to-a-stored-list-is-witnessed
kind: story
status: implemented
title: A narrowed external branch is witnessed on a row and input its subject guard holds for
summary: An external branch whose when_subject relates an input list to a stored list gets no outcome scenario, and every scenario through its state is refused ESS-SYNTH-004 (beyond10x/ess#727)
tags:
- from-consumer-report
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#727
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/input.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_t1_lane3_narrowed_input_relation.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_t1_lane3_pass2_near_miss.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/external_narrowed_by_input_relation.rs
revision: 14
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:33:51Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:33:52Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "active", to: "implemented", at: "2026-11-10T17:53:40Z", actor: "human:timo", revision: 14, decided_on: {"recorded":{"test_result":1,"review_outcome":3}}}
---
## Need

An `external:` branch narrowed by `when_subject: {predicate: P}` (`ess/29`) is, by design, witnessed
forced on a row and input for which `P` holds. Where `P` compares the stored row with the command's
input, synthesis does not do that. Two observable failures, one cause family, both on ess 0.92.0
(`ess/35`):

**1. Refused, and everything behind it refused too (the reported case).** The narrowed branch has
its own `when:` and a sibling reads the stored row. No candidate input carries the stored list's
element, so the outcome is `ESS-SYNTH-003` and every scenario routed through its target state is
`ESS-SYNTH-004`.

```yaml
- name: example.rollout.DecideAdmission
  input:
  - {name: id, type: String}
  - {name: missing, type: List<String>}
  - {name: bindings, type: List<String>}
  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when:
      all:
        - missing.count >= 1
        - forall: {in: missing, as: m, that: {not: {exists: {in: bindings, as: b, that: b == m}}}}
    when_subject:
      predicate:
        forall: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}
    moves: example.rollout.Rollout.refuse      # Pending -> Refused
    instance: id
  - name: admitted
    when: missing.count == 0
    when_subject:
      predicate:
        forall: {in: requires, as: c, that: {exists: {in: input.bindings, as: b, that: b == c}}}
    moves: example.rollout.Rollout.admit
    instance: id
  - name: incomplete
    error: example.rollout.AdmissionIncomplete
```

`requires: List<String>` is a stored field of `example.rollout.Rollout`, set at creation;
`StopRollout` moves `Refused -> Stopped`.

```text
$ ess verify conform synthesize --path spec-b --out suite-b.json
refused: refusal[ESS-SYNTH-003]: outcome example.rollout.DecideAdmission/refused has no scenario `example.rollout.DecideAdmission/outcome/refused`
  no candidate of the 60 tried satisfies ``refused` forced on a `example.rollout.Rollout` row it is taken on, sent an input no branch the stored row selects claims: none of `
refused: refusal[ESS-SYNTH-004]: outcome example.rollout.StopRollout/stopped has no scenario `example.rollout.StopRollout/outcome/stopped`
  it needs an instance of `example.rollout.Rollout` to move along `stop`, and the route runs through `example.rollout.DecideAdmission/refused`, which no input reaches
... (ESS-SYNTH-003 for transition refuse, ESS-SYNTH-004 for transition stop and three per-state refusals)
7 scenario(s) (0 authored), 10 refusal(s), written to suite-b.json
```

A witness exists: a row holding `requires: [requires.0]`, sent `missing: [requires.0]`,
`bindings: []`. The same search already builds `bindings: [.., requires.0]` from the stored list
for `admitted`.

The same refusal appears with no own `when:` when the guard needs a non-empty list
(`exists: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}`).

**2. A scenario the guard forbids is written (found while reproducing).** Where the narrowed branch
is the only branch of its command that reads the stored row, the scenario is emitted with the plain
witness and `P` is never checked. A correct target fails it.

```yaml
  outcomes:
  - name: refused
    external: at least one required capability has no implementation
    when: missing.count >= 1
    when_subject:
      predicate:
        forall: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}
    moves: example.rollout.Rollout.refuse
    instance: id
  - name: admitted
    moves: example.rollout.Rollout.admit
    instance: id
```

```text
$ ess verify conform synthesize --path spec --out suite.json
13 scenario(s) (0 authored), 6 refusal(s), written to suite.json
# DecideAdmission/outcome/refused: CreateRollout(requires = ["requires.0"]), then forced, DecideAdmission(missing = ["missing.0"])
$ ess verify conform run --target interpreted --path spec --suite suite.json --report-format 2
  failed example.rollout.DecideAdmission/outcome/refused
  ...
  13 scenarios: 6 passed, 7 failed, 0 error, 0 unsupported
      input: example.rollout.DecideAdmission(id = "id", missing = ["missing.0"])
      expected: - outcome = refused
      observed: - outcome = admitted
```

The scalar form `when_subject: {predicate: requires == input.missing}` (both `String`) writes the
same wrong scenario (`requires = "requires"`, `missing = "missing"`), so this is not about lists.

Cause, read in the code:

- `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:2937-2971` (`external_inputs`): with
  an own `when:` it grounds only the siblings' stored guards (`grounded(.., &hints(command))`, line
  2947). It neither grounds the branch's own narrowing predicate nor grounds a quantifier over the
  input (`grounded_over_input`, lines 1283-1295), both of which `inputs_for` does for the guarded
  branches (lines 1898-1899, `beside_another` 1930). With no own `when:` it falls to `linked_inputs`
  (line 2945), which grounds the siblings' guards only.
- `crates/verify/ess-conformance/src/synthesize.rs:5253-5260` (`unclaimed_external`): the row and
  input search `subject_fact::external_witness` runs only where `subject_fact::uses(command)`, and
  `uses` (`subject_fact.rs:50-57`) counts `SubjectField` and `SubjectPredicate` only, not
  `ExternalSubject`. A command whose only stored-row reader is the narrowed branch keeps the plain
  witness unchecked.

## Acceptance

Each is a test in `crates/verify/ess-conformance/tests/` on a brand-free fixture; each fails on
current main first.

1. For the first specification above, `synthesize` returns no `ESS-SYNTH-003` and no
   `ESS-SYNTH-004`; the scenario `example.rollout.DecideAdmission/outcome/refused` creates a row with
   a non-empty `requires` and sends a non-empty `missing` whose every element is an element of that
   row's `requires`; the interpreted target passes every scenario of the suite.
2. The same holds with the own `when:` removed and the guard written
   `exists: {in: input.missing, as: m, that: {exists: {in: requires, as: c, that: m == c}}}`.
3. For the second specification above, and for its scalar form
   `when_subject: {predicate: requires == input.missing}`, the emitted
   `DecideAdmission/outcome/refused` sends an input for which the guard holds on the arranged row,
   and the interpreted target passes every scenario of the suite (today 7 of 13 fail).
4. For a narrowed branch whose guard no row and input can satisfy
   (`when_subject: {predicate: {all: [requires == input.missing, requires != input.missing]}}`), the
   outcome is refused `ESS-SYNTH-003` and no scenario for it is written.
5. A target that ignores the narrowing (answers `refused` whenever forced) fails at least one
   scenario of the suites of 1 and 3, as `a_target_ignoring_the_narrowing_fails` does for its
   fixture.
6. The existing tests of `tests/external_narrowed_by_subject.rs` and
   `tests/external_beside_held_guard.rs` pass unchanged.

## Scope

Delivered in turn 1 of the zero-open-issues plan (lane story 3, row-set correction 2104be969b). The paths below are the ones
its commits changed, read from the diff, so every entry is cited.

- cited, confirmed: `crates/verify/ess-conformance/src/synthesize.rs`
- cited, confirmed: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs`
- cited, confirmed: `crates/verify/ess-conformance/tests/external_narrowed_by_input_relation.rs`
- cited, added: `crates/verify/ess-conformance/src/input.rs`
- cited, added: `crates/verify/ess-conformance/tests/adversary_t1_lane3_narrowed_input_relation.rs`
- cited, added: `crates/verify/ess-conformance/tests/adversary_t1_lane3_pass2_near_miss.rs`

Scoped but not touched:

- `crates/verify/ess-conformance/tests/fixtures/external-narrowed-by-input-relation.yaml`

Shared surface written by the coordinator, not a scope entry: `CHANGELOG.md`.

## Fit review

1. **The need, apart from the proposed syntax.** A behaviour ESS gets wrong: a provider-decided
   outcome that applies only to rows related to the input ("every capability reported missing is one
   the record requires") gets no scenario, or a scenario that contradicts its own guard. Minimal
   reproduction: the two specifications above, `repro/run.txt` variants A, B, D and E. The requester
   proposed no syntax; the request is "the witness for a forced external outcome grounds an input
   list from the stored list its guard compares it with, as the witness for an accepting branch
   does" (github:beyond10x/ess#727).
2. **Class: defect.** `docs/design/generic-language-features.md:549-551` says "Synthesis witnesses
   the branch forced on a row `P` selects", and lines 535-536 say the branch is taken only where "`P`
   holds of the addressed row". The emitted scenario of variant A has `P` false and the interpreter,
   which reads `P` before the provider (`crates/verify/ess-conformance/src/interpret/execute.rs:1249-1252`),
   answers `admitted`. `docs/design/guarded-external-outcomes.md:8-11`: "Synthesis satisfies its
   predicate ... An unsatisfiable guard produces a synthesis refusal."
3. **Can it already be expressed?** Yes, the specification validates (`ess specify validate --path
   spec-b` prints `valid`, `repro/run.txt`). The only way to get scenarios today is to delete the
   guard and state the rule outside the specification, which moves a checked figure out of it; by
   policy 1 that is no idiom (`.agents/skills/assessing-external-requests/SKILL.md`, question 3).
4. **Fit.** No grid cell is opened or refused: `when_subject:` already reads the input and stored
   fields (`docs/design/language-grids.md:24`, grid R row `when_subject:`, columns input and subject
   field both "yes"), and `forall`/`exists` over lists are admitted there
   (`website/docs/reference/predicates.md:726`). The change is in one consumer, synthesis, and makes
   the narrowed external branch use the grounding its sibling constructs already use
   (`subject_fact.rs:1898-1899` for `when_subject:` branches). General across siblings: the fix
   covers every row/input comparison in `P` (scalar, quantifier over the input, quantifier over the
   stored list), with and without an own `when:`, and commands with and without another stored-row
   reader. Other targets are unaffected: the interpreter already answers correctly (variant A
   output), generated Rust and Go "read the row, then `P`, then ask the context"
   (`generic-language-features.md:551-552`); no source, IR or suite construct changes.
5. **A second, unrelated adopter.** A payment refund declined by the processor only for line items
   of the stored order: `external: processor declined` beside
   `when_subject: {predicate: {forall: {in: input.lines, as: l, that: {exists: {in: order_lines, as: o, that: l == o}}}}}`.
6. **Cost.** No source format, no keyword, no new diagnostic, no diff classification, no generated
   API change. Suites of specifications that have this shape change: a refused or wrong scenario
   becomes a correct one, built from existing steps (`configure_external_outcome`,
   `execute_command`), so no suite format moves. A specification that today gets the wrong scenario
   of variant A and has an unsatisfiable guard gains an `ESS-SYNTH-003` (existing code).
7. **Considered.** (a) Change nothing: leaves a design statement false and writes scenarios a
   correct target fails. (b) Refuse a narrowed guard that reads the input at validation: narrows a
   published format (`ess/29` to `ess/35` admit it) and removes a checkable rule. (c) A fixture or
   seed key so the author supplies the witness: new authored surface for something the search can
   derive. (d) Chosen: ground the branch's own narrowing predicate in `external_inputs`, by
   `grounded` and `grounded_over_input` with the list widening `inputs_for` applies, on both the
   own-`when:` and the no-`when:` path, and send a narrowed branch through `external_witness`
   whatever its siblings read. It adds no surface and reuses the existing grounding. The requester's
   proposal is this design for the list case; it is widened to every row/input comparison and to the
   unchecked plain witness.

## Decisions

Accept, redesigned. The request (ground an input list from the stored list for a forced external
outcome) is taken and widened into one generic rule: a narrowed external branch is witnessed only on
a row and input its subject guard holds for, with the guard's row/input comparisons grounded on the
row as a `when_subject:` branch's are, and is refused `ESS-SYNTH-003` where none exists. What
changed from the request: it also covers scalar comparisons, the branch without an own `when:`, and
the command whose only stored-row reader is the narrowed branch, where today a scenario
contradicting the guard is written. No format version, no design page to write:
`docs/design/generic-language-features.md` § "An external branch narrowed by a subject guard
(`ess/29`)" already states the behaviour.

## Source

github:beyond10x/ess#727
