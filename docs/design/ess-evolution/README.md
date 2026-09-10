# ESS evolution

Owner: initiative:ess-evolution, created through AEP from the operator's 2026-09-10 implementation plan.

ESS keeps its repository, package identities, implemented behavior, formats, examples, fixtures,
generated snapshots and fuzz corpus. This is a migration, not a replacement compiler. Structural
Rust synthesis remains supported and is not evidence of a booted durable service.

A domain becomes a service only when a selected realization supplies execution, persistence,
transport, authority and external effects, and the application is built and started.

- [Decisions](adr-index.md)
- [Dependency policy](dependency-policy.md)
- [Feature preservation](feature-preservation.md)
- [Migration sequence](migration.md)
- [Acceptance](acceptance.md)

Private application specifications and source extractions remain in their owning repositories.
ESS examples are synthetic. Local cross-repository evidence is kept outside public documentation.
Nothing in this design declares a runtime migration complete.

