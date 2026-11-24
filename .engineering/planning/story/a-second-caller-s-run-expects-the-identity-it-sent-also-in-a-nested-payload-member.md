---
format: aep.planning-md/3
id: story:a-second-caller-s-run-expects-the-identity-it-sent-also-in-a-nested-payload-member
kind: story
status: draft
title: A second caller's run expects the identity it sent, also in a nested payload member
tags:
- gap-p1
- gap-reports-2026-11-24
relations:
- serves: vision:O2
revision: 1
---
## Need

A synthesized scenario runs a second time in the same scenario with the callers' roles reversed:
for two attribute-free actors that are both granted the command
(`crates/verify/ess-conformance/src/synthesize/grant.rs:419-519`, `cross_caller`), and for actors
carrying attributes (`synthesize/caller.rs:23-28`). The appended run draws a fresh value for every
caller-supplied identity it sends (`caller.rs:1457`, `Identities::drawn`) and rewrites the old
value wherever the serialized run carries it (`caller.rs:1463`, `redraw`).

`redraw` rewrites a value only under a member name held in `Identities::keys`
(`caller.rs:3486`, `keys.contains(name)`). `keys` holds plain names: fields declared at the
identity's type, and the `target` of every payload or `sets:` entry that copies the identity,
struct leaves included (`caller.rs:2492-2498`, `derived` at `caller.rs:2782-2848`, which inserts
`leaf.target`, here `id`). A struct payload field with a member the scenario does not determine
is asserted leaf by leaf under its dotted path (`synthesize.rs:11643-11651`,
`determined_leaves`; the key is built at `synthesize.rs:14138`, `format!("{prefix}.{}",
leaf.target)`), so `expect_event.payload` holds the key `data.id`. `data.id` is not in `keys`, so
`redraw` leaves the first run's value under it. The appended run sends the fresh identity and
expects the old one in `data.id`. Every implementation that echoes its input fails
`<Command>/outcome/<accepted>`, the interpreter included. That contradicts the module's own
statement that the old value is replaced "in an input, an event payload, a view row or an
expectation" (`caller.rs:3448-3449`).

By the same reading, an error payload has the same gap: `expect_error` fills its fields with
`determined_fields` (`synthesize.rs:11557-11568`), but `derived` reads only `payload` and `sets:`
(`caller.rs:2815-2828`) and the declared names leave error fields out (`caller.rs:2477-2491`). An
error field that copies a redrawn identity under a name of another type keeps the old value too.

