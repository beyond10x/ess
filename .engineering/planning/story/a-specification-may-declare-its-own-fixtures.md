---
format: aep.planning-md/1
id: story:a-specification-may-declare-its-own-fixtures
kind: story
status: archived
title: A specification may declare its own fixtures
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

Nothing in a specification can put a row in front of a test. A conformance scenario arranges state
through declared commands, which is right for a suite ESS synthesises — but a fake backend driven by
somebody else's test suite has to start from rows that no command in the model creates.

## What the adopting project did instead

Invented a domain of commands that exist in no backend:

```
adopter.fe.fixtures — SeedUser, SeedAccount, SeedAccountSettings, SeedRole,
                         SeedAccountRole, SeedAgent, SeedAgentGroup, SeedPresence, SeedTag
```

Nine commands, each `creates:` an entity and emits an event nobody publishes, with the domain's
summary saying plainly that no route serves them. They were declared rather than hidden in the
realization so the compiler would at least type them — but they are a lie inside a document whose
value is that it is not one.

## Acceptance

A specification may declare fixture state — typed rows against declared entities — which the
generated realization can load and which `ess specify validate` type-checks against the entity, and
which is visibly not part of the system's surface. A consumer reading the model can tell the
fixtures from the commands without reading a summary.

## Related

`ess verify conform` already has the harder half of this: it arranges state for its own scenarios.
What is missing is the same arrangement made available to a realization that somebody else drives.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
