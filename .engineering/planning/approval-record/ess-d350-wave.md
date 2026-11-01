---
format: aep.planning-md/3
id: approval-record:ess-d350-wave
kind: approval-record
status: approved
title: Wave D350 runs rows 15+25, 6+7, 14 and 17 of the decided triage as one wave, one pull request and release 0.81.0
summary: Orchestrator approval D350 of 2026-11-01 under the operator's grant
tags:
- non-interactive
relations:
- decides: story:a-null-when-absent-binary64-field-realizes
- decides: story:a-response-root-requires-a-declared-response
- decides: story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off
- decides: story:related-guard-wrong-state-beside-external
- decides: story:wrong-state-witness-unknown-and-own-stored-guards
- decides: story:the-published-schema-admits-the-name-aliases-the-parser-reads
revision: 2
transitions:
- {from: "draft", to: "approved", at: "2026-11-01T13:05:38Z", actor: "human:timo", revision: 2}
---
## Decision

D350, 2026-11-01: the orchestrator approved the next ESS wave after 0.80.0 under the operator's
grant (class C: a wave, bot commits, pull request and merge, release), with the operator's delivery
rule of 2026-10-28: "work in waves or batches. So we fix multiple stories against one integration
branch, bigger batches, multiple stories, and then we do one PR. We do not run expensive full gates
after every small change. We run targeted tests only for what we're currently implementing, and only
in the end, when everything ended up in the integration branch, we run a local full gate and then we
create the PR and auto merge it to the default main branch." The release that follows the merge is
`0.81.0`. Stories come from the adopted rows of the decided 2026-10-31 findings triage
(`approval-record:ess-d338-findings-triage`) that wave D343 did not take.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs. Skill: aep `implementing` 0.25.1, wave mode.

## Units

Four, on surfaces the store's `waves` verb finds disjoint. Base `main` dc60f4532 (0.80.0).
Integration branch `integration/ess-d350`.

- U1 (rows 15 and 25, medium and low; one unit because both land on
  `crates/generate/ess-gen/src/model_types.rs`, in different functions):
  `story:a-null-when-absent-binary64-field-realizes` (`fields_numeric_locations` uses `required()`,
  so a `null_when_absent` Binary64 location misses the `anyOf` member and realization refuses) and
  `story:a-response-root-requires-a-declared-response` (`body_root`'s response arm builds an empty
  object for a command with no response). Surface: `ess-gen/src/model_types.rs`,
  `schema-contract/src/realize.rs` (cited), new tests in `schema-contract/tests/` and
  `ess-gen/tests/command_view_roots.rs` (inferred). No ess-conformance build.
- U2 (rows 6 and 7, medium; one unit because both land on
  `crates/verify/ess-conformance/src/synthesize/related_guard.rs`):
  `story:an-outcome-arrangement-follows-the-bindings-its-steps-set-off` (`arranged_at` does not
  follow a binding its steps set off; pinned by `adversary_0730_pass1.rs:104`) and
  `story:related-guard-wrong-state-beside-external` (`wrong_state_overlap` / `refusal_arrangement`
  arrange no row beside an external branch; cases a4, a4b, a4c ignored in
  `adversary_selection_plan_w5_u4_pass1_open.rs`). Surface: `related_guard.rs` (cited),
  `synthesize.rs` `refusal_arrangement` and `binding_effects.rs` (inferred).
