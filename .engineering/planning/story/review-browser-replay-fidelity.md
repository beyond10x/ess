---
format: aep.planning-md/1
id: story:review-browser-replay-fidelity
kind: story
status: implemented
title: Make browser replay faithful to its declared semantic subset
tags:
- P1
- review-2026-09-05
relations:
- decomposes: epic:review-boundary-remediation
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/replay_fidelity_browser.rs
- confidence: cited
  path: crates/verify/ess-conformance/assets/coverage-player.js
- confidence: cited
  path: crates/verify/ess-conformance/assets/index.html
- confidence: cited
  path: crates/verify/ess-conformance/assets/player.js
- confidence: cited
  path: crates/verify/ess-conformance/src/web.rs
- confidence: cited
  path: docs/design/review-replay-subset.md
revision: 19
---
## Finding and source

F15 (P1) from `docs/reviews/2026-09-05-architecture-review.md:500`, reviewed at `fd06a4d61bfb7b4990617810655dc181d6a3ab00`. Evidence locations: `crates/verify/ess-conformance/src/web.rs:125`, `crates/verify/ess-conformance/src/web.rs:195`, `crates/verify/ess-conformance/assets/player.js:111`. Findings are attributed to the review; proposed implementation choices below are coordinator inferences.

## Acceptance

Replay displays the correct supported assignment/view state or an explicit unsupported marker for each F15 counterexample instead of silently computing a plausible state.

## Implementation boundary

Preserve typed literal assignments, apply sets with state moves, retain ordering and parameter filter semantics where supported. Establish an explicit replay subset and visible unknown state/calculations otherwise. Prefer common differential vectors or an existing evaluator; keep replay and implementation conformance distinct.

## Validation

Exercise the generic player itself for sets+move, literal values, view order and parameter filters, plus unsupported semantics; compare projected state with reference vectors. The billing WASM lab alone does not satisfy this acceptance.

Run package-scoped checks while implementing; the integration coordinator runs every step of `task check` and `task site-build` when required by `AGENTS.md`, retaining individual exit statuses and executed-case counts.

## Compatibility and exclusions

No production interpreter or universal evaluator; primitive comparison vectors must follow review-primitive-semantics if that migration has landed.

## Accepted implementation boundary

Root accepted the conservative declared-trace/explicit-unknown direction in reviewed candidate v2, SHA256 33bc30a78fcf90e29141d24a4a82d23b9f86c44576d474079478927defa74f88. The complete behavior and B01–B15 matrix in `docs/design/review-replay-subset.md` are binding once root publishes that accepted text. Both candidate attacks are recorded as `review-result:browser-replay-binding-pass1` and `review-result:browser-replay-binding-pass2`; the ledger carries 0, adds 0 and resolves the one B06 validation finding. Those reviews executed no runtime cases.

This chooses the existing acceptance’s explicit-unsupported alternative wherever replay/1 lacks literal, conversion, subject, filter or ordering semantics. Both actual current players must preserve declared values and trace order, clear uncertain API knowledge, display explicit unknowns/unexecuted expectations and use deterministic playback controls. No persisted format extension, stronger evaluator, primitive migration or implementation-conformance claim is selected.

The earlier source refresh through ESS 57e242e8a0eaa721968c3970099b4bc561cb91aa confirmed six existing write owners and the historical fixture matched reviewed source 9d84a425e3a0c052bb08975766c5dbd600d04ef0. That statement is historical. Current inspected source is a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 after namespace-observation PR #12: five browser owners and the historical fixture remain identical, while main.rs has namespace-import changes and unchanged web help/dispatch. Both proposed paths remain absent. Public-support support-check and source/release qualifications remain part of the current gate. The eight reservations below still supersede the earlier broad reservation for scheduling; preserve the new namespace flags, import branch and qualified report when editing only web-help wording. The historical scope and inspection note remain unchanged below.

