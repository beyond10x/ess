---
format: aep.planning-md/3
id: story:a-record-groups-fields-under-headings-and-a-group-may-start-collapsed
kind: story
status: draft
title: A record groups its fields under headings, sections may share one frame, and any headed group may start collapsed
summary: 'FormGroup gains collapsed:, record gains groups: (UI adopter composition report, P4); a layout column or area may draw its sections in one shared frame (#723 G25).'
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
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: cited
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-tui/src/view.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/model.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 10
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


### #723 G25: sections share one frame


A reader of a generated page sees sections that belong together, such as a customer's profile and
its open orders, drawn in one frame under one heading. Each section still loads, fails and empties
on its own.


Reproduced on ess 0.92.0 over the triage's minimal document (`i723-gap/g25/run.sh`, output in
`g25/run.txt`).

- Every section draws its own frame: `SectionFrame` renders one `<section class="ui-section">` per
  section (`crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl:401-491`, `:476`), and
  `.ui-section` carries the border, background and radius
  (`crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl:54`). The terminal draws one
  `Block::bordered()` per section (`crates/ui/ess-ui-tui/src/view.rs:335-378`).
- A layout groups sections but draws no frame around them. A column or an area already lists
  several sections (`crates/ui/ess-ui/src/model.rs:685-692` `LayoutColumn.sections`, `:720-725`
  `Areas.place: BTreeMap<String, Vec<String>>`). They are drawn as an unframed flex column
  (`styles.css.tmpl:48`, `:52`; `core.tsx.tmpl:316-343`). Two sections placed in one column
  generate one `LayoutColumn` holding two separate `SectionFrame`s (`run.txt` § 1).
- No key frames them together. `frame:` on a column, or a place entry written as a record, is
  refused (`error document_loads pages/orders.cards: data did not match any variant of untagged
  enum PageLayout`, `run.txt` § 2).
- One section holding the other parts as nested nodes (`record` with `item:` composites, each with
  its own `reads`) checks and draws one frame. But a nested composite cannot carry what a section
  declares for its part: `states` and `title` are refused
  (`` `collection`: unknown field `states` ``, `` unknown field `title` ``, `run.txt` § 3), and so
  are `load`, `live` and `depends_on`, which only a section has
  (`crates/ui/ess-ui/src/model.rs:750-771`). A nested read that fails draws no failed state:
  `Collection` and `MetricView` have no status branch
  (`templates/runtime/composites/collection.tsx.tmpl`, `metric.tsx.tmpl:36-39` prints `–`).

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

- `ess ui check` accepts `frame: shared` with an optional `title` on a layout column, and on an
  area's place entry written as `{sections: [...], frame: shared, title: ...}`. The list form
  stays the short form and means `frame: each`, so the default draws what is drawn today. It also
  accepts `collapsed: true` on a shared frame with a `title`. It refuses each of these by node
  path: a `frame` value other than `each` or `shared`; `title` or `collapsed` beside `frame: each`;
  `collapsed` without `title`; and a shared frame whose sections are not consecutive in the page's
  section order, naming the first section out of order. Observed by `ess ui check` over the
  `shared-column` and `shared-area` variants of `g25/run.sh` (exit 0) and by
  `cargo test -p ess-ui-check --test checks` for each refusal.
- React draws a shared frame as one framed element carrying the column's path
  (`pages/<page>/layout/columns/<name>`) or the area's (`pages/<page>/layout/areas/place/<area>`),
  with its `title` as the heading. It holds its sections in order. Each section keeps its own read
  states (loading, failed, forbidden, empty) and its own `title` as a subheading, and draws no
  frame of its own. A collapsed shared frame starts closed, opens on click and on Enter, and
  starts each `load: on_visible` read inside it when opened. Observed by a generator test in
  `crates/ui/ess-ui-react/tests/` that asserts one frame element at the column's path holding two
  `SectionFrame`s marked as framed, and a failed read in one of them drawing `Could not load.`
  while the other draws its rows.
- The terminal draws a shared frame as one bordered block titled by its `title`, its sections'
  lines under their own titles, at the position of its first section in the stack. A collapsed
  frame draws its heading line, opened by a key. Observed by a `crates/ui/ess-ui-tui` view test on
  a rendered buffer.
- A document without `frame` keeps its generated React project byte-identical. This is the story's
  existing byte-identity bullet, widened by the one key.

## Scope

