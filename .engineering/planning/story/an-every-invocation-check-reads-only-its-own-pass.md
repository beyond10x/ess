---
format: aep.planning-md/3
id: story:an-every-invocation-check-reads-only-its-own-pass
kind: story
status: draft
title: An every-invocation check in a swapped-caller pass reads only its own pass
summary: 'Inferred from the 0.72.0 gap Q fix: the forced-outcome cell''s every-invocation check spans both passes of a swapped caller.'
tags:
- defect
- ess-0.74.0
- severity-high
- zero-open-issues
relations:
- informed_by: approval-record:ess-0720-hold-for-gap-q
- decomposes: epic:zero-open-issues-2026-11
revision: 3
---
## Problem

Found while fixing the 0.72.0 swapped-caller count (`approval-record:ess-0720-hold-for-gap-q`).
That fix makes a swapped pass's `expect_invocation` count cumulative over the scenario
(`crates/verify/ess-conformance/src/synthesize/caller.rs`, `cumulative`), because runners count
every invocation since the scenario started.

The forced-outcome cell of a `drop` binding (`synthesize/binding_effects.rs`, `dropped`, the
`ScenarioStep::ExpectEveryInvocation` push near :3280) has the same window and was not given the
same treatment: an empty `selecting` and the mapped `input`, so it checks every invocation of the
binding since the scenario started. When `caller::append` appends the swapped pass, the second
pass's check also reads the first pass's invocation, which carried the first pass's instances.
`cumulative` raises only `ExpectInvocation` counts and `ExpectPublicationCount`; it leaves
`ExpectEveryInvocation` alone.

Reproduced on 0.92.0 (no longer inferred). Minimal model, three files:

- `system.yaml`: `format: ess/32`, `system: demo`, `domains: [demo.b]`.
- `components.yaml`: one component accepting every command; binding `refresh-on-note-added`,
  `when: {event: demo.b.NoteAdded}`, `invoke: {command: demo.b.Refresh}`,
  `mapping: {ticket_id: event.ticket_id, grant_id: event.grant_id}`, `delivery: at_least_once`,
  `on_failure: drop`.
- `b.yaml`: actor `Clerk` with attribute `account_id`; `DefineGrant` creates a `Grant` setting
  `account: {caller: account_id}` (this makes the suite run a swapped-caller pass); `OpenTicket`
  creates a `Ticket` under a grant; `AddNote(ticket_id, grant_id)` creates a `Note` and emits
  `NoteAdded {note_id, ticket_id, grant_id}`; `Refresh(ticket_id, grant_id)` has
  `upstream-down` (`external:`, error `Down`) and `refreshed` (`updates: Ticket`); a
  read-your-writes view `Tickets`.

```text
$ ess verify conform run --path . --target interpreted --report-format 2
  failed refresh-on-note-added/binding/on-failure
  10 scenarios: 8 passed, 1 failed, 0 error, 1 unsupported

ESS-CF-INVOCATION (second pass)
  expected: every invocation for the occurrence has demo.b.Refresh.ticket_id = "…0005"
  observed: [Refresh(grant_id = "grant_id", ticket_id = "…0002");
             Refresh(grant_id = "grant_id", ticket_id = "…0005")]
```

The suite holds two `expect_every_invocation` steps, the second naming the swapped pass's
`observed-…-swapped` instances, and the count step was correctly raised to 2. The interpreter is
right; the suite fails a correct implementation.

## Acceptance

- `an_every_invocation_check_reads_only_its_own_pass`: the model above, synthesized and run
  against `--target interpreted`, passes `refresh-on-note-added/binding/on-failure` (every
  scenario passes except ones unsupported for an unrelated reason). Before the fix the scenario
  fails with `ESS-CF-INVOCATION` on the second pass.
- The second pass's `expect_every_invocation` selects only its own occurrence (for example a
  `selecting` filled from the pass's own trigger payload) or is otherwise confined to the pass;
  the first pass's step is unchanged.
- A target whose second-pass invocation carries the first pass's `ticket_id` still fails the
  scenario (mutation check).
- `tests/binding_refusal_policy_swapped_caller.rs` keeps its verdicts; suite bytes move only for
  models whose `drop` cell is appended under a swapped caller.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/caller.rs` (`append` :1341, `cumulative` :1409)
- cited: `crates/verify/ess-conformance/src/synthesize/binding_effects.rs` (`dropped`, `ExpectEveryInvocation` :3280)
- inferred: `crates/verify/ess-conformance/tests/binding_drop_swapped_caller.rs` (new test with the model above)

confidence: high
