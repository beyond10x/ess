---
format: aep.planning-md/3
id: story:a-race-above-a-held-state-branch-is-witnessed
kind: story
status: active
title: 'A race declared above a held-state branch is witnessed with compensates: true, and a plain external below it is reported unreachable'
tags:
- adopter-report
refs:
- provider: github
  reference: beyond10x/ess#623
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- informed_by: story:external-branch-narrowed-by-subject-guard
- depends_on: story:an-identity-guard-beside-a-row-set-validates
- depends_on: story:enforced-reference-needs-no-missing-target-branch
scope:
- confidence: inferred
  path: crates/specify/ess-domain/src/command
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-21T22:08:00Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-21T22:08:01Z", actor: "human:timo", revision: 3}
---
## Outcome

A specification states "commit unless the head moved" and synthesis witnesses both answers, also
when the race refusal compensates; a plain `external:` declared below a held-state branch that
answers every row it could start from is reported unreachable instead of being filed as a witness
it is not.

Ask B of https://github.com/beyond10x/ess/issues/623 (downstream report, ess 0.36.0 → 0.66.0,
2026-10-21): a command `Commit` with a held-state branch `committed` (`when_subject_state:
Validated`) and an external refusal `stale` whose transition starts only from `Validated`; held
state answers first, so no input, fixture or seed reaches `stale`, and synthesis refuses 3
ESS-SYNTH-003 and 2 ESS-SYNTH-004 per component. The ask: a precedence between held-state and
external branches a specification can state.

The binding design answers it (`docs/design/generic-language-features.md` § G4): branches answer
in the order of what they read, held state (step 3) before external causes (step 5), and within a
step the first declared (lines 436-438). "Commit unless the head moved" needs no new key: the race
is `external:` beside `when_subject: {predicate: state == Validated}`, declared above `committed`;
both are step 3, so the race answers where the provider takes it, `committed` otherwise, and
"synthesis witnesses both with no `ESS-SYNTH-003`/`004`". "A plain `external:` declared below
`committed` stays unreachable, since step 3 answers every validated row first" (lines 506-512,
`ess/29`, shipped by `story:external-branch-narrowed-by-subject-guard`).

What is left, on the integration build (e44d2b75c, sizing pass 2, 2026-10-21):

- **The race with `compensates: true`** (the reported `stale` moves the row before answering):
  synthesis refuses ESS-SYNTH-003 and `Commit/stale` gets no scenario. The design's lines 506-512
  say nothing about `compensates:`. `story:external-branch-narrowed-by-subject-guard` records under
  "Not done": "a narrowed external refusal with `compensates: true` keeps the refusal every
  non-external compensation gets".
- **The plain shape** (`stale` an unnarrowed `external:` declared below `committed`): 0 refusals,
  and the suite carries a scenario `Commit/outcome/stale` whose expected answer is `committed`. A
  scenario filed under `stale` witnesses no `stale` answer, so the suite counts a branch the
  design calls unreachable as covered.

## Fit review

1. **Need.** An external race that overrides a held-state acceptance on the same rows, with the
   race's compensating move (the row leaves `Validated`).
2. **Class.** Gap for the compensating race; defect for the plain shape (a scenario that claims a
   branch it does not reach).
3. **Already expressible?** The non-compensating race is, from `ess/29` (design lines 506-512).
   The compensating race is not.
4. **Fit.** No new key and no new order: the requester's "declaration order decides" or `before:`
   is answered by the rule's step 3, where declaration order already decides.
5. **Second adopter.** Book a held slot unless the provider reports it taken meanwhile, releasing
   the hold.
6. **Cost.** Synthesis and, if validation refuses the compensating race today, a format gate at
   `ess/30`. The plain-shape change alters synthesized suites of specifications that declare an
   unreachable external branch: one scenario fewer, one refusal more.
7. **Alternatives.** (a) Change nothing. (b) `before:` on an external branch: a second ordering
   mechanism beside the rule. (c) The design's narrowed race, extended to `compensates: true`, and
   the plain shape reported as unreachable (chosen).

## Decisions

The design decides the plain case and is silent on `compensates:`. The readings below are taken
from it; each is the coordinator's call, taken (2026-10-22), and the implementor does not reopen
it.

- **`compensates: true` on the narrowed race is admitted and witnessed.** Reading: lines 506-512
  place the race by what it reads (the held row, step 3), and `compensates:` changes what a
  refusal does to its row, not what it reads or where it answers
  (`docs/design/refusal-with-effect.md`: a flag beside the outcome's other flags). So the design's
  "synthesis witnesses both with no `ESS-SYNTH-003`/`004`" holds with the flag set. The witness
  forces the provider on a `Validated` row and reads the row back in the compensating state
  (`story:every-witness-reads-back-its-outcome-effect`). If validation refuses the pair today
  (`refusal_mutated_state`), the admission is gated at `ess/30` and below it the refusal names
  `ess/30`; if validation admits it, no gate.
- **The plain shape is reported unreachable at every format.** Reading: lines 510-511, "stays
  unreachable". Synthesis files no scenario under `Commit/outcome/stale` that expects another
  outcome; it refuses `stale` as ESS-SYNTH-003, naming `committed` as the step-3 branch that
  answers every row `stale` could start from and the narrowed race declared above it as the
  repair. Today's misleading scenario is a defect at every format, so no format gate. Validation keeps admitting the plain shape: its declaration order agrees with the step
  order, which is all `story:declaration-order-agrees-with-every-step-pair` checks. A precedence
  check that forces the provider and expects `committed` may stay in the suite, filed under
  `committed`.
- No `before:` key and no declaration order across steps.

## Acceptance

Each line names the failing test written first, as a case in an existing test binary. Fixture: a
neutral revision ledger, Propose → Validate → Commit, `stale` "the head moved".

- `crates/verify/ess-conformance/tests/external_narrowed_by_subject.rs`:
  `a_compensating_race_above_a_held_state_branch_is_witnessed`: the race with `compensates: true`
  declared above `committed` synthesizes with no ESS-SYNTH-003/004, `Commit/stale` forces the
  provider on a `Validated` row and reads the row back in the compensating state, the suite passes
  against the interpreted target, and a target that answers `committed` there fails.
- `crates/specify/ess-domain/tests/external_narrowed_by_subject.rs`:
  `a_narrowed_race_may_compensate` (validates at the admitting format) and, only if validation
  refuses it today, `below_ess_30_a_compensating_narrowed_race_names_ess_30`.
- `crates/verify/ess-conformance/tests/external_beside_held_guard.rs`:
  `a_plain_external_below_a_held_state_branch_is_reported_unreachable`: no scenario under
  `Commit/outcome/stale` expects `committed`; the refusal is ESS-SYNTH-003 naming `committed` and
  the narrowed race.

## Scope

| unit | files | confidence |
|---|---|---|
| synthesis | `crates/verify/ess-conformance/src/synthesize/` (external-branch witnesses beside held state) | inferred |
| validation (if refused today) | `crates/specify/ess-domain/src/command/` refusal-with-effect checks | inferred |
| generation | `crates/generate/ess-synth/src/determined.rs` (a compensating refusal stays a named obligation, `refusal-with-effect.md:153`) | cited, unchanged |
| docs | `docs/design/generic-language-features.md` lines 506-512 gains the `compensates:` sentence; `website/docs/reference/spec-versions.md` if gated | inferred |

Crates: ess-conformance, possibly ess-domain (sizing: M).

## Order

Lane L4a, last: after `story:an-identity-guard-beside-a-row-set-validates` and
`story:enforced-reference-needs-no-missing-target-branch`.
