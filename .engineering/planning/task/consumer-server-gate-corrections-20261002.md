---
format: aep.planning-md/3
id: task:consumer-server-gate-corrections-20261002
kind: task
status: implemented
title: Correct the queued generated-server batch correctness failures
relations:
- decomposes: story:go-generated-behaviour
- informed_by: story:served-view-params
- serves: vision:O2
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T08:43:00Z", actor: "human:timo", revision: 2, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T08:43:00Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-02T09:49:31Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"test_result":2,"verification":4}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
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

## Delivery evidence

Published combined candidate: f0b220099119090795c93cae07c14e6209918f67, https://github.com/beyond10x/ess/pull/386. Superseded PRs 383 and 384 were closed through the bot App after exact-head ancestry verification. Both original PR bodies retain the consolidation reference. No code was discarded.

Local checks: 453 affected-package passes, two existing ignored; exact task test-xtask 362 passes, three existing ignored; eight concurrent nextest passes; strict Clippy and formatting pass. Read-only adversary found no concrete counterexample and ran no tests. Source correction commit 51b3d92ff plus merge of current main produce the final candidate; merge changed only the already-published UI read-filter design.

Remote correctness run 36987215656 remains pending at observation. Common security run 36987213274 failed on 2026-10-02 at 09:03:49Z: `b10x-gates: candidate exceeds scan limit`. A local signed common receipt passed but does not replace required remote admission. Integration remains blocked by the repository secret policy, not waived by local evidence.

## Remote correctness result

At candidate f0b220099119090795c93cae07c14e6209918f67 all four test shards, Checks, both test archives, both macOS ownership checks and the final ESS Gate completed SUCCESS in CI run36987215656. Documentation source/build and planning checks also completed SUCCESS. Required common Security and privacy remains FAILURE with the separately recorded candidate-exceeds-scan-limit refusal. No unchanged rerun or integration occurred.

## Integration

PR386 merged by b10x-bot[bot] as fa08de5bd816b3cc46694ec4d19d20d13e128764 after every required check completed SUCCESS, including the security-only retry after authorized secret refresh. Exact merge tree equals tested f0b220099 tree2105a56fd3b86a78d8db839ee04bc8826f663a38. No source release is claimed: the0.52 release remains outstanding with its own exact-commit checks/assets. Issue314 remains open for its separate store/entry requirement; newly accepted server defects316/379/385 stay separate local follow-on work.
