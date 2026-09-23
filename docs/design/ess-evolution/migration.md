# Migration sequence

Authority: approved ESS evolution plan ess-evolution-20260915 revision 1.

## 1. Preserve sources and reconcile scope

Record exact commits, dirty/index/untracked changes, lockfiles, generator identities, toolchains,
configuration and store inventories for Eventlog, Entity Runtime, Service SDK, ESS, Connectors v2
and AEP. Preserve existing edits. Evidence belongs to its exact source/dependency/configuration
vector. Select one authoritative planning journal per repository; never concatenate independent
histories. Establish owner-local work/designs and the coordinated cross-repository migration ADR.

## 2. Qualify existing Eventlog

File storage and atomic groups already exist. Qualify their current implementations rather than
rebuilding them. Groups have one tenant and durable identity with ordered stream appends; changed
content under that identity refuses. Earlier appends affect later expectations. Deterministic
locking does not reorder requests. Guards, required inline projections and bookkeeping commit or
roll back together. Transaction-scoped blob reads must not re-enter the outer store.

Preserve eventlog-file/1 bytes, sequence and previous-frame digest, process-safe writer locking,
referenced content durability, erasure/redaction and recovery boundaries. Only provably incomplete
uncommitted tails may recover; committed corruption and divergent histories refuse. Verify shared
file/SQLite/PostgreSQL behavior, including retries, blobs, snapshots and crash/reopen. Missing
backend execution or end-to-end crash evidence remains missing.

## 3. Asynchronous recorded ER execution

Add async recorded-store ports and an executor outside entity-core, then an Eventlog adapter.
Persist complete definitions, commands, results and nested emitted events, including accepted
zero-domain-event decisions. Observations append ordered history without advancing entity revision;
physical positions and entity revisions are distinct. Preserve global record-ID equality/conflict,
ordered atomic multi-entity batches, transaction-local expectations and verifiable replay. Record
content uses Eventlog blobs.

Retain explicit synchronous compatibility outside the kernel without nested block_on. Preserve
query/session contracts and readable legacy layouts; SQLite/PostgreSQL packages become compatible
facades with explicit imports. Missing legacy command history is represented as an import boundary,
not invented genesis evidence. New adapter envelope meaning requires its own format version.

## 4. Exactly one AEP migration story

One AEP story owns Eventlog file authority with Markdown projections. Provider prerequisites remain
in their owning repositories. Add aep plan store inventory, migration dry-run/apply, verification
and projection rebuild, with aep.project/2 for changed authority/default semantics. Retain legacy
configuration readers for Markdown, hybrid, SQLite and PostgreSQL migration.

Preserve identities, revisions, relations, bodies, evidence and available history. Refuse divergent
inputs. Stage destination state, verify equivalence, recheck source identity under the writer
fence, then switch configuration. Track .engineering/state/manifest.json, events.jsonl, blobs and
provider-required durable metadata. Ignore only disposable caches and runtime-local operational
files; durable authority remains tracked.

Markdown remains tracked and derived. Direct edits are drift. A committed mutation with a failed
projection returns the durable receipt; projection rebuild never repeats the mutation. Remove
hybrid selection from new runtime configuration.

## 5. Cut over the six real planning stores

Rehearse on preserved snapshots, then migrate in this order:

1. Eventlog.
2. Entity Runtime.
3. Service SDK.
4. ESS.
5. Connectors v2.
6. AEP, using the explicitly qualified new executable.

After each switch verify history, queries, mutation, restart and projection rebuild. Retain original
legacy recovery copies. Once new commands commit, recovery follows Eventlog authority; never
reactivate stale writers as rollback.

## 6. ESS and Service SDK convergence

Extract reusable ServiceIr lowering into ess-service-contract and add ess-entity-runtime for ER
definitions and binding plans. Crosswalk creation, updates, transitions, predicates, invariants,
branches, identity, relations, revisions, exact values, outcomes and zero/one/multiple events.
Implement missing ER semantics before admitting lowering; never weaken ESS to fit it.

Service SDK delegates decisions and replay to ER. Bindings retain authentication, authorization,
hosting, queries, content policy, transport and effects, preserving ordering and public entrypoints.
Opt-in service-runtime-ir/4 and service-realization-plan/4 select ER; existing readers and meanings
remain. Explicit legacy event-only conversion does not claim complete decision history.

## 7. Connectors and infrastructure acceptance

Migrate existing local SQLite metadata to ER/Eventlog SQLite while preserving identities,
credential references, revisions, fences and audit history. Linux Secret Service retains credential
custody. Preserve generated CLI behavior, existing provider reads, ordered credential-to-metadata
publication, executable verification, bounded supervision/startup, restart reuse, repair/revoke
races, protected input, schema-bound invocation and uncertain outcomes.

Advance exact ESS/AEP pins, generated sources and lockfiles, including the excluded CLI workspace.
Retain the digest-pinned AEP correction until equivalent upstream behavior is verified. Do not
expand provider or governance capabilities as a side effect.

Link selected service and local-process requirements to deterministic infrastructure projections
and independent observations. Rendering never applies infrastructure or infers actual state from
desired declarations. Preserve credential redaction and its mutation checks.

## Compiler minima and deferred work

Eventlog-backed runtimes move to Rust 1.91; independently supported pure libraries retain their
minima and separate checks. Generic protobuf/UI/Flutter is recorded in
task:deferred-protocol-ui-bindings, outside this initiative's completion. Local acceptance does not
authorize releases, publication or deployment.
