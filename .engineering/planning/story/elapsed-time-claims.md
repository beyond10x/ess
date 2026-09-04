---
format: aep.planning-md/1
id: story:elapsed-time-claims
kind: story
status: implemented
title: An authored scenario can claim a length of time, and a target has to answer for it
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/fixtures/go-billing/target.go
- confidence: cited
  path: crates/verify/ess-conformance/src/authored.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/report.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/scenario.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
revision: 5
---
## What is missing

`ess-scenario/1` can say what happened and in what order. It cannot say how long anything took.

The timeline's `at:` orders the file and reaches no runner, which the format's own commit message
records as deliberate. The consequence is that a scenario cannot claim *this must happen within N*,
*this must not happen for N*, or *this expires after N* — and for a call-distribution system several
of the most valuable claims are exactly those.

A migration of forty-five ACD scenarios into the format reported it as its one class of loss, and
enumerated the losses:

| duration | scenario | what is lost |
|---|---|---|
| 20 s hold before a blind transfer | `acd-requeue-after-hold-never-dialed` | the experiment: ~20 passes of a 1 Hz trigger sweep against a bridged call is DEV-741's independent variable, and it is the only thing the scenario adds to one that already passes |
| `WithWrapUp(5)`, `WithWrapUp(20)` | `acd-wrapup`, `acd-wrapup-no-reoffer` | the window; its ends' transitions are asserted and the window is not |
| `ExitWhen("call.duration", Gt, "1")` | `acd-queue-exit-on-duration` | the threshold that causes the exit |
| `CallbackTTL(1*time.Minute)` | `callback-expires-no-agent` | the TTL; the expiry is asserted and the minute is not |
| 5 s transfer-teardown margin | `callback-transfer-to-queue` | a margin that has to outlive a real backend timer |

A second translation hit the same wall from the other side and refused to fake it: no authored
assertion form can say a producer *halted*, only that a prefix was read. That one is out of scope
here and is a different shape.

## Why this is not a runner setting

A duration is not a deadline. A deadline is *when to give up*, which is a property of the machine a
suite runs on, and §37 puts every one of those on the runner's side for good reason. A length of
time a specification's own timer waits is a property of the system, and it reads the same on every
machine — so it belongs in the suite, and what belongs to the target is the *clock*.

## What a target has to be able to do

A duration claim has to mean something a target can check on a real system, without the format
inventing a clock it does not control. Three models were available and each is wrong as a
requirement:

| model | why not on its own |
|---|---|
| wall-clock time in the target | real, and it makes every suite slow and flaky |
| a logical clock the target advances | deterministic, and most targets have none to advance |
| an observation the target reports | honest, and it cannot make the twenty seconds happen |

The design asks for the third and permits the first two to produce it. The suite states a length and
the instant it is measured from; the target is asked to let the window close and to report a reading
it stands behind. A target that can do neither answers `Unsupported`, the scenario is reported
`unsupported`, and §28 makes the run fail — never passed. That last sentence is the whole feature: a
duration claim is the one construct where being ignored and being satisfied look identical from
outside.

## Acceptance

- An act names its instant with `mark:`; a later act states exactly one of `not_before:`, `within:`
  or `quiet: {for:, events:}` against it, and every window names its anchor.
- A window measured from an instant nothing marked is refused, as is one the file's own `at:`
  instants contradict.
- A refusal per new way a scenario can be wrong, each with a stable `ESS-AUTHOR-nnn` code and a test.
- The Rust runner and the emitted Go runner both execute the steps, and a target whose clock never
  moves fails exactly the scenario that claims a length of time.
- The forty-five scenarios in `sbf/specs/authored/` still compile with zero refusals.
