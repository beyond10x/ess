---
format: aep.planning-md/3
id: story:feature-request-330
kind: story
status: implemented
title: Resolve choice option enums from an explicitly supplied model
refs:
- provider: github
  reference: beyond10x/ess#330
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-10-04T11:42:46Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-10-04T11:42:46Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-10-06T09:43:24Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1,"review_outcome":1,"verification":1}}}
---
## Outcome

Choice options resolve a model enum when a model is supplied, while inline/local options remain checked for drift. Direct model lookup is still required; the supported inline workaround alone does not close issue330.

## Fit review

1. Need: options: factory.objective.RiskLevel should reuse the model's enum without duplicating variants.
2. Class: model-aware loading gap. expand.rs:587-632 resolves local document types before the checker receives a model (ess-ui-check/lib.rs:379).
3. Existing expression: local enums/inline variants plus --model are checked against ordinary form command-input enums (model.rs:399-465). Standalone choices and explicitly bound fields are outside that bounded guarantee.
4. Fit: add an explicit model-aware resolution contract, retaining the model-free loader's behavior. Define qualified-name resolution, collisions, invalid/non-enum types and behavior without --model before code. Do not silently turn missing model information into an empty list.
5. Second adopter: a release form shares the same deployment-environment enum as a served command.
6. Cost: loader/check/generation entrypoint coordination and stable diagnostics; avoid changing old model-free API meaning or inventing a new UI-owned copy of model types.
7. Alternatives: duplicate inline variants and check them; require local enum aliases; support the requested model-aware resolution. Workarounds remain documented but are not grounds alone to reject the consumer need.

## Decisions

Keep direct model lookup open and prepare a bounded loading design. Existing drift checks are verified by source only in this assessment; do not claim the requested feature shipped. No implementation assigned until resolution/order/collision behavior is concrete.

## Acceptance

Resolve qualified enum options with --model in form and standalone choices; retain inline variant drift checks; reject non-enum/unknown/ambiguous references and missing model with named diagnostics. Preserve local options without a model and deterministic emitted variant order.

## Evidence

Read-only assessment of55061600, zero executions. ess-ui/src/expand.rs:587-632; ess-ui-check/src/lib.rs:379; src/model.rs:399-465; tests/checks.rs:1491.
