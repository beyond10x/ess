# Collapsed groups, record groups and shared frames (`ess-ui/1`)

Story `a-record-groups-fields-under-headings-and-a-group-may-start-collapsed`, beyond10x/ess#723
G24 and G25. The construct, its checks and what each renderer does with it. Additive in
`ess-ui/1`; no source format change, no migration.

## The gap

A form groups fields under a heading (`groups:`, `label`). A record cannot, a group or a section
cannot start closed, and sections that belong together cannot share one frame. A section draws its
own frame in React (`SectionFrame`) and one `Block::bordered()` in the terminal; a layout column or
an area lists several sections and draws no frame around them.

## The construct

One rule for every container that has a heading: `collapsed: true` starts it closed.

```yaml
sections:
  - {name: history, title: History, collapsed: true, component: collection, reads: {view: t.History}}
  - name: profile
    component: record
    reads: {view: t.ById}
    groups:
      - {name: contact, label: Contact, fields: [email, phone]}
      - {name: audit, label: Audit, collapsed: true, fields: [created_at, created_by]}
layout:
  columns:
    - {name: side, frame: shared, title: Customer, collapsed: true, sections: [profile, orders]}
  # or: areas: {grid: [[a]], place: {a: {frame: shared, title: Customer, sections: [profile, orders]}}}
```

- `FormGroup.collapsed` and `Section.collapsed` are optional booleans; `Record.groups` is an
  optional list of `RecordGroup {name, label, fields, collapsed}`. A record group is read-only:
  `save`, `does` and the other form group keys are refused at the group's path.
- `LayoutColumn` gains `frame` (`each`, the default, or `shared`), `title` and `collapsed`. An
  area's place entry is a list of sections (the short form, `frame: each`) or a record
  `{sections, frame, title, collapsed}`.
- A shared frame draws its sections in one frame under its `title`. Each section keeps its own
  read, states and `title`, which becomes a subheading, and draws no frame of its own.

## Checks (`ess ui check`)

- `enum_values`: a `frame` other than `each` or `shared`.
- `frame_rules`: `title` or `collapsed` beside `frame: each`; `collapsed` without a `title` on a
  shared frame or a section; a shared frame whose sections are not consecutive in the page's
  section order, naming the first section out of order. Reported at the column's, the area's or
  the section's node path.
- The loader refuses a record group's `save` or `does` at the group's path.

## Renderers

- React: a heading with a button opens and closes a collapsed container (click, Enter); the body
  stays mounted and is hidden while closed, so a collapsed form group's fields still submit with
  the form and a `load: on_visible` read starts when the body is shown. A shared frame is one
  element at the column's or the area's path; its sections are marked `framed`.
- Terminal: a collapsed container is its heading line, `▸` while closed and `▾` while open.
  On the focused section `z` opens its closed frame, then its closed section, and closes the
  outermost of them once both are open; where it has neither it folds the record's or form's
  groups, as `Z` always does (the first closed group opens, and when none is closed they all
  close). The fields of a closed form group are neither drawn nor walked, and are still sent.
  A shared frame is one bordered block titled by its `title`, at the position of its first
  section in the stack, which is why its sections are consecutive; each member keeps a heading
  line of its own inside it.

## Not taken

Nested composites with `states` and `title` (a second unit of loading beside the section); a new
`group` node holding sections (a second grouping beside the layout); `collapsible`/`open`
spellings (two spellings of one idea).
