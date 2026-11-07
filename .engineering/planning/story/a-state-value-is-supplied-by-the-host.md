---
format: aep.planning-md/3
id: story:a-state-value-is-supplied-by-the-host
kind: story
status: draft
title: A state value may be supplied by the host application at start-up
summary: a page_state value from app configuration has no source; the page body renders empty
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
revision: 1
---
## Need

A UI adopter reported this on 2026-11-07, against 0.90.0.

A page reads its configuration through a `page_state` value (`backend_config`) that the host
application must supply. In the source application, that value comes from the application's
runtime configuration. A State `default` is literal JSON, and neither generated project has a
place where the host can seed a state value. Generated, the page body is empty.

## Acceptance

- A State may declare that the host supplies its initial value. It names the value, never the
  mechanism, so the document stays renderer-neutral.
- The React project exposes one typed place where the host provides those values at start-up, and
  the terminal takes them from its start-up configuration.
- A declared value the host does not provide fails at start-up with the state's name, rather than
  rendering an empty page.

## Scope

- inferred: `crates/ui/ess-ui/src/model.rs` (State), `crates/ui/ess-ui-check`, the React project's
  runtime state and entry templates, `crates/ui/ess-ui-tui` start-up.
