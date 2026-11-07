---
format: aep.planning-md/3
id: story:the-react-project-registers-each-declared-host-slot
kind: story
status: draft
title: The React project registers each declared host slot with a narrowed adapter
summary: 'ADR Effect (React): a typed registry, one entry per slot, and tsc fails on a missing, extra or undeclared access'
tags:
- backlog
- queue-0.91.0
- rank-2
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
revision: 4
---
## Need

The accepted ADR on host components, Effect (React) and section 3: the generated React project
registers the declared slots and hands each one only the access it declares. The design is
design:ui-host-slots, section Generators.

## Acceptance

- `ess generate ui --target react` on a document with slots emits a registry type with one entry
  per declared `host` name.
- Each entry receives a `DataAdapter` narrowed to the slot's declared reads and commands, and the
  page and scope state of its position.
- In the generated project, `tsc` fails in each of these cases:
  - a slot is left unregistered;
  - a name is registered that no slot declares;
  - a slot calls a read or command it did not declare.
- A registered slot renders at its position in the page.

## Scope

- inferred: `crates/ui/ess-ui-react/src/`, `crates/ui/ess-ui-react/templates/runtime/`
  (`data.ts.tmpl`), the React generation tests.
