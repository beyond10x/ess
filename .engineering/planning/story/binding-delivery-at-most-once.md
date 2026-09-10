---
format: aep.planning-md/1
id: story:binding-delivery-at-most-once
kind: story
status: proposed
title: A binding can say it delivers at most once
summary: Add a single-attempt delivery word for bindings whose invocation is one try with no redelivery, with conformance, docs and schema consequences
owner: timo
tags:
- consumer-integration
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
revision: 2
---
## Context
`ess/1` bindings require `delivery:`, and `ess-domain::binding::Delivery` has one variant, `AtLeastOnce` (crates/specify/ess-domain/src/binding.rs:194-200, "the only guarantee this build implements"). The review design left the door open: docs/design/ess-review-v0.1.md:95 writes `delivery: at_least_once # the only value v0.1 accepts`. The first consumer that needed the other word arrived on 2026-09-10: `specs/services/pusher` (the consumer repository) models two crossings that are single HTTP attempts — the pusher's authorization request to the backend (`request` with no retry, auth.js:177) and the backend's publish to the pusher (response never read, HttpBrowserPusher.groovy:153). ESS 0.20.0 refused `delivery: at_most_once` with `unknown variant, expected at_least_once`; the model now says `at_least_once` and carries a header comment stating that the word is false. A specification forced to state a stronger guarantee than the system gives is the failure F3 exists to prevent.

## Acceptance
An `ess/1` binding may declare `delivery: at_most_once`; `ess specify validate` and `compile` accept it and every projection (`docs`, `schema`, `openapi`, `asyncapi`, `docs-ir`, synthesized targets) renders the word; the conformance obligation for such a binding does not require `redeliver_event` and a synthesized suite contains no redelivery scenario for it, while an `at_least_once` binding keeps its current obligation and scenarios; the pusher model above validates with the word in place of the comment; `task check` exits 0.

## Scope
Cited: crates/specify/ess-domain/src/binding.rs (`Delivery`), crates/verify/ess-conformance/src/{web.rs,web_replay.rs,target.rs,reference.rs} (the places that spell or act on `at_least_once`), crates/generate/ess-gen and the schema/OpenAPI/AsyncAPI projectors where the delivery word is rendered, the ess-domain JSON schema, docs/design/ess-review-v0.1.md F3 (the design page that names the vocabulary), CHANGELOG.md `[Unreleased]`.
Inferred addition: one binding design note stating what `at_most_once` promises (one attempt, no redelivery, loss is `on_failure`'s business) and what it does not (it is not "exactly once"); fixtures that pin the rendered word in every projection.

## Boundaries
Do not change what `at_least_once` means or renders. Do not add a third word or a retry count. Do not invent an `exactly_once`. A new variant in a persisted `ess/1` field is accepted by old readers as a refusal, not as a silent reinterpretation: decide and document the format consequence explicitly per AGENTS.md ("A new format version is required when meaning … changes"); if the decision is that `ess/1` admits the new word without a version bump, say why in the design note. Keep ESS independent of AEP.

## Verification
Unit tests for the variant in `ess-domain` and its schema; a projection fixture per generator showing the word; a conformance test proving no `RedeliverEvent` step is synthesized for an `at_most_once` binding and one still is for `at_least_once`; a validation run of the the consumer repository pusher model at `specs/services/pusher` `20c198582aa509b85557b79de50a16cc2f6fbc9c` with `delivery: at_most_once` substituted on both crossings; then `task check`.
