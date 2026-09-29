---
format: aep.planning-md/3
id: story:collection-quantifiers-witnessed-everywhere
kind: story
status: draft
title: Every collection quantifier is witnessed with several elements, or says why not
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Quantifiers over command-input collections (`exists t in input.tags`) are witnessed with several elements, as stored collections are since beyond10x/ess#240, and a stored-collection quantifier that falls back to a one-entry row says so instead of passing silently.

## Acceptance

- a target reading only the first or last element of an input list or map, or swapping exists/forall, fails a scenario;
- where no several-entry row can be built (Boolean-keyed map, ordered comparison, collection not written from input), synthesis records a note naming the guard;
- nested quantifiers over a bound element are witnessed the same way.

## Origin

Correction 1 of #240 (0.43): witness.rs gives input-collection quantifiers one element; the stored-collection fallback records nothing.
