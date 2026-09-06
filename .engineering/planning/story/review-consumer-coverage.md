---
format: aep.planning-md/1
id: story:review-consumer-coverage
kind: story
status: draft
title: Require explicit consumer coverage for model extensions
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
- depends_on: story:review-semantic-diff-coverage
scope:
- confidence: inferred
  path: Cargo.lock
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: inferred
  path: docs/design/review-consumer-coverage.md
revision: 8
---
## Finding and source

F16 (P1) from `docs/reviews/2026-09-05-architecture-review.md:522`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `Taskfile.yml:138`, `crates/edge/ess-xtask/src/main.rs:54`, `docs/reviews/2026-09-05-architecture-review.md:532`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

The extension gate fails when a new semantic construct or field has no tested support or explicit refusal for an inventoried consumer.

## Implementation boundary

Maintain a typed consumer matrix for validation, IR, references, diff/impact, projections, synthesis and conformance, with links to executable cases or explicit unsupported/refusal evidence. Add a Rust xtask gate that checks the matrix against the actual authoritative model surface; avoid a parallel hand-maintained list silently omitting fields. A coverage record is not a substitute for running its behavioral test.

## Validation

Mutation proof: add a representative semantic field/consumer obligation without coverage and observe failure, then supply supported/refused behavior evidence and pass. Include concrete F01 rows. The existing fuzz story owns general document fuzzing rather than this matrix.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

Do not rewrite passing gates or claim every target supports every construct; unknown is a valid visible matrix entry with owned follow-up work.

## Scope

Derived 2026-09-06 by aep-drive:story-scoper from this revision 6 story, its implemented semantic-diff dependency and ESS coordinator d2057ffb944455d0ef3a90ab7c5043ae70027289, version 0.20.0 — cited.

- **Primary surface:** crates/edge/ess-xtask; the existing Command, run, workspace_root and schema seams own the repository command and its authoritative RawSpecFile schema derivation. This reservation includes the package manifest, a new inventory/matrix/checking module and focused extraction, accounting, executor and mutation tests — cited.
- **Gate wiring:** Taskfile.yml; projection-check and the ordered check task are current integration seams. Preserve their existing executions and add a distinct consumer-coverage lane after its executable contract is bound — cited.
- **Dependency reservation:** Cargo.lock, for a direct package-local AST-parser dependency and any required existing serde dependency entry; no root workspace manifest edit is established — inferred.
- **Binding design:** docs/design/review-consumer-coverage.md, still absent; bind reachable model roots, stable identities, macro/cfg/import resolution, exact consumer discovery boundaries, closed baseline unknowns, executable case/result identity, mutation proof and failure semantics before implementation — inferred.
- **Inventory approach:** derive reachable authored/resolved declaration fields and variants, cross-check the actual authored JsonSchema, and account for every discovered member/consumer pair. Do not use a Resolved-name prefix, an inhabited IR example, a generated schema file, or a future-field wildcard as authority — inferred.
- **Consumer correction:** include model-selected structural realization and normalization alongside existing validation, compilation, references/graph, diff/impact, individual projections, synthesis, conformance, composition, realization and deployment runtime consumers. Current composition runtime evidence establishes byte-buffer forwarding, not typed payload validation — cited.
- **Behavioral boundary:** supported and explicitly refused cells require exact executable cases and successful same-run behavior assertions; baseline unknowns require explicit owner/follow-up and never qualify as support. A panic or missing native toolchain must not satisfy an explicit refusal — inferred.
- **Minimal implementation option:** retain all model/consumer production and owner-package tests as read-only evidence inputs. Reuse only the exact assertions existing cases establish. If the chosen required cells lack those assertions, refresh scope to their exact test owners before dispatch; this package reservation does not authorize those edits — inferred.
- **Current-writer dependency:** fresh source and linked-case identity must be checked after coverage integration and before a new selection. The inspected source admits suite versions 1–4 and count-stage report/run version 2; it supplies no evidence of the uninspected suite-5 writer's behavior — cited.
- **Compatibility:** repository tooling and internal design only; preserve production formats, defaults and committed generated bytes. No AEP reader/writer migration, public documentation change or production repair is included in this candidate — inferred.
- **Would collide with:** xtask source, tests or manifest work; Taskfile gate ownership; lockfile dependency resolution; and the reserved binding. Owner-package test or model metadata choices would add concrete collisions and require a fresh scope first — inferred.
- **Confidence:** medium, because the command, gate, models and behavioral test owners are established, while accepted inventory granularity, consumer detection, baseline unknown eligibility and result acquisition are not yet bound — inferred.

## Candidate binding review

The coordinator accepted the four refreshed write reservations on 2026-09-06 and applied only
this Scope replacement; the story remains draft. The independent report is
target/review-boundaries-11/next-scope/consumer-coverage-report.md, SHA256
2194fea88d194d9c54b22e38f5990d6e2b00b64ed633ac039ebf833ca29abd61. Root independently
verified all 53 inputs and 49 opening Git blobs. Two document-only reviews are recorded as
review-result:consumer-coverage-binding-pass1 and review-result:consumer-coverage-binding-pass2;
verification-report:consumer-coverage-binding-wording records the narrow correction to actual
Draft7 definitions/#/definitions/ vocabulary. The current candidate is
target/review-boundaries-11/next-scope/consumer-binding-draft-v3.md, SHA256
63d6075781dc7f348fff838962c4392df3c8e9335eaf7b5d06f8a690f630812c. Its two-stage baseline
selection, exact wire/model inventory and actual attributed case execution remain requirements,
not measured gate results. Before implementation selection, refresh integrated coverage source
and case/profile identities, accept the binding and select the concrete first-stage work order.
No new story selection or baseline-unknown eligibility approval is implied by this scope update.
