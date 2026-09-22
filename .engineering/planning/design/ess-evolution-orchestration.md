---
format: aep.planning-md/1
id: design:ess-evolution-orchestration
kind: design
status: draft
title: Approved ESS evolution scope and orchestration
relations:
- designs: initiative:ess-evolution
revision: 1
---
## Approved scope and source

Plan ID: ess-evolution-20260915; approved revision: 1.
Root plan SHA-256: 7579145c3de5a1c6f8088fd7fb804d29dac8903ec505048f3ce595c45023b787.
Source: local-evidence:ess-evolution/plan-revisions/0001.md.
The operator explicitly approved the revised plan and Astra orchestration with Sol workers. This body is the public-safe projection of the full local root plan: the excluded local application's name is replaced with "the excluded second application". Absolute worktree paths and live operational state stay in the local handoff. This projection does not remove or reinterpret any implementation requirement.

This record owns the approved execution design and orchestration handoff under initiative:ess-evolution. Existing initiative/migration/test documents still require the scoped reconciliation in plan step 1. Creating this record is not implementation, runtime acceptance, a migrated store, or a published result.

# Complete the revised ESS evolution initiative

Plan ID: ess-evolution-20260915
Approved revision: 1
Approval: the operator approved the complete plan in this session and then authorized Astra orchestration with workers one tier below.
Resume entry point: ESS-EVOLUTION-HANDOFF.md
Live progress: .ess-evolution/CHECKPOINT.md
Orchestration rules: .ess-evolution/ORCHESTRATION.md

This document pins approved scope, sequence, compatibility decisions and acceptance. It is not evidence that implementation has happened. Owner-local AEP records own status and evidence. Amend scope explicitly; never silently edit this revision to match a partial implementation.

## Outcome and scope

Finish Eventlog-backed execution, migrate the real AEP planning stores, converge ESS and Service SDK, and verify Connectors v2 adoption.

- Remove the excluded second application and fake-backend requirements and notes from current planning/design documents.
- Archive the fake-backend epic and its 12 stories through AEP. Preserve historical AEP/Git records.
- Move generic protobuf, UI, and Flutter capabilities into a separate deferred follow-up, outside this initiative's completion criteria.
- Complete local acceptance; releases, external publication, and deployments remain separate.

## Implementation sequence

1. **Correct the governed plan and prepare the workspace.**
   Update the initiative, architecture, migration, acceptance, and ADR documents to reflect these decisions. Establish owner-local work items and the required cross-repository migration ADR. Use managed worktrees, preserve existing changes, and retain one authoritative planning-store lineage and writer per repository. Record exact source commits, toolchains, dependencies, configuration, and store inventories. Recover build space through reviewed disposable-output cleanup before full gates; approximately 4.4 GiB was free during planning. Refresh that observation before use.

2. **Qualify the existing Eventlog foundation.**
   Reuse the implemented file provider and atomic append groups. Verify file/SQLite/PostgreSQL parity for ordered transactions, repeated-stream expectations, idempotency conflicts, guards, projection rollback, crash recovery, blobs, and snapshot invalidation. Fix demonstrated gaps before advancing consumers. Preserve eventlog-file/1 bytes and reject corruption or divergent histories.

3. **Add asynchronous ER execution and Eventlog persistence.**
   Introduce an executor outside entity-core and an Eventlog adapter implementing asynchronous recorded-storage ports. Persist complete decisions, including accepted zero-event decisions, and ordered observations without advancing entity revisions. Preserve global record-ID conflict detection, transaction-local expectations, atomic multi-entity batches, and verifiable replay. Store record content through Eventlog's blob mechanism. Retain synchronous compatibility through an explicit bridge outside the kernel, and provide compatible SQLite/PostgreSQL facades with explicit legacy imports.

4. **Deliver exactly one owning AEP migration story.**
   Add Eventlog file authority with tracked Markdown projections. Cover legacy Markdown, hybrid, SQLite, and PostgreSQL inputs; preserve identities, revisions, relations, bodies, evidence, and available history. Represent incomplete legacy history as an explicit import boundary.

   Add inventory, dry-run migration, apply, verification, and projection-rebuild commands. Migration stages a destination, verifies equivalence, rechecks source identity under the writer fence, then switches configuration. Conflicting sources refuse migration. Direct Markdown edits report drift. Post-commit projection failures return the durable receipt; rebuilding never repeats the command.

5. **Cut over the real planning stores.**
   Rehearse against preserved snapshots, then migrate **Eventlog -> ER -> Service SDK -> ESS -> Connectors v2 -> AEP**. Use the qualified new AEP executable explicitly, migrating its own store last. Verify history, queries, mutations, restart, and projection rebuild after every cutover. Retain legacy recovery copies. Once new commands commit, recovery follows Eventlog authority rather than reactivating stale writers. Remove hybrid selection from the new runtime configuration.

