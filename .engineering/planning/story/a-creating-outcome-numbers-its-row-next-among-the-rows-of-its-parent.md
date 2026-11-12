---
format: aep.planning-md/3
id: story:a-creating-outcome-numbers-its-row-next-among-the-rows-of-its-parent
kind: story
status: draft
title: A creating outcome numbers its row one above the highest among the rows of its parent
summary: 'No value source gives a creating outcome the next number in a per-parent sequence: increment is refused, an ordered head reads the highest and no expression adds one, generated follows no stated rule (0.90.0 to 0.94.0)'
tags:
- backlog
- external-request
- from-consumer-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
revision: 1
---
## Need

A creating outcome cannot number its new row one above the highest number among the rows that
share a parent key: versions 1, 2, 3 of one document. The model can read that highest number and
cannot add one to it, so the rule "the next number" is in no model. An adopter reported it on
0.90.0 (2026-11-12, format `ess/35`); reproduced on the 0.94.0 release binary with `ess/37`.

A model with `Document` and `Version` (fields `document_id`, `number`, `at`) and a command
`AddVersion` whose creating outcome sets `number`, checked with `ess specify validate`:

| `number:` in the creating outcome's `sets:` | result on 0.94.0 |
|---|---|
| `input.number` (the caller sends it) | valid |
| `{increment: 1}` | `ESS-COMMAND-004`: "`{increment: …}` reads `demo.d.Version` as it was before outcome `added`, and a `creates:` outcome has no row before it" |
| `{related: {entity: demo.d.Version, where: document_id == input.document_id, order_by: [number desc, version_id], first: number}}` | valid: the highest number, not the next |
| `{expr: "coalesce({related: …}, 0) + 1"}` | `ESS-SPEC-012`: "`{` is no part of an expression" |
| `{generated: true}` | valid: the runtime mints an integer the model says nothing about |

The reporter also tried a counter on the parent row through `affects:` beside `creates:`
(`ESS-COMMAND-009 unsupported_construct` on 0.90.0; not re-run here, my probe's `affects:` entry
was malformed).

What the two valid forms cost the adopter:

- Caller-sent: every client counts, a form of an `ess-ui/1` document cannot compute the number,
  and two clients that read the same highest number send the same next one unless a row-set guard
  refuses the second.
- `{generated: true}`: a generic runtime shows a 15 or 16 digit integer as a version number. An
  authored scenario can pin "a revision counting its commits" (`ess/30`), and that rule is still
  not in the model.

The adopter removed the number from its pages and shows the version's date.

## Acceptance

One of the two forms, decided by the fit review:

A. An expression may read an ordered head: a name bound to a `{related: … first: <field>}` read
   is an operand of `{expr: …}`, so `coalesce(<head>, 0) + 1` is a value of a creating outcome.
B. A dedicated source for a creating outcome, such as `{next: {entity, where, of: <field>}}`: one
   above the highest `<field>` among the rows the `where:` selects, 1 where there is none.

Either way:

1. Validation admits the form on a `creates:` outcome for an `Integer` field and refuses it where
   the field's type has no successor.
2. Two creations under one parent get distinct consecutive numbers: the interpreted target and a
   generated server decide the number inside the command's own transaction, and the declaration
   says what a concurrent second creation sees (the reference states it; a `distinct:` or
   uniqueness declaration over parent key and number is admitted beside it).
3. Synthesis writes scenarios for the first row under a parent (number 1), a later row (highest
   plus 1) and a row under another parent (its own count), and each fails on a target that mints
   an arbitrary integer.
4. A deleted row's number: the reference says whether it is reused (the highest among the rows
   that exist) and a scenario holds that.
5. The reference page of value sources lists the form with the document-version example.

## Fit review

Not done (`.agents/skills/assessing-external-requests/SKILL.md`). It decides A or B, and whether
a per-parent sequence belongs to the language at all or is the caller's to send under a guard.
Related: `story:a-later-event-of-a-creating-outcome-carries-the-minted-identity` (another value a
creating outcome mints and the model cannot name).

The same report asked for a view that counts rows per hour or day of a Timestamp. That is
`story:a-group-key-buckets-a-timestamp-by-hour-day-or-week`, which now carries this report as its
second source.

## Evidence

Probe script, models and output on the 0.94.0 release binary: session scratch `ess/adopter12/`
(`probe.py`, one directory per row of the table with `system.yaml` and `validate.txt`; not in the
repository).
