---
format: aep.planning-md/3
id: story:a-setup-runs-refusal-names-the-real-cause-where-its-branch-has-a-scenario-of-its-own
kind: story
status: draft
title: A setup run's refusal names the real cause where its branch has a scenario of its own
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/generate/ess-entity-runtime/tests/fixtures/lowered-definitions-table.tsv
- confidence: inferred
  path: crates/specify/ess-compiler/tests/fixtures/selection-precedence-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_271_pass2.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/existing_instance_beside_related_guards.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/a-witnessed-route-branch-refusal.yaml
- confidence: cited
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_multiple_vias.rs
revision: 10
---
## Outcome

- A refusal whose route runs through a branch that has a scenario of its own no longer says the branch is one "which no input reaches". It says that no setup run of that branch arranges the row this scenario needs, and why.
- A refusal for a row beside that found none states how many candidates it tried, instead of a constant 1.
- Refusals whose route branch really has no scenario keep their text.

## Need

Cut from story-1 (`a-creating-branch-that-reads-several-related-rows-arranges-its-row-for-other-scenarios-wherever-its-own-scenario-is-witnessed`, decision D2) when its unit was written. It is still open after that unit (`fd8a86bcf3` on `unit/w1122-s3`).

Models were run with the unit's binary (`wave-1122/s3/ess-s3`). Each model directory holds its `run.txt`.

- `second-related-row/m-rs1/model.yaml`: the generic fixture `row-set-refusal-beside-a-named-related-row.yaml` of the wave plus a `FinishTicket` that moves the created `Ticket`. `PinTicket/outcome/pinned` is in the suite (8 scenarios), yet three refusals say:

  ```text
  refused: refusal[ESS-SYNTH-004]: outcome shop.store.FinishTicket/finished has no scenario `shop.store.FinishTicket/outcome/finished`
    it needs an instance of `shop.store.Ticket` to move along `finish`, and the route runs through `shop.store.PinTicket/pinned`, which no input reaches
    help: the route to this state runs through a branch no input reaches; see that outcome's own refusal
  ```

  `PinTicket/pinned` has no refusal of its own to see. The real cause (story-4) is that the setup run finds no row on which `pinned` alone is selected.
- `second-related-row/m-g22/model.yaml`: "no candidate of the 1 tried satisfies `a row of `shop.desk.Grant` … beside the row the scenario is arranged around`" for `ReceiveTicket/received`. In fact no Grant carrying the chosen name could be created at all (story-2), and the 1 is a constant.
- An adopter's specification shows the same: 12 ESS-SYNTH-004 refusals with "which no input reaches" remain on the unit's binary, and the receiving branch is refused "of the 1 tried".

Where the text is made, at `fd8a86bcf3`:
- `synthesize.rs` `Unreachable::Unwitnessable` is displayed at line 3315-3319 ("the route runs through `{outcome}`, which no input reaches"). The help line is at 3248.
- The cause of a failed setup run is dropped by `.map_err(|_| Unreachable::Unwitnessable { .. })` at 8802 (`created_owned_made`), 9013 and 9078 (`invoke`). Also by 9031, 9053 and 9558. The comment at 9017-9019 gives the reason: "That branch has a refusal of its own, under its own id". That reason does not hold where the branch has a scenario instead.
- `related_guard.rs` `beside_refused` (line 1491) sets `tried: 1` at line 1504. Two more constants of the same shape are at 3220 and 3815.

## Fit review

- Origin: an adopter's finding. The refusal text misled them: "1 tried" and "no input reaches" for a branch with a scenario. Adopted, redesigned: the request was for the text to name the real predicate. This story does that by carrying the cause the run already has, not by adding a code.
- Considered and rejected:
  - Always carrying the cause. That repeats a branch's own refusal under every scenario routed through it, which the comment at 9017 rightly avoids.
  - A new refusal code. ESS-SYNTH-004 and ESS-SYNTH-003 keep their meaning.

## Design

- `Unreachable::Unwitnessable` gains the cause of the failed run (`Option<String>`, the `RefusalCause` display). Every `map_err(|_| ..)` listed above keeps it.
- After synthesis has decided every scenario, one deterministic pass rewrites the text of each such refusal whose route branch has a scenario in the suite. The new text is "the route runs through `<branch>`, whose own scenario is witnessed, and no run of it arranges this row: <cause>", and the help line says the same. Every other refusal keeps its bytes.
- `beside_refused` takes the number of candidates `beside_row` tried, counted where it searches. So do the two constants at 3220 and 3815, or they lose "of the N tried" (D2).
- Refusal text is not a persisted format: no new `ess/N`, no `ValidationCode`. The suite's `refusals=` digests move where the text moves.

## Acceptance

- `m-rs1` as a fixture: the refusal of `FinishTicket/outcome/finished` names `PinTicket/pinned` as witnessed and carries the setup run's cause. A test in `tests/related_guard_multiple_vias.rs` asserts the new text and the absence of "which no input reaches".
- `m-g22`: the refusal of `ReceiveTicket/received` states the real number of candidates tried, or no number (D2). Tested.
- `adversary_271_pass2.rs` line 512 (`"which no input reaches"`) still passes. Its route branch has no scenario of its own; if it does, the pin is re-pinned with the cause named beside it.
- The doc comment at line 16 of `tests/existing_instance_beside_related_guards.rs` is updated to the new wording.
- `synthesis-suite-bytes-table.tsv` moves only in the `refusals=` digest of rows whose refusal names a witnessed route branch or a row beside. The candidates are the five models that print "which no input reaches" at 0.97.4:
  - `crates/edge/ess-cli/tests/fixtures/parse-refusals/every-position.yaml`
  - `crates/verify/ess-conformance/tests/fixtures/calendar-windows.yaml`
  - `claim-search/pick-frozen.yaml`
  - `external-narrowed-by-state-unreachable.yaml`
  - `rows-moved-by-many-commands.yaml`

  Plus any fixture added by this wave that prints either text. Every moved row is named with its cause in the change.
- The byte pins of `AGENTS.md` (`synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `tutorial_page`) pass, re-pinned only where a named row moved.

## Decisions

- D1. Which refusals get the new text.
  - A (recommended): only those whose route branch has a scenario in the suite, by a pass after synthesis.
  - B: every one, with the cause appended.
- D2. The constant `tried: 1`.
  - A (recommended): count the candidates in `beside_row` and the two sibling sites.
  - B: drop "of the N tried" from these three texts.
- D3. Order against story-2 and story-4, which remove most of the cases this text describes.
  - A (recommended): after them, so the pins move once.
  - B: before them.
