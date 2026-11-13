---
format: aep.planning-md/3
id: story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither
kind: story
status: active
title: A section's first row is a binding in the reference and both renderers, or in neither
summary: The reference lists section.<name>.selection only; the React runtime also exposes section.<name>.row for the header's title section, the terminal renderer never does (0.94.0, adopter report)
tags:
- backlog
- defect
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:27:27Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
- {from: "proposed", to: "active", at: "2026-11-12T21:27:28Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-accept-16"}
---
## Defect

The reference lists one member of a section a binding may read, `section.<name>.selection`
(`website/docs/reference/ess-ui.md`, Expressions). The generated React application also resolves
`section.<name>.row`, the first row the section shows, for some sections and not for others, and
the terminal renderer never resolves it. A document that binds `section.<name>.row.<field>` works
in one generated application, comes out empty in the other renderer and in a renderer an adopter
wrote from the reference, and `ess ui check` accepts it.

Reported by an adopter on 0.90.0 (2026-11-12): its own renderer built `section.<name>` from
selections only, and every such binding was empty (a read without its parameter, a link without a
target). Read from the source of 0.94.0 (main `9f8141cb8b`):

- React: `SectionInfo` in `crates/ui/ess-ui-react/templates/runtime/core.tsx.tmpl` has a member
  `row`, "the first row, reported by the section a header title reads its record from". A section
  reports it only with `reportsRow`, which `section_component` in
  `crates/ui/ess-ui-react/src/emit.rs` sets where the page header's title reads that section
  (`titles_header`) and the section has a read. The page scope passes the whole registry as
  `section`, so `section.<name>.row.<field>` resolves for that one section and is undefined for
  every other.
- Terminal: the `section` root in `crates/ui/ess-ui-tui/src/app.rs` builds a mapping with the one
  key `selection`.
- Check: `section.<name>.row.<field>` and `section.<name>.<anything>` pass with no finding
  (`story:ui-check-resolves-the-root-and-section-of-every-binding-path`).

## Acceptance

Form A, decided by the fit review (2026-11-12). `section.<name>.row` is part of the language.

1. The reference's table of expression forms lists `section.<name>.row`: the first row of a
   section that reads; absent while the section loads, when it is empty, and for a section without
   a read. `title_from` and this form are described with the same words.
2. Both renderers resolve it for every section with a read. The same document renders the same
   bound value in the React and the terminal renderer; a test holds one read parameter and one
   navigate parameter of this form against both.
3. `ess ui check` admits `row` beside `selection` under `section.<name>`. It refuses
   `section.<name>.row` where the section reads nothing, as `header_record` does for `title_from`.
   With `--model` it refuses a `<field>` the rows of the section's view do not carry, as it does
   for `title_from.field`.
4. `ess ui check` refuses a read whose params read its own section's row, and a cycle of sections
   that read each other's row, naming the sections.
5. `header.title_from` keeps its behaviour in both renderers.
6. The reference's table of `section.<name>` members is the list the check enforces
   (`story:ui-check-resolves-the-root-and-section-of-every-binding-path`).

## Fit review

Source read at main `582524ae95` (0.94.0 line), 2026-11-12.

1. **Need.** A page shows one record in a section, a read that answers one row. Another node of
   the page (a sibling section's read parameter, a link's target, a `visible` condition) needs a
   field of that record that the page's path does not carry. The requester proposed no syntax. It
   reported that its bindings `section.<name>.row.<field>` were empty in a renderer built from the
   reference: a read without its parameter, a link without a target. Minimal reproduction: the
   Defect section above; `ess ui check` on `section.deals.row.id` answers 0 errors (the table in
   `story:ui-check-resolves-the-root-and-section-of-every-binding-path`).
