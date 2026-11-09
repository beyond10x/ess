---
format: aep.planning-md/3
id: story:a-stored-guard-beside-an-input-guard-on-a-system-step-synthesizes
kind: story
status: draft
title: A stored-field guard beside an input guard on a system step synthesizes
summary: 'ESS-SYNTH-024: a stored-field guard beside an input guard on a system-issued step is refused.'
tags:
- adopter-report
- defect
- ess-0.71.0
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2, from a downstream repository specification (which
quarantines 9 scenarios for gaps 1 and 2): on a step the system issues, a guard over a stored field
beside a guard over an input field is refused as `ESS-SYNTH-024` instead of synthesized.

Reproduced on 0.92.0 with a neutral model. A `system: true` actor holds `RunJob(job_id, mode)`;
`Job.paused` is a stored Boolean that `OpenJob` sets from its input:

```yaml
- name: RunJob
  input:
    - {name: job_id, type: JobId}
    - {name: mode, type: Mode}            # enum [fast, slow]
  outcomes:
    - name: held
      when_subject: {predicate: paused == true}
      when: mode == fast
      error: JobHeld
    - name: finished
      moves: Job.finish
      instance: job_id
      emits: [JobFinished]
      payload: {JobFinished: {job_id: input.job_id}}
    - {name: closed, wrong_state: true, error: JobClosed}
```

```text
$ ess verify conform synthesize --path model.yaml --out suite.json
refused: refusal[ESS-SYNTH-024]: outcome jobs.run.RunJob/held has no scenario `jobs.run.RunJob/outcome/held`
  `jobs.run.RunJob/held` refuses a step the system issues itself, and an input guard beside the stored-field guard decides it: ...
7 scenario(s) (0 authored), 1 refusal(s), written to suite.json
$ ess verify conform run --target interpreted --path model.yaml --suite suite.json --report-format 2
jobs v1 against interpreted 0.92.0 — failed
  failed jobs.run.Job/state/Done/refuses/jobs.run.RunJob
  failed jobs.run.Job/transition/finish/by/jobs.run.RunJob/finished
  failed jobs.run.RunJob/outcome/finished
  7 scenarios: 4 passed, 3 failed, 0 error, 0 unsupported
```

Two things are going on:

- The `ESS-SYNTH-024` on `held` is the designed answer. A refusal that another input choice of the
  system would avoid is not an observable obligation (`docs/design/system-issued-commands.md:184`,
  `crates/verify/ess-conformance/src/synthesize/system_issued.rs:281`), and the refusal names the
  guard combination.
- The defect: the default branch beside it fails the reference target. `finished` arranges
  `paused = true` and awaits `RunJob` stating no `mode`, so the await cannot hold the system to
  `slow`. The interpreter reports `ESS-CF-AWAIT` "nothing reached by the runner's deadline", and a
  correct target that picks `fast` would fail as well. The rule "a default branch states every other
  branch's `when:` input" (CHANGELOG `### Fixed`, story:a-system-issued-awaited-step-honours-its-outcomes-input-guard)
  reads only `ResolvedCondition::When` and `ExternalWhen` (`system_issued.rs:361`, `guard`). The
  input guard of `SubjectPredicate { input: Some(..) }` (`crates/specify/ess-compiler/src/ir.rs:629`)
  is skipped, and the same holds for `Related` and `RelatedSet` input guards.
- Controls on 0.92.0: with `held` guarded by `when: mode == fast` alone, all 7 scenarios pass.

## Acceptance

- `a_stored_guard_beside_an_input_guard_on_a_system_step_synthesizes`, in
  `crates/verify/ess-conformance/tests/system_issued_synthesis.rs` with the model above inline:
  every synthesized scenario passes the interpreted target. The await of `RunJob/outcome/finished`
  (and of the setup awaits that reach `Done`) states `mode: slow`, or the scenario arranges
  `paused = false`.
- `RunJob/outcome/held` stays refused as `ESS-SYNTH-024` naming the input guard beside the
  stored-field guard.
- The same holds where the input guard sits beside `when_related:` instead of `when_subject:`.
- `tests/fixtures/system-guard.yaml` keeps its suite bytes.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- inferred: crates/verify/ess-conformance/tests/system_issued_synthesis.rs
- cited: docs/design/system-issued-commands.md
- inferred: CHANGELOG.md

confidence: high
