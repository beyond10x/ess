---
format: aep.planning-md/3
id: story:a-delete-command-may-take-its-input-from-the-query-string
kind: story
status: implemented
title: A command on delete may encode its input as query parameters
summary: ESS-HTTP-020 admits request encoding query on get only; A2A 11.5 lets DELETE take query parameters (a2a wave 2, H4, 2026-10-22).
tags:
- adopter-report
- ess-0.70.0
- gap
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- informed_by: story:a-command-binds-to-a-protocol-or-deployment-http-route
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-23T04:42:46Z", actor: "human:timo", revision: 5, correlation: "ess-0690"}
- {from: "proposed", to: "active", at: "2026-10-23T04:42:46Z", actor: "human:timo", revision: 6, correlation: "ess-0690"}
- {from: "active", to: "implemented", at: "2026-10-26T09:27:25Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":1,"verification":1}}}
---
## Need

Reported by the adopter session specifying A2A in beyond10x/a2a (a2a wave 2, story http-binding,
ESS 0.67.0, relayed 2026-10-22): **H4**, a2a wave-2 HTTP binding gaps list, line 11. A2A section
11.5 says GET and DELETE take path or query parameters; a `delete` command whose input is not all on
the path cannot be encoded `query`:

> ESS-HTTP-020: `query` admitted on `get` only

## Where it lives

- Refusal: `crates/specify/ess-http/src/lib.rs:3676-3684` ("`query` carries the input in the query
  string, the encoding of a command on `get` only").
- Rule table: `lib.rs:2506`; design `docs/design/http-presentation-binding.md:150-152`.
- `ESS-HTTP-014` (`lib.rs:2505`) states the same for `get`; a `delete` with unmapped input and no body
  needs the same allowance.
- The OpenAPI projection of a `query` request (`crates/generate/ess-gen/src/openapi.rs:211`) already
  spells `in: query` parameters; it is keyed on the encoding, not the method (not verified for
  `delete`).

## Format

Admits documents refused today, without changing the meaning of any valid one: additive within
`ess-http/1` and `ess-http-ir/1` (the method and encoding already exist in the IR). `ess/N`
unchanged.

## Further report

Source: a2a wave 3 (an adopter's server implementation, 2026-10-22), reproduced on 0.68.0.

- Need: the generated server reads every DELETE input from a JSON body; the protocol sends them in the query string.
- Reproduction (minimal spec, a deleting command with an `Optional<String>` input, bound `method: delete` with `request: {encoding: query}`):

```text
error[ESS-HTTP-020]: routes[1].request.encoding: `query` carries the input in the query string, the encoding of a command on `get` only; a command on `delete` is encoded `json` or `form`
```

- Where it lives: `crates/specify/ess-http/src/lib.rs:3679-3680` (traced: the message above).
