---
format: aep.planning-md/3
id: story:feature-request-281
kind: story
status: implemented
title: An ess-ui section has a heading, and a page can omit a section its kind contributes
tags:
- feature-request
- ui-spec
refs:
- provider: github
  reference: beyond10x/ess#281
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-06T09:46:18Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
- {from: "proposed", to: "active", at: "2026-10-06T09:46:19Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}}
- {from: "active", to: "implemented", at: "2026-10-06T09:46:20Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

An `ess-ui/1` section can carry a heading, and a page can do without a section its kind contributes.

## Acceptance

- The two #281 reductions pass `ess ui check`.

## Origin

beyond10x/ess#281 (uilab moving to `ess-ui/1`).

## Priority

UI spec: lower priority than every base-spec story (operator, 2026-10-01).

## Fit review

Per `.agents/skills/assessing-external-requests/SKILL.md` (fit review 2026-10-01; repros under `~/.cache/ess-gaps/fit2/`, run on ess 0.44.0 unless stated).

# Fit review: feature-request-281 (beyond10x/ess#281)

There is no `story:feature-request-281` in the read tree's store or on `origin/main`. `ess` 0.44.0 has no `ui` command (`ess ui --help` → unrecognized), so the reproductions use the already-installed `~/.cache/ess/toolchains/0.48.0/ess` (read-only). Schema citations are from the read tree `e3bc9a2ff`.

1. **Need.** Two items.
   - **Item 1:** a page section cannot carry display text that tells a reader what the region is. Two sections over the same view ("Due this week" / "Overdue") differ only in their node `name`, which is an identifier, not text. Repro `repro-281/r1-title.ui.yaml` → `unknown field title` (0.48.0).
   - **Item 2:** a page cannot drop a section its kind contributes. `dashboard_page` brings `board` (schema `schemas/ui/ess-ui.schema.yaml:490-492`), and `board` requires `reads` (`:787-795`). Repro `r2-board.ui.yaml` → `board: missing field reads`.
   - The requester's draft syntax (`title:` on the section), labelled as theirs, was given as evidence, not as a design.
2. **Class.**
   - Item 1: **gap**. See Q3 for why the workaround is not an idiom.
   - Item 2: **already expressible**, so convenience at most. It is a discoverability problem.
3. **Already expressible?**
   - Item 2: **yes**. A named-list removal, `{name: board, remove: true}`, is defined in the schema (`ess-ui.schema.yaml:112,121,393`; reference `website/docs/reference/ess-ui.md:116`) and tested (`crates/ui/ess-ui/tests/shorthands.rs:52`; implementation `crates/ui/ess-ui/src/expand.rs:932-1002`). Repro `r2-remove.ui.yaml`: `ess ui check` → 0 errors. `ess ui load` → 5 addressed nodes, against 6 with the board kept (`r2-keep.ui.yaml`), so the board is gone.
   - Item 1: **partially**. A widget whose body is `text` with `style: heading` plus the collection works (`r1-widget.ui.yaml` → 0 errors, 0.48.0), but it is a downgrade:
     - The read moves from the section into a widget body, so the section is no longer the unit of loading. Its `reads`, `states` and `live` (`ess-ui.schema.yaml:512-541`) do not apply.
     - The view is fixed inside the widget, so you need one widget per view.
   - `children` cannot serve as a heading, because it renders *after* the composite (`:529`; `crates/ui/ess-ui/src/model.rs:707`).
4. **Fit (item 1).** The proposed `title` fits as written:
   - `overlay` is the sibling construct: "a frame around one member of the composite union … frame fields sit beside the member's own props" (`:719`), and it spells its heading `title` (`:725`).
   - `Section` is the same kind of frame (`:521-523`).
   - The terminal renderer already uses the overlay's title with a fallback to its name (`crates/ui/ess-ui-tui/src/view.rs:608`), and it renders a section box with `section.name` (`view.rs:487`). A section title takes the same fallback.
   - No composite member declares a `title` prop (union members at `:968-970`; only `header` and `overlay` have one, at `:703` and `:725`), so there is no inline-prop collision.
   - `label` was rejected, because `metric.label` (`:772`) would collide with a section-level `label` written inline beside the member props.
   - It composes with `PageKind.sections` (same `Section` type, `:459`) and with named-list merging (a page can override an inherited section's title by name).
   - Targets: loader (`ess-ui` `SectionFrame`, `model.rs:715-720`), `ess ui check`, schema-generated reference (`ess ui docs`), TUI (`ess-ui-tui`), the React generator (`ess-ui-react`), and `ess ui test --playwright`.
5. **Second adopter.**
   - Item 1: a detail page with two collections over one view filtered differently ("Open tickets" / "Closed tickets"), or any localized UI where the section identifier cannot be the display text.
   - Item 2: answered by the idiom.
6. **Cost (item 1).**
   - One optional `string` field on `Section`, and no new diagnostic.
   - Renderer work in TUI and React (show it, fall back to nothing or the name).
   - Documents using it need an `ess` that knows the key, because the loader is `deny_unknown_fields` (`model.rs:716`). `requires:` covers that.
   - Whether an additive field needs `ess-ui/2`: I don't know. `ess-ui/1` first shipped in 0.47.0 (`git tag --contains 51aee77ac`), and I found no versioning rule for it.
   - Item 2: at most a hint on the `missing field reads` refusal for an inherited section, pointing at `{name: <n>, remove: true}`.
7. **Alternatives.**
   - Item 1:
     - (i) Change nothing, using the widget workaround: rejected because it gives up section loading semantics.
     - (ii) `label` on Section: rejected for the `metric.label` collision.
     - (iii) A `heading` node in `children` rendered first: a new ordering rule for one case.
     - (iv) Chosen: `title` on Section, mirroring overlay. This is the requester's draft syntax, kept unchanged.
   - Item 2:
     - (i) Change nothing: the idiom works (Q3).
     - (ii) A new kind: unnecessary.
     - (iii) A per-page `omit:` list: a second spelling for `remove: true`, rejected.
     - Chosen: the idiom, plus an optional refusal hint.

## Decisions

- **accept, redesigned (proposed):** Narrow the story to item 1.
  - Add an optional `title` to `Section`, mirroring `overlay.title` (the sibling frame construct, `ess-ui.schema.yaml:725`).
  - Renderers fall back to the section name the way the TUI does for overlays (`ess-ui-tui/src/view.rs:608`).
  - `label` was rejected because it collides with `metric.label` written inline.

  Item 2 is declined with the idiom: `{name: board, remove: true}` already removes an inherited kind section. It is in the schema and tested (`ess-ui/tests/shorthands.rs:52`), and `repro-281/r2-remove.ui.yaml` checks clean on 0.48.0. At most, add a hint on the `missing field reads` refusal pointing at it. Reply to the requester with the idiom.

  Item 2 is already fixed (it shipped with `ess-ui/1`). Item 1 is not. No duplicate. Same-format but unrelated: #284 (`ess ui check` grants). No story artifact exists yet.

- Coordinator (2026-10-01): adopted as proposed above. Item 2 is declined with the idiom `{name: board, remove: true}`. UI spec: priority 4.
