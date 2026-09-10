---
format: aep.planning-md/1
id: architecture-design:ess-evolution
kind: architecture-design
status: draft
title: ESS semantic and execution dependency architecture
relations:
- designs: initiative:ess-evolution
revision: 1
---
## Context
ESS already implements domain compilation, structural synthesis, CLI, composition, realization, schemas, documentation, infrastructure, conformance, diff and normalization. Preserve these while adding explicit service and UI models.

## Decision
See docs/design/ess-evolution/dependency-policy.md and adr-index.md. ESS core has no AEP, Service SDK, Eventlog or application dependency. ESS ER target uses pure entity-core definitions. ER kernel owns decisions and replay; executor coordinates asynchronous ports; Eventlog adapter maps complete recorded history. Eventlog owns storage. Service bindings own transport, auth, effects and hosting. Go remains native without Rust FFI or an ER service hop.

## Interfaces
Add ess-service-contract, ess-ui-contract, ess-entity-runtime, ess-protobuf and ess-flutter at the paths specified in the design. Preserve ess-ir/1, both CLI paths, realization /1 and /2 and existing runtime readers. New ER runtime and UI conformance artifacts are opt-in versions. Compilation and generation never start services or apply deployments.

## Evidence
The preservation mapping and exact baseline source vector precede implementation migrations. Runtime and compatibility acceptance remains outstanding.