- cited: crates/ui/ess-ui/src/model.rs
- cited: schemas/ui/ess-ui.schema.yaml
- cited: crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl
- cited: crates/ui/ess-ui-tui/src/view.rs
- cited: crates/ui/ess-ui-react/templates/runtime/styles.css.tmpl
- cited: crates/ui/ess-ui-react/src/emit.rs
- inferred: crates/ui/ess-ui-check/src/rules.rs
- inferred: crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl
- inferred: crates/ui/ess-ui-react/templates/runtime/composites/record.tsx.tmpl
- inferred: website/docs/reference/ess-ui.md
- inferred: crates/ui/ess-ui-check/tests/checks.rs

confidence: medium

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


### #723 G25

Tree `3bd1b6e9d2`, binary ess 0.92.0. Covers #723 item G25. G24 and the field groups keep the
review above.

1. **Need.** Several sections, each with its own read and its own states, are shown as one framed
   group under one heading. Reproduction above. The requester's words, theirs: "two sections cannot
   share one card".
2. **Class.** *Gap* in presentation. A layout places sections together but cannot frame them, and
   the one workaround drops the per-part states (§ 3).
3. **Already expressible?** Partly. One section holding the other reads as nested `item:`
   composites draws one frame and checks clean. It is no answer, because each part loses what a
   section declares and the reader sees: its empty and failed text, its heading, `load`, `live`
   and `depends_on` (Evidence). A failed part then looks like an empty one, which is a figure moved
   out of what the document can say (`docs/design/generic-language-features.md` § Policy,
   policy 1).
4. **Fit.** The language grids do not apply (`docs/design/language-grids.md:15`, `:63`). The
   frame lives on the construct that already groups sections, the layout's column and area, and
   not on a new grouping node. The heading is `title`, as on a section and a page. A framed group
   with a heading takes `collapsed:` by this story's own rule ("`collapsed: true` on every container
   that has a heading"), so the merge keeps one rule for every headed container. Siblings: a column
   and an area both take it; `stack` has no entry to put it on, and a stack page writes a one-column
   `areas` grid instead. Renderers: React draws it. The terminal, which stacks in section order
   (`crates/ui/ess-ui-tui/src/lib.rs:9-10`), draws it as one block, which is why the framed
   sections must be consecutive. A nested composite gaining `states` was not taken: it would make
   every composite a second unit of loading beside the section
   (`website/docs/reference/ess-ui.md:795`, "The section, not the page, is the unit of loading").
5. **Second adopter.** A customer page whose "Customer" frame holds the profile record and the
   open-orders list, each reading its own view. A dashboard whose side column is one frame of three
   metrics, one of which may fail alone.
6. **Cost.** One optional key and one optional heading on `LayoutColumn`, one record form for an
   area's place entry (the list stays its short form), additive in `ess-ui/1`. No source format
   change (`ess/37` untouched), no conformance change (`ess-conformance/64`, `65` untouched), no
   migration.
7. **Considered.** (a) Change nothing and answer with nested `item:` composites: loses per-part
   states. (b) A new `group` node holding sections: a second grouping beside the layout, and a page
   could nest groups. (c) `states`, `title` and `load` on every nested composite: a second unit of
   loading. (d) Chosen: `frame: shared` on the layout's own groups, merged here because the heading
   and the collapse rule are this story's, and because the story already owns the files the frame
   touches (`model.rs`, the schema, `core.tsx.tmpl`, `view.rs`), so two stories would be serialized
   on them anyway.

## Decisions

**Accept, redesigned.** Headings in a form are answered with the existing `groups:` idiom; the gap
part is `collapsed:` on a form group, a record group and a section, and `groups:` on a record,
spelled with the existing `FormGroup` and `Section.title` vocabulary instead of the requesters'
`group`/`title`/`open`/`collapsible`. No new format version.


### #723 G25

**Accept, redesigned, merged.** G25 lands in this story as `frame: shared` with `title`, and
optionally `collapsed`, on a layout column and an area's place entry, instead of a construct of the
requester's. The nested-composite workaround is recorded as no answer (policy 1). No new format
version. The story's title widens to the one above.

A refusal message noticed while reproducing is not part of G25: an unknown key under `layout` is
reported as `data did not match any variant of untagged enum PageLayout`, which names neither the
key nor the place. The first bullet's refusals must name the key, so this story replaces that
message for the keys it adds.

## Source

`github:beyond10x/ess#723`; the 2026-11-08 collected UI report.