- U3 (row 14, medium): `story:wrong-state-witness-unknown-and-own-stored-guards`. Surface:
  `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (cited).
- U4 (row 17, low): `story:the-published-schema-admits-the-name-aliases-the-parser-reads`. Surface:
  `crates/specify/ess-domain/src/binding.rs`, `component.rs`, `schemas/generated/ess.schema.json`
  (cited), `ess-xtask` consumer-coverage pins (inferred).

Every unit gates package-scoped through the host's one build slot (host-monitor D95: debuginfo 0,
`CARGO_BUILD_JOBS=8`, start only with / at 80G free or more, at most one other cargo, the tree's
`target/` under 15G; a failed check stops the command, nothing waits). Any build of ess-conformance
(U2, U3, and U4's ess-cli targets) runs only on a slot the orchestrator grants.

## Selection

`aep plan artifact waves --kind story --status draft --format json` (aep 0.78.0), after one
`aep:story-scoper` per unscoped candidate (rows 6, 7, 11, 12, 15, 21, 25; rows 8, 10, 14, 17, 19,
22, 27 already carried a Scope section). Verb output for the picked stories, verbatim:

- waves: `a-null-when-absent-binary64-field-realizes` wave 1, `a-response-root-requires-a-declared-response`
  wave 2, `an-outcome-arrangement-follows-the-bindings-its-steps-set-off` wave 3,
  `related-guard-wrong-state-beside-external` wave 11; `wrong-state-witness-unknown-and-own-stored-guards`
  and `the-published-schema-admits-the-name-aliases-the-parser-reads` collide with no picked story.
- collisions among the candidates read: `a-null-when-absent-binary64-field-realizes x
  a-response-root-requires-a-declared-response on crates/generate/ess-gen/src/model_types.rs (cited)`
  (one unit, U1); `an-outcome-arrangement-follows-the-bindings-its-steps-set-off x
  related-guard-wrong-state-beside-external on .../synthesize/related_guard.rs (cited)` (one unit,
  U2); `an-outcome-arrangement-follows-the-bindings-its-steps-set-off x
  binding-site-identity-comparisons on .../synthesize/binding_effects.rs (inferred)`;
  `binding-site-identity-comparisons x related-guard-wrong-state-beside-external on
  .../synthesize.rs (inferred)`; `a-parent-tenant-guard-holds-across-every-related-row-and-absent-field`
  collides with U2 on `.../synthesize.rs (inferred)`.
- unassessed among the candidates: `story:a-no-view-arranged-half-probes-the-row-through-the-command`
  (row 19; its Scope cites one path without a typed entry). 237 drafts are unassessed store-wide.

Left out:

- Row 12 (`story:binding-site-identity-comparisons`, medium) collides with U2 (verb, inferred) and
  would be a third ess-conformance build; next wave.
- Row 11 (`story:row-set-beside-named-rows-suite-tells-every-order-apart`) lands on
  `related_guard.rs`, U2's file; next wave.
- Row 8 collides with U2 on `synthesize.rs` (inferred); next wave.
- Row 10 (`story:concurrent-history-records-inputs`) adds an input to `ess-history/1`, whose reader
  and schema are closed: a format decision first; next wave.
- Rows 19, 21, 22, 27 (low): wave cap of four units.
- Row 9 stays blocked on `decision-blocker:decimal-canonical-serialization-crosses-repositories`.

## Review scope

Adversary passes review each unit's diff against its acceptance only (operator rule of
2026-10-31). Gaps inside the acceptance are fixed in the wave; anything outside is one
out-of-scope row in the pass's review-result, decided by the coordinator at wave close (adopt as a
story, decline with reason, duplicate).

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-d350`, the store commits on that branch, the release-prepare commit (version bump,
lock files, CHANGELOG, change record) on that branch before the final gate, one pull request to
`main` from a `queue/` branch through the App and its merge on a green pipeline, then the release
tag and publication through the repository's own process. Not the next wave.

Decider: operator

## Wave record

Scratch root `~/.cache/ess-d350-wave/`; trees under the worktree manager's `b10x/ess/` root.

| unit | tree id | branch | build dir | scratch | stage |
|---|---|---|---|---|---|
| int | ess-d350-int | integration/ess-d350 | tree `target/` | `~/.cache/ess-d350-wave/` | open |
| U1 | ess-d350-u1 | unit/ess-d350-u1 | tree `target/` | `~/.cache/ess-d350-wave/u1/` | planned |
| U2 | ess-d350-u2 | unit/ess-d350-u2 | tree `target/` | `~/.cache/ess-d350-wave/u2/` | planned |
| U3 | ess-d350-u3 | unit/ess-d350-u3 | tree `target/` | `~/.cache/ess-d350-wave/u3/` | planned |
| U4 | ess-d350-u4 | unit/ess-d350-u4 | tree `target/` | `~/.cache/ess-d350-wave/u4/` | planned |

Pre-flight 2026-11-01: / 92G free of 848G; one other cargo process machine-wide; the earlier ess
trees hold no `target/` (all 0 or 4K).
