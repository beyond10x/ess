---
format: aep.planning-md/3
id: story:format-gated-construct-is-named-at-validation
kind: story
status: draft
title: A construct whose projection waits for a later format is named at validation
revision: 1
---
## Context

`returns: true` on an outcome puts the command's `response:` into the OpenAPI success body only from
format `ess/22` (`crates/generate/ess-gen/src/openapi.rs`, the status table; beyond10x/ess#424).
Below `ess/22` the projection keeps the earlier `202` body by design. `ess specify validate`
accepts `returns: true` at `ess/17` to `ess/21` without a word, so an author on an older format
believes the response is on the wire when it is not. This came up as a reported regression of
#423 that was only an old format.

## Outcome

A construct whose projection depends on a later format than the one a system declares is named at
validation. The note says which construct, which format changes it and what the projection does
meanwhile, and the document stays valid.

## Acceptance

- `ess specify validate` on a system at `ess/17` with an outcome declaring `returns: true` exits 0
  and prints one note naming the outcome, `ess/22` and "the response stays out of the success body
  below `ess/22`". Test: a case in `crates/edge/ess-cli/tests/openapi_accounting.rs` (an existing
  binary).
- At `ess/22` or later there is no such note.
- The note is a structured diagnostic with a stable code, so `--format json` carries it.
- No new `tests/*.rs` file.
