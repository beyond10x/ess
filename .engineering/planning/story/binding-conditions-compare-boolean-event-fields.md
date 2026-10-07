---
format: aep.planning-md/3
id: story:binding-conditions-compare-boolean-event-fields
kind: story
status: draft
title: A binding condition may compare a Boolean event field with true or false
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

Gap 1 of https://github.com/beyond10x/ess/issues/492: `where: [event.is_bridged == false]` in a
binding condition validates and is witnessed like the enum comparisons (`condition-false` flips the
flag). Today it is refused as `ESS-BINDING-002` ("a binding condition compares a String or enum leaf
with a text literal").

Spec first; fit review owed (is an existing construct or idiom an answer? if so, the issue is told).
