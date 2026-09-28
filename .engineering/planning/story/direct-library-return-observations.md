---
format: aep.planning-md/3
id: story:direct-library-return-observations
kind: story
status: implemented
title: Observe typed pure-library returns in conformance
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-xtask
- confidence: cited
  path: crates/specify/ess-compiler
- confidence: cited
  path: crates/specify/ess-domain
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: cited
  path: docs/design
- confidence: cited
  path: docs/evidence/direct-library-returns
- confidence: cited
  path: models/toolchain
- confidence: cited
  path: schemas/generated/ess.schema.json
- confidence: cited
  path: website/docs/reference
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-09-28T07:40:54Z", actor: "human:timo", revision: 11}
- {from: "proposed", to: "active", at: "2026-09-28T07:40:54Z", actor: "human:timo", revision: 12}
- {from: "active", to: "implemented", at: "2026-09-28T08:19:01Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"test_result":1,"review_outcome":2}}}
---
## Acceptance

ER's actual pure-library results are observed by admitted Rust conformance suites, with literal response checks, complete typed response shape and independently bounded exact values, without inventing events or persistence, and all legacy source/suite formats keep their existing meanings.

## Authorized continuation

The operator requested merging the ER specification, using the current planning store, cleanup and release. Prior independently reviewed implementation is ESS pull request 182 at 6e0b577df9fa4142dffc4d616d96fe2ab5881e24, with retained red/green tests and reviews under docs/evidence/direct-library-returns. ESS 0.38.0 has since allocated source ess/16 and suites26/27, so that unmerged branch cannot reuse those identities. This work ports its actual behavior onto current main and allocates fresh ess/17, suite28/29 formats; authored scenario/4 remains available. Prior evidence retains its original version and exact bytes and is historical, not final admission evidence.

## Scope

Source/IR typed direct returns, Rust runner observations, typed response/resource validation, source/suite admission, explicit refusal in unsupported generators, associated schema and public format inventories. No AEP dependency. Keep native Binary64 admission explicit; ER wire numeric checks remain lossless serialized values. Preserve current main's presence, leaf, paging, retry and time semantics. The AEP reader compatibility story is admit-er-direct-return-evidence in that repository.

## Evidence

Prior source and independent reviews are recoverable from pull request 182. Final merged conformance, compatibility, schema/format inventory, toolchain model, MSRV, static checks and CI must be rerun against current main. AEP writes and lifecycle records are coordinator-owned.
