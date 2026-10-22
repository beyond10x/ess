---
format: aep.planning-md/3
id: story:a-view-may-scope-rows-to-the-reading-actor
kind: story
status: draft
title: A view may show each actor only its own rows
summary: No per-row view scoping by reading actor (a2a wave 1, M5).
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 2
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **M5** (a2a wave-1 gaps list, line 39): per-row view scoping. The protocol (section 13.1)
  requires that a client sees only its own tasks; a view cannot filter its rows by the reading
  actor. No construct.

## Where it lives

No construct. View grants decide who may read a view, not which rows.

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch. Not a
transport question: the rule is the same over every binding.

## Acceptance (provisional)

- A view may filter rows by a field compared with the reading actor's identity or attribute;
  synthesis checks that a second actor does not see the first actor's rows.
- A filter naming an actor attribute the actor does not declare is refused by name.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1. Likely a new format version.