Actual Firefox/BiDi execution of both explicit suite/4 and suite/5 emitted players is mandatory for B01–B15, with the accepted admitted-fixture route for control steps that the authored frontend cannot emit. Preserve every historical, coverage, lineage, refusal and pairing family. No browser skip, billing-WASM substitution or source-token check substitutes for those cases. Root selects the wave and assigns a new managed unit and mutable resources after the current gate; this body does not activate or dispatch implementation.

## Scope

Confirmed by implementation at 1a416a607f84cfb531cb4ed2c7279a5ad189801a and integration at 3408bbf049d10215487c50f6f7b5597486b14127. The original implementor confirmed all eight reservations, and source review retained the same boundary. Earlier inferred reservations are now observed source/test/document changes; no additional path was required. The original dispatch scope remains below as history.

| Path | Confirmed result |
|---|---|
| `crates/verify/ess-conformance/src/web.rs` | README wording only; projection and emitted suite/model formats preserved. |
| `crates/verify/ess-conformance/assets/player.js` | Typed input/declaration display, conservative unknown propagation and playback controls. |
| `crates/verify/ess-conformance/assets/coverage-player.js` | Same replay behavior after unchanged paired-input admission. |
| `crates/verify/ess-conformance/assets/index.html` | Visible declared, unknown and unexecuted presentation. |
| `crates/edge/ess-cli/tests/coverage_browser.rs` | Existing assertions retained; historical player bytes explicitly installed for the historical replay case. |
| `crates/edge/ess-cli/tests/replay_fidelity_browser.rs` | New actual Firefox vectors for the complete binding, plus retained independent regression cases. |
| `crates/edge/ess-cli/src/main.rs` | Only web-help wording; namespace import and command dispatch preserved. |
| `docs/design/review-replay-subset.md` | Clarifies wrong-state policy versus actual refusing outcomes and capture event-field identity; no stronger projection semantics. |

All eight entries are cited by the source diff and retained implementor report. Original inferred labels and inspection notes remain visible below; path reservations did not expand. The historical player, admission/harness, compiler/scenario owners and Cargo.lock remain unchanged. Root owns the separate planning, changelog and wave records.

## Dispatch scope (retained history)

Derived 2026-09-07 by `aep-drive:story-scoper` 0.8.0. Original reviewed source is ESS 9d84a425e3a0c052bb08975766c5dbd600d04ef0; the earlier public-support refresh at 57e242e8a0eaa721968c3970099b4bc561cb91aa is historical. Current inspected source is a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3 after namespace-observation PR #12. Five existing browser owners and the historical player remain byte-identical; CLI main.rs has namespace-import changes while its web help/dispatch remains unchanged. Both proposed paths remain absent. All eight path reservations and cited/inferred marks are unchanged.

