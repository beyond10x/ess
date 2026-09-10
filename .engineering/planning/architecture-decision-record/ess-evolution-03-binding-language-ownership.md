---
format: aep.planning-md/1
id: architecture-decision-record:ess-evolution-03-binding-language-ownership
kind: architecture-decision-record
status: proposed
title: 03 — Bindings and language ownership
relations:
- decides: initiative:ess-evolution
revision: 1
---
## Context
Shared semantics must not couple ESS to adopting hosts.

## Decision
Compiler/generator code stays Rust. Native support belongs to its runtime or actual adopter; use precise implementation/protocol binding terminology.

## Rejected alternatives
ESS depending on Service SDK; mandatory Rust FFI for Go; empty language SDK scaffolds.

## Compatibility and migration
SDK names and public entrypoints remain compatible; facades point toward ESS. Dependency tests must prove the direction.

## Acceptance evidence
Operator decision: supplied implementation plan, 2026-09-10. See docs/design/ess-evolution/feature-preservation.md and acceptance.md. Implementation and runtime acceptance are not yet established.
