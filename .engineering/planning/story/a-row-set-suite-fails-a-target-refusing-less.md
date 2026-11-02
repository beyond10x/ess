---
format: aep.planning-md/3
id: story:a-row-set-suite-fails-a-target-refusing-less
kind: story
status: draft
title: A row-set suite beside named rows fails a target that refuses less
tags:
- ess-d368-close
relations:
- serves: vision:O2
revision: 1
---
## Finding

Found by wave D368 U4 (first-round implementor note), out of that story's Acceptance: the row-set suite beside named rows is checked against targets that refuse more than the rule; a target that refuses less (accepts a send the row set refuses) is not among the mutants its tests try.

## Acceptance

- A test runs the suite of a row-set-beside-named-rows model against a target that accepts a send the row set refuses, for each refusal order, and the suite fails each.
- Where a case passes, the gap is fixed or withdrawn with an `ESS-SYNTH-001` reason naming it.
