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
  path: crates/verify/ess-conformance/tests/refusal_policy.rs
revision: 7
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

- `cargo test -p ess-conformance --locked --test refusal_policy`, test
  `a_related_refusal_reads_back_the_rewritten_row`: for the model above, each
  `refresh-on-ticket-moved/binding/refusal/<outcome>` scenario (`grant-missing`, `grant-closed`)
  holds a `snapshot_subject` of `Tickets` for the ticket after the trigger and an
  `expect_subject_unchanged` on `Tickets` after the invocation count, so it compares `flag: false`,
  `note` as sent and `grant_id` as moved; run against `--target interpreted`, every scenario of the
  suite passes.
- Mutation check, same test: a target whose `Refresh` sets `flag: true` while answering
  `grant-closed` fails `refresh-on-ticket-moved/binding/refusal/grant-closed`.
- Same test target, the model above with the `Tickets` view removed: the two refusal scenarios keep
  today's steps (no `snapshot_subject`, no `expect_subject_unchanged`), and the synthesis result's
  `notes` hold a `Note::PartialObservation` for each of them naming the unobserved Ticket fields.
- `cargo test -p ess-conformance --locked --test synthesis_suite_bytes_table` (and the
  `synthesis_suite_bytes_table_cli_read`, `adversary_related_values_all_bytes` and
  `external_beside_held_guard` targets) pass, and every row the change re-pins belongs to a model
  whose suite holds a `<binding>/binding/refusal/<outcome>` scenario over a `when_related` refusal
  of a command that `updates` a row, with that cause named beside the pin.

## Scope

Re-pins suite-bytes rows for models with a per-refusal `when_related` policy on an updating
command.

- cited: crates/verify/ess-conformance/src/synthesize/refusal_policy.rs
- cited: crates/verify/ess-conformance/src/synthesize/binding_effects.rs
- inferred: crates/verify/ess-conformance/tests/refusal_policy.rs

confidence: medium

`refusal_policy.rs`: `refused_row` :384, `witness`. `binding_effects.rs`: `related_refused` :1629;
the `dropped` snapshot pattern :3255-3300 is the one to copy. `tests/refusal_policy.rs` gains the
new case.

## Decisions

- An unobserved row is reported with the existing `Note::PartialObservation` (`synthesize.rs`
  :309), whose documented meaning already covers "a refusal that changed it is not checked", so no
  note variant is added and `synthesize.rs` is not touched.

