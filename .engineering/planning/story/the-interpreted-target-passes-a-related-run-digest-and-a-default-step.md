---
format: aep.planning-md/3
id: story:the-interpreted-target-passes-a-related-run-digest-and-a-default-step
kind: story
status: draft
title: The interpreted target passes a whole-model related-run digest and default step
summary: 15 interpreted scenarios fail on DigestMismatch and an await deadline in a whole model (workbench report items 37, 38)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Need

On the reporter's whole model (not minimised, not in the repository; the reporter holds it),
synthesis on 0.92.0 gives 879 scenarios and the interpreted target passes 861, fails 15 and leaves
3 unsupported. Two failure shapes, items 37 and 38 of the 2026-11-09 workbench report:

- `MarkConforming/outcome/conforming` is answered with the model's own `DigestMismatch` refusal and
  the view keeps showing the subject `Validated` (item 37). The report's diagnosis: the scenario's
  setup does not record the related run whose digest the guard compares, or records it with another
  digest. Inferred, not checked: the accepting branch sits behind a `when_related:` refusal comparing
  a related row's stored field with the subject's, and synthesis arranges a related row the guard
  does not accept, so the interpreted target answers the refusal. `DigestMismatch` is a name in the
  reporter's model; ESS has no such error.
- `TakeDefault/outcome/no-default` ends `nothing reached by the runner's deadline, after 50
  await(s)` (item 38), the await-outcome timeout of
  `crates/verify/ess-conformance/src/runner/await_outcome.rs:141`, recorded when every await
  answers `NotIssued`: the scenario awaits an outcome the system issues itself (a binding or a
  system-issued step), and the interpreted target never issues it.

Neither is reproduced in the repository: no example exists for items 37 and 38
(`.engineering/repro/workbench-2026-11-09/examples/`), and no smaller model was given.

## Acceptance

- Each failure has a minimised specification under `.engineering/repro/` on which
  `ess verify conform synthesize --path spec --out suite.yaml` and
  `ess verify conform run --target interpreted --path spec --report-format 2` show the failing
  scenario on the current release.
- Each minimised failure is then classified as a defect of synthesis (the scenario expects what the
  model does not do) or of the interpreted target (the target answers against the model), and a
  regression test in `crates/verify/ess-conformance/tests/` fails before the fix and passes after.
- On the minimised specifications the interpreted run reports 0 failed for those scenarios.

## Scope

- inferred: `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (item 37, related-row setup)
- inferred: `crates/verify/ess-conformance/src/interpret/` (either item, if the target is wrong)
- inferred: `crates/verify/ess-conformance/src/runner/await_outcome.rs` (item 38, the awaited step)

confidence: low

## Fit review

Pending minimisation: questions 1 and 2 (need and class) cannot be answered without a reproduction.
An external request: it is assessed with `.agents/skills/assessing-external-requests/SKILL.md`
before this story leaves draft.

## Source

`.engineering/repro/workbench-2026-11-09/report.md`, items 37 and 38.
