---
format: aep.planning-md/3
id: story:the-interpreted-target-passes-a-related-run-digest-guard
kind: story
status: draft
title: The interpreted target passes a related-run digest guard
summary: 'Workbench item 37: MarkConforming observes another model digest on the interpreted target; not reproduced from the report, needs the adopter''s model (turn-3 split, backlog)'
tags:
- backlog
- defect
- external-request
- from-consumer-report
- needs-evidence
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

Workbench report item 37 (`.engineering/repro/workbench-2026-11-09/report.md`): on the
interpreted target, `MarkConforming/outcome/conforming` observes a different model digest
(`DigestMismatch`), the view stays `Validated`, and 15 scenarios of the reporter's whole model
fail. The report has only the model; the reporter's log is not in the repository.

`story:the-interpreted-target-passes-a-related-run-digest-and-a-default-step` (turn 3) fixed item
38 but could not reproduce item 37. About 30 guessed variants all pass synthesis and the
interpreted target. They covered:

- a digest guard against a literal, an input, and an optional digest;
- a stored reference, `via: input.run_id`, and row-set `exists: false`, `count` and `forall`;
- String, Bytes, Uuid, Integer and Timestamp digest newtypes;
- a run-state guard beside the digest guard;
- a system-issued run creator.

That story kept a guessed minimised model, `examples/37/spec/system.yaml`, and a regression pin,
`crates/verify/ess-conformance/tests/related_guard_accepting_arrangement.rs`. Both are green on the
base, so neither is a reproduction.

## Acceptance

- The failing shape is in the repository as a minimised model that fails on the release before the
  fix. It comes from the reporter's model or log; ask the adopter for it.
- On that model, the synthesized `MarkConforming/outcome/conforming` scenario arranges the related
  run with the digest the caller sends, and passes `--target interpreted` (0 failed).
- `related_guard_accepting_arrangement` stays green.
