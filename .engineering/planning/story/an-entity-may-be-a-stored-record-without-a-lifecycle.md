---
format: aep.planning-md/3
id: story:an-entity-may-be-a-stored-record-without-a-lifecycle
kind: story
status: draft
title: An entity may be a stored record without a lifecycle
summary: 'An entity that omits lifecycle: is refused (missing field) on 0.93.0; records with no states of their own need a one-state lifecycle (adopter report)'
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-domain-language
revision: 1
---
## Need

A downstream adopter retrofits a service whose stored records mostly have no states of their own:
a row exists or it does not. Every entity must declare `lifecycle:` today, so each such record
carries a one-state lifecycle that states nothing (`{initial: Open, states: [Open], terminal:
[Open], transitions: []}`), and the adopter counts about 60 open notes on it.

Reproduced on 0.93.0 (`ess/37`; `crates/verify/ess-conformance/tests/fixtures/view-paging.yaml`
with the entity's `lifecycle:` line removed):

```text
$ ess specify validate --path system.yaml
system.yaml was refused:
  - system.yaml: missing field `lifecycle`
```

`provided: true` on the entity does not lift it (adopter's observation, not rerun here).

## Acceptance

- At a new source format an entity may omit `lifecycle:`. It then has no `state` field: a view
  cannot project `state`, and `moves:`, `into:`, `when_subject_state:` and `wrong_state:` naming it
  are refused by name.
- `creates:`, `updates:` and `deletes:` on such an entity validate, compile and synthesize; the
  suite's scenarios for it carry no state step.
- JSON Schema, OpenAPI and the generated types carry no state member for it.
- Below the new format the omission is refused as `unsupported_format_version`, not as a serde
  error. A model that declares `lifecycle:` keeps its IR bytes and its suite bytes.
- Open for the scoper: whether the omission is a distinct entity kind in the IR or lowers to one
  hidden state; an ADR states the choice.
