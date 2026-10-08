---
format: aep.planning-md/3
id: story:related-identity-readable-in-when-related
kind: story
status: draft
title: A when_related predicate reads the related row's identity
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#505
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A `when_related` predicate reached `via:` a reference can read the related row's identity, so a
command can refuse an answer whose named issuer differs from the server it was sent to.

## Evidence

GitHub https://github.com/beyond10x/ess/issues/505, reproduced on ess 0.56.0: `predicate: issuer != input.iss`
is refused with ESS-COMMAND-003 (`unobservable_fact`), because the identity is not one of the
related entity's stored fields; `predicate: active == false` validates.

## Acceptance

- The related entity's identity field is an observable root inside its `when_related` predicate;
  the issue's reproducer validates.
- Synthesis witnesses the predicate with a related row whose identity equals and one whose
  identity differs from the input.
- The interpreted target executes it; `ess verify diff` rates the added refusal as it rates any
  other added related guard.
