---
format: aep.planning-md/3
id: story:an-http-binding-routes-an-operation-of-a-pinned-sibling-specification
kind: story
status: draft
title: An ess-http binding may route an operation of a pinned sibling specification
summary: ess-http/1 refuses a route naming a pinned sibling's command or view as ESS-HTTP-004 (adopter report)
tags:
- backlog
- external-request
- from-adopter-report
relations:
- serves: vision:O2
- decomposes: epic:adopter-gaps-wire-and-answers
revision: 1
---
## Need

A downstream adopter splits one service's model over several specifications that pin each other.
Some of the service's HTTP routes serve an operation that a pinned sibling specification declares.
An `ess-http/1` binding routes only operations of its own specification: a route that names a
command or view of the sibling is refused as `ESS-HTTP-004` ("names one the specification does not
declare", `crates/specify/ess-http/src/lib.rs:3082`). The adopter lists six such routes as unbound.

## Acceptance

- An `ess-http/1` route may name a command or a view of a specification its own specification
  pins, by the pinned name. The route validates, and the projected OpenAPI document carries the
  operation with the sibling's types.
- The pin's digest decides which declaration is meant: a route naming an operation the pinned
  version does not declare stays `ESS-HTTP-004`, and the diagnostic names the pin.
- Conformance over the binding runs the sibling operation's scenarios through the route.
- A binding that names only its own operations keeps its bytes.

## Evidence

The refusal code and its text are read from `crates/specify/ess-http/src/lib.rs:3082`. The
adopter's six routes were reported by the adopter's session on 2026-11-11 and not reproduced in
this repository; the scoper builds a two-specification fixture that shows the refusal before the
story is proposed.
