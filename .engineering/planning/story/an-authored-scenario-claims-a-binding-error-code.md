---
format: aep.planning-md/3
id: story:an-authored-scenario-claims-a-binding-error-code
kind: story
status: draft
title: An authored scenario claims a binding's error code, including JSON-RPC -32602 and -32601
summary: No code in an authored error claim and no transport-level refusal scenarios (a2a wave 1, J4, J6); blocked on the transport and IDL compile-targets ADR.
tags:
- adopter-report
- ess-0.70.0
- feature-request
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
- depends_on: story:a-command-binds-to-json-rpc-and-grpc-beside-http
- informed_by: architecture-decision-record:transport-and-idl-compile-targets
revision: 2
---
**Blocked on `architecture-decision-record:transport-and-idl-compile-targets`. Not to be implemented
before that record is accepted.**

## Need

Reported by the adopter session specifying A2A in beyond10x/a2a, in the a2a wave-1 gaps the
orchestrator relayed on 2026-10-22 (ESS 0.67.0, a2a main 3a18050).

- **J4** (a2a wave-1 gaps list, line 25): an authored error claim cannot name the error's code.
- **J6** (a2a wave-1 gaps list, line 27): there is no way to state the protocol-level refusals
  JSON-RPC defines, -32602 (invalid params) and -32601 (method not found), as scenarios.

Both are one need: an authored scenario checks the code a binding answers, including refusals that
come from the transport rather than from a model outcome.

## Where it lives

The codes themselves are bound per outcome by `story:a-command-binds-to-json-rpc-and-grpc-beside-http`;
this story depends on it. Related: `story:one-refusal-answers-with-any-of-several-wire-codes`
(several string wire codes for one refusal).

## Fit review

Owed: assessed with `.agents/skills/assessing-external-requests/SKILL.md` before dispatch.

## Acceptance (provisional, fixed after the ADR)

- An authored error claim may name the binding's code; the scenario fails when the target answers
  another code.
- A scenario can send a malformed request or an unknown method under a JSON-RPC binding and claim
  -32602 or -32601.

## Schedule

Gap; tagged `ess-0.69.0` as filed from a2a wave 1, not dispatchable before the ADR is accepted.
