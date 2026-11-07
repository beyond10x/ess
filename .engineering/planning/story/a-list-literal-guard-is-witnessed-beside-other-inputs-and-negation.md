---
format: aep.planning-md/3
id: story:a-list-literal-guard-is-witnessed-beside-other-inputs-and-negation
kind: story
status: draft
title: A list-literal guard is witnessed across lists, beside other leaves and under negation
summary: 'Three U4 adversary cases: two lists, a refuting leaf beside the literals, not exists unchecked (0.90.0; from-finding)'
tags:
- backlog
- from-finding
- queue-0.91.0
- rank-1
relations:
- serves: vision:O2
revision: 2
---
## Need

Found by the U4 adversary of wave-2026-11-07-ess (ESS 0.90.0), beyond U4's acceptance. Origin is
undecided; reading the code, the new witness candidate does not reach these cases, which suggests
they predate U4. Synthesis emits no failing scenario in any of them. Each is either a refusal or a
mutant the suite does not catch:

1. Two literals in each of two lists of one command: the guarded branch is refused ESS-SYNTH-003,
   and `updated` gets four ESS-SYNTH-022 refusals. Each candidate fills only one list
   (`crates/verify/ess-conformance/src/witness.rs` ~1057).
2. Two literals beside a guard on another input whose base value refutes it (`level > 3`): refused
   ESS-SYNTH-003. The candidate keeps every other input at its base value.
3. `exists telephony` and `not exists voice`: no scenario sends `[telephony, voice]`, so a target
   that ignores the negated child passes every scenario. `member_literals` (~3000) skips literals
   under `not`.

The three cases are `adversary_u4_literals_across_two_lists_are_witnessed`,
`adversary_u4_literals_beside_another_leaf_are_witnessed` and
`adversary_u4_exists_and_not_exists_holds_one_excludes_other`. They were removed from
`adversary_u4_list_literals.rs` before merge, so no ignored test ships, and the lead keeps them
outside the repository. The story starts by restoring them.

## Acceptance

- The three cases are restored and pass.
- One witness candidate fills every list the guard names.
- Other inputs are solved together with the literals.
- A negated literal is sent in at least one scenario that the guard must refuse.

## Scope

- inferred: `crates/verify/ess-conformance/src/witness.rs`, `src/synthesize/related_list.rs`.
