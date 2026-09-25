# Dependency policy

Authority: approved ESS evolution plan ess-evolution-20260915 revision 1.

| Layer | Responsibility and admitted dependency direction |
| --- | --- |
| ESS semantic core | Parsing, typing, resolution, validation and semantic plans; no AEP, Service SDK, Eventlog or application dependencies |
| ESS ER target | ESS service semantics and pure entity-core types become validated ER definitions and binding plans |
| ER kernel | Deterministic decisions, invariants and replay; no ESS, Eventlog, transport, database or async runtime |
| ER executor | Async execution through mandatory recorded-storage ports outside the kernel |
| ER Eventlog adapter | Complete ordered entity history over Eventlog, with record content in blobs and explicit legacy boundaries |
| Eventlog | Durable appends, concurrency, atomic groups, blobs, snapshots and projections; file, SQLite and PostgreSQL providers |
| Service binding | Authentication, authorization, transport, hosting, queries, content and effects; decisions/replay delegate to ER |
| AEP and Connectors | Product behavior and projections; explicitly assemble ER/Eventlog authority |

Host-owned Eventlog effect-delivery records are not another entity state machine. Compiler and
generator code stays Rust. Native support belongs to its runtime or actual consumer; no empty
language-support scaffolds are introduced.

## Focused additions

| Package and proposed area | Public responsibility |
| --- | --- |
| crates/specify/ess-service-contract | Selected component surfaces become validated language-neutral ServiceIr: operations, results, errors, events, views, ownership and external obligations |
| crates/generate/ess-entity-runtime | Service/entity semantics become ER definitions and binding plans with explicit unresolved obligations |

Record final typed interfaces and source scopes in owner-local designs before implementation.
Preserve existing package identities. Reuse handles, source locations, semantic analysis, output
ownership and capability reporting. Keep EssIr, composition, CLI, realization, deployment, InfraSpec
and observed InfraIr distinct. No universal facet registry, arbitrary metadata bags or forced
ess-ir/2 migration.

Preserve current command spellings, both Clap synthesis and ess-cli/1, realization readers,
conformance defaults and SDK entrypoints. Opt-in service-runtime-ir/4 and service-realization-plan/4
admit ER while prior format meanings stay fixed. Changed AEP authority/default semantics use
aep.project/2 with legacy migration readers.

Runtime packages depending on Eventlog use Rust 1.91; independent pure libraries retain supported
minima and explicitly tested dependency closures. Generic protobuf/UI/Flutter additions are deferred
to task:deferred-protocol-ui-bindings. Generation writes artifacts; execution, deployment and
publication remain explicit operations.
