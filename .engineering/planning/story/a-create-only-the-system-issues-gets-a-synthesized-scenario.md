---
format: aep.planning-md/3
id: story:a-create-only-the-system-issues-gets-a-synthesized-scenario
kind: story
status: archived
title: A create only the system issues gets a synthesized scenario
summary: 'ESS-SYNTH-023: a creating command only the system issues gets no scenario.'
tags:
- adopter-report
- defect
- ess-0.71.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-synthesis
- decomposes: epic:zero-open-issues-2026-11
revision: 5
transitions:
- {from: "draft", to: "archived", at: "2026-11-09T23:06:54Z", actor: "human:timo", revision: 5}
---
## Problem

Adopter report, 2026-10-26, ess 0.69.2: a downstream specification quarantines 9 scenarios for this
gap and one other. A creating command that only a `system: true` actor holds gets no synthesized
scenario (`ESS-SYNTH-023`). Every scenario that needs a row it creates is refused with it too, so
the create and everything downstream of it go unexercised.

This is the documented stage-1 limit of `docs/design/system-issued-commands.md` ("The instance",
lines 128-134; the `ESS-SYNTH-023` row at line 377): an awaited step names an existing record by
`instance:`, so a system-issued `creates:` outcome has nothing to await on.

Reproduction on 0.92.0 (`ess/27`): an entity `Check` that a system-only `OpenCheck` creates and a
caller-held `CloseCheck` moves.

```yaml
actors:
  - name: inspection.check.Clerk
    may: [inspection.check.CloseCheck]
  - name: inspection.check.Scheduler
    system: true
    may: [inspection.check.OpenCheck]
commands:
  - name: inspection.check.OpenCheck
    input:
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: inspection.check.Check
        instance: check_id
        sets: {note: input.note}
        emits: [inspection.check.CheckOpened]
        payload:
          inspection.check.CheckOpened: {check_id: {generated: true}}
  - name: inspection.check.CloseCheck
    input:
      - {name: check_id, type: inspection.check.CheckId}
    outcomes:
      - name: closed
        moves: inspection.check.Check.close
        instance: check_id
        emits: [inspection.check.CheckClosed]
        payload:
          inspection.check.CheckClosed: {check_id: input.check_id}
      - {name: not-open, wrong_state: true, error: inspection.check.NotOpen}
```

`ess verify conform synthesize --path spec.yaml --out suite.yaml` writes 1 scenario
(`CloseCheck/outcome/not-open`) and 4 `ESS-SYNTH-023` refusals: `OpenCheck/outcome/opened`,
`CloseCheck/outcome/closed`, `Check/transition/close/by/CloseCheck/closed` and
`Check/state/Closed/refuses/CloseCheck`.

## Acceptance

- The reproduction is checked in as
  `crates/verify/ess-conformance/tests/fixtures/system-issued-create.yaml`.
- `docs/design/system-issued-commands.md` states how a scenario awaits a system-issued create: what
  the await names in place of an instance, and how the created identity is captured for later steps.
- `crates/verify/ess-conformance/tests/system_issued_synthesis.rs`,
  `a_system_only_create_is_awaited_in_a_scenario`: for the fixture, synthesis records no
  `ESS-SYNTH-023`; `OpenCheck/outcome/opened` awaits the create and checks its outcome, its
  `CheckOpened` event and the `Checks` view row; `CloseCheck/outcome/closed` reaches its row through
  that awaited create; the interpreted target passes all of them. Red on 0.92.0.
- A system-issued command with no subject (only `instances:` or `affects:`) keeps its
  `ESS-SYNTH-023` refusal, and suites of models without a system-issued create keep their bytes.

## Scope

- cited: crates/verify/ess-conformance/src/synthesize/system_issued.rs
- cited: crates/verify/ess-conformance/src/system_issued.rs
- cited: docs/design/system-issued-commands.md
- inferred: crates/verify/ess-conformance/src/synthesize.rs
- inferred: crates/verify/ess-conformance/src/interpret/
- inferred: crates/verify/ess-conformance/src/go/runtime.go
- inferred: crates/verify/ess-conformance/src/ts/runtime.ts
- inferred: crates/verify/ess-conformance/tests/system_issued_synthesis.rs

confidence: medium


## Duplicate (2026-11-09)

The same gap as `story:a-system-issued-creating-command-is-witnessed`, which keeps the reproduction
(0.92.0: four ESS-SYNTH-023 refusals on a system-only creating command, the documented stage-1 limit
in `docs/design/system-issued-commands.md:128-134`) and stays in the plan of
`epic:zero-open-issues-2026-11`. This draft leaves the plan as archived.
