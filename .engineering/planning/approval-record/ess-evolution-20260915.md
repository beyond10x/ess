---
format: aep.planning-md/2
id: approval-record:ess-evolution-20260915
kind: approval-record
status: draft
title: Operator approval of revised ESS evolution plan and orchestration
relations:
- decides: initiative:ess-evolution
- decides: design:ess-evolution-orchestration
revision: 1
---
## Decision and authority

The operator approved plan ess-evolution-20260915 revision 1 in the current interactive session on 2026-09-15 local time. The approval was explicit: "yes, perfectly covered!" The operator then requested a durable root plan, Astra as planner/orchestrator, and multiple implementation sub-agents one tier below Astra, followed by a clean-context handoff.

The plan's full local source has SHA-256 7579145c3de5a1c6f8088fd7fb804d29dac8903ec505048f3ce595c45023b787. design:ess-evolution-orchestration contains its public-safe projection and the coordinator/worker rules. This is a record of actual operator instructions, not a non-interactive approval bypass.

## Authorized decisions

- Complete the revised initiative through local acceptance, including real planning-store migration in Eventlog, Entity Runtime, Service SDK, ESS, Connectors v2 and AEP.
- Exclude the second application and fake-backend from current scope; remove their current planning requirements and retire their dedicated artifacts while preserving AEP/Git history.
- Track generic protobuf, UI and Flutter as separate deferred follow-up.
- Raise affected Eventlog-backed runtimes to Rust 1.91 while preserving independent pure-library compatibility.
- Persist the plan and checkpoint before implementation; use gpt-6-astra for planning/orchestration and gpt-5.6-sol for implementation and independent review.
- Preserve dirty primary checkouts, maintain one planning writer per repository, use managed worktrees and qualified gates.

## Limits and current result

Publication, releases, deployment, broader organization rollout, and historical record erasure are not covered. Implementation and migration evidence remain outstanding. Local planning persistence and the handoff are the current task; no implementation wave has been launched. A future unit or wave record must identify its exact scope, source vector, resources and applicable authorization.
