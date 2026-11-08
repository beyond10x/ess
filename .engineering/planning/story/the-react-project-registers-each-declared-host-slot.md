---
format: aep.planning-md/3
id: story:the-react-project-registers-each-declared-host-slot
kind: story
status: active
title: The React project registers each declared host slot with a narrowed adapter
summary: 'ADR Effect (React): a typed registry, one entry per slot, and tsc fails on a missing, extra or undeclared access'
tags:
- backlog
- queue-0.92.0
- rank-2
relations:
- derived_from: design:ui-host-slots
- serves: vision:O2
- depends_on: story:a-ui-document-declares-a-host-slot
scope:
- confidence: cited
  path: crates/ui/ess-ui-react/src
- confidence: cited
  path: crates/ui/ess-ui-react/templates/runtime
- confidence: cited
  path: crates/ui/ess-ui-react/tests/host_slots.rs
revision: 11
transitions:
- {from: "draft", to: "proposed", at: "2026-11-08T08:54:30Z", actor: "human:timo", revision: 10}
- {from: "proposed", to: "active", at: "2026-11-08T08:54:30Z", actor: "human:timo", revision: 11}
---
## Need

The accepted ADR on host components, Effect (React) and section 3: the generated React project
registers the declared slots and hands each one only the access it declares. The design is
design:ui-host-slots, section Generators.

## Acceptance

- `ess generate ui --target react` on a document with slots no longer refuses it: it emits a
  registry type with exactly one entry per declared `host` name, and the generated app entry
  requires a value of that type.
- Each entry is a component whose props are `{ adapter, path, page, scope }`: `adapter` is a
  `DataAdapter` narrowed to the slot's declared reads and commands, `path` is the slot's document
  path, `page` the page's route parameters, and `scope` the `useScope().values` in force at the
  slot's position (on a page, overlay, tab or field with no scope layer of its own, the values of
  the nearest enclosing layer, else the root scope's; `SectionStates`, a display configuration,
  is not passed). A render test (the crate's existing node render harness) registers a probe component and
  asserts it received each of the four with those values.
- In the generated project, `tsc` fails in each of these cases, one fixture each:
  - a slot is left unregistered;
  - a name is registered that no slot declares;
  - a slot calls a read or command it did not declare (`adapter.<name>` outside the narrowed type).
- A registered slot renders inside an element with `data-ui-path` equal to the slot's path and
  `data-ui-host` equal to its `host`, at its position among the section's siblings; the render test
  asserts both attributes and the order.
- The fact "a host slot's path is rendered by React and driven by nobody" is owned by
  story:the-terminal-draws-a-host-slot-as-a-labelled-region in `crates/ui/ess-ui-test/src/parity.rs`;
  this story renders the `data-ui-path` that rule names and does not edit `parity.rs`.

## Scope

- inferred: `crates/ui/ess-ui-react/src/` (the refusal at `lib.rs:85`, `emit.rs`),
  `crates/ui/ess-ui-react/templates/runtime/` (`data.ts.tmpl`, `core.tsx.tmpl`, the app entry),
  `crates/ui/ess-ui-react/tests/host_slots.rs` (rewritten: the refusal becomes generation).
- Shared, trivially merged: `CHANGELOG.md`, `website/docs/reference/ess-ui.md`.
- Not touched: `crates/ui/ess-ui-test/` (owned by the terminal story).
