---
format: aep.planning-md/3
id: story:a-delete-may-have-a-no-op-branch-that-emits-nothing
kind: story
status: draft
title: A delete of an absent row may succeed and emit nothing
summary: A no-op delete branch is refused with ESS-SYNTH-001 (a2a wave 1, M3).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
scope:
- confidence: cited
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/unknown_instance.rs
revision: 4
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **M3** (a2a wave-1 gaps list, line 36): a delete whose target row does not exist should succeed
  and emit nothing (an idempotent delete). The adopter's no-op branch was refused with
  ESS-SYNTH-001 (4 refusals), so every delete emits `PushNotificationConfigDeleted`, including a
  delete of nothing.

Narrowed on 2026-10-22: the idempotent delete is already expressible (see Reproduction). What is
missing is (1) that its scenario checks the no-op wrote nothing, and (2) that the refusal an
author meets on the wrong spelling points at the right one.

## Reproduction (0.68.0, 2026-10-22)

Brand-free fixture, `format: ess/27`: entity `demo.jobs.Job` with a `read_your_writes` view
`demo.jobs.Jobs`, `CreateJob`, and `DeleteJob` with branches `deleted` (`deletes: demo.jobs.Job`,
`instance: job_id`, emits `JobDeleted`) and

```yaml
- {name: absent, unknown_instance: true, refuses: false}
```

`ess specify validate`: valid. `ess verify conform synthesize` then `run --target interpreted
--report-format 2`: 3 scenarios, 3 passed, including `demo.jobs.DeleteJob/outcome/absent`.

The `absent` scenario sends the delete for an identity nothing stores and expects the outcome and
no event (`crates/verify/ess-conformance/src/synthesize.rs:13639`, `unknown_instance`; the steps
`execute_command`, `expect_outcome`, `expect_no_event`). Nothing reads the entity afterwards, so a
target that writes a row on the no-op branch still passes (inferred from the steps; no mutation
target was run).

The spelling an author reaches first, `{name: absent, instance: job_id, accepts: nothing}`, is
refused with ESS-COMMAND-005: "declares `instance: job_id` and none of `creates`, `moves` or
`updates`, so the field names the identity of nothing", hint "say what this branch does to which
entity, or drop the `instance`" (`crates/specify/ess-domain/src/command.rs:7605-7614`). Neither
the message nor the hint names `deletes`, and neither names the `unknown_instance` idiom.

## Where it lives

- `crates/verify/ess-conformance/src/synthesize.rs:13639` (`unknown_instance`): the scenario of a
  non-refusing `unknown_instance:` branch.
- `crates/specify/ess-domain/src/command.rs:7605-7614`: ESS-COMMAND-005's message and hint for an
  `instance:` without an effect.
- Related, not the same: `story:an-outcome-that-only-stores-is-observed-through-a-view` (a write
  that emits nothing, refused with ESS-COMMAND-007) and
  `story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view` (a preserving branch of
  an existing row).

## Fit review

2026-10-22: reproduced first, as the earlier note asked. **Already expressible**: the idiom is
`unknown_instance: true, refuses: false` on the absent-row branch. The remaining work is a
conformance defect (the no-op is not checked for writes) and a diagnostic defect (the hint omits
`deletes`). No authored surface, no format version.

## Decisions

2026-10-22: **accept, narrowed.** (1) The `absent` scenario of a non-refusing `unknown_instance:`
branch also checks that no row with the sent identity exists afterwards, read through an
immediate view of the entity where one exists; where none exists, the entity is named in the
scenario's unobserved note. (2) ESS-COMMAND-005's message lists `deletes` among the effects, and
its hint names `deletes:` and, for a branch that answers an absent row, `unknown_instance: true,
refuses: false`. No new `ValidationCode`.

## Acceptance

Fixture: the Reproduction's model, written by the test.

- **A no-op that writes is caught.** Transition: a mutation target stores a `Job` row for the sent
  identity on the `absent` branch. Before: `DeleteJob/outcome/absent` passes. After: it fails.
  Test: `crates/verify/ess-conformance/tests/unknown_instance.rs`, case
  `a_no_op_branch_that_writes_a_row_fails`. Failing before: the mutation target passes the suite
  on the current tree.
- **The honest no-op still passes.** Transition: the interpreted target on the same suite. After:
  3 scenarios, 3 passed. Same file, case `a_no_op_delete_of_an_absent_row_passes`. Models whose
  entity has no immediate view keep their suite bytes apart from the unobserved note.
- **The hint names `deletes`.** Transition: `ess specify validate` on the `instance: job_id,
  accepts: nothing` spelling. Before: message and hint name only `creates`, `moves`, `updates`.
  After: both name `deletes`, and the hint names `unknown_instance: true, refuses: false`. Test:
  the `ess-domain` test pinning ESS-COMMAND-005's text, case
  `an_instance_without_an_effect_names_deletes`; `task test-xtask` passes (a diagnostic changed).

## Schedule

Release 0.70.0 (wave A unit A10, with
`story:a-branch-may-leave-its-addressed-record-unchanged-without-a-view`). The release tag on the
artifact marks it.
