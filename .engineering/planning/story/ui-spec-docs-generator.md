---
format: aep.planning-md/3
id: story:ui-spec-docs-generator
kind: story
status: implemented
title: The ess-ui reference is generated from the schema as one styled page
relations:
- serves: vision:O2
- decomposes: epic:ess-ui-renderer-neutral-ui
- depends_on: story:ui-spec-schema
scope:
- confidence: inferred
  path: crates/ui/ess-ui-docs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-09-30T00:35:23Z", actor: "human:timo", revision: 3}
- {from: "proposed", to: "active", at: "2026-09-30T01:41:54Z", actor: "human:timo", revision: 4}
- {from: "active", to: "implemented", at: "2026-09-30T05:07:12Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1,"review_outcome":2,"verification":1}}}
---
## Outcome

A hand author reads one good-looking reference of `ess-ui/1`, generated from the schema as data.

## Acceptance

- `ess ui docs --out <file.html>` renders `schemas/ui/ess-ui.schema.yaml` into one self-contained HTML
  page: chapters by `group` and `order`, per construct its summary, doc, a properties table (name,
  readable type such as "list of AgentGroupId" or "one of: thin | fat | hybrid", required, default,
  note), the shorthands it accepts with their expansions, a highlighted example, and cross-links
  from each type to the construct it names; a quick-reference page at the end.
- A test fails when a construct lacks a summary, doc or example, or when a type names a construct
  that does not exist.
- The page is also emitted as Markdown for the documentation site.
