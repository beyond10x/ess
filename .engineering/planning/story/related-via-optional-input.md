---
format: aep.planning-md/3
id: story:related-via-optional-input
kind: story
status: active
title: when_related reads through an Optional input; absence reads no row (ess/22)
refs:
- provider: github
  reference: beyond10x/ess#304
relations:
- decomposes: epic:ui-live-apps
- serves: vision:O2
- depends_on: story:feature-request-287
- supersedes: story:feature-request-304
scope:
- confidence: cited
  path: crates/generate/ess-gen/src/openapi.rs
- confidence: inferred
  path: crates/specify/ess-compiler/src/ir.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-compiler/tests/related_guard_ir.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_guard.rs
- confidence: cited
  path: crates/specify/ess-domain/src/command/related_value.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/execute/related.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/related_guard.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/related_guard_optional.rs
- confidence: inferred
  path: docs/design/cross-record-and-stored-field-guards.md
- confidence: inferred
  path: website/docs/reference/predicates.md
revision: 15
transitions:
- {from: "draft", to: "proposed", at: "2026-10-01T20:11:54Z", actor: "human:timo", revision: 7, decided_on: {"recorded":{"review_outcome":4}}}
- {from: "proposed", to: "active", at: "2026-10-01T20:11:54Z", actor: "human:timo", revision: 8, decided_on: {"recorded":{"review_outcome":4}}}
---
## Outcome

