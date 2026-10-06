---
format: aep.planning-md/3
id: story:fast-line-specification-completeness-measure
kind: story
status: archived
title: No measure of how complete a specification is
tags:
- fast-line
revision: 2
transitions:
- {from: "draft", to: "archived", at: "2026-10-06T09:48:10Z", actor: "human:timo", revision: 2}
---
## Outcome

A specification author can see how complete a specification is: which declared commands, outcomes and views are covered by synthesized or authored scenarios and which are not.

## Origin

Fast-line intake, 2026-10-04. A downstream team asked on 2026-09-25 for a way to tell how finished a specification is; the maintainer reply listed it as a known limit with no measure yet.

## Open question

- Whether existing coverage reporting (`ess-conformance` report/2 coverage knowledge) already answers part of this and only needs a CLI view.
