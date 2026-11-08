---
format: aep.planning-md/3
id: story:a-used-by-list-reads-several-views
kind: story
status: draft
title: A used-by list and a delete confirm read several declared views
summary: references.reads and confirm.references take a list of views, each row marked by its kind; no UI join, a view keeps one source (UI adopter composition report, P6)
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:ess-ui-renderer-neutral-ui
scope:
- confidence: inferred
  path: crates/ui/ess-ui-check/src/model.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/src/emit.rs
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/confirm.tsx.tmpl
- confidence: inferred
  path: crates/ui/ess-ui-react/templates/runtime/composites/references.tsx.tmpl
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

Before a record is deleted, and in a record's usage tab, a user must see everything that still
refers to it. In most systems several kinds of record refer to one: a trigger is used by call flows
and by queues, a tag by contacts and by deals. Each kind is its own entity with its own view. The
`references` composite and `confirm.references` read exactly one view
(`crates/ui/ess-ui/src/model.rs:2062-2070`, `1818`), so a used-by list over two kinds is a
hand-written host component that merges reads in the browser.

Evidence: a UI adopter's collected report (2026-11-08) ranks a "composed read" first with 39 lines.
Of those, 20 host lines are exactly the used-by and references lists (14 + 6), and 3 checklist
lines ask for a used-by list derived from other reads. The rest of the 39 is covered by constructs
that exist (below), so this story claims the 23.

Minimal reproduction (brand-free): views `flows.ByTrigger` and `queues.ByTrigger`, each a view
over its own entity filtered by a `trigger_id` param. `{component: references, reads:
[flows.ByTrigger, queues.ByTrigger]}` is refused by `ess ui check` (ess 0.90.0): `references:
invalid length 2, expected struct Reads`.

## Fit review

Tree `b33454b88`, binary ess 0.90.0.

1. **Need.** One used-by list (and one delete confirm) shows the rows of several declared views,
   each row marked with the kind it came from and linking to it. Requester's proposed syntax,
   labelled as theirs: a UI `views: [{join: {from, with: [{view, match: "<expr>", as}], shape}}]`
   and `confirm: {lists: <join>}`.
2. **Class.** *Gap* for the several-kinds used-by list. The rest of the "composed read" lines are
   *convenience*: a label of a referenced row is `label_from` (`model.rs:2562-2570`); a count of
   another entity's rows is an aggregate view (`docs/design/aggregate-views.md`); the detail of a
   chosen entry is a record reading a `ById` view with the choice as its param.
3. **Already expressible?** For one referring kind, yes: a view over that entity filtered by the
   referenced id, shown by `references`. For several kinds, no: the composite takes one `Reads`
   (run above). A join inside one ESS view is refused by standing decision: a view has one
   `source:` (`docs/design/read-api-view-idioms.md:177-206`, `story:feature-request-447`;
   `aggregate-views.md:1037`); one hop through a reference is already `field: {related: …}`
   (`ess/28`).
4. **Fit.** Concatenating several declared views keeps every row a row of a checked view, so the
   figure the reader sees stays checked (`docs/design/generic-language-features.md` § Policy,
   policy 1); a UI-side `join` with match expressions would compute rows no view declares and no
   suite checks, so it is refused. Vocabulary: `reads` takes one `Reads` or a list of them, as a
   preload already lists `views` (`model.rs:351-375`); each row shows its source's label, and
   `navigates` links each row to its own kind. Siblings: `references`, `confirm.references`, and a
   usage tab holding a `references` node; a `collection` stays one view (paging, sorting and
   selection over a union of sources is a different question, refused by name). Where the
   reference sits inside an untyped `Json` payload, no view filter can reach it; that needs the
   model to type the payload first (see `story:a-form-field-over-a-union-draws-the-chosen-variants-fields`
   for the UI side); whether a view filter reaches a member of a struct field: I don't know.
   Targets: React, terminal, `ess ui check --model` (each view must take the confirm's or the
   page's record id as a param).
5. **Second adopter.** A CRM: deleting a tag lists the contacts and the deals that carry it.
6. **Cost.** `references.reads` and `confirm.references` widen to one-or-list; additive in
   `ess-ui/1`; no ESS source format; a finding for a listed view that takes no matching param.
7. **Considered.** (a) change nothing: a host component per used-by list. (b) a union or join view
   in the ESS model: reopens "a view has one source" for a presentation need and costs a source
   format, synthesis and every target. (c) the requester's UI join: unchecked rows. (d) chosen: a
   used-by list over several declared views.

## Decisions

**Accept, redesigned.** The several-kinds used-by list is adopted as a list of declared views on
`references` and `confirm.references`; the requester's `join`/`match`/`shape` and a union view in
the model are refused, with the reasons in 3 and 4. The remaining composed-read lines are answered
with `label_from`, aggregate views and a `ById` record, and not filed.

## Acceptance

- `ess ui check` accepts `references.reads` and `confirm.references` given a list of views; with
  `--model` it refuses, by node path, a listed view that is not in the model or whose params cannot
  be bound from the page or confirm.
- The React `references` composite shows the rows of every listed view in list order, each with its
  source's label, and each row navigates to its own record when `navigates` is true (a React test
  over fixtures for two views).
- A delete confirm with two listed views shows both kinds' rows before the command runs.
- The terminal renderer shows the same rows in an `ess ui test` case.
- A `references` node with a single `reads` keeps its generated React project byte-identical.
- A `collection` given a list of reads is still refused, naming `references` as the composite that
  lists several views.

## Scope

- `crates/ui/ess-ui/src/model.rs` — `References.reads`, `Confirm.references` (inferred)
- `schemas/ui/ess-ui.schema.yaml` (inferred)
- `crates/ui/ess-ui-check/src/model.rs` (inferred)
- `crates/ui/ess-ui-react/templates/runtime/composites/references.tsx.tmpl`,
  `crates/ui/ess-ui-react/templates/runtime/composites/confirm.tsx.tmpl`,
  `crates/ui/ess-ui-react/src/emit.rs` (inferred)
- `crates/ui/ess-ui-tui/src/view.rs` (inferred)
- `website/docs/reference/ess-ui.md` (inferred)
