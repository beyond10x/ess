---
format: aep.planning-md/3
id: story:a-host-slot-reads-a-list-of-views
kind: story
status: draft
title: A host slot reads a list of views
summary: reads on a host slot accepts several views; adopter hosts need 4-6 each (downstream report 2026-11-09)
tags:
- backlog
- external-request
- from-consumer-report
relations:
- derived_from: design:ui-host-slots
- depends_on: story:a-host-slot-reads-and-commands-only-what-it-declares
- depends_on: story:the-react-project-registers-each-declared-host-slot
- serves: vision:O2
revision: 1
---
## Need

An adopter's host components each read several views. `HostSlot.reads` is one `Reads`: one view,
written `t.View` or `{view, params}` (`crates/ui/ess-ui/src/model.rs:1055`). The generated React
registry types each slot's adapter to that one view, so a component that reads another view fails
`tsc`, and at run time the adapter refuses the read with status 403. The adopter's hosts need
4 to 6 views each: flow-step-editor 4, trigger-references 5, widget-filter-form 6. Together with
the form-value gap, this blocks about 45 of their 84 host entries
(a downstream report, 2026-11-09).

## Acceptance

- `reads` on a host slot accepts a list of `Reads`, each `t.View` or `{view, params}`. A single
  `Reads`, as written today, keeps its meaning and its IR bytes.
- `ess ui check --model` applies `view_in_model`, `section_readable`, `page_actor_grants` and
  `read_params` to every entry of the list, and counts each listed view as host-covered.
- The React project's adapter for the slot reads each listed view and refuses any other with
  status 403. A component that reads an unlisted view fails `tsc`.
- A list that names one view twice is refused by name.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (`HostSlot.reads`), `crates/ui/ess-ui-check/src/`
  (the model checks over a slot's reads, coverage), `crates/ui/ess-ui-react/src/emit.rs` and its
  hosts runtime template (the narrowed adapter type), `crates/ui/ess-ui-react/tests/host_slots.rs`,
  design:ui-host-slots, `schemas/ui/ess-ui.schema.yaml`.

## Fit review

Pending. An external request: it is assessed with
`.agents/skills/assessing-external-requests/SKILL.md` before this story leaves draft.
