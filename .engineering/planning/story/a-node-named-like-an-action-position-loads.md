---
format: aep.planning-md/3
id: story:a-node-named-like-an-action-position-loads
kind: story
status: active
title: A state or region named like an action position loads
summary: The shorthand expander reads a named map entry called action or form as that construct (UI-adopter wave 1 U4 implementor note, pre-existing, backlog)
tags:
- backlog
- defect
- from-finding
- severity-medium
- zero-open-issues
relations:
- informed_by: story:a-refused-node-is-reported-at-its-own-path
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
scope:
- confidence: cited
  path: crates/ui/ess-ui/src/expand.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/locate.rs
- confidence: inferred
  path: crates/ui/ess-ui/src/schema.rs
- confidence: inferred
  path: crates/ui/ess-ui/tests/
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:40:18Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:40:20Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

The loader's local-shorthand expander (`crates/ui/ess-ui/src/expand.rs`, `Local::mapping`) reads
a map entry by its key alone. A page state, a shell region or another named entry called
`action`, `form` or another action position, and holding a map, is expanded as an Action, so a
valid document is refused. The load-error-path unit of UI-adopter wave 1
(story:a-refused-node-is-reported-at-its-own-path) found this. Its fix in `locate.rs` derives
named-node maps from the schema's `containers`, so it reports the right path, but it leaves the
expander's name-based reading in place. Pre-existing at 83391ebaeb; still reproduces on 0.92.0.

Minimal reproduction. In `tmpl.yaml`, `REGION` and `STATE` are replaced per variant:

```yaml
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      REGION: {kind: page_outlet}
navigation:
  home: a
  sections: [{name: all, pages: [a]}]
pages:
  a:
    kind: list_page
    title: A
    state:
      STATE: {type: string, class: page_state, default: x}
    sections:
      - {name: rec, component: record, reads: {view: probe.Items}, fields: [title]}
```

```text
REGION=main,   STATE=picked  -> 0 error(s)                         exit 0
REGION=main,   STATE=action  -> error document_loads pages/a/state/action: an action without a `name` needs one of opens, does, navigate, export, upload or sets to derive it from; …
REGION=main,   STATE=form    -> error document_loads pages/a/state/form: (same message)
REGION=action, STATE=picked  -> error document_loads shells/app/regions/action: (same message)
REGION=form,   STATE=picked  -> error document_loads shells/app/regions/form: (same message)
```

## Acceptance

- `ess ui check` over each of the four renamed variants above reports the same findings as the
  control (0 errors), and `ess ui load` prints the same document apart from the name.
- The same holds for every key the schema lists as an action position (`Schema::positions`
  `actions`, shape `One`), used as a page-state, region, page or widget name. One test iterates
  over that set.

## Scope

- cited: `crates/ui/ess-ui/src/expand.rs`
- inferred: `crates/ui/ess-ui/src/schema.rs`
- inferred: `crates/ui/ess-ui/src/locate.rs` (reuse its schema-derived named containers)
- inferred: `crates/ui/ess-ui/tests/`

confidence: high

## Schedule

Waved under epic:zero-open-issues-2026-11 (accepted 2026-11-10).

