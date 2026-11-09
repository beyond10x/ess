---
format: aep.planning-md/3
id: story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed
kind: story
status: draft
title: A record groups its fields under headings, and a group or a section may start collapsed
summary: 'FormGroup gains collapsed:, record gains groups: (UI adopter composition report, P4)'
tags:
- backlog
- external-request
- zero-open-issues
refs:
- provider: github
  reference: beyond10x/ess#723
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 5
---
## Need

Long forms, records and pages group what they show under headings, and a rarely used group or
section starts closed ("Advanced", "History"). A form can already group fields under a heading:
`groups:` with a `label`, saved with the form (`crates/ui/ess-ui/src/model.rs:2636-2656`). Three
things are missing on ess 0.92.0:

- a form group cannot start collapsed: `collapsible` on a form group is refused, `unknown field
  collapsible, expected one of name, label, fields, save, does, actions, endpoint, help`;
- a read-only `record` has `fields`, `tabs` and `item` but no headed groups (`model.rs:1392-1408`);
- a section cannot start collapsed: `collapsible: true` on a section is refused,
  `` `record`: unknown field `collapsible`, expected one of `reads`, `fields`, `tabs`, `item`, `actions` ``
  (`repro/run.txt`, variant `g24-collapsible-section`; #723 G24).

Evidence: a UI adopter's collected report (2026-11-08) counts 8 lines for a field group with a
collapsible heading, among them a record's cards grouped under three headings; #723 G24 asks for a
collapsible section.

## Acceptance

- `ess ui check` accepts `collapsed: true` on a form group, on a record group and on a section, and
  `groups:` on a record; it refuses a save key (`save`, `does`) on a record group by node path, and
  `collapsed` on a section without a `title` (nothing to click) by path.
- React draws a `collapsed: true` group or section closed under its heading, opens it on click and
  on Enter, and still submits a collapsed form group's fields with the form. A collapsed section
  with `load: on_visible` starts its read when opened.
- React and the terminal renderer draw a record's groups under their labels, in order; the terminal
  draws a collapsed group or section as its heading line, opened by a key.
- A document without any of the keys keeps its generated React project byte-identical.

## Scope

- cited: `crates/ui/ess-ui/src/model.rs`
- inferred: `schemas/ui/ess-ui.schema.yaml`
- inferred: `crates/ui/ess-ui-check/src/rules.rs`
- inferred: `crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl`
- inferred: `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl`
- inferred: `crates/ui/ess-ui-tui/src/view.rs`
- inferred: `website/docs/reference/ess-ui.md`
- confidence: medium

## Fit review

Tree `995e29579d`, binary ess 0.92.0. Covers #723 item G24 and the 2026-11-08 report this story
was first drafted from.

1. **Need.** Fields of a form or record, and sections of a page, shown under headings, a group or
   section optionally starting closed. Reproductions above. Proposed syntax, the requesters':
   `{group: {title, collapsible: true, open: true, fields: [...]}}` and "a collapsible section".
2. **Class.** Headings in a form: *convenience*, already `groups:` with `label`. Collapse (group and
   section), and headings in a record: *gap* in presentation.
3. **Already expressible?** Form headings: yes (`groups`, `save: with_form` is the default,
   `schemas/ui/ess-ui.schema.yaml` FormGroup). Collapse and record groups: no (runs above). A
   section with `load: on_demand` draws a Load button, not a heading that opens and closes.
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). One key,
   `collapsed: true`, on every container that has a heading: form group, record group, section (the
   siblings where the same question arises; a tab is already hidden until chosen, an overlay is
   closed until opened, so neither needs it). Reuse `FormGroup` rather than a new `group` node:
   `label` is the heading (not `title`); a section's heading is its existing `title`.
5. **Second adopter.** An account settings page with "Profile", "Security" and a collapsed
   "Advanced" group; an order page whose "History" section starts closed.
6. **Cost.** One optional key on `FormGroup` and `Section`, one optional list on `Record`; additive
   in `ess-ui/1`; no migration.
7. **Considered.** (a) Change nothing: headings only in forms, never collapsed. (b) The requester's
   `group` node with `title`/`open`, and `collapsible` on a section: two spellings of one idea. (c)
   Chosen: `collapsed:` on every headed container, `groups:` on a record.

## Decisions

**Accept, redesigned.** Headings in a form are answered with the existing `groups:` idiom; the gap
part is `collapsed:` on a form group, a record group and a section, and `groups:` on a record,
spelled with the existing `FormGroup` and `Section.title` vocabulary instead of the requesters'
`group`/`title`/`open`/`collapsible`. No new format version.

## Source

`github:beyond10x/ess#723`; the 2026-11-08 collected UI report.
