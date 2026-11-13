---
format: aep.planning-md/3
id: story:a-state-value-is-supplied-by-the-host
kind: story
status: active
title: A state value may be supplied by the host application at start-up
summary: a page_state value from app configuration has no source; the page body renders empty
tags:
- backlog
- external-request
relations:
- serves: vision:O2
- informed_by: epic:downstream-reported-gaps
- decomposes: epic:ess-ui-renderer-neutral-ui
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-11-12T21:47:55Z", actor: "human:timo", revision: 3, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
- {from: "proposed", to: "active", at: "2026-11-12T21:47:56Z", actor: "human:timo", revision: 4, executor: "agent:b10x", correlation: "operator-2026-11-12-round-2"}
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

## Decisions

**Accepted under the operator's procedure for external-request drafts (2026-11-12).** He set
it that day: the session that maintains the repository recommends per draft, an independent
adversarial review reads each draft, a draft both read as accept is accepted on his behalf, and
a draft where the two differ goes to him. This draft was number 30 of a second list of 13
(numbers 26 to 38); both read accept. The two transitions carry the correlation
`operator-2026-11-12-round-2` and the executor `agent:b10x`.

The fit review of `.agents/skills/assessing-external-requests/SKILL.md` is not written yet. It
is owed before this story is dispatched and says which form of acceptance applies (as proposed
or redesigned).

Question carried into the fit review, not a condition of the acceptance: whether a view of the
specification that the page reads (a state with `store: server`) serves the need in place of a
value the host supplies.

The story named `epic:downstream-reported-gaps`, archived on 2026-11-02. It now decomposes
`epic:ess-ui-renderer-neutral-ui`; the older `informed_by` edge stays as history.
