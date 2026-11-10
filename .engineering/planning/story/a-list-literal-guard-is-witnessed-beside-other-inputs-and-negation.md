---
format: aep.planning-md/3
id: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation
kind: story
status: active
title: A list-literal guard is witnessed across lists, beside other leaves and under negation
summary: 'Three U4 adversary cases: two lists, a refuting leaf beside the literals, not exists unchecked (0.90.0; from-finding)'
tags:
- backlog
- defect
- from-finding
- queue-0.91.0
- rank-1
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/verify/ess-conformance/src/synthesize/related_list.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/witness.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_u4_list_literals.rs
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:38:20Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:38:21Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The U4 adversary of wave-2026-11-07-ess (ESS 0.90.0) found three cases beyond U4's acceptance.
Reading the code, the new witness candidate does not reach them, so they predate U4. Synthesis emits
no failing scenario in any of them; each is either a refusal or a mutant the suite does not catch.
All three are re-checked on 0.92.0 as variants of
`crates/verify/ess-conformance/tests/fixtures/list-identity-guard-beside-updates.yaml`, with the
guard of `UpdateRolePermissions/updated-with-telephony` replaced:

1. Two literals in each of two lists (input `extras: List<String>` added):

   ```yaml
   when:
     all:
       - exists: {in: input.permissions, as: e, that: 'e == "telephony"'}
       - exists: {in: input.permissions, as: e, that: 'e == "voice"'}
       - exists: {in: input.extras, as: e, that: 'e == "telephony"'}
       - exists: {in: input.extras, as: e, that: 'e == "voice"'}
   ```

   The guarded branch is refused ESS-SYNTH-003, and `updated` gets four ESS-SYNTH-022 refusals.
   Each `member_literals` candidate fills one list and leaves every other at its base
   (`crates/verify/ess-conformance/src/witness.rs:1052-1061`).
2. Two literals beside a guard on another input whose base value refutes it (input `level:
   Integer`, guard `all: [<telephony>, <voice>, level > 3]`): refused ESS-SYNTH-003. The candidate
   keeps every other input at its base value.
3. `all: [<telephony>, not: {exists: {in: input.permissions, as: e, that: 'e == "voice"'}}]`: no
   scenario supplies or sends `voice`, so a target that ignores the negated child passes every
   scenario. `updated` also gets ESS-SYNTH-022 for that child. `member_literals`
   (`witness.rs:2999`) walks only `all`/`any`/`exists`, so it skips literals under `not`.

The adversary's three tests (`adversary_u4_literals_across_two_lists_are_witnessed`,
`adversary_u4_literals_beside_another_leaf_are_witnessed`,
`adversary_u4_exists_and_not_exists_holds_one_excludes_other`) were removed before merge and are not
in the repository; the variants above replace them.

## Acceptance

In `crates/verify/ess-conformance/tests/adversary_u4_list_literals.rs`, built with its `guarded`
and `exists` helpers:

- `adversary_u4_literals_across_two_lists_are_witnessed`: `check` passes for variant 1 (no
  ESS-SYNTH-019, no failing scenario, `updated-with-telephony` and `updated` witnessed, no unbacked
  literal) and synthesis records no ESS-SYNTH-003 or ESS-SYNTH-022 for the command.
- `adversary_u4_literals_beside_another_leaf_are_witnessed`: `check` passes for variant 2; the
  guarded scenario sends both literals and `level > 3` together.
- `adversary_u4_exists_and_not_exists_holds_one_excludes_other`: for variant 3, some scenario
  sends `permissions` holding both `telephony` and `voice` and expects `updated`, and
  `mutant_caught` holds against the fixture's original guard (the negated child dropped).
- All three are red on 0.92.0; every other suite keeps its bytes.

## Scope

- cited: crates/verify/ess-conformance/src/witness.rs
- inferred: crates/verify/ess-conformance/src/synthesize/related_list.rs
- cited: crates/verify/ess-conformance/tests/adversary_u4_list_literals.rs

confidence: high
