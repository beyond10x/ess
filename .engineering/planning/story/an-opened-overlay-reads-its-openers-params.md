---
format: aep.planning-md/3
id: story:an-opened-overlay-reads-its-openers-params
kind: story
status: draft
title: An overlay's params values are read where the action that opens it stands, in the check and in both targets
summary: 'A defect since 0.96.0: ess ui check judges params.<name> of an overlay''s params values against the overlay''s own params, and the terminal and React targets send an empty value; fixed by 6b3c131773 (OP-D1 A)'
tags:
- defect
- ess-ui
- regression
relations:
- serves: vision:O2
- informed_by: story:ui-check-resolves-the-root-and-section-of-every-binding-path
revision: 1
---
## Defect

An overlay's `params:` values are "values passed by the opener" (`schemas/ui/ess-ui.schema.yaml:757`), and the targets and the check
read them as if they stood inside the overlay. Since 0.96.0 `ess ui check` judges each `params.<name>` of an overlay's `params:`
values against the overlay's own params, so an overlay opened from another place cannot pass on a value that place holds (`expr_paths`
refuses it: "in scope: `parent_id`"), and the terminal and React targets send an empty value for it.

## Reported case

Measured 2026-11-25 on `4255c8b431` (queue/0.99.1), a page whose row action opens an overlay with a `params:` value
`params.<name>`:

- `ess ui check`: `expression_paths` 2 cases fail with the message "in scope: `parent_id`";
- terminal target (`ess-ui-tui`, `tui` 1 case): the opener sends `parent_id: Null`;
- React target (`ess-ui-react`, `overlay_params` 1 case): the confirm sends `{}`.

The check was added by `63285479e5` (2026-11-13, first released in 0.96.0) with
`story:ui-check-resolves-the-root-and-section-of-every-binding-path`, which has no line on an overlay's own params; 0.94.0 and 0.95.0
have no `expr_paths`; 0.98.2 refuses the same case.

## Fit review

1. **Need.** An overlay is the unit that receives its opener's values; its `params:` map is the only place they are written (an Action's
   `opens:` names the overlay only, `ess-ui.schema.yaml:1314`).
2. **Class.** A defect: the reference decides the scope (the opener's: `ess-ui.schema.yaml:757`, `website/docs/reference/ess-ui.md:60`
   and `:1165`, `ess-ui.schema.yaml:1805`'s `must_resolve_in`: "in scope where the expression stands", and the value stands at the opener).
3. **Expressible today.** No: the value cannot be written.
4. **Fit.** The reference needs no change beyond one rendered line; the check, the terminal target and the React target evaluate the
   values where the opener that ran stands.
5. **Existing stories.** `story:ui-check-resolves-the-root-and-section-of-every-binding-path` borders: it introduced the check and
   says nothing on an overlay's own params. None covers the defect.
6. **Release train:** 0.99.x patch (a defect of a shipped release; no format and no schema change).

## Decisions

- OP-D1: A, taken by the coordinator on 2026-11-25 (D112). Every opener: `ess ui check` judges each `params.<name>` of an overlay's
  `params:` values at every action that opens it, and reports each opener that lacks the name, naming that opener's path
  (fail-closed). B (the intersection, reported once), C (a `params:` on the opening action: a schema change) and D (the union: not
  fail-closed) were not taken. A is reversible; B and C only add to it.

## Acceptance

- an_opener_passes_what_it_holds: a page whose row action opens an overlay with the value `params.<name>` passes `ess ui check`, and
  the terminal and React targets send that value.
- every_opener_is_judged: an overlay opened from two places, one of which lacks the name, is refused naming that opener's path.
- an_unopened_overlay_is_held_to_the_page: an overlay nothing on its page opens is held to the page's parameters.
- the_reference_says_it: the rendered reference line states that the values are read where the opener stands.

## Delivery

Commit `6b3c131773` (fix(ui): an overlay's params are read where its opener stands) on
`fix/an-opened-overlay-reads-its-openers-params`: three red cases made green (`ess-ui-check` `expression_paths` 4 passed, `ess-ui-tui`
`tui` 1 passed, `ess-ui-react` `overlay_params` 2 passed), the `ess-ui*` crates, `tutorial_page`, `generate_ui_http` and `test-xtask`
green, `fmt-check` and clippy clean; `CHANGELOG.md` has the entry under Fixed.

## Not covered

- `state.<x>` in an overlay's `params:` values, and shell overlays: see
  `story:an-overlay-s-params-values-that-read-state-or-belong-to-a-shell-overlay-are-read-where-their-opener-stands`.

## Scope

- cited: `crates/ui/ess-ui-check/src/sites.rs`, `crates/ui/ess-ui-react/src/emit.rs`, `crates/ui/ess-ui-react/templates/runtime/overlays.tsx.tmpl`,
  `crates/ui/ess-ui-tui/src/app.rs`, `schemas/ui/ess-ui.schema.yaml`, `website/docs/reference/ess-ui.md`
