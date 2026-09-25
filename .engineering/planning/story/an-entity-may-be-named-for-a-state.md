---
format: aep.planning-md/2
id: story:an-entity-may-be-named-for-a-state
kind: story
status: archived
title: An entity may be named for a state
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The defect

ESS allocates `<Entity>State` for an entity's lifecycle state type. An entity actually named
`<X>State`, where `<X>` is also an entity, therefore collides with a name the emitter reserves — and
the refusal rejects the entire workspace:

```console
$ ess generate synthesize --path docs/ess --target rust
rust target cannot emit this workspace
adopter.widget.DeskWidgetMetric.State, adopter.widget.DeskWidgetMetricState:
  `DeskWidgetMetricState` is allocated 2 times in `domain:adopter.widget` (type, entity)
```

`--target go` emitted the same workspace without complaint, so the refusal is per target.

## Why the name was not the author's choice

`desk_widget_metric_states` is a table GORM generates for a `hasMany`. The entity is named for
the table it is, which is the rule the whole specification was written under. Renaming it to satisfy
an emitter puts a name in the model that matches nothing in the database.

## Acceptance

Either the generated state type takes a spelling that cannot collide with a declared entity, or the
collision is reported by `ess specify validate` — where an author can act on it — rather than by an
emitter after the model is finished, and rather than only by two of three targets.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