A `when_related` guard reads through an Optional input: an absent reference reads no row and selects no `when_related` branch; a present one is checked as today (ess/21, story 1 of the design in beyond10x/ess#304).

## Fit review

origin/main moved to `8700d0808` (0.50.0) during the review; among the cited files only `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (+121, #272) and the new story `feature-request-304.md` changed. Binaries run: `~/.cache/ess/toolchains/0.49.0/ess`, `0.48.0/ess`. Scratch specs: `~/.cache/uilab-todo/fit-304/`.

1. **Need.** #304: a command takes an Optional reference; present → it must name a row in a given state; absent → nothing is checked (reproduction `optional-via.yaml`, `AddTask` with `repair_of: Optional<CandidateId>`). uilab: the reference is a stored field of the addressed row and may be absent: a task stores `blocked_by: Optional<TaskId>`, and `CompleteTask` is refused while that task is not `Done` (`a-subject-via.yaml`). #304's "What happens" does not reproduce: on 0.48.0 and 0.49.0 `optional-via.yaml` is refused at validate with `ESS-COMMAND-002 type_mismatch: input.repair_of is Optional<…>, and when_related reads the one row an identity names`; nothing reaches a not-found branch (ask the reporter).
2. **Class: gap, both.** The refusal is documented: `website/docs/reference/predicates.md:775` ("must be a required input"), `docs/design/cross-record-and-stored-field-guards.md:517-520` ("One hop, from the input only"), `crates/specify/ess-domain/src/command/related_guard.rs:501-513`, `:110-125`. The story draft's "Likely accept as a defect" (`feature-request-304.md:33`) is wrong.
3. **Already expressible?** #304 only by changing the API (split into `AddTask` and `AddRepairTask`: `optional-via-split-idiom.yaml` valid, 8 scenarios); no substitute for a retrofit of one optional field. uilab: no. On 0.49.0: `via: blocked_by` refused (`ESS-COMMAND-002`); a client-supplied `blocker` input checked against the stored value refused (`ESS-COMMAND-004`), and without that check a client can lie; a `blocked` flag kept by `affects:` validates (`c-affects-flag.yaml`) but synthesis refuses the completing branch (`ESS-SYNTH-001`, as open #288) and the Go target fails the workspace ("affects: this target cannot change the rows a filter selects", `crates/generate/ess-synth/src/set_effects.rs:1-60`); a separate Block entity needs a lookup by field (rule 2, design doc `:441-473`).
4. **Fit.** `{related: {via}}` already reads a subject field (bare name) or an input (`input.<f>`) (`related_value.rs:17-31`); the IR has `ResolvedRelatedVia::Subject` (`crates/specify/ess-compiler/src/ir.rs:1106-1121`); `when_related` is the one place that cannot say it (backlog: `related-guard-vocabulary-aligns.md`, acceptance line 1). #285 makes the same Optional reference read as absent in ess/21 (`feature-request-285.md:38`). A subject via is read after the subject loads, the step #282 adds after held state (`feature-request-282.md:38`). Targets today: Rust, Go, Web, Clap keep a `when_related` command a hand-written obligation (`crates/generate/ess-synth/src/determined.rs:108`); Entity Runtime refuses it by name (`ess-entity-runtime/src/lib.rs:1281-1284`); the interpreter declines a stored-field via (`interpret/execute.rs:524-528`); diff, docs and plan render `via` generically (`ess-diff/src/diff.rs:976`, `ess-gen/src/docs.rs:1744`, `ess-synth/src/plan.rs:722-740`). Family F's row-set form (`feature-request-237.md`) cannot tell a missing row from a failed test; lookup by identity stays `via`.
5. **Second adopter.** Optional input: an expense may name a budget which, if named, must be `Open`. Stored reference: a shipment dispatched only while its stored payment is `Captured`; a change merged only while the change it depends on (Optional) is `Merged`.
6. **Cost.** No format of its own: goes into the ess/21 bundle (`.engineering/waves/downstream-gaps.md:40`); below ess/21 the new forms are refused with `unsupported_format_version` naming ess/21. No new keys, no new diagnostic codes, only messages; no migration; IR for ess/20 and below byte-identical; no new `ScenarioId`, no suite-format bump; generated Rust/Go signatures unchanged, obligation text only.
7. **Designs.** (a) change nothing: rejected. (b) #304 as filed (Optional input only): rejected (leaves the stored-field case and the sibling gap). (c) family F's row-set form: rejected for lookup by identity. **(d) chosen: one design for `via`, covering both.**


## Decisions

**Accept, redesigned.** One design, two stories, both ess/21.

- `when_related.via` is `input.<f>` or a bare `<f>` (a stored field of the addressed subject as it was just before the branch), the `{related:}` spelling; `OutcomeCondition::Related.via` becomes `RelatedVia`. Either may be `Optional<Id>`; `ResolvedRelatedVia.type_ref` carries the declared type.
- Absent reference: nothing is read; no `when_related` branch is selected; selection carries on. Validation requires the absent case to reach exactly one branch, else `non_exhaustive_branches` naming the absent `via`. Present reference: unchanged (`exists: false` still required wherever there is a predicate branch).
- Precedence: input via unchanged (step 1); subject via answers at #282's new step, after existence (3) and held state (4), before accepting branches (5). Subject via refused on `creates:` and on a command with no supplied subject; `when_subject*` and `unknown_instance` beside it stay refused in this cut.
- Story 1 (#304): the Optional input via. Story 2 (uilab): the subject via, Optional included; takes over acceptance line 1 of `related-guard-vocabulary-aligns`; depends on #282.
- Synthesis witnesses: absent (a segment on the scenario of the branch it selects, between related rows in the refusing state; no new id); present accepted (`<cmd>/outcome/<default>`); present wrong state (`<cmd>/outcome/<refusal>`; for a subject via the referenced row sits between decoys in the opposite state and the subject is `Open`); missing row (`<cmd>/outcome/<exists-false>`; for a subject via witnessed where some writer stores an unchecked identity, else a note that the branch is unreachable, else `ESS-SYNTH-003` naming why).
- Targets: validation, compiler, synthesis implement it; the interpreter handles an absent input via; it executes a subject via from story:related-via-stored-reference on; Rust, Go, Web, Clap keep the command an obligation (obligation text only); Entity Runtime keeps `RelatedGuardUnsupported`; diff, docs, OpenAPI get "when present" wording.


## Acceptance

`crates/specify/ess-domain/tests/related_guard.rs`: `issue_304_an_optional_input_via_validates_under_ess_22`, `issue_304_an_optional_via_below_ess_22_is_refused_naming_ess_22`, `issue_304_an_absent_reference_reaching_no_branch_is_non_exhaustive`; `issue_211_via_is_an_input_field_typed_as_another_entitys_identity` stays green. `crates/specify/ess-compiler/tests/related_guard_ir.rs`: `issue_304_ess_20_related_fixtures_compile_to_identical_ir`. New `crates/verify/ess-conformance/tests/related_guard_optional.rs`: `issue_304_absent_present_and_missing_are_each_witnessed`, `issue_304_a_target_treating_absent_as_missing_fails`, `issue_304_a_target_ignoring_a_present_reference_fails`. `crates/verify/ess-conformance/tests/interpreted_command_execution.rs`: `issue_304_an_absent_optional_reference_reads_no_related_row`. `crates/generate/ess-gen/tests/related_guard.rs`: `issue_304_an_optional_reference_is_documented_as_checked_when_present`.

The stored-reference (subject via) tests belong to story:related-via-stored-reference; generation of either via to story:related-guard-behaviour.

## Scope

Cited: `ess-domain/src/command/related_guard.rs` (parsing, type check :501, format gate, absent coverage), `related_value.rs:17-31,66-68`, `ess-compiler/src/resolve.rs:2835-2850`, `ir.rs:1104-1121` (doc), `ess-conformance/src/synthesize/related_guard.rs`, `synthesize/related.rs` (:325, :416, :542), `synthesize.rs:6281`, `ess-conformance/src/interpret/execute.rs:495-560`, `ess-synth/src/plan.rs:722-740`, `ess-gen/src/docs.rs:1744`, `ess-gen/src/openapi.rs:1155`, `ess-diff/src/diff.rs:976`, `website/docs/reference/predicates.md:27,770-810`, `docs/design/cross-record-and-stored-field-guards.md:517-539,605-628`. Inferred: `ess-domain/src/command/mod.rs`, `system.rs` (V21, the bundle's), `spec-versions.md`, `formats.md`, `schemas/generated/ess.schema.json` (coordinator-owned). Must not change: `determined.rs:108`, Entity Runtime `lib.rs:1281`, suite format `ess-conformance/10` and the `ScenarioId` set, IR bytes and suites for ess/20 and below.


## Sequencing

Base 0.50.0+. #287 touches `synthesize/related_guard.rs` and `related.rs`: the synthesis half waits for it. #310, #306: none. Within ess/21: story 1 before #285 or in its unit (shared `related_value.rs`); story 2 after #282 (owns the precedence step); #283 and family F's row-set form edit the same files: one at a time. `ess-gen/src/openapi.rs` (`:1155`, "when present" wording) is also edited by story:served-view-params (`:560-564` comment): different regions, whichever lands second rebases.

`CHANGELOG.md` is a merge-time edit (epic).

## Bounded prerequisite disposition for the serial batch, 2026-10-03

Read-only ancestry inspection found singleton core1f131e170, release integration482bc33c2 and tag0.51.0 ancestors of current main1ff305685. feature-request-287 itself records that released core and remaining ignored cross-caller reinstall, Uuid and ordered-view scenario-withdrawal limits. Optional-input requires the landed singleton synthesis change and serialized edits to related witness arrangement; its named acceptance does not require the retained failing shapes. The coordinator therefore treats that landed core as satisfying this unit's sequencing prerequisite while preserving the broader #287 artifact as active and preserving every ignored limitation as unresolved. This is a bounded planning disposition, not a new test result or whole-story completion. The depends_on edge remains visible with this qualification.

Current main's interpreter declines present related predicates; later guard-generation work must implement them or integrate a bounded reviewed equivalent. Candidate ee50829da is not landed and relies on earlier held-subject work on a74-commit line: do not import that line wholesale under this five-wave approval. Optional absence must skip both row lookup and related-branch selection, rather than fall into the unconditional decline. Merge readiness still depends on the ess/21 format bundle decision.

## Precedence reconciliation for the approved serial batch

The earlier "input via unchanged (step 1)" phrase preserves the early lookup/missing-row behavior. It does not override the accepted ess/21 held-state-before-related-predicate-refusal decision in story:feature-request-282, implemented first in this batch. Optional absence skips every related lookup/branch regardless of placement; required/present input lookup retains the early missing-row refusal, while present-row predicate refusal follows the held-state decision under ess/21. Below ess/21 preserve the existing supported contract. Source: this story's #282 dependency in its design and story:feature-request-282 Decisions/Acceptance. This is a coordinator clarification of the overlapping accepted decisions; implementation must prove the distinctions rather than introduce a separate order.

## Current allocation and acceptance ownership

The accepted bundle now allocates these syntax additions to source ess/22; ess/21 is reserved for one-time responses. This supersedes historical ess/21 references in the original fit, cost, decisions and sequencing paragraphs without erasing those dated assessments. Older formats through21 must refuse each newly admitted Optional-input/stored-reference form by its source location; required input-via behavior and serialized bytes of unchanged old models remain stable. The named acceptance tests above use ess_22 accordingly.

Execution order remains #282, Optional input, stored reference, then generated guard behavior319. Optional-input owns absent/present/missing input reference admission, interpretation, synthesis and documented target obligations. Stored-reference owns the corresponding addressed pre-branch field lookup, subject existence/state precedence, Optional stored absence and stored-reference fault controls. Common declaration/IR infrastructure may be introduced in the first slice but does not claim the second slice's acceptance. Input absence never means a missing related row: it performs no lookup and selects no related branch. For present references, preserve #282's distinction between early missing-row refusal and later present-row predicate evaluation.

Named Rust/Go/Web generation obligations are permitted only at the intermediate304 boundary because319 implements those targets in this same bundle. They cannot satisfy final generated-target acceptance or close the combined issue. No duplicate implementation of the already integrated stored-field interpreter is needed; compose on current #292/nested correction/282 source and retain all existing controls. This is planning reconciliation based on read-only current-source inspection, not a new test result.

## Prepared tests refresh after reviewed precedence integration

Reviewed #282 candidate31e7362f89464846f069591b313529bbc2f65f9d is integrated fe251b2a4 with14 interpreter-command checks,546 neighboring checks, strict lint, and final independent approval. The prepared Optional-input test commit4cf695b0cb2cc637c6fcd1415cce490385a006b5 remains the starting unit. Existing worker review_292 now refreshes that managed unit onto10fdf93606e568a172d6edfd8cd5b2cd2908dfd7, preserving all prepared304 tests and integrated282 controls. Only source preparation and merge-conflict resolution are assigned; no production implementation or baseline execution is claimed. External412 baseline and the remaining five-file report migration execute first under the serial resource contract. Source22 admission, old-reader refusal, declared Optional type retention, no lookup on absence and present-reference precedence remain the acceptance authority. Stored-reference304 and generated behavior319 remain later units.
