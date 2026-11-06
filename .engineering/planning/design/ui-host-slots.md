---
format: aep.planning-md/3
id: design:ui-host-slots
kind: design
status: draft
title: 'Host slots in ess-ui/1: shape, check, coverage and generator seams'
summary: Design for the accepted host-slot ADR; six stories for 0.90.0 after issue 761
relations:
- derived_from: architecture-decision-record:a-ui-document-declares-the-host-components-it-does-not-generate
- serves: vision:O2
revision: 1
---
## Purpose

This page designs the host slot that the accepted
architecture-decision-record:a-ui-document-declares-the-host-components-it-does-not-generate
adds to `ess-ui/1`. A host slot is a declared place in a UI document where the host application
supplies its own component. ESS does not draw it and never counts it as covered by the
specification.

## Shape

A host slot is one more node shape. It stands wherever a node may stand: a page, a section, an
overlay, a tab, or a field. It is told apart from a composite and a primitive by its `host:` key,
and a node that carries `host:` beside `component:` or a primitive key is refused by name.

| key | required | meaning |
|---|---|---|
| `host` | yes | the component name the host registers; unique in the document |
| `reason` | yes, non-empty | why the generator cannot draw it |
| `stands_in_for` | yes | one of `COMPOSITE_KINDS` (`crates/ui/ess-ui/src/model.rs:15-28`), a primitive, or `none` |
| `reads` | no | the views it reads, in the shape a composite's `reads` takes |
| `commands` | no | the commands it sends, in the shape a generated node names them |

The slot's scope is its node path, so the document carries no separate scope key. An older reader
refuses the `host` key by name as an unknown key. `schemas/ui/ess-ui.schema.yaml` and
`website/docs/reference/ess-ui.md` gain the shape.

## Check (`ess ui check`)

- **Declaration (ADR section 1):**
  - A host slot without `reason`, or with an empty one, is refused.
  - Two slots with one `host` name are refused.
  - `ess ui check --hosts <file>` lists the names a host registers. It refuses a registered name
    that no slot declares, as it refuses an undeclared view or command.
- **Coverage (ADR section 2):** a row that a host slot's reads or commands reach is counted as
  host-covered. That is a third count beside covered and uncovered, and it is never added to the
  covered count. The text report and the JSON report list each slot with its path, `host`,
  `reason` and `stands_in_for`.
- **Access (ADR section 3):** with `--model`, a slot's reads and commands are checked against the
  specification and its `ess-http/1` binding, exactly as a generated node's are.
- **Stand-in (ADR section 4):** a `stands_in_for` value that names no composite, no primitive and
  is not `none` is refused. `stands_in_for: none` requires a `reason`. The report lists the `none`
  slots as the generator's backlog.

## Generators (ADR Effect)

**React:** the generated project exports a registry type with exactly one entry per declared
`host` name.

- Each entry is typed by what that slot receives: a `DataAdapter`
  (`crates/ui/ess-ui-react/templates/runtime/data.ts.tmpl:58`) narrowed to the slot's declared
  reads and commands, and the page and scope state of its position.
- A missing registration fails the project's type check (`tsc`), so it is a build error rather
  than a blank region.
- An extra registration is refused the same way.

**Terminal:** the renderer draws a labelled placeholder region at the slot's path. The label shows
`host` and `reason`.

- `ess-ui-test/1` can address the region by that path.
- A test step that acts inside the region is refused by name, because the terminal does not draw
  the host component.

## Specification

Unchanged. A host slot reads and commands only what the specification already serves.

## Stories

All six are drafts for 0.90.0, ranked after the issue 761 story.

1. story:a-ui-document-declares-a-host-slot (ADR section 1: shape, schema, reference page,
   refusals)
2. story:a-host-slot-names-the-generator-item-it-stands-in-for (ADR section 4)
3. story:host-covered-rows-are-their-own-count (ADR section 2)
4. story:a-host-slot-reads-and-commands-only-what-it-declares (ADR section 3, check side)
5. story:the-react-project-registers-each-declared-host-slot (Effect, React)
6. story:the-terminal-draws-a-host-slot-as-a-labelled-region (Effect, terminal and `ess-ui-test`)

Stories 2 to 6 depend on story 1. Stories 5 and 6 can run side by side.