2. **Class.** Defect: the reference lists one member, `section.<name>.selection`
   (`website/docs/reference/ess-ui.md:65`); React resolves `row` for one section of a page
   (`core.tsx.tmpl:178`, `:527`; `emit.rs:2408-2431`); the terminal resolves `selection` only
   (`crates/ui/ess-ui-tui/src/app.rs:4527-4565`). One document, two values. Behind it is a gap: no
   listed form reaches a field of the record another section shows. `params.<name>` carries path
   params (`ess-ui.md:60`); `section.<name>.selection` is a collection's selected row (the terminal
   computes it for `Composite::Collection` only, `app.rs:4545-4562`).
3. **Already expressible?** Not for a binding. The concept exists for one reader:
   `header.title_from: {section, field}` is "a field of the first row of a section that reads"
   (`ess-ui.md:1097`; `crates/ui/ess-ui/src/model.rs:1840-1845`). Both renderers draw it (React
   `core.tsx.tmpl:658`; terminal `crates/ui/ess-ui-tui/src/view.rs:475-481`) and the check holds it
   (`header_record`, `crates/ui/ess-ui-check/src/rules.rs:1122-1140`; the field against the view's
   row keys, `crates/ui/ess-ui-check/src/model.rs:754-758`). No idiom gives a binding that record.
4. **Fit.** The language grids (`docs/design/language-grids.md`) cover the specification language
   and have no row for the binding expressions of a UI document (no match for `ess-ui` in that
   file), so the triage is on the reference's table of expression forms (`ess-ui.md:57-70`).
   - Vocabulary: form A reuses the concept and the words of `title_from` (the first row of a
     section that reads) and the spelling `row` of `row.<field>` (`ess-ui.md:62`).
   - Siblings: every section with a read, not only the section the header's title names.
   - Composition: every expression site (read params, navigate params, `visible`, guards); the
     check story lists them.
   - Targets: React (every section with a read reports its first row; `reportsRow` goes), the
     terminal (`row` beside `selection`, with the guard that keeps a section's own params from
     reading itself, `app.rs:4548-4557`), the check (members `selection` and `row`; with `--model`
     the `<field>` against the view's row keys, as `title_from.field`), the reference table,
     `ess ui test`.
   - A section whose read params read its own row, or two sections that read each other's row,
     never settle: the check refuses both by name.
   - Form B closes the divergence and leaves the gap of point 2; closing that gap then needs a new
     construct (a page-level read), which is more surface than one member of an existing root.
5. **Second adopter.** An order page: section `order` reads the order by id; section `customer`
   reads the customer with `id: section.order.row.customer_id`; a link opens the invoice with
   `invoice_id: section.order.row.invoice_id`.
6. **Cost.** One expression form in the reference; no new key and no new composite. Whether
   `ess-ui/1` moves for an added expression member: I don't know; the implementor follows how
   `actor.<field>` was added. Generated React changes for every section with a read. A document
   that binds `section.<name>.row` on the title's section keeps its value in React and gains it in
   the terminal; no migration. A document that binds another member becomes an error, under the
   check story.
7. **Alternatives.** (a) Change nothing: two renderers, two values; refused. (b) Form B: removes
   the divergence, keeps the gap, refuses documents in use, and its replacement is a new construct.
   (c) Form A: chosen. (d) `section.<name>.selection` of a record section is its row: gives
   `selection` a second meaning against `ess-ui.md:63` and `:65`; refused.

## Evidence

Source lines named above; probe output in session scratch `ess/adopter9/` (not in the repository).

## Decisions

**Accepted by the operator (2026-11-12),** with fifteen other drafts from one list of 25 that
he was shown with a recommendation and an outside review per draft. He accepted the drafts both
recommended. The two transitions carry the correlation `operator-2026-11-12-accept-16` and the executor
`agent:b10x`. That acceptance left the choice between A and B to the fit review; the recommendation
shown to the operator was A.

**Fit review (2026-11-12): accept as proposed, form A.** `section.<name>.row` joins the language
as the binding form of the record `header.title_from` already reads. Form B is refused: it leaves a
field of another section's record unreachable and needs a new construct to close that. The review
added acceptance points 3 (a section that reads nothing; the field against the view) and 4 (a
section reading its own row, and cycles). The requester is told in one message before the work
starts.
