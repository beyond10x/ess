---
format: aep.planning-md/3
id: story:a-view-route-answers-declared-refusals
kind: story
status: draft
title: A view may declare refusals, and its route answers them
summary: 'A view route has empty: only; outcomes and errors on a view are ESS-HTTP-022 (adopter report)'
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

A downstream adopter's read routes answer errors a view cannot state: `404` when the parent record
a list is scoped to does not exist, `400` for a refused query, `429` when reads are throttled. A
view route in `ess-http/1` has `empty:` for a `single` view with no row and nothing else:
`outcomes`, `errors` and `alike` on a view are refused as `ESS-HTTP-022`
(`crates/specify/ess-http/src/lib.rs:3094`). The adopter counts about 37 open notes on it.

## Acceptance

- A view may declare named refusals with a condition over its parameters and over records it reads
  by a parameter (for example "the parent a parameter names does not exist"), each naming a
  declared error.
- An `ess-http/1` view route may give each such refusal a status and a body, in the forms an error
  outcome of a command route takes; OpenAPI lists the responses.
- Synthesis writes one scenario per refusal that reads the view in the refused condition and
  expects the error; the interpreted target answers it.
- A refusal the model cannot condition (throttling) is out of scope and stays undeclared.
- A view without refusals, and a binding without them, keep their bytes.

## Evidence

The `ESS-HTTP-022` row is read from `crates/specify/ess-http/src/lib.rs:3094` at 0.93.0. The
adopter's refusals (view route with `outcomes`) were observed by the adopter's session and are not
rerun here; the scoper reruns one before the story is proposed.
