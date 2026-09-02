---
format: aep.planning-md/1
id: story:own-planning-store
kind: story
status: implemented
title: ESS plans in a store of its own
summary: The .engineering store, pinned to aep 0.42.0, so cross-repository work has a plan that is not a chat.
owner: ess
tags:
- store
relations:
- decomposes: epic:entity-relations
revision: 4
---
# Story: ESS plans in a store of its own

## Outcome

Anybody working on ESS finds its plan in `.engineering/planning/`, mutated only through `aep artifact`, pinned to a protocol tree by commit — the same shape `aep` and `agentplugins` already use.

## Context

Until 2026-09-02 ESS had no `.engineering/` directory (`ls -d .engineering` in the worktree at `2979e6e`: no such file). The epic that needs this store, `epic:entity-relations`, is the first cross-repository change since the 2026-09-01 split, and a plan that lives only in a chat is the thing that split was meant to end.

## Acceptance

- `.engineering/project.yaml` names the `aep` protocol tree by a 40-hex commit and `development.standard`.
- `aep artifact validate` exits 0 in this repository.
- `AGENTS.md` names the store and says mutations go through `aep artifact`.

## Out of Scope

Migrating `docs/plan/` pages into artifacts. They stay as design and plan documents; the store references them.

## Ambiguities

- `inferable` — the pin: `a054945cf55229861b7e7b9e83e94343278cbc02` is `aep` tag `0.42.0`.

## Open Questions

None.