- **Primary surface:** the existing conformance web emitter and its two current generic players — cited; both current emission routes still contain the F15 behavior.
- **Emitter:** `crates/verify/ess-conformance/src/web.rs` — cited; `emit`, `emit_input`, `model`, `set_source` and `readme` own emitted assets, the shared reduced projection and its explanation. Reserve README wording and package-local projection/asset checks; preserve model and suite bytes.
- **Legacy-route player:** `crates/verify/ess-conformance/assets/player.js` — cited; `literal`, `groupSteps`, `applyAct`, `evaluateViews`, `mark` and playback controls own the current default player's behavior.
- **Coverage-route player:** `crates/verify/ess-conformance/assets/coverage-player.js` — cited; admission precedes replay creation, followed by the same defective assignment, subject-selection and view calculations.
- **Visible presentation:** `crates/verify/ess-conformance/assets/index.html` — cited; state, notes, view results and progress must distinguish declared inputs, unknown projections and unexecuted assertions.
- **Existing browser compatibility test:** `crates/edge/ess-cli/tests/coverage_browser.rs` — cited; its emitted-player-equals-historical-fixture assertion must become an explicit historical-player test while retaining every existing admission, pairing, lineage and old/new-metadata control.
- **New actual-player vectors:** `crates/edge/ess-cli/tests/replay_fidelity_browser.rs` — inferred; one Rust integration test owner containing the finite independent vectors and fixture sources, using the existing Firefox harness unchanged.
- **CLI description:** `crates/edge/ess-cli/src/main.rs` — inferred; update only the web command's claim that a green walk establishes specification coherence. Preserve dispatch, selection and suite-format defaults, including the newly landed namespace-import flags, acquisition branch and coverage report.
- **Binding document:** `docs/design/review-replay-subset.md` — inferred; record the finite subset, unknown propagation, literal/identity/conversion limitations, compatibility decision and executable matrix.
- **Read-only dependencies:** paired-model admission, coverage admission, the existing Firefox harness, compiler/scenario declarations and the retained historical player remain source evidence and regression controls — cited; this candidate requires no implementation changes to those owners.
- **Confidence:** high — cited; current owners and generic-browser infrastructure are present, and the proposed new test/document paths are explicitly identified.
- **Would collide with:** changes to either player, the shared HTML/emitter, CLI web help or generic-browser compatibility tests — cited.
- **Boundary:** no persisted replay extension, primitive migration, production interpreter, universal evaluator, new story or coordinator-owned planning mutation — inferred; stronger computed semantics require a separately accepted format consequence before implementation.

## Earlier scope (retained history)

Derived 2026-09-06 by story-scoper against ESS e339f4ce3d5046443ff8e99b0ea72099bc23e410.
Entries distinguish cited surfaces from inferred implementation reservations.

- **Primary surface:** `crates/verify/ess-conformance` — cited; the story names this owner, and replay projection, browser behavior and existing projection tests remain package-local.
- **Producer files and symbols:** `src/web.rs:54` (`emit`), `:75` (`model`), `:108` (`outcome`), `:123` (`subject`), `:143` (`view`), `:181` (`set_source`) within the primary surface — cited; these determine the model information available to replay.
- **Browser files and symbols:** `assets/player.js:56` (`literal`), `:68` (`groupSteps`), `:111` (`applyAct`), `:185` (`parseFilter`), `:204` (`evaluateViews`), `:262` (`mark`), and `assets/index.html:176` (state), `:200` (refusal heading), `:205` (views) within the primary surface — cited; these own the demonstrated F15 behavior and visible limitations.
- **Validation surface:** `src/web.rs:229`, `:256`, `:275` and `tests/synthesis.rs:2231` within the primary surface — cited; existing checks inspect emitted assets or collect canonical artifacts and do not exercise browser semantics.
- **Required new validation:** owner-package generic-player browser cases and independent expected vectors for every F15 counterexample, including reset/back replay and visible unsupported results — inferred; exact filenames depend on the browser harness delivered by coverage work.
- **Binding document:** `docs/design/review-replay-subset.md` — inferred; absent at this baseline and required to choose supported assignment/value/filter/order/parameter semantics, unknown propagation, markers and any replay format migration.
- **Scope limit:** this is the minimum unconditional reservation; a stronger persisted replay representation or changed CLI selection requires refreshed reservations for concrete CLI, format-catalog and public-documentation paths before dispatch — inferred.
- **Confidence:** medium — inferred; defect locations and owners are established, but accepted replay/1 intentionally leaves stronger representation and format consequence to this story.
- **Would collide with:** the conformance package's web emitter, paired-model admission, player assets, generic-browser harness and compatibility fixtures; the same package's primitive-comparison and synthesis-vector work — cited.
- **Sequencing:** refresh source/API assumptions after coverage-writer integration; preserve complete original-byte admission, pairing, selection, coverage/refusal display and historical compatibility fixtures — inferred.
- **Coordinator-owned records:** planning journal, wave selection, integration evidence and shipment records remain outside the implementation reservation — inferred.