Minimal reproduction (written fresh, not the reporter's; not run, the assessment was read-only):
the model below, at `format: ess/37` on 0.98.0. Two actors, both granted both commands, carrying no
attributes. `SubmitTicket` addresses the ticket by the input `ticket_id` and echoes it in
`TicketSubmitted.data.id`. `data.label` is generated, so `data` is asserted leaf by leaf.

```yaml
format: ess/37
system: demo
version: v1
summary: An input identity echoed into a nested event member, sent by two actors.
domain: demo.tickets
types:
  - {name: demo.tickets.TicketId, kind: newtype, of: Uuid}
  - name: demo.tickets.Receipt
    kind: struct
    fields:
      - {name: id, type: demo.tickets.TicketId}
      - {name: label, type: String}
entities:
  - name: demo.tickets.Ticket
    identity: {name: ticket_id, type: demo.tickets.TicketId}
    fields:
      - {name: note, type: String}
    lifecycle: {initial: Open, states: [Open], terminal: [Open]}
events:
  - name: demo.tickets.TicketOpened
    fields:
      - {name: ticket_id, type: demo.tickets.TicketId}
  - name: demo.tickets.TicketSubmitted
    fields:
      - {name: data, type: demo.tickets.Receipt}
actors:
  - {name: demo.tickets.Clerk, may: [demo.tickets.OpenTicket, demo.tickets.SubmitTicket]}
  - {name: demo.tickets.Agent, may: [demo.tickets.OpenTicket, demo.tickets.SubmitTicket]}
commands:
  - name: demo.tickets.OpenTicket
    input:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: note, type: String}
    outcomes:
      - name: opened
        creates: demo.tickets.Ticket
        instance: ticket_id
        sets: {note: input.note}
        emits: [demo.tickets.TicketOpened]
        payload:
          demo.tickets.TicketOpened: {ticket_id: input.ticket_id}
  - name: demo.tickets.SubmitTicket
    input:
      - {name: ticket_id, type: demo.tickets.TicketId}
      - {name: note, type: String}
    outcomes:
      - name: submitted
        updates: demo.tickets.Ticket
        instance: ticket_id
        sets: {note: input.note}
        emits: [demo.tickets.TicketSubmitted]
        payload:
          demo.tickets.TicketSubmitted:
            data: {id: input.ticket_id, label: {generated: true}}
```

```console
$ ess verify conform synthesize --path model.yaml --out suite.json
$ ess verify conform run --path model.yaml --target interpreted --report-format 2
```

In `demo.tickets.SubmitTicket/outcome/submitted` of `suite.json`, the second `execute_command` of
`SubmitTicket` (the reversed run, its instances suffixed `-swapped`) sends `ticket_id` = W2, and
the `expect_event` after it carries `"data.id": W1`, the first run's value. The right bytes:
`"data.id": W2`. The run reports that scenario failed. Two control variants, each expected to
pass by the same reading: `label: submitted` in place of `{generated: true}` (the struct is then
determined whole, the payload key is `data` and its member `id` is in `keys`), and a top-level
event field `id: input.ticket_id` in place of `data` (`id` is in `keys`).

## Outcome

Wherever a reversed or appended run carries a copy of a redrawn identity, at any depth of an event
payload and in an error's fields, it carries the fresh value. A suite that had no such copy keeps
its bytes.

## Acceptance

- A new test, `crates/verify/ess-conformance/tests/a_swapped_run_redraws_nested_payload_leaves.rs`,
  synthesizes the model above, finds the reversed run's `expect_event` of
  `demo.tickets.TicketSubmitted` in `demo.tickets.SubmitTicket/outcome/submitted`, and asserts that
  its `data.id` equals the `ticket_id` the reversed run sends and differs from the first run's. It
  fails before the change and passes after.
- The same test runs the suite against the interpreter (`Interpreted`), and the scenario passes.
- The same assertion holds for actors carrying attributes (the `caller.rs` swap): the model with
  `attributes:` on one actor and a `{caller: …}` read, as `tests/caller_fresh_identity.rs` builds.
- The same assertion holds for an error field copying the identity under a name no declared field
  of the identity's type has, in a refusal scenario with an appended run.
- The two control variants keep their bytes: the whole-struct and the top-level forms produce the
  same suite before and after the change.
- `cargo test -p ess-conformance --test caller_struct_identity --test caller_fresh_identity --test
  a_caller_swap_redraw_keeps_unread_members_and_related_ties --test leaf_payloads` pass unchanged.
- The slow lane's `synthesis_suite_bytes_table` (`task test-slow`) is run once. Each row that
  moves is named in the change with the scenario whose stale value it corrects. A row that moves
  for any other reason fails the change.
- Each test of the change runs under 60 s, and the test step it belongs to within 120 s.

## Fit review

Reading tree: `integration/wave-2026-11-24` at `bcfc51555`.

1. **Need.** A reversed caller run expects the first run's identity in a nested payload member, so
   a correct implementation fails the outcome scenario. Reproduction above. Report: third report,
   finding 1 (the reporter declared it a known failure on 0.98.0).
2. **Class: defect.** `caller.rs:3448-3449` says the fresh value replaces the old one in "an event
   payload". `leaf_payloads.rs:4-8` and `synthesize.rs:14091-14097` say a nested member is
   asserted under its dotted path. The two rules were never joined.
3. **Already expressible?** Not applicable to a defect. The reporter's workaround is a declared
   known failure (`ess-known-failures/1`), which removes the scenario from the verdict.
4. **Fit.** No new surface. One rule is widened: `keys` also holds the dotted path of each struct
   leaf that copies an identity, and error-payload targets, so `redraw`'s exact name match reaches
   them. The rule stays a name match, so the guard against rewriting metadata that equals a witness
   (`caller.rs:3451-3453`) stays in force. Siblings that use the same `append`: grant
   cross-caller runs, attribute swaps, disclosure follow-ups (`append_followup`, `caller.rs:1367`).
5. **Second adopter.** Any CloudEvents-style envelope (`data: {id, time}` with a generated time)
   whose command is granted to two roles.
6. **Cost.** No format change, no diagnostic. Suite bytes move only for a scenario that carries the
   defect today. Whether a repository model is one cannot be established by reading. The bytes
   table answers it.
7. **Considered.** (a) A dotted key counts as typed when its last segment is in `keys`. This was
   rejected: it widens the match to any leaf of that name, including leaves that do not copy the
   identity. (b) Chosen: `derived` records the exact dotted target path and error targets. (c) Change
   nothing and document the known failure. This was rejected because it is a defect.

## Decisions

**Accept as a defect; hotfix material.** The fix is hotfix-sized (S). Release form: a patch on the
line after 0.98.1 if no row of `synthesis_suite_bytes_table` moves. If a row moves, the fix still
ships first, but the byte-pin tables move with it, so it is released as the next minor version, not
a patch (rule given by the coordinator). No open design question.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/caller.rs` (`derived` 2782-2848, `Identities::of` 2477-2498, `redraw_under` 3466-3504)
- cited: `crates/verify/ess-conformance/src/synthesize.rs` (`determined_leaves` 14098, `collect_leaves` 14125, `expect_error` 11557), read only
- inferred: `crates/verify/ess-conformance/tests/a_swapped_run_redraws_nested_payload_leaves.rs` (new)
- inferred: `CHANGELOG.md`, `release-notes/<version>.md`
- Size: S (about 40 changed lines of code, about 200 of test).

## Depends on

Nothing. It lands first.

## Source

A downstream application, third report, finding 1 (ESS 0.98.0).
