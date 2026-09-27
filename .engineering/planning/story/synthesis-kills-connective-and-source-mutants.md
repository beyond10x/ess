---
format: aep.planning-md/2
id: story:synthesis-kills-connective-and-source-mutants
kind: story
status: draft
title: Synthesis kills dropped-source and connective mutants, and witnesses partial views
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 2
---
## Scope

- #154: a state dropped from an all-states transition gets a `…/state/<s>/refuses/…` scenario.
- #155: `any:` gets one witness per disjunct with the others false; `all:` a boundary witness with
  exactly one conjunct false (MC/DC over the connective).
- #132: wrong-state preservation over the fields the declared views publish, and a `help:` line
  that points at views.

## Acceptance

- A spec mutated by dropping one source state from an all-states `from:` synthesizes a scenario
  the unmutated implementation fails.
- `any` → `all` and `all` → `any` mutants each change the synthesized suite.
- The repro in #132 synthesizes its terminal refusal scenario over an eventual identity/state view.
