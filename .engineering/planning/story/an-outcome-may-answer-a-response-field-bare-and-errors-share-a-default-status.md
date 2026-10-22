---
format: aep.planning-md/3
id: story:an-outcome-may-answer-a-response-field-bare-and-errors-share-a-default-status
kind: story
status: draft
title: An ess-http/1 outcome may answer one response field bare, and error outcomes may share a default status
summary: 'body: response keeps the response object wrapper; upstream answers the bare message and puts errors under default (a2a wave 2, H6, 2026-10-22).'
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 4
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H6**, a2a wave-2 HTTP binding gaps list, line 13. Upstream answers
the bare response message (a `v1Task`); ESS answers `{outcome, published, response}`, and errors
cannot be put under one `default` status. No refusal: the adopter did not try, because
`outcomes: {<o>: {body: response}}` still keeps the `{task: …}` wrapper (the command's response
object, whose one field is the task).

## Where it lives

- Body forms: `crates/specify/ess-http/src/lib.rs:484-497`: `Envelope` (the default) and `Response`,
  "the command's declared response object, bare". The gap is one level further: answering the value
  of the response's single field, not the response object.
- Status per outcome: `ESS-HTTP-023` and `-025` (`lib.rs:2509`, `lib.rs:2511`) hold each outcome to
  its own status; there is no `default` status for every error outcome, as OpenAPI's `default`
  response and the upstream document use.
- Overlaps `story:response-field-declares-its-value-source` (a2a wave 1, M7), which concerns the
  response's fields, not its wire body; checked, not a duplicate.

## Format

A new body form (for example `response.<field>`) and a default status are new keys or words in
`ess-http/1`; documents valid today keep their bytes, so additive within `/1` under the stage-2
precedent. `ess/N` unchanged. Fit review decides.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22), reproduced on 0.68.0.

- Count: 39 generated error sites in the adopter's server answer with ESS's envelope at status 502 where the protocol declares another status.
- Reproduction (minimal spec, a command with an `external:` refusal and no `outcomes:` in its binding): the generated `--layout crate --http` server answers that refusal with status 502 and the envelope body. With `outcomes: {unsupported: {status: 400, body: {error: error}}}` declared, the same server answers 400 with the error object. So a declared per-outcome status works; the 39 sites are refusals the binding lists no outcome for.
- Where it lives: an `external:` refusal derives `UPSTREAM` = `"502"`, `crates/specify/ess-http/src/status.rs:28` and `:74` (traced through the generated output). The missing piece is the default this story names: one status and body shared by every error outcome a route does not list, so 39 sites need not be listed one by one.
