---
format: aep.planning-md/3
id: story:a-no-view-arranged-half-probes-the-row-through-the-command
kind: story
status: active
title: A no-view arranged half probes the row through the command
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/synthesize/existence.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_arrangement_pass2.rs
- confidence: cited
  path: docs/design/outcome-shapes.md
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T20:19:29Z", actor: "human:timo", revision: 4}
- {from: "proposed", to: "active", at: "2026-11-01T20:19:30Z", actor: "human:timo", revision: 5}
---
# Story: a no-view arranged half probes the row through the command

## Outcome

Where no identity view shows an arranged row, the arranged half of an input-refusal scenario still detects a target that changes the record on refusal, by following the refusal with a probe the model decides from the row (for example the command's own valid send, whose outcome depends on the row's state).

## Why

Adversary pass 2 on the 0.41 arrangement unit (`review-result:adversary-a-arrangement-pass-2`): with no identity view, the arranged half requires only the error and no event (`crates/verify/ess-conformance/src/synthesize/existence.rs:336`), so a target that silently moves the record on refusal passes; the next valid send would reveal it. Reached only by a built model; nothing in the repository.

## Acceptance

- `issue_209_no_view_arranged_half_catches_a_target_moving_the_record_on_refusal` in `tests/adversary_arrangement_pass2.rs` is flipped from asserting today's pass to asserting failure.

## Scope

Derived 2026-11-01 by `story-scoper` against `integration/ess-d357` at 78d58fb47 (ESS 0.81.0). Every line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).

- **Primary surface:** `crates/verify/ess-conformance/src/synthesize/existence.rs`, the function `arranged_refusal` — cited
- **Files:** `crates/verify/ess-conformance/src/synthesize/existence.rs:435-452` — cited. The no-view branch: `observed` is false, so an empty `subject_fact::Preservation` is used and nothing reads the row back. The story's `:336` has drifted; the doc sentence it means is now `:337-338`.
- **Files:** `crates/verify/ess-conformance/src/synthesize/existence.rs:486-497` — inferred. The probe step (the addressing branch's valid send for the arranged instance, with the outcome it requires) is added after `ExpectNoEvent`/`preservation.after`, before `models.mark(InvocationPhase::Act, …)`.
- **Files:** `crates/verify/ess-conformance/src/synthesize/existence.rs:329-346`, `:347-396` — cited. The doc and the caller `refusals_on_an_arranged_row`, which already holds the `addressing` branch (`:363`) the probe would send. The call site is `:324`.
- **Symbols:** `arranged_refusal`, `refusals_on_an_arranged_row`, `subject_fact::Preservation` — cited
- **Also likely:** `crates/verify/ess-conformance/src/synthesize.rs` helpers `reach` (`:8290`), `reach_in_state` (`:8007`), `supply` (`:7824`), `expect_error` (`:9229`), `not_emitted` (`:9101`), read only — inferred
- **Tests:** `crates/verify/ess-conformance/tests/adversary_arrangement_pass2.rs:542-562` — cited. `issue_209_no_view_arranged_half_does_not_yet_catch_a_target_moving_the_record_on_refusal` is renamed `issue_209_no_view_arranged_half_catches_a_target_moving_the_record_on_refusal` and flipped to require that both `vault.acct.Configure/outcome/secret-too-short` and `…/missing-configuration` fail against `Mode::MovesOnRefusal` (`:355`). The control `issue_209_no_view_suite_passes_the_correct_target` (`:526`) stays green.
- **Documents:** `docs/design/outcome-shapes.md:281-283` — cited ("with no such view nothing reads the row back" turns false). `CHANGELOG.md` `[Unreleased]`, one line — inferred
- **Generated / pinned bytes:** none move — inferred (all 352 + 78 bytes-table models synthesized with `ess` 0.80.0; none has the no-view arranged-half shape)
- **Confidence:** high for the source site and the test; medium for "no pinned bytes move"
- **Would collide with:** units editing `synthesize/existence.rs` `:329-498`, `tests/adversary_arrangement_pass2.rs`, or `docs/design/outcome-shapes.md` §Precedence (`:276-290`)
- **Not in this story:** `held_state_refusals` falls back the same way without a view (`existence.rs:613-618`); one committed model reaches it (`tests/fixtures/occurrence-clock-offers.yaml`, `demo.offers.AcceptOffer/outcome/malformed`) — cited
