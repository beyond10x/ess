# Dependency policy

| Layer | Responsibility and admitted dependency direction |
| --- | --- |
| ESS semantic core | Parsing, typing, resolution, validation and semantic plans; no AEP, Service SDK, Eventlog or application dependencies |
| ESS ER target | ESS semantic plans and ER pure entity-core types; validated definitions, binding plans and generated wiring |
| ER kernel | Decisions, invariants and replay; no ESS, Eventlog, transport or database |
| ER executor | Asynchronous execution through recorded storage ports outside the kernel |
| ER Eventlog adapter | Maps complete entity history to stored records; implements ER ports using Eventlog |
| Eventlog | Durable appends, concurrency, group idempotency, blobs, snapshots and projection storage; file, SQLite and PostgreSQL providers |
| Rust service binding | Transport, authentication, authorization, effects and hosting; delegates entity execution to ER |
| Babelconnect Go | Native Go implementation checked against specifications; no Rust FFI or mandatory ER service hop |
| AEP and Connectors | Product behavior and projections; assemble selected ER/Eventlog implementations |

Host-owned Eventlog effect-delivery records do not constitute another entity state machine.
Compiler and generator implementations stay Rust. Native support belongs to its runtime or real
consumer; no empty Go/Dart support packages are created.

## Focused additions

| Package and path | Public responsibility |
| --- | --- |
| crates/specify/ess-service-contract | Selected component surfaces become validated language-neutral ServiceIr: operations, results, errors, events, views, state ownership and external obligations |
| crates/specify/ess-ui-contract | ess-ui/1 becomes UiIr: pages, elements, navigation, forms, presentation states, view references and command bindings |
| crates/realize/ess-entity-runtime | ESS service/entity semantics become ER definitions, binding plan, Rust assembly and unresolved-obligation diagnostics |
| crates/generate/ess-protobuf | Import/account for existing protobuf and project admitted semantics through explicit protocol bindings |
| crates/generate/ess-flutter | Typed widget/SDK catalog binds UiIr to generated Dart composition, routes, actions and views |

Reuse existing handles, source locations, semantic analysis, output ownership and capability reporting.
Keep EssIr, composition, CLI, realization, deployment, InfraSpec and observed InfraIr distinct.
No universal facet registry, arbitrary string identity bags or forced ess-ir/2 migration.
Domain and UI references remain typed. Existing SDK package identities remain compatible.

Keep existing command spellings; add beneath specify, generate and verify. Preserve both Clap
synthesis and ess-cli/1, both realization readers and all conformance defaults. ER runtime and UI
scenario formats are opt-in additions; service-runtime-ir/3 bytes keep their existing meaning.
Generation writes files. Execution, deployment and publication remain explicit operations.