## Fresh inspection note

The read-only `count_writer_scope8` report at ESS e339f4ce3d5046443ff8e99b0ea72099bc23e410 confirms F15 remains open. Accepted coverage transport intentionally preserves replay/1 limitations. Assignment literals in `ResolvedPayloadValue::Literal` are String (`ess-compiler/src/ir.rs:744`), not an already typed JSON scalar; conversion metadata, precise subject sources, ranking and filter AST are absent from replay/1. A richer projection requires a separate versioned binding before implementation. Runtime corrections using the existing shape must expose information loss instead of guessing. Refresh reservations and source assumptions after coverage integration. This is preparation, with no source implementation or F15 test result claimed.


## Dispatch preparation at namespace-integrated source

The complete refreshed accepted-binding publication draft, implementor brief and exact input/output readback are retained under target/review-boundaries-14/preparation/browser-replay-candidate/dispatch-refresh/. Source anchor: a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3. No accepted behavior or B01–B15 vector changes; no third candidate attack or runtime result is claimed. Root selects the next N=1 wave and publishes the binding/story before filling the final dispatch commit, managed unit/branch/lease, target, scratch, private Cargo/TMP/browser roots and measured resource allowance. Unfilled slots are not defaults or authorization. The retained full Rust 1.98.1 bin/lib/WASM and Firefox 153.0 payload manifests are exact tool inputs, to be verified before/after future producers. Preparation itself performs no build/browser launch or planning mutation.


## Wave 15 implementation selection

The coordinator selects this story alone under the existing standing implementation and publication approvals. The accepted binding is published at docs/design/review-replay-subset.md; all eight exact reservations apply. The source anchor remains a45b4081de9352e0b2f0b7a8ec87bb91f99b6cc3, whose runtime bytes remain unchanged in published closure 4a53fca3dbb3df37d200ea8c6191549f135fb06f. The earlier preparation statements describe their historical stage. Full B01–B15 execution through both actual Firefox players, package checks, independent source review and the integration gate remain required before completion. See docs/plan/2026-09-07-review-boundaries-15.md for the exact unit, resource and lifecycle assignment.


## Wave 15 completion evidence

The integrated source at 3408bbf049d10215487c50f6f7b5597486b14127 passed all eleven fresh gate lanes: site-build, formatting, strict Clippy, workspace tests, rustdoc, examples, projection, support, release consistency, action contract and planning validation. The workspace runner executed 2,155 tests across 195 groups, with zero failures or ignored cases. Every lane returned zero and retained identical tracked source bytes. Completion was observed at 2026-09-07T10:40:43.825866+00:00. Exact commands, direct exits, durations, source/tool readbacks and raw outputs are retained under target/review-boundaries-15/gate-3408bbf049d1-attempt1; coordinator-readback.json records the independent log-hash and count readback. Unit commit: 1a416a607f84cfb531cb4ed2c7279a5ad189801a. This evidence establishes implementation completion; remote publication and managed retirement are recorded separately after their actual outcomes.

The first source pass was interrupted after seven cases found one optional-null renderer defect on both routes. Its exact interim report is retained; it is not represented as a completed source review. Root independently reproduced the two failures, the original implementor added the null guards, and every retained assertion remained. The final separate source pass added three browser cases and found no further product defect. No third source pass was opened.

The unit's package checks executed 287 conformance cases and 34 final browser cases, with zero failures or ignored cases; formatting and strict package Clippy returned zero. These are package results, distinct from the complete integration gate recorded above. B01–B15 apply to both actual players. The legacy fixture SHA256 remains 990575f6db31d13fecd7d06df51f3d34190cc8df33fa1a320efe682900d2e23f, and original suite/model/replay fixture byte comparisons remain in the retained handoff. This closes the accepted explicit-unknown replay subset, without claiming implementation execution or a universal evaluator.
