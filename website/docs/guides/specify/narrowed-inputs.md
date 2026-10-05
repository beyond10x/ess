---
title: An input refused when absent is present afterwards
sidebar_position: 6
description: An Optional input reads as its value type in a branch only ever taken with the input present, so it fills a required field without a conversion.
---

# An input refused when absent is present afterwards

From source `ess/16`, an `Optional<T>` input reads as `T` in a branch that is only ever taken with
the input present. There are two such branches:

- the default branch, written with no `when:`, when a sibling refuses exactly the input's
  absence with `error:` — `when: not defined(x)` or `when: missing(x)`;
- a branch whose own guard requires the input: `when: defined(x)`, or an `all:` with it as a
  member.

```yaml
commands:
  - name: demo.notes.SubmitNote
    input:
      - {name: account_id, type: Optional<demo.notes.AccountId>}
      - {name: text, type: String}
    outcomes:
      - name: account-missing
        when: not defined(account_id)
        error: demo.notes.AccountMissing
      - name: submitted                       # the default: the account is present here
        creates: demo.notes.Note
        instance: note_id
        sets: {account_id: input.account_id, text: input.text}
        emits: [demo.notes.NoteSubmitted]
        payload:
          demo.notes.NoteSubmitted: {note_id: {generated: true}, account_id: input.account_id, text: input.text}
```

`input.account_id` fills the required `account_id` fields with no `conversions:` entry. A
conversion from `Optional<AccountId>` to `AccountId` would admit the same copy on every command,
whether anything refuses the absence first or not. Narrowing does not depend on the order outcomes
are declared in. It applies to `input.<x>` in `payload:`, in `sets:` and in a leaf of a nested
mapping, and only to a top-level input.

Nothing else narrows. A refusal of the absence *and* something else, such as
`{all: ["not defined(x)", "kind == Draft"]}`, leaves some absent requests to the default branch. A
sibling that succeeds when `x` is absent refuses nothing. A guarded branch other than the default can
match a request the refusal also matches. That includes a branch written `when: true`: it is a
guard that always holds, not the default, and a runtime that tries branches in declared order would
take it before a refusal declared after it. Write the default with no `when:`.

The synthesized suite checks the narrowing: the refusal's scenario sends no `account_id` and requires
`account-missing`, and the success scenario sends one. Below `ess/16` the copy is refused as a
`type_mismatch`, as it always has been.
[Design](https://github.com/beyond10x/ess/blob/main/docs/design/optional-input-narrowing.md).
