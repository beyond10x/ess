---
format: aep.planning-md/3
id: review-result:adversary-d357-u3-pass-1
kind: review-result
status: active
title: 'D357 U3 adversary pass 1: the no-view probe of an arranged refusal half'
relations:
- reviews: story:a-no-view-arranged-half-probes-the-row-through-the-command
revision: 1
---
unit: D357 U3 (story:a-no-view-arranged-half-probes-the-row-through-the-command), 1188481a0 on unit/ess-d357-u3, reviewed against 700b4faa6
verdict: NEEDS-CHANGE (1 warning, 1 note, 1 out of scope known); fixed at 1962a5ff0
cases: executed 220→224, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory u3/adv-scratch (cargo logs only)
needs-coordinator: none

**Cases.** `crates/verify/ess-conformance/tests/adversary_d357_u3_pass1.rs`: a no-view model with a `Staged` state where `configure` runs from `New` and `Staged`; two controls (the correct target passes; the story's own move-to-configured mutant is caught) and two red cases: a target that swaps the record between two states `Configure` runs from on refusal passed; a model whose first addressing branch can never be reached had both arranged halves withdrawn although a later branch has a valid send. Suite run beside the unit's targets: lib 172, pass1 14, pass2 14, external_beside_held_guard 17, both bytes tables green.

**Findings.**
- F1 (warning, introduced), `existence.rs:339` (also :440, `docs/design/outcome-shapes.md:284`, `CHANGELOG.md`): the probe sends only the addressing branch's valid input, so a move into another state that branch runs from passes, while the text claimed any change to the record is caught. Decision: narrow the claim, do not extend the probe. Fixed at 1962a5ff0: the four places claim a move out of every state the probing branch runs from, or a change in what it answers, and say a move between two such states is not seen. The red case is kept outside the repository for a later wave's story.
- F2 (note, introduced), `existence.rs:530`: only the first non-creating addressing branch was tried. Fixed at 1962a5ff0: every branch that addresses the arranged record and runs from its state is tried in declaration order; withdrawal names every branch tried. The red case is green and committed.

**Attacked, not broken.** `ExpectNoEvent` checks only the last command, so the probe's event does not trip it; models with an identity view skip the probe and keep their bytes; the withdrawal test checks code, scenario id, branch path and reason; the flipped pass-2 case names both scenario ids.

**Fix-round run (1962a5ff0).** lib 172, adversary_arrangement_pass1 14, adversary_arrangement_pass2 14, adversary_d357_u3_pass1 3, external_beside_held_guard 17, synthesis_suite_bytes_table 2, synthesis_suite_bytes_table_cli_read 1; clippy -D warnings and fmt-check clean. No second pass: the fix is the adversary's own red case turned green plus a narrowed claim.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":339,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the no-view probe sends only the addressing branch's valid input, so a target that moves the record on refusal into another state that branch also runs from passes, while the doc comment, :440, outcome-shapes.md:284 and CHANGELOG claimed any change is caught; claim narrowed at 1962a5ff0"},
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":530,"category":"boundary","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"probe tried only the first non-creating addressing branch, so a dead first branch withdrew both arranged halves although a later branch has a valid send; fixed at 1962a5ff0"},
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":613,"category":"judgement","severity":"note","verdict":null,"origin":"undecided","message":"out of scope - the same no-view fallback in held_state_refusals"}
]
```
