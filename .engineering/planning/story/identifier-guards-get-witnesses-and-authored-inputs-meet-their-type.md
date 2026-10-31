---
format: aep.planning-md/3
id: story:identifier-guards-get-witnesses-and-authored-inputs-meet-their-type
kind: story
status: active
title: Guards over constrained identifiers get witnesses, and authored inputs are held to their type
summary: Five synthesis and authoring gaps from one hosting-lifecycle domain (downstream gateway specification, 0.77.0); next wave.
tags:
- adopter-report
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: decision-blocker:an-out-of-type-command-input-may-have-a-declared-error
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-31T19:58:59Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-31T19:59:00Z", actor: "human:timo", revision: 3}
---
## Need

Reported by a downstream gateway specification session (2026-10-30, ESS 0.77.0 f692aab), from one
hosting-lifecycle domain whose commands refuse an identifier that is empty, holds a space, is over
256 bytes or holds a byte outside printable ASCII. Five findings, each worked around downstream:

1. An authored scenario's input is not held to its declared type: an identifier holding a space and
   an empty evidence text are admitted although the declared type excludes them.
2. A constrained input type gets no witness for a guard the type excludes: an input-guarded refusal
   over a constrained text type is refused `ESS-SYNTH-003`, so the type had to be loosened.
3. The guard grammar cannot say "every byte is printable ASCII"; the refusal for such a byte stays
   a target error.
4. An input-guarded refusal beside an exhaustive answer partition with no default branch gets no
   witness (`ESS-SYNTH-003`): synthesis asks the refusal's witness to refute every accepting
   guard although the refusal answers first (reported at `synthesize.rs:8567`, `admits_plain`).
5. `ESS-SYNTH-022` finds no input for 257-byte children of an `any:` guard; the reporter withdrew
   the refusals by authoring three scenarios that send a 257-byte value.

## Triage

Reported on 0.77.0, which is `main` (f692aab); not re-run in wave D325 (no standalone
reproduction; the reporter's specification carries the workarounds). Next wave: the fit review
rebuilds one brand-free reproduction per finding and designs them into generic features.

## Fit review (wave D335 U2, 2026-10-31)

Brand-free reproductions on 0.77.0 (one per finding) are in the unit's probes.

| | F1 authored input outside its type | F2 guard the input type excludes | F3 every character in a set | F4 refusal beside an exhaustive partition | F5 257-byte child of `any:` |
|---|---|---|---|---|---|
| reproduced | `conform author`: 0 refusals for `"a b"`, `""`, `"nolabel"` against alphabet, length and prefix types; the interpreted run passes | ESS-SYNTH-003, no candidate of 7 satisfies the guard | ESS-SPEC-012 for an `only` operator; the idiom gives 2 scenarios, 0 refusals | ESS-SYNTH-003 on the refusal `invalid-id` beside `answer == Accepted` / `== Rejected` with no default | ESS-SYNTH-022 with a printable-ASCII alphabet and two fields; no alphabet, one field, or an alphabet with wide characters each give 0 refusals |
| class | defect: generated decoders refuse such a value (`ess-synth/src/rust/invariant.rs:204-206`); an authored act's literal is only shape-checked (`authored.rs:4883`), `setup:` values are type-checked (`:3218`) | gap; today matches `predicates.md:1826` ("an input that still breaks an invariant is never sent") | convenience for an ASCII set; a real gap only for a non-ASCII set | defect: `synthesize.rs:8710-8724` refutes every sibling and held-state predicate (`decision.rs:29-35`), against refusal-first precedence | defect: `predicates.md:408` promises the witness; candidates outside the alphabet exhaust the 64-candidate cap before `admitted_inputs` (`witness.rs:2736`) drops them |
| expressible today | no | only by loosening the type and guarding | yes: `utf8_bytes != count` plus one `contains` per excluded byte | no | only by authored scenarios |
| cost | gate on `ess-scenario/6`; reuses ESS-AUTHOR-015 | behaviour change touches every target's decoding (blocker below); message only now | docs | none; suite bytes unchanged | none; suite bytes unchanged |

## Decisions

**accept F1, F4, F5; accept F2's message and defer its behaviour; decline F3 with the idiom**
(wave D335, the coordinator's call inside `approval-record:ess-d335-next-adopter-gaps`).

- F4 (defect, no format): a refusal guarded on the input refutes only the refusals that answer
  before it; the accepting branches and held-state predicates it answers before are not refuted.
  The search keeps today's rule first and falls back to the new one, so every witness found today
  is unchanged. ESS-SYNTH-003 stops saying "satisfies" where candidates failed by matching a
  sibling.
- F5 (defect, no format): the byte-length ladder builds a wide-led text only from a wide character
  the leaf's alphabet holds, else only the plain text.
- F1 (from `ess-scenario/6`): a literal input of an authored act that breaks its type's alphabet,
  prefix or invariant at any depth is ESS-AUTHOR-015 naming the field and the constraint. Below
  `/6` nothing changes.
- F2: when every candidate meeting a guard was dropped for breaking the input's type, ESS-SYNTH-003
  names the type, its first broken constraint and the rule that an out-of-type input is never sent.
  Whether a command may declare the error it answers for an out-of-type input is
  `decision-blocker:an-out-of-type-command-input-may-have-a-declared-error`.
- F3: declined; `predicates.md` gains a checked example of the idiom (`key.utf8_bytes != key.count`
  plus the `contains` children). A set holding non-ASCII characters goes to the same blocker.
- No `ess/34`.

## Acceptance

- F4: `crates/verify/ess-conformance/tests/held_state_input_refusal.rs`,
  `an_input_refusal_beside_an_exhaustive_held_state_partition_is_witnessed`.
- F5: `crates/verify/ess-conformance/tests/expression_utf8_bytes.rs`,
  `a_byte_length_child_is_witnessed_under_an_alphabet_without_wide_characters`.
- F1: `crates/verify/ess-conformance/tests/authored.rs`,
  `an_authored_input_outside_its_declared_type_is_refused_from_scenario_6` and
  `below_scenario_6_an_out_of_type_input_still_compiles`; `crates/edge/ess-cli/tests/authored_scenarios.rs`,
  `an_out_of_type_authored_input_is_refused_by_author`.
- F2: `crates/verify/ess-conformance/tests/string_alphabet_and_length.rs`,
  `a_refusal_guard_its_input_type_excludes_names_the_type`.
- F3: the new block runs in `crates/edge/ess-cli/tests/predicate_reference_page.rs`.
- `synthesis_suite_bytes_table` does not move for F4 and F5.
