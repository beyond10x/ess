---
format: aep.planning-md/1
id: epic:specification-runs-as-a-fake-backend
kind: epic
status: archived
title: A specification can be run as a fake backend
summary: What ESS is missing before a model can serve a frontend without a human writing the behaviour.
revision: 3
---
## What was tried

A fake of a real backend — an adopter's Grails monolith — compiled from its ESS model and driven
from a browser, so a frontend end-to-end suite could stop running against a live environment. The
model is the adopter's services repository `docs/ess/`: 102 entities, 892 fields, validated and checked against the
database schema.

It works, and almost none of it was generated. 381 capabilities synthesised, 34 obligations, and
every one of the 34 written by hand in Rust. The REST surface a browser calls — envelope, status
codes, pagination, id translation — was written by hand in JavaScript, because ESS projects a
command-shaped HTTP surface and says in its own generated description that it does not invent a
resource.

## The question this epic answers

Not *can ESS describe a system* — it can. **Can a specification be run?** Today it cannot: it can be
compiled into types and refusals, and a human supplies the behaviour. For CRUD the specification
already determines the behaviour, and the emitter declines to take it.

## The register

Five blocking gaps, four secondary, three defects, each filed as a story that decomposes this.
Close the five and ESS generates the fake; gap 1 alone yields behaviour, 2 to 4 make it reachable
from a browser as REST.

Everything here was measured against ESS 0.23.0 while building the thing, not reasoned about. Four
things that looked like gaps were probed and are not: per-field `wire:` naming, cross-domain
`creates:`/`moves:`/views, a domain spanning two files, and `views` / `reached_by: network`.

## Provenance

Filed from the adopter's services repository by the session that hit them. The adopting model, the
hand-written realization and the shim are on the services repository's `ess-docs` branch and its frontend branch
branch `fake-backend` — both being removed, so the citations below are to ESS's own sources and to
command output quoted in each story, not to that work.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
