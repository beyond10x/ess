---
format: aep.planning-md/3
id: story:an-outcome-schema-names-how-its-outcome-is-selected
kind: story
status: draft
title: An outcome's response schema names structurally how the outcome is selected
summary: External and unknown-identity outcomes are marked only in description prose in the generated OpenAPI.
tags:
- downstream
- ess-issues-3
revision: 1
---
## Problem

The generated OpenAPI contract says how an outcome is selected only in prose. The schema
`description` reads "Decided outside the request: <cause>." for an `external:` outcome and
"Taken when the request names an identity no record carries." for an `unknown_instance:`
outcome. A consumer that needs to know which outcomes a black-box client cannot bring about
matches that English text, and a rewording of the generator's prose silently changes what the
consumer decides. The error status per binding (HTTP, JSON-RPC, gRPC) is likewise prose in the
error summary. Reported 2026-10-29 by an adopter on ESS 0.75.0, whose test kit matches the
description text.

Checked on the 0.77.0 head (84671d6) with `ess generate --kind openapi` over the adopter's model
at a2a d9e658f:

- `a2a.discovery.GetExtendedAgentCard.not-supported.Response` carries only `description`,
  `additionalProperties`, `properties`, `required` and `type`.
- The `x-ess-*` members written are consistency, entities, field, kind, map-key, may-invoke,
  may-read, name, provenance, reached-by and relation. None of them says how an outcome is
  selected. The prose comes from `crates/generate/ess-gen/src/openapi.rs:2616`.

## Acceptance

- Each outcome response schema carries a structural member naming how it is selected (for
  example `x-ess-selected-by: external | unknown-identity | guard | default`), drift-checked
  with the generated contract.
- A consumer can list the external outcomes of a command from the contract without reading any
  `description`.
