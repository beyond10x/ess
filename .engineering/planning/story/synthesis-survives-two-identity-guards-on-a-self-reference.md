---
format: aep.planning-md/3
id: story:synthesis-survives-two-identity-guards-on-a-self-reference
kind: story
status: draft
title: Synthesis does not overflow its stack on two identity-addressed when_related guards, one on an optional self-reference
refs:
- provider: github
  reference: beyond10x/ess#474
relations:
- serves: vision:O2
revision: 1
---
## Outcome

`ess verify conform synthesize` finishes, with scenarios or named refusals, on a command with two
identity-addressed `when_related` guards where the second follows an optional self-reference
(https://github.com/beyond10x/ess/issues/474). Reported on 0.53.0; the implementor reproduces it on
the newest release before any change, and the story is closed as not reproduced if it does not.

## Acceptance

- The issue's `shop.tasks` reproducer synthesizes without a stack overflow, each outcome witnessed
  or refused by name, and a regression test holds it.
