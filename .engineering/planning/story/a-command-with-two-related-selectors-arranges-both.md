---
format: aep.planning-md/3
id: story:a-command-with-two-related-selectors-arranges-both
kind: story
status: draft
title: A command with two related selectors gets scenarios for both
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

An adopter's command carries two `when_related` selectors, each guarding its own outcome (one row set per group key, one per workspace). ess 0.56.0 synthesizes scenarios for one selector only and prints `a scenario arranges the rows of one selector per command in this cut`, so every outcome the second selector separates has no witness.

## Reproduction

Not yet minimised. As for `story:a-related-selector-compares-a-row-field-with-a-subject-field`, a neutral reproducer goes into `.engineering/repro/` with the fit review.

## Next

Assess with `.agents/skills/assessing-external-requests/SKILL.md` before any build. Expected in synthesis (`crates/verify/ess-conformance/src/synthesize/`), so scheduled after the other session's synthesis waves leave those files.
