---
format: aep.planning-md/3
id: story:a-list-parameter-selects-rows-outside-a-group-key
kind: story
status: draft
title: A list parameter selects rows outside a group key, with absent meaning unfiltered
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#492
relations:
- serves: vision:O2
revision: 1
---
## Outcome

Gap 3 of https://github.com/beyond10x/ess/issues/492: the list selector of #438 is arranged over a
row filter of an ungrouped or grouped view, and `not defined(param.<p>)` is accepted as the absent
rule beside `param.<p>.count == 0`. Today `ess verify conform synthesize` refuses the view with
`ESS-SYNTH-017`.

Spec first; fit review owed.
