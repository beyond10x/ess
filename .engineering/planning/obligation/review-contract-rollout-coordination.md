---
format: aep.planning-md/1
id: obligation:review-contract-rollout-coordination
kind: obligation
status: open
title: Coordinate relying readers before changed ESS formats become defaults
relations:
- decomposes: epic:review-boundary-remediation
revision: 5
---
## Outstanding outcome

Any remediation that changes persisted meaning or bytes verified by another repository needs the Atlas ADR and relying-party rollout order required by `AGENTS.md:47` and the organization authority. Atlas `ROADMAP.md:45` at `a8fb936ddcb35c8971311610e5c63cc86d612fab` records the ESS-to-AEP evidence seam.

## Discharge condition

Recorded compatibility and rollout evidence shows that every affected relying reader supports the selected contract before its default writer is enabled.

## Procedure and required evidence

For each selected migration, inventory exact reader/writer revisions, create the Atlas ADR and governed work in each affected repository, prove old/new compatibility using actual adapters, then enable the default writer only in the approved rollout order. Record the checked commits and release evidence. If a remediation preserves bytes and meaning, document that result explicitly instead of manufacturing a migration.

## Known scope

F01 semantic delta/impact vocabulary and artifact provenance, F03 suite/report coverage and count semantics, F06 observation completeness and F08 exact numbers are migration candidates; F04 persisted import accounting and F11 evidence-kind meaning may also require coordination. This list remains subject to each design's actual reader inventory.

Current F03 inspection establishes both AEP aep-ess-evidence::adapt_json and aep-cli::recorded_from_report, plus the closed domain result and predicates, at advertised Git object 00c742e4179593738a2e8aa69e2ecc07d3c89402. The F01 scoper found no external executable delta/impact parser in inspected published siblings, but Service SDK consumes generator APIs and verifies complete output bytes. Source inspection is not execution or deployment evidence. Detailed source/revision inventory is in the owning stories; refresh relying revisions at rollout.

## Scheduling limit

This obligation is not a blocker on byte-preserving P0 fixes or drafting designs. It remains open through the affected release readiness checks and cannot be replaced by an ESS-only green test run.

## F01 compatibility checkpoint

F01's corrected reader/writer/profile and 26-relation dependency graph passed the complete ESS gate at acb7859e3202ffdc1ca840dde67f7ca4da33c746 and the exact-source SDK old/new compatibility experiment. Atlas ADR 0036's final decision is published at 7b67e8e2437ec9956135930435875a8a76139c3f. SDK 48833c6d14ec37cb3b614fca05cf7dd78f63b743 remained unchanged; all seven candidate ESS paths were checked, both generator/checker pairs succeeded, cross-checkers reported measured whole-file drift, regeneration converged, source-bound controls were unchanged and both generated-service compilations passed. The integrated verification report records exact contracts, actual checks and limits. ESS source publication remains pending on the wave page; no SDK pin, release or deployment is implied.

This is evidence for F01 only. The obligation remains open for the conformance, observation, number and other selected migrations; the independent Atlas catalog-intent reconciliation obligation also remains open.
