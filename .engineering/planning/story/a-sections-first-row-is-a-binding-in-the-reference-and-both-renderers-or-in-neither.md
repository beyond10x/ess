---
format: aep.planning-md/3
id: story:a-sections-first-row-is-a-binding-in-the-reference-and-both-renderers-or-in-neither
kind: story
status: draft
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
revision: 1
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

One of the two, decided by the fit review:

A. `section.<name>.row` is part of the language: the reference lists it and says what it is (the
   first row the section shows; absent while the section loads, is empty or has no read), both
   renderers resolve it for every section with a read, and the check admits it.
B. It is not: the React runtime no longer exposes `row` under the `section` root (the header title
   reads its record another way), and the check refuses `section.<name>.row`.

Either way:

1. The same document renders the same bound value in the React and the terminal renderer; a test
   holds one binding of this form against both.
2. The reference's table of `section.<name>` members is the list the check enforces.

## Fit review

Not done (`.agents/skills/assessing-external-requests/SKILL.md`). It decides A or B: whether "the
record a detail page shows" should be reachable from another section or the header by a binding
(a detail page's sections and header actions conditioned on the page's record, see also
`story:a-visible-expression-resolves-its-row-and-actor-paths-wherever-an-action-stands`, second
point), or whether a page-level read is the form for that.

## Evidence

Source lines named above; probe output in session scratch `ess/adopter9/` (not in the repository).
