---
format: aep.planning-md/3
id: story:validate-refuses-a-branch-implied-by-an-earlier-input-refusal
kind: story
status: active
title: Validate refuses a branch implied by an earlier input-guarded refusal
summary: A when branch whose guard implies an earlier input-guarded refusal guard in the same step is UnreachableBranch at validate (ess#702)
refs:
- provider: github
  reference: beyond10x/ess#702
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-29T07:28:23Z", actor: "human:timo", revision: 2, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
- {from: "proposed", to: "active", at: "2026-10-29T07:28:23Z", actor: "human:timo", revision: 3, executor: "agent:ess-issues-lead", correlation: "ess-issues-2"}
---
## Problem

`ess specify validate` accepts a `when:` branch whose guard implies the guard of an input-guarded
refusal the precedence plan answers before it in the same step. No input reaches the later branch.
Only `ess verify conform synthesize` names it (ESS-SYNTH-003, decided by trying candidate inputs in
`crates/verify/ess-conformance/src/synthesize.rs`, `reach` and `Shadow::record`), synthesize
exits 0, and `run` stays green with one scenario fewer. Reported by a downstream consumer
(beyond10x/ess#702), reproduced on 0.76.0 with a toy specification: `closed: route == X` declared
before `blocked: route == X and flag_a == false and flag_b == true`.

## Fit review

1. Need: validate refuses a branch no input can reach, as it already does for other unreachable
   branches (`UnreachableBranch`, `crates/specify/ess-domain/src/command.rs:4180`).
2. Class: gap in validation. `validate_step_order` (`command.rs:3898`) already asks the finite
   prover (`command/finite.rs:218`) whether two input guards can hold together; implication of an
   earlier input-guarded refusal's guard is not asked.
3. Already expressible: no. The specification is valid today and the dead branch ships.
4. Fit: no authored surface and no format. A new refusal under the existing `UnreachableBranch`
   code, with the SYNTH-003 wording.
5. Second case: any command with a broad refusal declared before a narrower one on the same input.
6. Cost: a specification valid today may be refused. D186: the adopter replay must show no adopter
   refused; if one is, this ships as a warning and the refusal waits.
7. Considered: a warning only (kept as the fallback of D186).

## Decisions

Accept as proposed. Within one precedence step, a `when:` branch whose input guard implies the
input guard of an input-guarded refusal declared before it (equal guards count) is refused as
`UnreachableBranch` at the later branch, naming both branches and both guards in the SYNTH-003
wording ("is answered first by `<earlier>` (<guard>), an input-guarded refusal declared before
it"; for an accepting later branch, "the input-guarded refusal taken first"). Implication is
decided by the finite prover over input guards only; where it cannot decide, validate stays silent
and synthesis keeps SYNTH-003. A later branch that is reachable for some input (the earlier guard
is narrower) is not refused.

## Acceptance

- A later refusal implied by an earlier input-guarded refusal is refused:
  `crates/specify/ess-domain/tests/implied_input_guard.rs`,
  `a_branch_implied_by_an_earlier_input_refusal_is_unreachable`.
- An accepting later branch gets the "taken first" wording: same file,
  `an_accepting_branch_implied_by_an_earlier_refusal_is_unreachable`.
- Equal guards count: same file, `an_equal_guard_declared_later_is_unreachable`.
- A narrower earlier guard leaves the later branch valid: same file,
  `a_broader_later_branch_stays_valid`.
- Guards the prover cannot decide are not refused: same file,
  `an_undecidable_implication_is_not_refused`.
- Run: `cargo test -p ess-domain --locked --test implied_input_guard`; the diagnostics pins and
  the conformance table tests stay green, and no committed example or corpus model is newly refused.
