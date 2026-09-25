---
format: aep.planning-md/2
id: story:a-generated-module-keeps-its-name
kind: story
status: archived
title: A generated module keeps its name
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The defect

Adding a domain whose leaf name collides with an existing one silently renames **every** sibling's
generated module.

Before adding `adopter.fe.automation`, beside the existing `adopter.automation`:

```
adopter-types/src/  bootstrap.rs  fixtures.rs  queues.rs  routing.rs
```

After:

```
adopter-types/src/  fe_automation.rs  fe_bootstrap.rs  fe_fixtures.rs  fe_queues.rs  fe_routing.rs
```

Four modules an adopter imports were renamed by adding a fifth domain that has nothing to do with
them. Every hand-written `use adopter_types::queues::…` stopped compiling.

## Why disambiguating is right and doing it this way is not

The rename itself is correct — two domains cannot share a module. What is wrong is that it is
retroactive and silent: the modules that were already unambiguous change spelling, and nothing in
the output says a rename happened.

## Acceptance

Either a domain's module name is derived from its full qualified name from the start, so it never
changes, or a regeneration that renames an existing module reports which names changed.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
