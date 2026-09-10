---
format: aep.planning-md/1
id: migration-plan:ess-evolution
kind: migration-plan
status: draft
title: Incremental ESS runtime and application migration
relations:
- delivers: initiative:ess-evolution
revision: 1
---
## Plan
See docs/design/ess-evolution/migration.md. Providers precede consumers. Keep old readers and original inputs until migration acceptance passes. Never dual-write authorities. Preserve Babelconnect's working-tree edits.

## AEP
One owning story includes configuration, inventory dry-run, identity/revision/relation/body/evidence/history migration, divergence refusal, staged verification, projection drift/rebuild and removal of hybrid runtime selection. Database/Eventlog/ER prerequisites stay in their owning repositories. Journal plus blobs are authoritative and tracked. Direct Markdown edits are drift. Post-commit projection failure reports a receipt; projection retry must not repeat mutation. A snapshot-only import is an explicit legacy boundary.

## Services
Extract reusable lowering from Service SDK into ESS and retain compatible SDK entrypoints. New runtime version selects ER explicitly. Legacy event-only histories are converted explicitly and never described as complete DecisionRecords.

## Applications
Protobuf remains authoritative until descriptor, wire and consumer gates pass. Generate only owned Flutter composition around existing widgets. Keep server business state, Go transport/downstream adapters and separate native/web media bindings. Connectors retains describe/invoke/serve and reviewed local management behavior. Advance its exact ESS dependency and lockfiles only with compatibility evidence.

## Rollout
Local acceptance is the completion boundary. No releases, deployment or documentation publication is authorized by completion of a generator.

