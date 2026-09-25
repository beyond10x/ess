---
format: aep.planning-md/2
id: story:a-resource-has-an-http-shape-of-its-own
kind: story
status: archived
title: A resource has an HTTP shape of its own
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The gap

`ess generate --kind openapi` projects one path per command:

```
/telephony/commands/PurgeStaleCall:
  post: …
```

and the generated document states the position in its own `description`:

> Every path here is one semantic command, so the method is always POST and the path is the
> command's wire name under its domain's: a command is not a resource, and this document does not
> invent one.

That is defensible for a command surface. It means a specification cannot describe the surface of
almost every existing HTTP API, and cannot be projected into one.

## What the adopting project had to hand-write

A JavaScript facade, because nothing in ESS carries any of it:

| | |
|---|---|
| the envelope | `{items, pagination, success}` / `{item, success}` / `{message, error, success}` |
| status per outcome | 201 on create, 200 on update, 400 on a refusal the input decides, 404 on a missing row |
| the identifier on the wire | a `uuid` column, not the entity's identity |

Every frontend call went through it. None of it is checked by anything ESS generates.

## Acceptance

A domain may declare a REST presentation — resource path, the method and status per command
outcome, the envelope a list and an item are wrapped in — and `ess generate --kind openapi` projects
it as resources, and the `web` target serves it.

## Precedent

`ess specify cli` already does exactly this for a different presentation: a typed binding from
commands to a command-line surface, validated against the model. The construct being asked for here
is its HTTP sibling.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
