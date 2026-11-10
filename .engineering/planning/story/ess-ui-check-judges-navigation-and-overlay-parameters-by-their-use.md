---
format: aep.planning-md/3
id: story:ess-ui-check-judges-navigation-and-overlay-parameters-by-their-use
kind: story
status: active
title: ess ui check judges navigation and overlay parameters by their use
summary: A nav link to a page needing an id passes; a param bound only into a command is called unread (workbench report items 6, 10)
tags:
- backlog
- defect
- external-request
- from-consumer-report
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ui-component-diagnostics-cite-their-own-view-and-reader
scope:
- confidence: cited
  path: crates/ui/ess-ui-check/src/lib.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/names.rs
- confidence: cited
  path: crates/ui/ess-ui-check/src/rules.rs
- confidence: cited
  path: crates/ui/ess-ui-check/tests/checks.rs
- confidence: inferred
  path: schemas/ui/ess-ui.schema.yaml
- confidence: inferred
  path: website/docs/reference/ess-ui.md
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:43:49Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:43:50Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Two `ess ui check` answers disagree with what the document does (items 6 and 10 of the downstream
gap report of 2026-11-09). Reproduced on 0.92.0 with the report's minimal examples `ui06` and
`ui10` (`.engineering/repro/workbench-2026-11-09/examples/`).

**A page that requires a parameter is accepted in navigation.** `ui06` lists `items.detail`, a
`detail_page` with `params: {id: string}`, in `navigation.sections[].pages`:

```yaml
navigation:
  home: items.list
  sections:
  - {name: main, label: Main, icon: home, pages: [items.list, items.detail]}
pages:
  items.detail:
    kind: detail_page
    params: {id: string}
    # ...
```

```
$ ess ui check --path ui/gap.ui.yaml                 # exit 0
ui/gap.ui.yaml: 0 error(s), 0 warning(s)
$ ess ui check --path ui/gap.ui.yaml --model spec    # exit 0
ui/gap.ui.yaml: 0 error(s), 0 warning(s)
```

The React menu link is built with `hrefFor(link.page, link.params ?? {})`
(`crates/ui/ess-ui-react/templates/runtime/navigation.tsx.tmpl:116`), so it opens the page with no
id. The navigation checks (`crates/ui/ess-ui-check/src/rules.rs:496`, `nav_resolves`, `nav_unique`)
look only at whether the name resolves.

**A form overlay's parameter that the form sends is called unread, as an error.** `ui10` opens a
form overlay from a row action, passing `params: {id: row.id}`, with `does: gap.demo.FinishItem`
(input `id`) and `fields: []`:

```
$ ess ui check --path ui/gap.ui.yaml                 # exit 1
error overlay_params pages/items.list/overlays/finish: `params.id` is passed to this overlay and read nowhere inside it; it reads no parameter
```

Both renderers send every overlay parameter as command input: React's form submits
`{ ...params, ...input }` (`crates/ui/ess-ui-react/templates/runtime/composites/form.tsx.tmpl:104,
121, 131, 148`) and its confirm `{ ...params, ... }` (`composites/confirm.tsx.tmpl:101, 104`); the
terminal does the same (`crates/ui/ess-ui-tui/src/app.rs:3448-3453`, "Like React's
{...params, ...draft}"). `overlay_params` (`crates/ui/ess-ui-check/src/names.rs:300`) counts only
`params.<key>` text found in the overlay's body, frame and title (`read_params`, :342), so the one
use the renderers make of the parameter is not seen, and the finding is an error that fails the
check.

## Acceptance

- `ess ui check` on `ui06` exits 1 with an error at `navigation/sections/main/pages` naming
  `items.detail` and its parameter `id`. `navigation.home` naming such a page is the same error.
  Listing the page under `navigation.hidden` passes.
- `ess ui check` on `ui10` exits 0 with no `overlay_params` finding: a parameter of an overlay whose
  form or confirm declares `does` counts as read. With `--model`, a parameter that is no input of
  that command and is read nowhere else is still reported.
- The existing `crates/ui/ess-ui-check/tests/checks.rs::overlay_params` case (a record overlay
  passing a misspelt `iten_id`) still reports it.

## Scope

- cited: `crates/ui/ess-ui-check/src/rules.rs` (`navigation`, :496)
- cited: `crates/ui/ess-ui-check/src/names.rs` (`overlay_params` :300, `read_params` :342)
- cited: `crates/ui/ess-ui-check/src/lib.rs` (rule table, :180, a new navigation-parameter rule)
- cited: `crates/ui/ess-ui-check/tests/checks.rs` (`overlay_params` :1486, new cases for both)
- inferred: `schemas/ui/ess-ui.schema.yaml` (the Navigation checks gain the new rule)
- inferred: `website/docs/reference/ess-ui.md` (generated reference, Navigation checks table)

confidence: high

## Fit review

1. **Need.** These are two diagnostics that disagree with what the renderers do. A menu entry
   reaches a page without the parameter its route needs (`navigation.tsx.tmpl:116`), and the check
   calls a parameter unread although both renderers send it to the overlay's command
   (`form.tsx.tmpl:104`, `app.rs:3448-3453`). The minimal reproductions are `ui06` and `ui10`
   above. The report proposed no syntax ("Needs: a check error", "a param bound into `does` counts
   as read").
2. **Class.** Both are defects. `overlay_params` is documented as "every key of an overlay's
   `params` is read inside the overlay" (`names.rs:11-12`), and the renderers do read the key:
   they submit it. The Navigation reference says a page "reached by link or redirect only" belongs
   in `hidden` (`website/docs/reference/ess-ui.md:436`), and a page that needs a parameter can
   only be reached that way.
3. **Already expressible?** Yes for the document. `ui06` should list the page under `hidden`, and
   `ui10` could add a dummy read of `params.id`. Neither answers the diagnostic, which accepts the
   first and refuses the second.
4. **Fit.** No authored surface is added. One check is added to the navigation rules and one
   existing check is narrowed. Every renderer is unaffected. The overlay change follows what both
   renderers already do, so the check and the renderers say the same thing.
5. **A second adopter.** Any document with a detail page and a confirm dialog hits both: a
   `customers.detail` page listed in its menu, and a `delete` confirm overlay passing
   `params: {id: row.id}` to `customers.Delete`.
6. **Cost.** One new diagnostic code. One diagnostic stops firing in a case where it was wrong. No
   format version, no migration, no generated-code change. A document that passed before and lists
   a parameterised page in navigation now fails `ess ui check`, which is the intent.
7. **Alternatives.** (a) Change nothing: the false error forces a dummy read, and the menu keeps a
   broken link. (b) Make the navigation case a warning: rejected, because the link is broken on
   every renderer. (c) Have the menu prompt for the id: new renderer behaviour with no design.
   The chosen design (an error, and counting a sent parameter as read) adds no surface.

The related issues #322 (names resolved inside pages) and #324 (overlay params evaluated without
the opener's row) are closed. This story fixes the `overlay_params` rule that #322 introduced, and
links to both. The overlay half overlaps
`story:ui-component-diagnostics-cite-their-own-view-and-reader` (its acceptance: "A parameter that
only the form submission reads is not flagged by `overlay_params`"). One of the two takes that half,
and the other names it.

## Decisions

- **accept, redesigned**: the report's two "Needs" are taken as defects in the checker. The
  navigation case becomes an error, and a parameter of an overlay whose form or confirm declares
  `does` counts as read. No authored syntax changes.

## Source

`.engineering/repro/workbench-2026-11-09/report.md` items 6 and 10, with examples `ui06` and `ui10`.
