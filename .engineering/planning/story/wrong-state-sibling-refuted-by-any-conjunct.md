---
format: aep.planning-md/3
id: story:wrong-state-sibling-refuted-by-any-conjunct
kind: story
status: implemented
title: A wrong_state scenario refutes each sibling branch through any one of its conjuncts
tags:
- defect
refs:
- provider: github
  reference: beyond10x/ess#516
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-09T01:14:11Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-09T01:14:11Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-09T01:14:12Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

Synthesis builds the `wrong_state` scenario of a command whose sibling branches combine `when:`
with `when_subject:`, by refuting each sibling through any one of its conjuncts.

## Evidence

https://github.com/beyond10x/ess/issues/516 (ess 0.56.0): `ESS-SYNTH-003` "no candidate of the 4 tried satisfies `none
of: not (defined(nonce)), defined(nonce), defined(nonce), …`" for
`AuthenticationRequest/state/Sent/refuses/ReceiveTokenResponse`. The candidate search demands
every sibling's `when:` be refuted alone, ignoring that its `when_subject:` conjunct is already
false for the arranged row; `nonce-missing` (`not defined(nonce)`) and a sibling requiring
`defined(nonce)` cannot both be refuted by `when:` alone.

## Acceptance

- The issue's command yields its `wrong_state` scenario; the scenario's row and input refute each
  sibling through at least one conjunct, and the interpreted target passes it.
- A sibling whose every conjunct holds still makes the scenario refused with the same code.
- Existing suites keep their bytes where the old search found a candidate.

## Scope

`crates/verify/ess-conformance/src/synthesize/` (the wrong-state family and its candidate search),
`crates/verify/ess-conformance/tests/`.
