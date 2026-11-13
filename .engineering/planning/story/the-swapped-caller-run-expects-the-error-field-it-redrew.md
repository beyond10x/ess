---
format: aep.planning-md/3
id: story:the-swapped-caller-run-expects-the-error-field-it-redrew
kind: story
status: draft
title: The swapped-caller run expects the error field of the input value it redrew
summary: A refusal's error field copied from an input keeps the first run's value in the swapped-caller run, so a correct model fails its own suite on the interpreter (a consumer's conformance run, ess 0.94.0)
tags:
- backlog
- conformance
- defect
- external-request
- from-consumer-report
- severity-high
relations:
- serves: vision:O2
- informed_by: story:caller-swap-draws-fresh-guarded-inputs
revision: 1
---
## Defect

A correct model fails its own synthesized suite on `--target interpreted` (ess 0.94.0). The failed
scenario witnesses a refusal whose error carries a field copied from an input, in a model whose
commands read the caller. Such a scenario runs twice in one scenario, the second time with the two
callers swapped, and the second run draws a fresh value for the input. The `execute_command` step
of the second run sends the fresh value; its `expect_error` step still names the first run's value.
The interpreter answers with the value it was sent, so the check fails.

Evidence: a consumer's conformance run (2026-11-13, ess 0.94.0, suite `ess-conformance/52`):
207 scenarios, 204 passed, 1 failed, 0 error, 2 unsupported. The failed scenario is of the shape
below, with two further `when_related` refusals on another input before the one under test.

## Reproduction

A model of our own (ess 0.94.0, `format: ess/35`): a clerk writes notes and opens tickets, and
`AnswerTicket` takes the ticket and a cited note. It refuses with `NoSuchNote {note_id}` when no
note carries the cited identity. No command addresses a note by an input.

```yaml
format: ess/35
system: demo
version: v1
domain: demo.desk
summary: A clerk opens a ticket and answers it, citing a note.
types:
  - {name: demo.desk.TicketId, kind: newtype, of: Uuid}
  - {name: demo.desk.NoteId, kind: newtype, of: Uuid}
  - {name: demo.desk.ClerkId, kind: newtype, of: String}
entities:
  - name: demo.desk.Note
    identity: {name: note_id, type: demo.desk.NoteId}
    fields:
      - {name: written_by, type: demo.desk.ClerkId}
    lifecycle:
      initial: Written
      states: [Written]
      terminal: [Written]
  - name: demo.desk.Ticket
    identity: {name: ticket_id, type: demo.desk.TicketId}
    fields:
      - {name: opened_by, type: demo.desk.ClerkId}
    lifecycle:
      initial: Open
      states: [Open, Answered]
      terminal: [Answered]
      transitions:
        - {name: answer, from: [Open], to: Answered}
errors:
  - name: demo.desk.NoSuchTicket
    summary: No ticket carries the identity.
    fields:
      - {name: ticket_id, type: demo.desk.TicketId}
  - name: demo.desk.NoSuchNote
    summary: No note carries the identity.
    fields:
      - {name: note_id, type: demo.desk.NoteId}
  - name: demo.desk.StateConflict
    summary: The ticket cannot move from its held state.
    fields: []
events:
  - name: demo.desk.NoteWritten
    fields:
      - {name: note_id, type: demo.desk.NoteId}
  - name: demo.desk.TicketOpened
    fields:
      - {name: ticket_id, type: demo.desk.TicketId}
  - name: demo.desk.TicketAnswered
    fields:
      - {name: ticket_id, type: demo.desk.TicketId}
actors:
  - name: demo.desk.Clerk
    attributes:
      - {name: clerk_id, type: demo.desk.ClerkId}
    may:
      - demo.desk.WriteNote
      - demo.desk.OpenTicket
      - demo.desk.AnswerTicket
commands:
  - name: demo.desk.WriteNote
    input: []
    outcomes:
      - name: written
        creates: demo.desk.Note
        instance: note_id
        sets: {written_by: caller.clerk_id}
        emits: [demo.desk.NoteWritten]
        payload: {demo.desk.NoteWritten: {note_id: {generated: true}}}
  - name: demo.desk.OpenTicket
    input: []
    outcomes:
      - name: opened
        creates: demo.desk.Ticket
        instance: ticket_id
        sets: {opened_by: caller.clerk_id}
        emits: [demo.desk.TicketOpened]
        payload: {demo.desk.TicketOpened: {ticket_id: {generated: true}}}
  - name: demo.desk.AnswerTicket
    input:
      - {name: ticket_id, type: demo.desk.TicketId}
      - {name: cited_note, type: demo.desk.NoteId}
    outcomes:
      - name: no-such-ticket
        unknown_instance: true
        error: demo.desk.NoSuchTicket
        payload:
          demo.desk.NoSuchTicket: {ticket_id: input.ticket_id}
      - name: no-such-note
        when_related: {via: input.cited_note, exists: false}
        error: demo.desk.NoSuchNote
        payload:
          demo.desk.NoSuchNote: {note_id: input.cited_note}
      - {name: wrong-state, wrong_state: true, error: demo.desk.StateConflict}
      - name: answered
        moves: demo.desk.Ticket.answer
        instance: ticket_id
        emits: [demo.desk.TicketAnswered]
        payload:
          demo.desk.TicketAnswered: {ticket_id: input.ticket_id}
views:
  - name: demo.desk.Tickets
    source: demo.desk.Ticket
    consistency: read_your_writes
    fields:
      - {name: ticket_id, type: demo.desk.TicketId}
      - {name: state, type: demo.desk.Ticket.State}
  - name: demo.desk.Notes
    source: demo.desk.Note
    consistency: read_your_writes
    fields:
      - {name: note_id, type: demo.desk.NoteId}
```

```
ess verify conform synthesize --path model.yaml --target ir --out suite.json
ess verify conform run --path model.yaml --suite suite.json --target interpreted
```

```
  failed demo.desk.AnswerTicket/outcome/no-such-note
  6 scenarios: 5 passed, 1 failed, 0 error, 0 unsupported
```

The scenario's command and error steps in `suite.json` (`ess-conformance/34`), by step index:

```
11 execute_command AnswerTicket caller clerk_id-262145  cited_note = ...c0e2e0d903a1, ticket_id = instance ticket
13 expect_error    NoSuchNote   note_id = ...c0e2e0d903a1
30 execute_command AnswerTicket caller clerk_id-262144  cited_note = ...76cfd14780f9, ticket_id = instance ticket-swapped
32 expect_error    NoSuchNote   note_id = ...c0e2e0d903a1
```

The failed check, from `--format json`:

```
expected: error = demo.desk.NoSuchNote, error.note_id = "00000000-0000-4000-8000-c0e2e0d903a1"
input:    demo.desk.AnswerTicket(cited_note = "00000000-0000-4000-8000-76cfd14780f9", ticket_id = "00000000-0000-4000-8000-000000000006")
observed: error.note_id = "00000000-0000-4000-8000-76cfd14780f9"
```

Variants of the same model:

- the error field named `missing_note`: fails the same way;
- the input typed `Optional<demo.desk.NoteId>`: fails the same way (5 scenarios, 4 passed);
- the error field named like the input, `cited_note`: passes, the second run expects the fresh
  value;
- the same refusal on an input whose type is the identity of an entity a command addresses by an
  input (a model with `order_id` sent to a second command as its `instance`): passes.

A model no actor of which declares an attribute gets no second run and passes.

## Cause, read from the code

`crates/verify/ess-conformance/src/synthesize/caller.rs` on main `1762e21e03`. The second run is a
copy of the first in which `redraw` replaces a drawn value only under a name of the key set that
`Identities::of` builds (line 2162). The set holds:

- the names of command inputs, event fields, entity fields and view fields and params whose type
  is the type of an input some command creates or addresses a row by (lines 2207 to 2222). The
  fields of the model's errors are not read there;
- the names of inputs a guard compares (`cited_note` above);
- the targets `derived` finds (line 2449): fields an outcome's `payload` or `sets` copies from such
  an input (line 2491). An outcome's `error_payload` is not read there.

So an error field is redrawn only when its name is in the set for another reason. That is why the
field named like the input passes and the two other names fail. This reading is not yet pinned by a
test.

Stories on the same code, all implemented: `story:caller-swap-draws-fresh-guarded-inputs` (the rule
that draws the fresh value for a guarded input), `story:a-caller-swap-redraw-keeps-unread-members-and-related-ties`,
`story:a-swapped-caller-run-renames-only-the-names-it-binds`.

## Until it is fixed

- Naming the error field like the input makes the suite pass; that changes the error's published
  field name, so it is a choice for a model that has no consumer of the error yet.
- Otherwise the scenario fails on the interpreter. A generated server that answers with the value
  it was sent would fail it the same way; that was not run here. Whether an `ess-known-failures/1` declaration (`--known-failing`) is the right holder for it was
  not tried here.

## Acceptance (sketch, to be fixed by the fit review)

- A test synthesizes the model above and runs it on `--target interpreted`: every scenario passes,
  and in the second run of `AnswerTicket/outcome/no-such-note` the `expect_error` field equals the
  value that run's `execute_command` sends. It fails before the change.
- The same holds for the error field named `missing_note`, for the `Optional` input, and for an
  error field a refusal copies from a member of a struct input.
- An error field copied from a stored field of a row the second run arranges (a `SubjectField` of
  the refusal's payload) carries that run's value too, or the test shows that it already does.
- The suites of the repository's fixtures change only in `expect_error` fields of second runs; the
  pinned digests that move are listed in the change.

## Fit review

Owed. This draft records the defect and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the acceptance is fixed there.
