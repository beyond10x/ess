---
format: aep.planning-md/1
id: initiative:ess-evolution
kind: initiative
status: draft
title: Evolve ESS without losing capabilities
relations:
- informed_by: epic:review-boundary-remediation
- informed_by: story:cli-presentation-binding
revision: 1
---
## Outcome
Implement the operator's ESS evolution plan while retaining ESS names, packages, supported formats and implemented behavior. Deliver repository-local Eventlog file authority for AEP, recorded ER execution, explicit ESS service/protocol/UI bindings, and verified adoption in Connectors v2 and Babelconnect.

## Authority
The operator supplied and authorized the implementation plan on 2026-09-10. This is an interactive implementation session; no approval bypass is claimed. Umbrella design: docs/design/ess-evolution/README.md. ADR index: docs/design/ess-evolution/adr-index.md. Migrations and acceptance remain separate claims.

## Existing work
Build on epic:review-boundary-remediation and story:cli-presentation-binding. Connectors owns epic:local-cli-contracts, specification:local-cli-wave-20260909 and the reviewed local CLI binding stories. Do not create a second product backlog.

## Sequence
0. Immutable source vector, feature preservation mapping and governed design.
1. Eventlog atomic groups and file provider; ER asynchronous recorded executor and persistence adapter.
2. Exactly one AEP migration story: Eventlog file authority with Markdown projections.
3. ESS service semantics extraction, ER lowering and Service SDK convergence.
4. Protobuf candidates/accounting, typed UI, Flutter composition and independent conformance.
5. Complete both application adoptions and desired-versus-observed infrastructure links.

## Completion
Local acceptance against exact source, dependency and configuration evidence. Linux, web and Android execution and real disposable SIP/WebRTC media evidence are required for Babelconnect. Missing evidence is not success. Releases, publishing, Atlas delivery and deployment are separate work.

## Progress
Baseline discovery is in progress. No runtime migration or application acceptance is claimed.

