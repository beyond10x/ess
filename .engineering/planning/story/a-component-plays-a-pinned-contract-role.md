---
format: aep.planning-md/3
id: story:a-component-plays-a-pinned-contract-role
kind: story
status: draft
title: A component plays a component of a pinned contract specification, checked by compose
tags:
- adopter-report
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#525
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 2
---
## Outcome

A service specification can state that one of its components plays a component of another specification it pins by digest (a contract role), and which of its own commands stands where in that contract, and `ess specify compose` checks the claim against both compiled surfaces instead of a comment asserting it.

## Evidence

https://github.com/beyond10x/ess/issues/525 (ess 0.57.0, neutral reproducer `repro.sh` in the issue: system `demo`, component `relay`, contract system `wire`, component `wire-server`). Every form is refused today: a `plays:` key on the component (unknown field), a binding to `wire.server.Subscribe` (ESS-BINDING-001), the contract's domain listed in `demo` (ESS-SPEC-004), `roles:` in `ess-composition/3` (unknown field), and a command-to-command conformance (`unresolved_semantic_reference`). Composition admits type conformance only (`docs/design/composition-type-conformance.md`).

## Fit review

Not yet written. Run `.agents/skills/assessing-external-requests/SKILL.md` before this story is proposed: the requester's two syntaxes (`ess-composition/4` `roles:`, or a component `plays:` key) are evidence of the need, not the design. The design decides where the role lives (composition or system), what "stands where" means for commands (gates, decides, delivers), and the format consequence.

## Acceptance

- The issue's reproducer, with the chosen form, composes and records the role in the composition IR at both digests; a role naming a command or component the contract does not declare is refused, naming it.
- A design page under `docs/design/` precedes the code.
- The format consequence is decided explicitly, with an old-reader test.
