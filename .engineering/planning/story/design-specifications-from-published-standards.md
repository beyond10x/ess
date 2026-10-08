---
format: aep.planning-md/3
id: story:design-specifications-from-published-standards
kind: story
status: draft
title: 'Design: specifications extracted from a published standard'
refs:
- provider: github
  reference: beyond10x/ess#504
relations:
- serves: vision:O2
- depends_on: story:requirement-completeness-without-prose-in-the-model
revision: 3
---
## Outcome

A binding design page under `docs/design/` for https://github.com/beyond10x/ess/issues/504: a pinned
source text as an input (`ess-source/1`), requirement addressing owned by ESS, typed dispositions,
completeness statuses that say what was shown (checked, answered, modelled, unmapped, out of scope,
partial), requirement ids carried end to end, and generated statements per source. The design
settles what `story:requirement-completeness-without-prose-in-the-model` (#503 part C) left open and
splits the work into stories. No code.

## Decisions

- Design only; not scheduled in any release plan.
- The requester's extractor and six test texts are asked for when the design needs fixtures.
- Gaps met while modelling map to https://github.com/beyond10x/ess/issues/473 and
  https://github.com/beyond10x/ess/issues/498; `via:` reading a related row's identity waits for a
  reproduction.

## Acceptance

- The design page states each of the six areas of #504 with formats, ids and refusals, and names
  the stories it splits into.
