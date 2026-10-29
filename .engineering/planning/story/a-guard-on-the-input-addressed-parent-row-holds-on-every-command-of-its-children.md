---
format: aep.planning-md/3
id: story:a-guard-on-the-input-addressed-parent-row-holds-on-every-command-of-its-children
kind: story
status: draft
title: A guard on the input-addressed parent row holds on the creator, reader and deleter of its child rows
summary: Creator predicate guard starves the reader's found (ESS-SYNTH-001); when_subject beside a row set is ESS-COMMAND-009; the deleter's synthesized scenario fails past when_subject.
tags:
- downstream
- ess-issues-3
relations:
- informed_by: story:every-guard-combination-validates-under-the-rule
revision: 2
---
## Problem

A downstream protocol answers "not found" when a request names a parent row of another tenant.
The parent row is addressed through the input (`input.task_id`), and its child rows (push
configurations) are created, read and deleted by three commands. Each of the three loses the
tenant rule for a different reason. Reported 2026-10-29 by an adopter on ESS 0.75.0. Checked on
the 0.77.0 head (84671d6, `ess 0.77.0`) against the adopter's model at a2a d9e658f, with the
missing guard added by hand:

- Creator, `when_related: {via: input.task_id, predicate: <tenant differs>}` declared after
  `task-not-found`: it validates and synthesizes. `other-tenant` and `created` both get
  scenarios, and `created` arranges the parent under the tenant it sends. On 0.67.0 this was
  ESS-SYNTH-003. With the guard present, the reader's `found` loses its scenario: ESS-SYNTH-001,
  "`task_id` created by `CreateTaskPushNotificationConfig`, whose `task-not-found` refuses a
  missing `a2a.tasks.Task` ... is `row set`, which has no arrangement of rows its selector
  selects".
- Reader: `when_subject:` (the same tenant predicate) beside the row-set guard `config-not-found`
  is refused, ESS-COMMAND-009: "selects on a row set ... and on a `when_subject` guard; one
  command reads one kind of related row in this cut".
  `story:every-guard-combination-validates-under-the-rule` leaves this pair out.
- Deleter: the same `when_subject:` validates and synthesizes. The synthesized `deleted` scenario
  creates its rows under `tenant-1` and deletes with input tenant `tenant`. Against the
  interpreted target it fails with "expected outcome = deleted, observed outcome = other-tenant",
  plus three ESS-CF-VIEW checks.

Repros: `~/.cache/scratch/<session>/a2a-w4/ess-gaps-conformance-gate.txt` (adopter). The 0.77.0
check is `ess-issues-2/a2a-gaps/a2a-A` with `synthA*.out` and `runA3.json` in the same scratch
directory.

## Acceptance

- A creator guarded by a predicate on the input-addressed parent row still lets synthesis arrange
  child rows for the commands that read them, so the reader's `found` has its scenario.
- `when_subject:` beside a row-set `when_related:` validates.
- A synthesized accepting scenario past a `when_subject:` refusal sends an input that satisfies
  none of the earlier refusals, and passes against the interpreted target.

## Downstream update 2026-10-29

The adopter's next release scopes its configurations view to the caller's tenant (the view takes
a `tenant` parameter). Since then, the synthesized scenarios for the reader's `found` and the
deleter's `deleted` arrange their configuration rows under fresh tenants on a task created under
another tenant. The adopter's server answers those scenarios "not found", and the adopter lists
them as rearranged in its conformance runner.

Checked on the 0.77.0 head (84671d6) against the adopter's main at 2ad75d3. The steps are
(command, input tenant):

- `GetTaskPushNotificationConfig/outcome/found`: SendMessage `tenant`; three
  CreateTaskPushNotificationConfig calls under `tenant-1`, `tenant-2` and `tenant-3`; then
  GetTaskPushNotificationConfig with no tenant. Before the view took `tenant`, the three creates
  sent no tenant.
- `DeleteTaskPushNotificationConfig/outcome/deleted`: SendMessage under `tenant`, `tenant-1` and
  `tenant-2`; five creates under `tenant-1` to `tenant-5`; then the delete with no tenant.
- The interpreted target passes both, because the model leaves the parent-row tenant guard
  unmapped, as described above. A target that enforces the rule refuses them.

Added acceptance:

- Synthesis arranges a child row under the tenant (or other input) its parent row was created
  under, whenever the parent row's guard or a view parameter reads that input. It does not
  diversify the input across tenants it never created a parent row under.
