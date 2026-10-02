---
format: aep.planning-md/3
id: task:consumer-server-gate-corrections-20261002
kind: task
status: active
title: Correct the queued generated-server batch correctness failures
relations:
- decomposes: story:go-generated-behaviour
- informed_by: story:served-view-params
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T08:43:00Z", actor: "human:timo", revision: 2, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T08:43:00Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
## Outcome

The grouped generated-server candidate containing PRs 383, 384 and 386 passes its affected correctness checks, including the gatepass Entity Runtime consumer and generated Go view-parameter test.

## Acceptance

The previously failing `ess-entity-runtime::lowering` tests and `ess-synth::view_params_served` test pass with assertions that reflect the model, all other tests in those affected crates pass, and strict Clippy/formatting plus the exact failing Checks task pass without weakening assertions or skipping tests.

## Evidence and scope

Actions run 36972424134 at d414cfc213d8871b04ac18b08c47fee3dc0b71ca fails `complete_real_fixture_inventory_keeps_versions_slots_events_effects_and_fulfillment`, `admit_visitor_reuses_the_normalized_badge_for_action_and_event`, and `a_view_param_reaches_the_port_from_the_query_string_go`. Its Checks lane also failed and must be read before correction. The head includes PR 384 and the PR 383 badge assignment. Preserve all source and generated behavior that already passed; update only the demonstrated failing assumptions or implementation defects.

Owning stories: go-generated-behaviour and served-view-params. Source scope is crates/generate/ess-entity-runtime/tests/lowering.rs and the named failure surfaces under crates/generate/ess-synth. No authored surface change is planned.

## Execution

One managed tree `ess-backlog-servers-20261002`, branch `batch/consumer-servers-20261002`, based on d414cfc213d8871b04ac18b08c47fee3dc0b71ca. Build output stays in that tree's target directory; coordinator owns store writes and publication.
