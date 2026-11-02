---
format: aep.planning-md/3
id: story:an-eventual-event-compares-a-captured-identity
kind: story
status: draft
title: An eventual event compares a downstream identity copied from a captured instance
relations:
- serves: vision:O2
revision: 1
---
## Finding

Out of scope in wave D357 U4 (`story:binding-site-identity-comparisons`, decided before the unit): identities further down a binding chain, such as the `job_id` of an event published by the invoked command (`Started.job_id`), are expected through `EventuallyEvent`, which carries literals only, so a downstream event's identity copied from a captured instance is not compared.

## Acceptance

- An eventual-event expectation can name a captured instance for an identity field, and binding scenarios use it for identities the downstream event copies from an instance the scenario holds.
- A target that publishes a wrong downstream identity fails at least one binding scenario; the correct interpreted target passes.
- Models whose bindings publish no downstream identity keep their suite bytes.
