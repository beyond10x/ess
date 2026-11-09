---
format: aep.planning-md/3
id: story:a-host-slot-in-a-reused-template-is-one-registration
kind: story
status: draft
title: A host slot in a reused template is one registration with one identity per instance path
summary: 'Reused template holding a slot is refused declared twice in 0.90.0 (U2 adversary; coordinator: draft for 0.91.0)'
tags:
- backlog
- defect
- from-finding
- queue-0.91.0
- rank-10
- severity-medium
- zero-open-issues
relations:
- derived_from: design:ui-host-slots
- depends_on: story:a-ui-document-declares-a-host-slot
- serves: vision:O2
- depends_on: story:the-react-project-registers-each-declared-host-slot
- depends_on: story:the-terminal-draws-a-host-slot-as-a-labelled-region
- decomposes: epic:zero-open-issues-2026-11
revision: 4
---
## Need

Found by the U2 adversary on ESS 0.90.0; still reproduces on 0.92.0. A host slot inside a widget
or page template that a document uses twice is refused as "declared twice". The loader's
`check_host_slots` (`crates/ui/ess-ui/src/path.rs`) walks the expanded document, so each use of
the template is a separate slot with the same `host` name. This is the fail-closed reading of
design:ui-host-slots, where a host name is "unique in the document". The coordinator kept that
refusal for 0.90.0 and asked for this story.

A reused template is the natural place for a bespoke part repeated across pages. The refusal stops
an adopter from declaring such a part once.

Minimal reproduction:

```yaml
format: ess-ui/1
app: probe
model: probe.system
placement_profile: fat
shells:
  app:
    regions:
      main: {kind: page_outlet}
navigation:
  home: a
  sections: [{name: all, pages: [a, b]}]
widgets:
  route_panel:
    summary: A route map with a caption.
    body:
      - {name: caption, primitive: text, text: Route}
      - {name: map, host: RouteMap, reason: a map widget, stands_in_for: chart}
pages:
  a:
    kind: list_page
    title: A
    sections:
      - {name: panel, component: route_panel}
  b:
    kind: list_page
    title: B
    sections:
      - {name: panel, component: route_panel}
```

```text
$ ess ui check --path ui.yaml
error host_slots pages/b/sections/panel/body/map: host slot `RouteMap` is declared twice, here and at `pages/a/sections/panel/body/map`: a host name is unique in the document
ui.yaml: 1 error(s), 0 warning(s)          # exit 1
```

The same document with page `b` removed passes and lists the slot once.

## Acceptance

- `ess ui check --path ui.yaml` over the document above exits 0. Its text report lists `RouteMap`
  once for each instance path, `pages/a/sections/panel/body/map` and
  `pages/b/sections/panel/body/map`, and its JSON report carries both paths.
- `ess generate ui --target react` over it writes a `src/hosts.ts` whose `HostComponents` has
  exactly one `RouteMap` entry. The generated pages pass each instance its own `path`.
- The terminal draws the placeholder at both instance paths, and an `ess-ui-test/1` `text` check
  at each path reads its label.
- With `--model`, coverage counts the slot's `reads` and `commands` as host-covered from each
  instance.
- Two slots with the same `host` that come from different declarations (two different widgets, or
  two literal page slots) are still refused as `host_slots`, "declared twice".
- design:ui-host-slots states that the rule is per declaration and that the instance path is the
  identity.

## Scope

- cited: `crates/ui/ess-ui/src/path.rs`
- inferred: `crates/ui/ess-ui/src/expand.rs`
- inferred: `crates/ui/ess-ui-check/src/`
- inferred: `crates/ui/ess-ui-react/src/emit.rs`
- inferred: `crates/ui/ess-ui-tui/src/view.rs`
- inferred: `.engineering/planning/design/ui-host-slots.md`

confidence: medium
