---
format: aep.planning-md/3
id: story:feature-request-309
kind: story
status: active
title: Aggregate with two group keys filled from one input is refused for a move the source does not have
tags:
- feature-request
refs:
- provider: github
  reference: beyond10x/ess#309
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/aggregate.rs
- confidence: inferred
  path: crates/verify/ess-conformance/tests/aggregate_shared_input_keys.rs
- confidence: inferred
  path: docs/design/aggregate-views.md
revision: 8
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T08:44:47Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T08:44:47Z", actor: "human:timo", revision: 7, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

An aggregate whose group keys copy the same creating input is synthesized with a consistent key tuple and an exact observable expectation.

## Acceptance

The named regression `aggregate_shared_input_keys` compiles a minimal single-state entity with two group keys copied from one input, synthesizes its aggregate scenario without ESS-SYNTH-017, and runs it successfully against interpreted commands with independently computed aggregate views over their resulting state. Independent additional keys remain discriminated; reordered declarations and Optional shared keys preserve their meaning. Existing tests that detect truly rewritten keys continue to fail incorrect targets. The view target must not reproduce the tuple planner's algorithm.

## Origin

GitHub issue beyond10x/ess#309, reported from the #272 adversary pass on ESS 0.49.0. The report states that a source with no moving commands is diagnosed as having a move rewrite its origin key.

## Fit review

1. Need: creation copies one input into two fields but tuple planning treats them as independent. The assignments collide at `crates/verify/ess-conformance/src/synthesize/aggregate.rs:1875-1891`, after independent tuples at `:1242-1245`.
2. Class: defect. `docs/design/aggregate-views.md:664-666` says arrangement follows creating sets mappings. The rewrite diagnostic at `aggregate.rs:2529-2572` describes a move even on a source with no moves.
3. Existing expression: the authored model already expresses equal copies without new syntax. The pending regression must demonstrate that its valid shape can be witnessed; no alternative authored spelling is required.
4. Fit: repair tuple planning upstream, preserving `kept_as_planned` and its exactness guard. Keys determined by the same input must vary and omit together; independent keys still need distinct tuples. This changes synthesis, not authored models, reader formats, runtime behavior, or generated APIs.
5. Second adopter: an order records origin-zone and shipping-zone from one selected zone; grouped reporting observes their equality. The semantics use the same sets mapping documented above.
6. Cost: no keyword, persisted field, format bump, migration or target API change. Regression and existing aggregate tests cover the changed planner.
7. Alternatives: change nothing retains a false refusal; weaken preservation checks risks unsound aggregate assertions; model the equality constraints in the tuple planner preserves exactness and is selected.

## Decisions

Accept as proposed. Fix the synthesis defect without weakening refusal checks or changing the model to suit the generator. The implementation is authorized by the operator's 2026-10-02 full-backlog delivery instruction. #361 and #362 are separate shapes and are not implicitly resolved by this correction.

## Scope

- Cited: `crates/verify/ess-conformance/src/synthesize/aggregate.rs`, tuple assignment, row arrangement and key-preservation check.
- Inferred: `crates/verify/ess-conformance/tests/aggregate_shared_input_keys.rs`, new regression through assembly, synthesis and interpreter.
- Inferred: `docs/design/aggregate-views.md`, qualify the independent-key pattern where creating input mappings require equality.
- Confidence high: the source walk identifies the contradictory assignments and refusal. Execution remains to be established by the regression.

## Verification refinement

The implementation's first focused red run fails five regressions with the reported nonexistent-move diagnostic. A first treatment synthesizes those scenarios, but the repository interpreter refuses every view (`Interpreted::query_view`: views are not interpreted yet). The acceptance therefore uses the existing interpreter for commands and an independent aggregate view over its resulting state. This preserves the actual acceptance claim, an exact runnable aggregate scenario, while making the unsupported pre-existing view seam explicit; it is not a reason to omit the Passed assertion.
