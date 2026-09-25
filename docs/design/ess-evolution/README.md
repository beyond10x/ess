# ESS evolution

Owner: initiative:ess-evolution. Current authority is the operator-approved plan
ess-evolution-20260915 revision 1, recorded by approval-record:ess-evolution-20260915 and
design:ess-evolution-orchestration. It supersedes the broader 2026-09-10 completion scope.

ESS keeps its repository, package identities, implemented behavior, formats, examples, fixtures,
generated snapshots and fuzz corpus. Structural Rust synthesis remains supported; durable service
acceptance additionally builds and starts the selected realization with execution, persistence,
transport, authority and external effects.

Current work covers Eventlog qualification, asynchronous recorded ER execution, exactly one AEP
migration story, actual cutover of all six participating planning stores, ESS/Service SDK
convergence, Connectors v2 adoption and infrastructure acceptance. Synthetic billing/gatepass
fixtures remain required service regressions.

Generic protobuf, UI and Flutter capabilities are deferred in
[task:deferred-protocol-ui-bindings](../../../.engineering/planning/task/deferred-protocol-ui-bindings.md).
They do not block this initiative. The excluded adopter and fake-backend work have no current
acceptance requirement; archived artifacts and historical journals remain intact.

- [Decisions](adr-index.md)
- [Dependency policy](dependency-policy.md)
- [Feature preservation](feature-preservation.md)
- [Migration sequence](migration.md)
- [Acceptance](acceptance.md)

Private source material and exact local evidence stay with their owners. Local acceptance is the
completion boundary; publication, releases and deployment are separate. This design records
requirements, not completed migrations.
