# Migration sequence

## 0. Preserve the source and acceptance baseline

Record exact commits, dirty/index/untracked changes, lockfiles, generator identities and toolchains
for ESS, ER, Eventlog, AEP, Service SDK, Connectors v2, Babelconnect and downstream server/controller
sources. Preserve user edits. Evidence is tied to the actual source/dependency/configuration vector;
unrelated repository movement does not invalidate another slice's completed evidence.

## 1. Shared persistence and execution

Eventlog gains a separate atomic-group capability. One tenant, ordered stream appends, one group
identity; changed content under that identity is refused. Earlier appends in the group affect later
expectations. Lock streams deterministically but apply in request order. Guards, required inline
projections and bookkeeping commit or roll back together. Never loop over independent commits.
Provide transaction-scoped blob reads, without re-entering the outer locked store.

File storage uses versioned JSONL transaction frames with sequence and previous-frame digest.
Process-safe locking serializes writers; referenced content and journal are durable before success.
Use the existing blob port; indexes/snapshots are disposable. Recover only provably incomplete,
uncommitted trailing writes; committed corruption and divergent Git histories are refusals. No
concatenating merge driver. Preserve active-store erasure/redaction; Git history is a retained archive.

ER adds asynchronous recorded-store ports and executor outside entity-core, then an Eventlog adapter.
Retain explicit synchronous compatibility adapters, without nested block_on in the kernel. Store
complete pinned definition, command, result and emitted events, including accepted zero-event
history. Observations do not change state. Entity revision is distinct from physical position.
Preserve record-ID equality/conflict, ordered atomic batches, query/transaction capabilities and
readable legacy provider layouts. SQLite/PostgreSQL packages become compatible facades.

## 2. AEP: one bounded story

Exactly one AEP story owns Eventlog file authority with Markdown projections. Provider prerequisites
stay outside it. Include configuration/defaults, dry-run inventories of Markdown/hybrid/database
stores, identity/revision/relation/body/evidence/history migration, divergent-input refusal, staged
verification before switching configuration, projection generation/drift/rebuild and removal of
hybrid runtime selection. Preserve available history; snapshot-only data enters at a legacy-import
boundary. Never invent past commands.

Track .engineering/state/manifest.json, events.jsonl and blobs; ignore only disposable .cache.
Markdown remains tracked but derived. Direct edits are drift, never silent import or overwrite.
Post-commit projection failure carries the durable receipt. Projection retry never repeats mutation.
Use one planning-store writer per repository. After acceptance, AEP leaves the critical path.

## 3. Service semantics and ER convergence

Extract reusable Service SDK lowering into ess-service-contract. Keep authorization/hosting/content
policy and external effects in bindings. Crosswalk create/update/transitions, predicates/invariants/
branches, identities/relations/revisions, exact values, zero/one/multiple events and external outcomes.
Implement missing ER semantics in its kernel; never weaken ESS for lowering. Add ess-entity-runtime;
service-engine delegates decisions and replay. Preserve auth/admission ordering/query/effect/transport
behavior and public entrypoints. New artifacts opt into ER; old readers and explicit conversion
remain. Event-only legacy service history is not a full decision history.

## 4. Protocol, UI and independent tests

Imports produce candidates, source references, accounting and unresolved questions. Ground private
application contracts in client and downstream source. Protobuf stays authoritative until descriptor,
wire and consumer compatibility pass; then ESS and explicit bindings generate existing proto paths
and pinned Buf emits language code. Preserve names, numbers, reserved fields, presence, oneofs,
enums, JSON mappings, streaming and HTTP annotations.

UiIr covers every implemented screen/journey and loading/empty/failure/permission/reconnect state.
Generate one complete journey at a time around existing theme roles, translations, widgets, SDK and
media interfaces. Own only generated composition/routes/actions/views. Keep local drafts/navigation
separate from server-owned agent/call business state. Independent cache vectors verify patch handling.
Extend conformance's Go runner and fault injection with UI observations and Flutter/Playwright emitters.
Expected behavior stays upstream of lowering; targets report observations, runners judge them.

## 5. Applications and infrastructure

Connectors retains adapter-owned kinds and GitLab/Kubernetes/SQL reads. Bind generated Handler,
protected Sources and dynamic validation to real setup, adapter supervision, connection management
and schema-bound invocation. Linux Secret Service holds credentials; ER/Eventlog holds references
and metadata. Preserve ordered keyring-to-metadata publication, executable verification, bounded
startup, restart policy, revocation fencing, protected-input and uncertain-outcome rules. Preserve
describe/invoke/serve. Broader modeled governance/mutations remain outside selected acceptance.
Advance exact ESS pins and lockfiles only after compatibility, including excluded CLI workspace.

Babelconnect keeps Go AgentView, Subscribe/Send, acknowledgement versus streamed-error distinction,
Connect/gRPC-web and CGO_ENABLED=0. Preserve backend-call-ID correlation/field ownership, server Faye
and WebSocket behavior, authoritative presence lookup, outbound-only auto-answer, inbound accept/
reject, native/web auth, embed origins and media permissions. No Rust FFI. Keep SIP/media in native
bindings. Generate composition without redesigning the application.

Link selected deployment requirements to existing infrastructure expectations/projections. Cover
Babelconnect server/web and Helm inputs, and Connectors local processes. Render manifests and use
captured/synthetic observations locally. Never infer observed state from desired declarations.

