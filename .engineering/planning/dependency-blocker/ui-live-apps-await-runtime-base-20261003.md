---
format: aep.planning-md/3
id: dependency-blocker:ui-live-apps-await-runtime-base-20261003
kind: dependency-blocker
status: cleared
title: Serial guard waves await the independently reviewed runtime base
relations:
- blocks: story:feature-request-282
- blocks: story:related-via-optional-input
revision: 3
transitions:
- {from: "open", to: "cleared", at: "2026-10-03T14:51:51Z", actor: "human:timo", revision: 3}
---
## Missing prerequisite

The approved serial #282/#304 implementation overlaps active work owned by a separate ESS coordinator in crates/verify/ess-conformance/src/interpret/execute.rs, synthesize.rs and synthesize/{caller,subject_fact,related_guard,existence}. That owner is completing reviewed runtime/#312 changes on batch/consumer-runtime-20261002. Operator explicitly requested cross-session coordination to prevent conflicts.

## Agreed boundary

Both coordinators agreed the separately reviewed runtime batch should land independently first; these five approved units must not silently import the broader runtime carrier. No reliable landing ETA was supplied. Its initial_state/conformance-envelope changes require a new source-base reconciliation; the Optional story's ScenarioId and older-source-format constraints remain binding. No runtime source series has been imported by this batch. Source-only transfer is only an alternative proposal requiring concrete scope/dependency reconciliation, not an approved shortcut.

## Evidence and next action

Peer coordination messages are recorded in runbook:ui-live-apps-serial-integration-20261003. The runtime owner supplied reviewed source provenance, including RelatedField85754ad031178695fb72169a9850dc7bf60fbed8, but explicitly warned against whole-carrier import and is still completing312. Once a stable reviewed runtime baseline lands, inspect its exact commit, reconcile this batch against current main, update test/format expectations without weakening acceptance, then resume #282 and #304 serially. The #318 server decoder correction is disjoint and continues. All ess-transports work belongs exclusively to the third Claude session; releasePR398 belongs to another coordinator and is outside this batch.

## Resolved source handoff

The operator transferred the final frozen60-commit source handoff and authorized completion of the whole held bundle by this sole integrator. Source inventory was independently audited, current released main merged, and all60 selected deltas plus companion designs applied on the same integration branch. Local bot checkpoint48d5cc77b38f919381d04830475ff6ea4959289f is the stable shared source base. The older318 and transport/duplicate exclusions remain preserved. Locked offline dependency resolution and source formatting pass; combined runtime checks and regeneration are still due and are not claimed by clearing this source-availability blocker.

The other runtime writer has handed off and stopped. No competing owner holds these runtime source files. The previously missing reviewed, immutable base is now available for the accepted serial282/304 work, so this dependency can clear. Transport ownership and aggregate completion are separate unresolved dependencies, not cleared here.
