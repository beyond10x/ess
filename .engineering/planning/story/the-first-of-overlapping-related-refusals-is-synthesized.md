---
format: aep.planning-md/3
id: story:the-first-of-overlapping-related-refusals-is-synthesized
kind: story
status: draft
title: The first of several overlapping related refusals over one row is synthesized
summary: Synthesis refuses reachable related refusals that share a guard with a later one (ESS-SYNTH-003), as in 0.68.0 (found by the U14 adversary, 2026-10-23).
tags:
- defect
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

Found while landing `story:an-authored-step-expecting-a-shadowed-related-refusal-is-refused`
(0.69.0, adversary pass 1, finding F3). A command declares refusals over one related row whose guards
overlap, for example `staff-closed` and `locker-closed` both reading `accepting == false` through
the same `via:`. The interpreter answers the first declared one, as the precedence design says
(`docs/design/cross-record-and-stored-field-guards.md:828-830`, `:868`). Synthesis refuses the
reachable first refusal as well as the shadowed one. Present since at least 0.68.0; still on 0.92.0.

## Problem

Reproduction: `crates/verify/ess-conformance/tests/fixtures/related-refusal-shadowed.yaml`, whose
command reads:

```yaml
- name: parcels.locker.DropParcel
  input:
    - {name: parcel_code, type: parcels.locker.ParcelCode}
    - {name: locker_id, type: parcels.locker.LockerId}
    - {name: recipient, type: String}
  outcomes:
    - {name: no-locker, when_related: {via: input.locker_id, exists: false}, error: parcels.locker.NoSuchLocker}
    - name: staff-closed
      when_related: {via: input.locker_id, predicate: accepting == false}
      when: recipient == staff
      error: parcels.locker.StaffLockerClosed
    - {name: locker-closed, when_related: {via: input.locker_id, predicate: accepting == false}, error: parcels.locker.LockerNotAccepting}
    - {name: locker-shut, when_related: {via: input.locker_id, predicate: accepting == false}, error: parcels.locker.LockerShut}
    - {name: locker-full, when_related: {via: input.locker_id, predicate: {all: [accepting == false, site == Depot]}}, error: parcels.locker.LockerFull}
    - {name: at-depot, when_related: {via: input.locker_id, predicate: site == Depot}, error: parcels.locker.LockerAtDepot}
    - name: dropped
      creates: parcels.locker.Parcel
      ...
```

`ess verify conform synthesize --path related-refusal-shadowed.yaml --out suite.yaml` on 0.92.0
(exit 0) refuses all four, the two reachable ones included:

```text
refused: refusal[ESS-SYNTH-003]: outcome parcels.locker.DropParcel/staff-closed has no scenario `parcels.locker.DropParcel/outcome/staff-closed`
  no candidate of the 4 tried satisfies ``parcels.locker.Locker` stored related row selecting this branch, over the rows 4 bounded arrangements left`
refused: refusal[ESS-SYNTH-003]: outcome parcels.locker.DropParcel/locker-closed has no scenario ...
refused: refusal[ESS-SYNTH-003]: outcome parcels.locker.DropParcel/locker-shut has no scenario ...
refused: refusal[ESS-SYNTH-003]: outcome parcels.locker.DropParcel/locker-full has no scenario ...
6 scenario(s) (0 authored), 4 refusal(s)
```

`staff-closed` is reached by a closed locker and `recipient: staff`, `locker-closed` by a closed
locker and any other recipient; an authored act reaching `locker-closed` passes on the interpreter
(`authored_outcome_guards.rs`). `locker-shut` and `locker-full` are shadowed by `locker-closed`.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/related_guard.rs:1867-1888` (`answering_refusal`):
  precedence is applied only to refusals the plan reads before the accepting phase; the accepting
  phase related refusals have no order among themselves, so the goal for the first one also asks its
  row to miss the later one's guard.
- Pinned: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`, row
  `related-refusal-shadowed.yaml` (`refusals=4`).
- Failing case:
  `adversary_u14_pass1::the_shadowed_reproduction_synthesizes_its_reachable_refusals`
  (`#[ignore]`, naming this story).

## Acceptance

- The reproduction synthesizes a scenario for every reachable refusal (`staff-closed`,
  `locker-closed`) that passes on the interpreted target, and refuses only the shadowed ones
  (`locker-shut`, `locker-full`), naming the branch that answers first.
- The ignored adversary case runs and passes.
- No existing pinned suite digest moves except the reproduction's own row (`refusals=2`).

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/related_guard.rs`
- cited: `crates/verify/ess-conformance/tests/adversary_u14_pass1.rs`
- cited: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv`

confidence: high

## Format

None.