6. **Converge ESS service semantics and Service SDK execution.**
   Extract reusable lowering into ess-service-contract; add ess-entity-runtime to produce ER definitions and binding plans. Crosswalk creation, updates, transitions, predicates, invariants, identities, relations, exact values, outcomes, and event multiplicity. Implement missing semantics in ER before admitting the lowering. Make Service SDK delegate decisions and replay to ER while retaining authentication, authorization, hosting, queries, content policy, and external effects in bindings. Preserve existing SDK entrypoints and legacy artifact readers.

7. **Complete Connectors adoption and infrastructure acceptance.**
   Migrate Connectors v2's local metadata authority to ER over Eventlog SQLite, preserving existing identities, credential references, fences, revisions, and audit history. Keep credentials in the qualified Secret Service binding. Preserve generated CLI behavior, supervision, restart reuse, repair/revoke races, protected inputs, bounded queries, and uncertain outcomes. Advance exact ESS/AEP pins, generated sources, and lockfiles, including the excluded CLI workspace; retain existing pinned corrections until equivalent upstream behavior is verified. Link selected service and local-process requirements to deterministic infrastructure projections and independently supplied observations.

## Interfaces and compatibility

- Add asynchronous recorded-store/executor interfaces and a versioned ER-Eventlog record envelope.
- Add aep plan store operations for inspection, migration, verification, and rebuild. Introduce aep.project/2 for the new authority/default semantics; retain legacy configuration readers for migration.
- Introduce opt-in service-runtime-ir/4 and service-realization-plan/4 for ER execution. Preserve existing format meanings and canonical fixtures.
- Raise Eventlog-backed runtime minima to Rust 1.91. Keep independent pure libraries on their existing supported minima and test those boundaries separately.
- Preserve ESS package identities, existing formats, dependency boundaries, and feature-preservation accounting.
- Track the AEP Eventlog authority under .engineering/state/: manifest.json, events.jsonl, blobs and any provider-required durable metadata. Markdown under .engineering/planning/ remains tracked and derived. Only disposable caches and runtime-local operational files may be excluded; never exclude durable authority to make a gate pass.
- The proposed new command and format names above are implementation requirements, not commands already available in installed AEP 0.55.0.

## Verification

- **Storage:** concurrency, atomic rollback, identical retries, changed-content conflicts, zero-event records, observations, tampering, crash/reopen, corrupted blobs, and stale snapshots. Execute PostgreSQL lanes against disposable databases.
- **Migration:** every legacy configuration; divergent inputs; exact preserved history; interruption before and after cutover; Markdown drift; deleted projections; committed mutation with failed projection; safe retry.
- **Services:** generated billing/gatepass fixtures build, start, serve HTTP, persist, restart, and retain authorization, query, and effect behavior. These are synthetic regression fixtures; they do not reinstate either excluded adopter.
- **Connectors:** existing provider reads plus disposable keyring/provider fixtures covering restart, repair, revocation, protected input, uncertainty, and redaction.
- **Gates:** ESS task check with consumer coverage enabled and task site-build; ER/AEP/Service SDK task check; Eventlog bash scripts/gate.sh; Connectors cargo run --locked -p connectors-build -- gate --msrv. Update and exercise affected compiler-minimum checks.
- Retain independent fault sensitivity: wrong field mapping, dropped state/event updates, incorrect transitions and bypassed revocation must fail their corresponding tests.
- Projections never deploy or infer observed infrastructure from desired declarations. Keep credential redaction and its mutation checks.

## Completion criteria

Close the revised initiative only after all six real planning stores use verified Eventlog authority, ER-backed services and Connectors pass acceptance, and every required gate succeeds against the recorded source/configuration vector.

Record actual evidence and lifecycle transitions through AEP. Deferred capabilities remain separately tracked. Missing or skipped required evidence prevents completion. Retain an explicit managed-worktree handoff with commits and verification logs until authorized publication permits cleanup.

## Authority and preservation

The operator chose real participating-store migration, current-plan cleanup with historical records retained, separate deferral of protobuf/UI/Flutter, and Rust 1.91 for affected runtimes. These decisions supersede the older broader ESS evolution prose.

The root copy is local and may name excluded local applications. Its governed public-safe copy uses a generic label for the excluded second application, records this projection explicitly, and binds back to the root revision by digest. Neither copy grants publication authority. Existing primary-checkout changes and old journals are not disposable and must not be concatenated or overwritten during integration.

## Orchestration contract

# ESS evolution orchestration

Plan: ess-evolution-20260915, revision 1.
Applies only to this initiative. The approved root plan controls scope and sequence; owner-local AEP artifacts control lifecycle and evidence; CHECKPOINT.md is a bounded recovery index.

## Roles and execution

