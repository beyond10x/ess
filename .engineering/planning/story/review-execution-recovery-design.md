---
format: aep.planning-md/1
id: story:review-execution-recovery-design
kind: story
status: active
title: Specify the recovery contract for finite deployment execution
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: inferred
  path: docs/design/review-execution-recovery.md
revision: 5
---
## Finding and source

F11 (P1) from `docs/reviews/2026-09-05-architecture-review.md:394`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/edge/ess-cli/src/main.rs:1613`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

A binding design enumerates recoverable outcomes for partial deployment failure and retries instead of treating a caller-supplied desired plan as proof of applied state.

## Implementation boundary

Design only: define the supported finite execution workflow, observed versus desired versus applied claims, unknown state, manual drift, retry/removal behavior and per-release versus multi-release atomicity. First model any newly introduced execution evidence in a validated typed home; do not invent receipt fields in this story. Name fake-executor acceptance vectors and exact source modules the follow-on implementation will need.

## Validation

Review a failure after each external action, interrupted evidence persistence, equal desired plans after manual change and removal retries. Each case must have one stated observable recovery outcome and a test strategy. Record unresolved ownership/cardinality/authority semantics as UNMAPPED rather than guesses.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No continuous controller or live deployment. This design alone does not close F11's execution weakness: decompose the implementation through AEP after its typed contract is validated; the parent keeps that obligation open.

## Scope

Derived 2026-09-06 by `aep-drive:story-scoper` against clean ESS `ba43fda29de637ad9323d96c4bb9aac10f48ae64` — cited.

- **Primary surface:** design documentation only; define finite deployment recovery, distinguish desired/observed/applied claims, and specify observable outcomes and fake-executor verification for partial failure, interruption, drift, retries and removals — cited.
- **Documents/edit token:** `docs/design/review-execution-recovery.md` — inferred; the story names this proposed binding document, which does not yet exist.
- **Existing symbols to ground the design:** `DeploymentCommand::Reconcile`, `deployment`, `reconcile_release`, `fetch_helm_chart`, `run_external`, `DeploymentIr`, `DeploymentRelease`, `ReleaseManifest`, `Evidence`, `EvidenceKind`, `Identifier` and `Digest`; these are read-only evidence, not implementation scope — cited.
- **Typed-contract boundary:** existing delivery types establish validated intent and release-artifact evidence; they do not establish a deployment execution receipt or applied-state authority. Preserve this distinction rather than adding receipt fields to existing envelopes — cited.
- **Modeling prerequisite:** any concrete new execution-evidence model requires an explicitly scoped ESS modeling step, resolved identity/ownership/cardinality/authority semantics, and successful validation before receipt fields or implementation decomposition are admitted. Unresolved semantics remain `UNMAPPED` — cited.
- **Implementation follow-through:** identify the process/filesystem seams and a complete failure/retry matrix, but leave `obligation:review-execution-recovery-implementation` open until that matrix passes against an integrated implementation commit — cited.
- **Exclusions:** no executor, persisted format, test, live deployment, continuous controller or public Website change in this unit; ESS acquires no AEP dependency — cited.
- **Confidence:** high — the acceptance explicitly requires a binding design, and the current finite executor and validated delivery boundaries were inspected — cited.
- **Would collide with:** another writer of `docs/design/review-execution-recovery.md`; the planning journal, wave page and change record remain coordinator-owned. Read-only CLI references do not reserve the CLI package against the separate empty-scenarios unit — inferred.
