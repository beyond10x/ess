---
format: aep.planning-md/3
id: story:a-node-draws-text-written-in-markdown
kind: story
status: draft
title: A node draws text written in Markdown, read-only
summary: 'ess-ui/1 draws no Markdown: text has four styles and rich_text is an input (a consumer''s UI report, item 7)'
tags:
- backlog
- ess-ui
- external-request
- feature-request
- from-consumer-report
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
revision: 1
---
## Need

A page shows text that somebody wrote with structure: a help text, a release note, the answer of an
assistant, the description of a ticket. The stored value is Markdown, and the reader expects
headings, lists, emphasis, links and code as such, not the source characters.

`ess-ui/1` has no node that draws it. The `text` primitive has four styles (`body`, `caption`,
`heading`, `mono`) and value formats for numbers, money and time
(`website/docs/reference/ess-ui.md`, section text). `rich_text` is an input: its `syntax` is one of
`plain`, `expression`, `ssml`, `json`, `curl`, it `binds` a draft field (section rich_text), and the
React project draws a `textarea` for it, as a section
(`crates/ui/ess-ui-react/templates/runtime/composites/rich_text.tsx.tmpl`) and as a field
(`templates/runtime/fields.tsx.tmpl`, case `rich_text`). So a consumer's document shows Markdown
source as plain text or leaves the node to a host slot.

Evidence: a consumer's UI document (2026-11-13, ess 0.94.0), item 7 of its collected report.

Base document for the reproductions (ess 0.94.0, brand-free): one entity `Request` with a
`Pending` state, a view `PendingRequests` (fields `id`, `title`, `state`), a page `requests.inbox`.
`ess ui check --model` on the base: `0 error(s), 0 warning(s)`.

Reproduction 1, a record section with `fields: [{field: title, as: markdown}]`:

```
error enum_values pages/requests.inbox/sections/help/fields/title: `markdown` is not a value of `as`; the schema's `Field.as` lists audio, badge, bytes, choice, color, cron, date, date_range, duration, file, json, link, list, map, number, percent, rich_text, secret, tags, text, time, timestamp, toggle
```

Reproduction 2, a widget body node
`{name: body, primitive: text, text: "**Read** the [guide](guide)", format: markdown}`:

```
error document_loads widgets/note/body/body: data did not match any variant of untagged enum TextFormat
```

Reproduction 3, a section `component: rich_text` with `syntax: markdown`:

```
error document_loads pages/requests.inbox/sections/help: `rich_text`: data did not match any variant of untagged enum RichTextSyntax
```

The same record field with `as: rich_text` checks clean (`0 error(s), 0 warning(s)`) and is the
input control.

## What a design has to settle

- Where it stands: a value of `Field.as`, a format or a style of the `text` primitive, a primitive of
  its own, or a read-only mode of `rich_text`; and whether one key serves a literal text of the
  document and a field of a row.
- Which Markdown: the subset every renderer draws (headings, lists, emphasis, links, code), and what
  happens to the rest. Raw HTML in the value is not drawn as HTML: the value comes from a row and is
  not trusted.
- Where a link in the text may lead: a page of the document, an address outside it, and what a
  guard and `ess ui check` say about either.
- What the terminal renderer draws (structure in cells, or the source), what `degrades` says where a
  renderer has no markup (`no_rich_text` exists for the input), and what `ess-ui-test/1` reads at the
  node: the drawn text without the markup characters.
- The two load errors above name a Rust type (`untagged enum TextFormat`, `RichTextSyntax`) and not
  the values the key takes, as the `enum_values` finding of reproduction 1 does. Whether that is
  fixed with this story or is a defect of its own is decided in the fit review.

## Acceptance (sketch, to be fixed by the fit review)

- A document that shows a row's Markdown field with the chosen key checks clean; the React project
  draws a heading, a list, emphasis, a link and code for it and draws no HTML element the value
  names; the terminal draws the same text readable.
- `expect: {at: <the node>, text: …}` reads the drawn text in both renderers.
- A value of the key outside the schema is reported with the values the key takes.

## Fit review

Owed. This draft records the need and its reproduction only. It is an external request: the seven
questions of `.agents/skills/assessing-external-requests/SKILL.md` are answered here before the
story is accepted or dispatched, and the design below the need is chosen there, not in this draft.
