---
format: aep.planning-md/3
id: story:a-related-refusal-compares-the-rewritten-row
kind: story
status: draft
title: A related refusal reads back the stored fields of the row it rewrites
tags:
- defect
- ess-d368-close
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv
- confidence: inferred
  path: crates/verify/ess-conformance/tests/refusal_policy.rs
revision: 5
---
## Finding

Found by the unit that made binding refusal witnesses compare their trigger identities
(`story:a-refusal-witness-compares-its-trigger-identities`), out of that story's Acceptance.
Where a binding's invoked command `updates` a row and a per-refusal policy (`drop:` or a bounded
`retry:`) answers a `when_related` refusal, the synthesized `<binding>/binding/refusal/<outcome>`
scenario compares the refusal and the invocation count. It never reads that row back, so a target
whose refused command still writes the row passes.

Reproduction on 0.92.0 (three files, system `demo`, domain `demo.b`):

- `Grant {grant_id, open: Boolean}`, created by `DefineGrant(open)`.
- `Ticket {ticket_id, grant_id, flag: Boolean, note: String}`, created by
  `OpenTicket(grant_id, note)` with `flag: false`. `MoveTicket(ticket_id, grant_id)` updates
  `grant_id` and emits `TicketMoved {ticket_id, grant_id}`.
- `Refresh(ticket_id, grant_id)`: `grant-missing` (`when_related {via: input.grant_id, exists:
  false}`), `grant-closed` (`when_related {via: input.grant_id, predicate: open == false}`), and
  `refreshed`, which `updates: Ticket`, `instance: ticket_id`, `sets: {flag: true}`.
- Binding `refresh-on-ticket-moved`: `TicketMoved` → `Refresh`, mapping both fields,
  `on_failure: {drop: [grant-missing, grant-closed], retry: {except: [grant-missing, grant-closed]}}`.
- A read-your-writes view `Tickets` publishing every Ticket field.

```text
$ ess verify conform run --path . --target interpreted   → 12 scenarios: 12 passed
refresh-on-ticket-moved/binding/refusal/grant-closed:
  OpenTicket → DefineGrant(open: true) → DefineGrant(open: false) → DefineGrant(open: true)
  MoveTicket {grant_id: <closed grant>, ticket_id: ticket} → expect_event_values TicketMoved
  expect_invocation refresh-on-ticket-moved Refresh count 1
  (no query_view / snapshot_subject / expect_subject_unchanged on Tickets)
```

`grant-missing` has the same shape. By contrast, the single-policy `drop` cell
(`binding_effects.rs` `dropped`) snapshots the destination row before the trigger and expects it
unchanged. The per-refusal witness built through `refusal_policy.rs` `refused_row` (:384) and
`binding_effects.rs` `related_refused` (:1629) does not.

## Acceptance

- `a_related_refusal_reads_back_the_rewritten_row`: for the model above, each
  `refresh-on-ticket-moved/binding/refusal/<outcome>` scenario reads the ticket back after the
  refusal and compares its stored fields with what they were after the trigger (`flag: false`,
  `note` as sent, `grant_id` as moved). A target whose `Refresh` sets `flag: true` while answering
  `grant-closed` fails it; the interpreted target passes every scenario.
- Where no read-your-writes view shows the row by identity, the scenario keeps today's shape and
  the synthesis notes say the row is not compared.
- Committed models whose suites move are named, and their bytes pins move with the change.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/refusal_policy.rs` (`refused_row` :384, `witness`)
- cited: `crates/verify/ess-conformance/src/synthesize/binding_effects.rs` (`related_refused` :1629; `dropped` snapshot pattern :3255-3300)
- inferred: `crates/verify/ess-conformance/tests/refusal_policy.rs` (new case)
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-suite-bytes-table.tsv` (binding refusal rows)

confidence: medium
