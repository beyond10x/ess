---
format: aep.planning-md/1
id: task:verify-observed-bindings
kind: task
status: implemented
title: Validate semantic implementation bindings against scoped Kubernetes observations
relations:
- serves: vision:O2
revision: 8
---
Implement the user-authorized first backend deployment connection as native ESS, reproducible without AI. The exact adopted types and checks are specified in docs/design/observed-component-bindings.md before code is added. Current ess-realization/1 already owns component-to-implementation artifacts (crates/specify/ess-realization/src/lib.rs:201,399); InfraIr separately owns scoped observed workload/container/image identities (crates/infra/infra-compiler/src/ir.rs). The CLI is the existing boundary consuming both.

Scope: native observed-binding document admission, offline and explicitly requested live verification, deterministic JSON and generated reference views, stable satisfied/violated/unknown outcomes with nonzero unknown/failure exit, tests over realistic mutations and failed collection, and adopter-facing documentation. Reuse realization and infrastructure readers. Do not relax runtime cardinality to guess profile semantics, merge IR domains, add a Python/shell semantic checker, fabricate build provenance, change cluster resources, or publish a release tag. Private adopter data stays outside the public repository.

This is one bounded implementation task, not a multi-story decomposition; no critic panel is needed for parallel story comparison. User authorization on 2026-09-07: implement the discussed backend mapping and rerunnable checks. Planning status is not a claim of runtime conformance. Run task check and task site-build before publication, record exact results, and consume a published source pin from the Internal specs repositories. Use the bot wrapper and managed worktree lifecycle.

Design established in docs/design/observed-component-bindings.md. The first slice validates workload-template references and explicitly leaves tag/source-to-image provenance unknown. It does not promise running Pod image-ID or build attestation verification. This boundary follows the actual native namespace collector coverage and the existing artifact model; no observed fact is silently promoted to a semantic or deployment guarantee. The accepted design includes a report/exit-code contract and exact test cases for positive, negative and unknown results.

Adopter validation exposed an existing ess-realization/1 refusal: EmptyDeclaration at entrypoints (at least one entrypoint is required) and PrimaryEntrypoint (exactly one entrypoint must be primary; found 0). The opt-in ess-realization/2 and ess-realization-ir/2 now admit implementation-only selections without actors or conformance claims. V1 admission and canonical bytes stay unchanged; V2 adds the format tag to the digest tuple. The design appendix records the native fix rather than inventing a runtime entrypoint. Existing deployment runtime cardinality is untouched. Tests cover both strict V1 and explicit V2 admission.

Full offline validation exposed an existing Firefox harness race: replay_fidelity_browser/b01_capture_event_field_need_not_equal_entity_identity_field failed at support/browser.rs with HTTP/1.1 404 Not Found during the WebSocket upgrade. The harness used TCP readiness before /session was registered. Its native readiness loop now retries only that startup 404 under the original deadline, preserves the response as evidence, and retains strict validation of the eventual 101 response. No scenario is skipped or verdict weakened. Environment quota failures are handled by an external TMPDIR and RUSTC_WRAPPER empty for this gate, without repository policy changes.

Final output review found that the new verifier's preflight check could race another publisher because the existing namespace scanner deliberately overwrites its requested output. Expose admitted sanitized collection bytes separately from the scanner's existing publication behavior. The new verifier publishes those bytes with exclusive create-new at the final write; Markdown shares that same writer. A regression admits an absent path, simulates a competing acquisition publishing during collection, then verifies the late output is refused and retained. This changes no observation bytes or legacy scanner overwrite contract.
