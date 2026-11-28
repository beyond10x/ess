---
format: aep.planning-md/3
id: story:a-ui-expression-joins-texts
kind: story
status: draft
title: A UI expression joins texts
summary: An ess-ui/1 expression has no concatenation and no template, so a text built from parts is a model view field or a host slot (a consumer specification's UI list, item 1)
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- informed_by: story:text-functions-compute-values
revision: 2
---
## Need

A text shown or copied is sometimes built from parts: a fixed text and a value of the row
(`Request r1`), two values with a separator, an address made of a prefix and an identity. An
`ess-ui/1` expression is a path, a literal or a comparison; its operators are `==`, `!=`,
`in [..]`, `not`, `and`, `or` (`website/docs/reference/ess-ui.md`, Expressions). Nothing joins two
texts, so the part-built text is a field the model's view computes, or a host slot.

Evidence: a consumer specification's UI document (2026-11-13, ess 0.94.0), item 1 of its third collected list.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a command `ApproveRequest {id}`, a view `PendingRequests` (fields `id`, `title`,
`state`, no parameter), a page `requests.inbox` whose section `list` is a `collection` over that
view with the row action `approve`, and a page `requests.detail` with a param `id`. Fixture rows
`r1` (`request-one`) and `r2` (`request-two`). `ess ui check --model` on the base:
`0 error(s), 0 warning(s)`. Each reproduction adds one row action
`{name: link, copy: "<expression>", label: Copy the link}` to the section `list`.

| `copy:` | `ess ui check --model` | the terminal's notice after `act` on row `r1` |
|---|---|---|
| `row.title + row.id` | 0 errors, 0 warnings | `copied` (empty) |
| `'request ' + row.id` | 0 errors, 0 warnings | `copied 'request ' + row.id` (the source text) |
| `{row.title}/{row.id}` | 0 errors, 0 warnings | `copied {row.title}/{row.id}` (the source text) |
| `concat(row.title, row.id)` | 0 errors, 0 warnings | `copied` (empty) |

The check passes all four; that part is the defect
`story:ui-check-reports-an-expression-that-reads-a-path-and-is-no-expression`.

What works today: a view field of the model computed with `+` over texts
(`story:text-functions-compute-values`, active, documents it) and shown or copied by its path. That
moves a text of the user interface into the model.

## What a design has to settle

- Whether the UI expression language computes at all, or stays paths and comparisons and the
  answer is the view field. The fit review weighs a text that belongs to one screen against a
  second expression language to keep equal with the model's.
- If it joins: one form, and the same in every `expr` position (`text`, `label`, `copy`, `args`,
  a navigate param). Candidates are the model language's `s + t` and a template with holes; two
  spellings for one idea are refused by the review's question 4.
- What an absent part gives (an absent Optional, a row still loading): the whole text absent, or
  the part empty.
- What `ess ui check` resolves and types: each part's path, and that each part is a text or has
  one documented text form.
- What both renderers and `ess-ui-test/1` show for it.

## Acceptance (sketch, to be fixed by the fit review)

- Either the reference says that a text built from parts is a view field of the model and
  `ess ui check` answers the four documents above with a finding that says so, or a document
  states the join once, both renderers show and copy the joined text, and a test pins it.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.

## Covered findings

**Requirement 5 of a downstream page generator's report of 2026-11-25.** A column or record field
shows text composed from its row's members, and a list field shows text composed from each item's
members: `[{name: cpu, value: 6}, {name: mem, value: 0}]` reads `cpu 6 · mem 0`, with no host slot,
and a test reads the cell in both renderers. Read against this story (tree `2e055bcd0c`,
read-only):

- A field shows one `field` through `as` and `labels` (`crates/ui/ess-ui/src/model.rs:2502-2546`);
  no key of it holds an expression. The requester's way today is a host slot per place, whose
  `host` names are unique, and the terminal draws the slot's label, so a test cannot read the cell
  there (`website/docs/reference/ess-ui-test.md:412-446`, last row).
- The active stories
  `story:a-structured-value-reads-as-one-line-of-its-members-in-every-renderer` and
  `story:a-list-field-names-the-text-that-joins-its-items-in-every-renderer` end the JSON text:
  the list above reads `name: cpu, value: 6, name: mem, value: 0`. Which members show, and how
  they read, is what is left.
- What this story's draft did not name: the position (a field's text; the draft lists `text`,
  `label`, `copy`, `args` and a navigate param) and the scope (each item of a list value, not only
  the row).
- Whether a view field of the model can compute text over each item of a list is not known from
  this reading: the model language's list functions were not read for it.
- Conflict, not decided here: the reference's Expressions section
  (`website/docs/reference/ess-ui.md`, "Expressions": "short paths plus a few operators", and a
  string that joins words with an operator but reads no path is a finding) and the fourth answer of
  `story:a-duration-field-shows-the-time-between-two-instants-of-its-row-the-clock-standing-in-for-an-absent-end`
  (a UI arithmetic would be a second expression language) hold the other side of the first
  decision below.
- Not adopted from the request: the glyph between items (` · `) and the way a member reads (a name
  then a value, no colon) are how text looks. `ess-ui` is not extended for styling (XT-D3).
- Second adopter: a deployments table whose list of `{region, count}` reads `eu 3 · us 1`.

## Decisions

Would add to `ess-ui/1`, none accepted (each waits for the owner's read before a release): under
XT-D1 A, a join form in the expression grammar usable at a field's text; under XT-D2 A, the same
form with a root for the item; under XT-D2 B, a key of a list field that names the members of each
item that show.

- **XT-D1. Does a field show text composed from its row?** (A) Yes: a UI expression that joins
  texts (the model language's `+` or a template, as this story's design question asks), accepted at
  a field's text and in every other `expr` position. (B) No: the model's view computes the text and
  the field shows it (`story:text-functions-compute-values`); the reference says so and `ess ui
  check` answers a joined string with a finding that names the idiom. A conflicts with the
  Expressions section above. **Recommended: B.** Open, the coordinator's. Architecture of the
  expression language: the owner's.
- **XT-D2. What does a list field of records show for each item?** (A) A text expression over each
  item, the form of XT-D1 A with a root for the item. (B) The list field names the members of each
  item that show, beside `separator` of the active list story; no computation. (C) The view
  reduces the list to text in the model, if the model language can, or a host slot. **Recommended:
  B.** Open, the coordinator's.
- **XT-D3. Is how a member reads inside an item (`name: value`, or the name and the value without a
  colon) a renderer convention or a key of the document?** (A) A renderer convention, as the active
  structured-value story fixes it; the document picks members and the join only. (B) A key of the
  document. B is about looks and is not adopted. **Recommended: A.** Open, the coordinator's.
