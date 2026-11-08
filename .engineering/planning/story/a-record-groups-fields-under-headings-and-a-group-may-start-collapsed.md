---
format: aep.planning-md/3
id: story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed
kind: story
status: draft
title: A record groups its fields under headings, and a form or record group may start collapsed
summary: 'FormGroup gains collapsed:, record gains groups: (UI adopter composition report, P4)'
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
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
revision: 2
---
## Need

Long forms and records group their fields under headings, and a rarely used group starts closed
("Advanced", "After the call"). A form can already group fields under a heading: `groups:` with a
`label`, saved with the form (`crates/ui/ess-ui/src/model.rs:2636-2656`; checked clean by
`ess ui check`, ess 0.90.0). Two things are missing: a group cannot start collapsed (`collapsible`
on a form group is refused: `unknown field collapsible, expected one of name, label, fields, save,
does, actions, endpoint, help`), and a read-only `record` has `fields`, `tabs` and `item` but no
headed groups (`model.rs:1392-1408`).

Evidence: a UI adopter's collected report (2026-11-08) counts 8 lines for a field group with a
collapsible heading (1 host line, 7 checklist lines), among them a record's cards grouped under
three headings.

## Fit review

Tree `b33454b88`, binary ess 0.90.0.

1. **Need.** Fields of a form or record shown under headings, a group optionally starting closed.
   Requester's proposed syntax, labelled as theirs: `{group: {title, collapsible: true, open: true,
   fields: [...]}}`.
2. **Class.** Headings in a form: *convenience*, already `groups:` with `label`. Collapse, and
   headings in a record: *gap* in presentation.
3. **Already expressible?** Form headings: yes (`groups`, `save: with_form` is the default,
   `schemas/ui/ess-ui.schema.yaml` FormGroup). Collapse and record groups: no (runs above).
4. **Fit.** Reuse `FormGroup` rather than a new `group` node: `label` is the heading (not `title`),
   one key `collapsed: true` says it starts closed (a group with the key is collapsible), and
   `record` gains `groups:` of the same shape minus the save keys. Siblings: form, record, and a
   tab's fields; an overlay drawer holds one of those, so it needs nothing. Renderer-neutral: the
   terminal renderer draws a collapsed group as a heading line that expands on a key.
5. **Second adopter.** An account settings page with "Profile", "Security" and a collapsed
   "Advanced" group; an order record grouped under "Customer", "Lines", "Delivery".
6. **Cost.** One optional key on `FormGroup`, one optional list on `Record`; additive in
   `ess-ui/1`; no migration.
7. **Considered.** (a) change nothing: headings only in forms, never collapsed. (b) the requester's
   `group` node with `title`/`open`: a second spelling of `groups`/`label`. (c) chosen: extend
   `FormGroup` and give `record` the same `groups`.

## Decisions

**Accept, redesigned.** Headings in a form are answered with the existing `groups:` idiom; the gap
part is `collapsed:` on a group and `groups:` on a record, spelled with the existing `FormGroup`
vocabulary instead of the requester's `group`/`title`/`open`. Backlog: not scheduled.

## Acceptance

- `ess ui check` accepts `collapsed: true` on a form group and `groups:` on a record, and refuses a
  save key (`save`, `does`) on a record group by node path.
- React draws a `collapsed: true` group closed with its heading, opens it on click, and still
  submits its fields with the form.
- React and the terminal renderer draw a record's groups under their labels, in order.
- A document without either key keeps its generated React project byte-identical.

## Scope

- `crates/ui/ess-ui/src/model.rs` — `FormGroup`, `Record` (inferred)
- `schemas/ui/ess-ui.schema.yaml` (inferred)
- `crates/ui/ess-ui-check/src/rules.rs` (inferred)
- `crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl`,
  `crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl` (inferred)
- `crates/ui/ess-ui-tui/src/view.rs` (inferred)
- `website/docs/reference/ess-ui.md` (inferred)
