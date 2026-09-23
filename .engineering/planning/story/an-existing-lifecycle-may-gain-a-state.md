---
format: aep.planning-md/1
id: story:an-existing-lifecycle-may-gain-a-state
kind: story
status: archived
title: An existing lifecycle may gain a state
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The limitation

`ESS-ENTITY-005 missing_causation` refuses a transition no command outcome takes. That is right
inside one specification and wrong across two composed by an `ess-inputs.yaml` manifest.

An adopter with a derived tree — entities read out of a database schema, validated on its own — and
a second tree adding the commands a browser calls cannot give an entity a `Deleted` state: the state
belongs in the derived tree beside the entity, the command that causes it belongs in the second, and
the derived tree then stops validating alone.

## What it forced

A delete command with no `moves:` at all:

```yaml
- name: deleted
  summary: "the row is deleted and the deleted item rendered with 200"
  emits:
    - adopter.fe.queues.QueueDeleted
```

The row goes; the model says only that an event was published. The lifecycle is a lie of omission,
and the composed model cannot say otherwise without breaking the tree it composes with.

## Acceptance

Either causation is checked over the composed model rather than per tree, or a tree may declare that
a transition is caused elsewhere — so a derived tree stays valid alone without understating what the
entity does.

## Note

The composition mechanism itself worked well. Cross-domain `creates:`, `moves:` and views all
validate, a domain may span two files, and the manifest's containment rule is clear. This is the one
place composing a second tree cost accuracy rather than only effort.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
