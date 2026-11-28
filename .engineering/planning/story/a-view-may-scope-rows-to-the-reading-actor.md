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
- decomposes: epic:adopter-gaps-wire-and-answers
- informed_by: story:a-view-selects-rows-by-a-caller-attribute
revision: 3
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

## Related (2026-11-25)

The same need was reported again on 0.99.0 from the HTTP side (a view parameter the service takes from
the bearer token, twentieth gap report, shape 1a), and is drafted with its acceptance in
`story:a-view-selects-rows-by-a-caller-attribute`. The security-marked decisions are on
`decision-blocker:the-caller-reaches-a-view-or-a-route-only-through-declared-attributes` (HC-D1 to
HC-D3, with the owner). This story is not changed; when the owner has picked under HC-D1, one of the two
is archived into the other.