- Planner and orchestrator: gpt-6-astra. Owns decomposition, decisions within approved scope, scheduling, shared coordination changes, AEP writes, integration and acceptance.
- Implementors: gpt-5.6-sol, high reasoning by default. Implement the assigned source/tests in a managed worktree.
- Independent reviewers and story scopers: gpt-5.6-sol, high reasoning, fresh contexts. A reviewer must not review its own implementation.
- At most three active child agents alongside Astra. The current harness has four total slots. This is a ceiling, not a target: measured disk, build cost and service capacity may reduce concurrency. Initially allow only one heavy build at a time.
- Use explicit model overrides and fork_turns="none" for new workers. Give each a persisted brief path and the minimum task context; do not fork the full orchestration conversation.
- Astra alone dispatches further agents. Workers do not fan out, change approved scope, write planning stores, update root orchestration files, publish, deploy or select an alternative model.
- If an assigned model is unavailable, report the constraint. Never silently change models.
- These are coordinator operating rules, not claims of engine-enforced execution. No model-backed AEP driver run has been started.

## Work selection and review

Use aep-plan:planning and aep-drive:wave (inspected version 0.8.1), plus the repository's applicable AGENTS.md and worktree skill. Read current installed versions again only if the environment changes. Approved initiative scope and delegation are already granted; record each bounded wave before dispatch and preserve the existing release/publication exclusion.

Scope ready units with read-only scopers when needed; record cited or inferred paths through AEP, check dependencies and collisions, and retain the wave computation. Fix wrong scope entries before dispatch. A different directory name is not proof of independent work. Shared manifests, lockfiles, generated catalogs and planning journals require serialized ownership.

Read the installed aep-drive implementor, adversary and story-scoper charters before using their roles. The available collaboration tool has no subagent_type parameter: use its explicit model override and load the relevant charter by path in the persisted brief. Record this adaptation rather than claiming native plugin-agent dispatch.

The implementor returns source/tests and a compact handoff. Astra creates the required local bot commit and hands its exact submitted commit to an independent reviewer in a separate managed checkout. The reviewer may add regression tests in its assigned checkout but may not repair implementation or relax assertions. Corrections return to the implementor. Follow the wave skill's two-pass review rule; a red unit is not integrated and is not declared complete.

Run affected checks while developing. Run the actual whole repository gate on each stable integration slice; capture its own exit status and whether each required lane really executed. A skipped lane is not passing acceptance. Never replace the required whole gate with an invented equivalent command sequence.

## Durable records

Before a dispatch, write its brief and the wave/unit record. Every unit has a stable ID, owning repository/AEP reference, dependencies, exact base and submitted commits, managed worktree ID/path, branch, build directory, scratch directory, agent role/model/session, stage, acceptance commands, review commit/verdict, evidence links, blocker and next action.

Distinguish queued, scoped, implementing, implementation-ready, reviewing, correcting, accepted, integrated and blocked. These are local scheduling labels, not invented AEP statuses. Read each artifact's real lifecycle from AEP before moving it.

Update CHECKPOINT.md after each handoff, integration, blocker, scope amendment and before ending a session. Keep it approximately 120 lines or fewer. Move detail to wave/unit records and logs, and link it. Preserve CHECKPOINT.previous.md and replace the current checkpoint atomically; read it back.

Approved plan snapshots live under plan-revisions/. Changes require an explicit revision/amendment with a reason and authority, plus updated digests and governed copy. Do not silently rewrite an approved snapshot. Root files are outside Git; retain the governed copy and local snapshots. They are durable local files, not remotely backed-up publication.

AEP artifact mutations use only its CLI and one writer per repository. Preserve original journals as distinct lineages. No worker merges, concatenates or directly edits them. Generic paths and evidence references go into public artifacts; absolute workstation paths and excluded local application names stay in this local root control directory.

## Resume without the old conversation

1. Read ESS-EVOLUTION-HANDOFF.md, ESS-EVOLUTION.md and CHECKPOINT.md; read this file before dispatch.
2. Verify the approved plan digest and revision. Check the named governed copy and current owner records for drift.
3. Reconcile recorded Git heads, dirty state, managed worktree status/leases and actual active agents/processes. Recorded agent IDs may be stale.
4. Inspect any existing work before assigning it again. Resume or reassign its remaining work from the actual tree; do not recreate an already implemented unit because a session vanished.
5. Use the checkpoint's next eligible action. Open only its brief, owner artifact and relevant evidence. Do not load whole journals, complete prior chat histories or every worker log.
6. If a checkpoint contradicts code or AEP, reconcile the affected unit using exact evidence. Do not mark later units ready on an unverified summary.

## Storage, gates and handoff

Use the worktree CLI for all managed lifecycle operations. Maintain and release only your own lease. Build and scratch paths must be recorded and owned by the assigned unit. Inspect capacity before large builds; never clean another live agent's output.

Retain review/gate logs before removing disposable output. Publication is outside the approved scope, so local commits alone do not permit worktree finish/GC. Leave an explicit handoff until an authorized recovery destination has the wanted commits. Only exact reviewed IDs go to gc --apply; never force-remove a tree.

Common Gates checks and bot identity remain mandatory where applicable. No request here authorizes remote publication, release tags, deployments, production credential migration or a wider organization rollout.
