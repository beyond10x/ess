---
format: aep.planning-md/1
id: initiative:ess-evolution
kind: initiative
status: active
title: Evolve ESS without losing capabilities
relations:
- informed_by: epic:review-boundary-remediation
- informed_by: story:cli-presentation-binding
revision: 7
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

The preserved design and complete 96-consumer baseline mapping have been reviewed for integration against ESS main 988f219e90c9a7b105daec2345f77d6035075e92. That release changed no consumer-accounting authority files. The mapping retains exact profile entrypoints and claim boundaries and does not grant semantic support; automatic preservation-drift enforcement remains a follow-up.

Eventlog prerequisites are implemented and published: main 7d6bedc738b6037c7a1ba5218bc2a0d3271878c3 contains atomic groups and the file provider, with 129 local tests passed, no failures/skips, and formatter/strict Clippy and file-provider MSRV evidence. Its story:atomic-append-groups and story:file-eventlog records contain publication receipts. ER recorded execution and adapter work remains outstanding, followed by the one AEP migration story and the later service/UI/application work.

Integration retains original candidate cd80f202c4cd9ed1b25027e008ffbb2bc096c8cd as a Git parent. The independently advanced main planning store is the selected authority; the ten candidate artifacts were recreated with one AEP writer and their original relations, rather than concatenating journals. Original observations remain in that parent, while these writes record integration now. This is an interactive operator-authorized review and integration, not completed evolution.

## Integration test portability

At the operator’s request, integration verification will restore TMPDIR=/tmp for the full local gate. Scope is limited to tests/support/input_discovery_cases.rs::socket: bind through a short temporary directory symlink to the destination parent instead of renaming a socket across filesystems. This retains the selected-socket refusal assertions and avoids moving all ownership fixtures onto disk. Keep all reviewed source and planning changes. Verify the previously failing input-discovery test, all 39 output-ownership tests, and the complete task check gate with native-only linker flags. The interrupted disk-based run is not passing evidence.

## Integration verification

The complete local task check passed on 2026-09-10 against staged tree 66bd733886b558be00066658cb41fc83a734be25, including all 152 consumer cases and the final fuzz/release/action checks. Log: local-evidence:ess-evolution-20260910/ess-integration-gate-native-config.log. task site-build also passed; log: local-evidence:ess-evolution-20260910/ess-integration-site.log. Source, compiler and public-site inputs relevant to that site build are retained.

TMPDIR=/tmp is restored. The socket fixture binds through a short temporary directory symlink directly into the destination filesystem. Its targeted selected-socket test passed, and all 39 output-ownership tests passed in 47.31 seconds. The full workspace run passed those tests too. Native linker flags come from the existing machine Cargo configuration; all four compiler-wrapper variables are explicitly empty, build jobs are two, debug information and incremental compilation are disabled. Earlier interrupted or failed environment runs remain in the cache logs and are not passing evidence.

The integration retains the reviewed 96-consumer mapping and original candidate parent, with the current main planning journal preserved as authority. The next bounded continuation is story:ess-evolution-preservation-gate; broad evolution remains active.

## Integration publication

Reviewed integration and socket-fixture portability fix are published on ESS main 24d2fe714958c8cde63ea78122c31e28bcc682bc, with original candidate cd80f202c4cd9ed1b25027e008ffbb2bc096c8cd retained as a parent. The preservation-gate story has completed local verification and is awaiting publication.
