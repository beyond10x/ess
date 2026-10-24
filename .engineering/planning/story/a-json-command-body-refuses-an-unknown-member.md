---
format: aep.planning-md/3
id: story:a-json-command-body-refuses-an-unknown-member
kind: story
status: draft
title: A JSON command body refuses a member no input field names
summary: 'Generated servers answer 201 to an unknown JSON member under the default unknown: refuse; the design says malformed.'
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Need

Found while landing `story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status`
(0.69.0, adversary pass 1, note N1). On an `ess-http/1` route with the default JSON encoding and the
default `unknown: refuse`, both generated servers answer `201` to a command body that carries a
member no input field names, such as `{"item_id":"i1","count":1,"x":1}`. The design says such a
request is malformed: step 6 of the answer order lists "an unknown key under `refuse`" among the
inputs that cannot be read (`docs/design/http-presentation-binding.md:555`), and decision 14 says
"A JSON body stays closed, as ESS's bodies are today" (`:691-692`).

## Where it lives

- Observed by the adversary's runtime probe against both generated servers (Rust and Go), on the
  integration branch at `d3f407440a`; not reduced to a line of the generators yet.
- Existing HTTP binding tests send extra members on JSON routes and expect success, so a fix moves
  their expectations; which of the two the design means is the first question.

## Acceptance

- A JSON command body with a member no input field names, on a route whose `unknown` is `refuse`,
  is answered with `refusals.malformed`, else `400`, by the generated Rust and Go servers.
- The OpenAPI request schema for such a route says `additionalProperties: false`, or the design
  records why it does not.
- A conformance or HTTP binding scenario holds both servers to it.

## Format

None expected; the binding already declares `unknown: refuse` as the default.
