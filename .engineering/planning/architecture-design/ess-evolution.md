---
format: aep.planning-md/2
id: architecture-design:ess-evolution
kind: architecture-design
status: draft
title: ESS semantic and execution dependency architecture
relations:
- designs: initiative:ess-evolution
revision: 2
---
## Context

ESS already provides domain compilation, structural synthesis, CLI, composition, realization, schemas, documentation, infrastructure, conformance, diff and normalization. Preserve these while introducing the service execution capabilities admitted by approved ESS evolution revision 1.

## Decision

See docs/design/ess-evolution/dependency-policy.md and adr-index.md. ESS semantic core has no AEP, Service SDK, Eventlog or application dependency. The ESS ER target uses pure entity-core definitions. ER kernel owns deterministic decisions and replay; an executor coordinates asynchronous recorded ports outside the kernel; an Eventlog adapter stores complete history. Service bindings own transport, authentication, authorization, content, queries, hosting and external effects.

## Interfaces

Add ess-service-contract and ess-entity-runtime. Preserve existing public ESS formats, both CLI paths, realization readers and SDK entrypoints. New opt-in service-runtime-ir/4 and service-realization-plan/4 select ER. AEP authority/default changes use aep.project/2 with explicit legacy readers. Eventlog-backed runtime packages move to Rust 1.91; independent pure packages retain supported minima. Generic protobuf/UI/Flutter additions are deferred to task:deferred-protocol-ui-bindings.

## Evidence and authority

approval-record:ess-evolution-20260915 records operator approval; design:ess-evolution-orchestration binds the exact full plan. Owner-local designs and the coordinated migration ADR precede new contract implementation. Preserve the feature mapping and actual source/configuration vector. Runtime and compatibility acceptance remains outstanding; compilation and generation never apply deployments.
