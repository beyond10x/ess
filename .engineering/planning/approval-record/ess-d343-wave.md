---
format: aep.planning-md/3
id: approval-record:ess-d343-wave
kind: approval-record
status: approved
title: Wave D343 runs rows 3, 2, 5 and 20 of the decided triage as one wave, one pull request and release 0.80.0
summary: Orchestrator approval D343 of 2026-11-01 under the operator's grant; row 4 already held
tags:
- non-interactive
relations:
- decides: story:a-bound-command-input-decodes-the-published-request-body
- decides: story:entity-runtime-lowering-enforces-input-type-invariants
- decides: story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent
- decides: story:short-names-combine-with-all-commands-and-all-views
- decides: story:an-authored-text-literal-outside-its-alphabet-is-refused
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-01T06:49:17Z", actor: "human:timo", revision: 2}
---
## Decision

D343, 2026-11-01: the orchestrator approved the next ESS wave after 0.79.0 under the operator's
grant (class C: a wave, bot commits, pull request and merge, release), with the operator's delivery
rule of 2026-10-28 (several stories on one integration branch, targeted tests per unit, one local
final gate at the end, one pull request auto-merged to `main` on a green pipeline, one release).
The release that follows the merge is `0.80.0`. Stories come from the adopted rows of the decided
2026-10-31 findings triage (`approval-record:ess-d338-findings-triage`); row 3 (beyond10x/ess#656,
an adopter's first priority) goes first.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs.

## Units

At most four, on non-colliding surfaces. Base `main` 45a1beba7 (0.79.0).

- U1 (row 3, beyond10x/ess#656, first): `story:a-bound-command-input-decodes-the-published-request-body`,
  accept, redesigned (its `## Fit review` and `## Decisions`): `ess generate types --http <binding>`
  makes each body root the body the bound OpenAPI document publishes; unbound output keeps its
  bytes. Surface: `crates/generate/ess-gen/src/model_types.rs`, `crates/edge/ess-cli/src/model_types.rs` (cited).
- U2 (row 2): `story:entity-runtime-lowering-enforces-input-type-invariants`: an unstored input's
  type invariant lowers to an entity-core precondition or is refused by name; none is dropped.
  Surface: `crates/generate/ess-entity-runtime/src/{lib.rs,subset.rs}` and its generated reference page (cited).
- U3 (row 5): `story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent`, accepted
  as written. Surface: `crates/verify/ess-conformance/src/synthesize.rs` arrangement functions
  (cited), `synthesize/related_guard.rs` `on_identity` and `synthesize/repeated_identity.rs` (inferred).
- U4 (row 20, beyond10x/ess#409): `story:short-names-combine-with-all-commands-and-all-views`.
  Surface: `crates/generate/schema-contract/src/realize.rs` `with_short_names` (cited).

Every unit gates package-scoped; ess-conformance cargo runs only on a slot the orchestrator grants.

## Selection

Picked from the adopted rows of the 2026-10-31 triage, row 3 first. One `aep:story-scoper` per
candidate; scope written to each story as a `## Scope` section and typed entries; `aep plan
artifact waves --kind story --status draft` (aep 0.78.0) reports no collision between the four
picked stories (U1, U2 and U4 in its wave 1, U3 in its wave 7, ordered there by collisions with
other drafts, none of them picked).

- Row 4 (`story:an-authored-text-literal-outside-its-alphabet-is-refused`, narrowed to the
  entity-invariant half) is already held: f9fa4d12d (first tagged 0.63.0) refuses the downstream
  repro as `conflicting_declaration`, ESS-COMMAND-004 (run with ess 0.77.0 on
  `.engineering/repro/invariant-broken-by-literal/entity-invariant.yaml`); the alphabet half is held
  by `layered_literal` (code read). Closed as implemented in this wave's opening store commit; no unit.
- Row 8 (`story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field`) collides
  with U3 on `crates/verify/ess-conformance/src/synthesize.rs` (inferred, both sides) and its
  no-parent-guard variant did not reproduce; scoped and left for the next wave.
- Rows 15 and 25 land on `crates/generate/ess-gen/src/model_types.rs`, U1's file; next wave.
- Rows 6, 7, 11, 14 (synthesis) land on `related_guard.rs`, `row_set.rs` or `subject_fact.rs`
  beside U3; one ess-conformance unit per wave while the host's disk is held.
- Row 9 stays blocked on `decision-blocker:decimal-canonical-serialization-crosses-repositories`.

## Review scope

Adversary passes review each unit's diff against its acceptance only (operator rule of
2026-10-31). Gaps inside the acceptance are fixed in the wave; anything outside is one
out-of-scope row in the pass's review-result, decided by the coordinator at wave close (adopt as a
story, decline with reason, duplicate).

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-d343`, the store commits on that branch, the release-prepare commit (version bump,
lock files, CHANGELOG, change record) on that branch before the final gate, one pull request to
`main` from a `queue/` branch through the App and its merge on a green pipeline, then the release
tag and publication through the repository's own process. Not the next wave.

Decider: operator
