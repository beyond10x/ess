---
format: aep.planning-md/3
id: story:ui-spec-schema
kind: story
status: active
title: ess-ui/1 has a schema, a loading model and a brand-free example
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
scope:
- confidence: inferred
  path: crates/ui/ess-ui
- confidence: inferred
  path: examples/partner-portal
- confidence: inferred
  path: schemas/ui
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:23Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-09-30T00:35:25Z", actor: "human:timo", revision: 7}
---
## Outcome

`ess-ui/1` exists as a schema in this repository, with a Rust model that loads and expands a
document, and a brand-free example application that uses every construct.

## Acceptance

- `schemas/ui/ess-ui.schema.yaml` defines the document: app, model reference, shells and regions,
  navigation, pages and page kinds, layout, sections, overlays, the composite kinds, app-defined
  widgets with typed params, the closed primitive set, reads (including unbound placeholders),
  does, live, channels, section lifecycle states, degrades, state placement (`store:` and a
  `placement_profile`), node naming and paths. Every construct carries `summary`, `doc`, per-property
  `note`, an `example` and a `group`/`order`; every type is YAML structure; every shorthand is data.
- `crates/ui/ess-ui` loads a document with clap-free library code, expands shorthands, widgets and
  page kinds, and exposes each node with its canonical path; a test loads the example and asserts
  the expansion of one instance of every shorthand.
- `examples/partner-portal/ui.yaml` (a SaaS partner portal: partners, deals, invoices, tickets, a
  live activity feed) uses every construct and loads without error.
- No product or company name appears in the schema, the crate or the example.
- Anything whose order is meaningful (sections, columns, navigation entries, fields, actions) is a list of
  named nodes; maps are used only where order carries no meaning, so a generated reader that sorts map
  keys cannot reorder a page.
- Composite kinds are one union discriminated by `component`; until `story:union-tag-inline-with-fields`
  lands, the Rust model reads it with a hand-written `serde(tag)` enum.

## Input

A draft of the schema and a retrofit of a real application exist outside this repository; only
brand-free material may be copied in.
