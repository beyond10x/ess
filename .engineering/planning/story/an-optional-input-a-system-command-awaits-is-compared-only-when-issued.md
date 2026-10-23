---
format: aep.planning-md/3
id: story:an-optional-input-a-system-command-awaits-is-compared-only-when-issued
kind: story
status: active
title: 'An Optional input copied by sets: is compared with the awaited answer only when the target issued it'
summary: Conformance errors 12 scenarios when an awaited command's Optional input is absent (adopter item 1, beyond10x/a2a); defect, 0.69.0.
tags:
- adopter-report
- defect
- ess-0.70.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/await_outcome.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/system_issued_interpreter.rs
revision: 10
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T01:33:11Z", actor: "human:timo", revision: 9, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T01:33:11Z", actor: "human:timo", revision: 10, correlation: "ess-0690"}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (item 1 of 9, 2026-10-22). A
system-issued command awaits an agent command whose Optional struct input is copied into the
target row by `sets:`. Every scenario that drives that await errors in the interpreted run:

```text
error a2a.tasks.CompleteTask/outcome/moved
    ESS-CF-SUITE: `status_message`: the awaited answer carried no `status_message`
```

12 of the run's 14 errors are this one (CompleteTask, FailTask, RejectTask, RequireAuth,
RequireInput and siblings, `outcome/moved`). The run: 120 scenarios, 106 passed, 14 error, exit 3.

Minimal reproduction (brand-free): an entity `Job` with `status` and `note: Optional<Note>`; an
agent command `Finish(job_id, note: Optional<Note>)` that `sets: {note: input.note}`; a system
command that awaits `Finish`. Synthesize and run against the interpreted target.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize/system_issued.rs:337-345`: an input field that is
  not a path becomes `Source::Awaited(field)` without checking whether the field is Optional.
- `crates/verify/ess-conformance/src/interpret/await_outcome.rs:348-350`: the interpreter issues
  the awaited command with the Optional input absent first.
- `crates/verify/ess-conformance/src/runner.rs:3335`: the runner message; there is no "missing
  means absent" fallback.

## 0.68.0

No. Rerun on 0.67.0 and 0.68.0 (the 0.68.0 tree's binary): the same 12 errors.
`git log 0.67.0..0.68.0` touches none of the three files.

## Cause

Verified by code read plus run. Synthesis expects the awaited command's input `status_message` in
the answer; the interpreter issues that command with the Optional absent; the runner compares
anyway and errors. The struct type of the field is irrelevant: any Optional input copied by
`sets:` hits it.

## Fit review

1. **Need.** A conformance suite must accept a target that issues an awaited command without an
   Optional input the specification allows it to omit. The adopter proposed no syntax.
2. **Class: defect.** The specification declares the input Optional, so an answer without it is a
   valid answer; the suite treats it as missing evidence (`system_issued.rs:337-345`).
3. **Already expressible?** No authored idiom avoids it short of making the input required, which
   changes the domain. Nothing to express: this is synthesis and runner behaviour.
4. **Fit.** No authored surface. The fix stays inside `Source::Awaited` handling: compare the
   field only when the target's answer carries it, and assert absence-implies-unchanged (or
   cleared, per the `sets:` semantics) when it does not. Applies to every target alike, since the
   comparison is in the runner.
5. **Second adopter.** A ticketing service whose `Resolve(ticket_id, comment: Optional<Text>)` is
   awaited by an escalation timer.
6. **Cost.** No format version, no new diagnostic. Possibly a suite-format note if the comparison
   rule is recorded in the suite (I don't know whether it is; the implementor checks
   `system_issued.rs` serialisation).
7. **Alternatives.** (a) Change nothing: every adopter with an Optional awaited input has an
   erroring suite. (b) Have the interpreter always supply the Optional: hides the absent branch,
   which is the more interesting one. (c) Chosen: compare only what was issued.

## Decisions

**accept.** Compare an Optional input copied by `sets:` with the awaited answer only when the
target issued it; when absent, check the row against the absent-input semantics instead.

2026-10-22: the comparison rule is not recorded in the suite. No suite-format version, no new
scenario step, no change to the serialized `Source::Awaited`; the rule lives in synthesis
(`synthesize/system_issued.rs:337-345`) and the runner (`runner.rs:3325-3335`). This story and
`story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant` are one unit (both land on
`runner.rs`) and land before `story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain`,
which depends on them.

## Acceptance

Fixture: the minimal reproduction above (`Job`, `Finish(job_id, note: Optional<Note>)` with
`sets: {note: input.note}`, a system command awaiting `Finish`), written by the test itself.

- **Absent input stops erroring.** Transition: the interpreted target issues `Finish` without
  `note`. Before: the `outcome/moved` scenario errors with `ESS-CF-SUITE: note: the awaited answer
  carried no note`. After: it passes, and the row's `note` is checked against the absent-input
  semantics of `sets:`. Test: `crates/verify/ess-conformance/tests/system_issued_interpreter.rs`,
  case `an_optional_awaited_input_left_out_is_not_compared`. Failing before: the case errors on
  the current tree with the line above.
- **An issued input is still compared.** Transition: a mutation target issues `Finish` with
  `note` and stores a different value. After: the scenario fails. Same file, case
  `an_optional_awaited_input_issued_with_a_wrong_value_fails`. Mutation check: removing the
  comparison for an issued Optional (comparing nothing) turns this case green, so the case is red
  against that mutant.
- **Suite bytes unchanged.** Transition: synthesis of a model with no Optional awaited input.
  After: the suite is byte-identical to the current tree's and its format version is unchanged;
  the existing pins in `tests/system_issued_synthesis.rs` and `tests/system_issued_interpreter.rs`
  keep their verdicts, or the change names each re-pinned case.

## Schedule

Release 0.70.0 (defect, wave A unit A2 with its sibling on `runner.rs`). The store has no
target-release field; the release tag on the artifact marks it.

## a2a wave 1

Relayed by the orchestrator on 2026-10-22 from the adopter session specifying A2A in
beyond10x/a2a (ESS 0.67.0, a2a main 3a18050):

- **M8, first half** (a2a wave-1 gaps list, line 42): the 12 known conformance errors "awaited
  answer carried no status_message" on agent moves. The same 12 errors as this story's Need. The
  other half of M8 (4 errors on grant/denied) is
  `story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant`.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22), read from code on 0.67.0, reproduced on 0.68.0.

- Count: 12 scenarios error in the interpreted run of the adopter's suite on 0.68.0 (168 scenarios: 152 passed, 0 failed, 16 errors, exit 3). The 12 read `ESS-CF-SUITE: \`status_message\`: the awaited answer carried no \`status_message\`` (CompleteTask, FailTask, RejectTask and siblings, `outcome/moved`). The other 4 errors are `story:a-refused-send-keeps-the-earlier-read-token-for-a-denied-grant`.
- Adopter file:line: `ess/domains/tasks.yaml:1189` (`status_message`, an `Optional` input the awaited command copies by `sets:`).
- ESS file:line, unchanged from the Scope above: `crates/verify/ess-conformance/src/synthesize/system_issued.rs:337-345`, `crates/verify/ess-conformance/src/interpret/await_outcome.rs:348-350`, `crates/verify/ess-conformance/src/runner.rs:3335` (cited from code; the run confirms the symptom, not the line).
