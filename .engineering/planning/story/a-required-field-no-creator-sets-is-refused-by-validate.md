---
format: aep.planning-md/3
id: story:a-required-field-no-creator-sets-is-refused-by-validate
kind: story
status: active
title: Validate refuses a required entity field that no creating command sets
summary: 'Part b of the unfillable-required-field report (adopter item 5): validate exits 0 for a required field no creator sets; a new ValidationCode refuses it; defect, 0.70.0.'
tags:
- adopter-report
- defect
- ess-0.70.0
- severity-medium
- zero-open-issues
relations:
- serves: vision:O2
- informed_by: story:an-unfillable-required-field-is-named-by-validate-and-the-view
- supersedes: story:creates-leaving-required-field-unset-is-refused
- decomposes: epic:adopter-gaps-domain-language
- decomposes: epic:zero-open-issues-2026-11
- depends_on: story:ess-37-and-conformance-64-65-are-open-for-the-epics-constructs
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/validate_unset_required_field.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json
- confidence: cited
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: crates/specify/ess-domain/tests/created_invariant_fields.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/error.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
- confidence: inferred
  path: website/docs/reference/diagnostics.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-11-10T00:41:24Z", actor: "human:timo", revision: 12, decided_on: {"recorded":{"review_outcome":1}}}
- {from: "proposed", to: "active", at: "2026-11-10T00:41:25Z", actor: "human:timo", revision: 13, decided_on: {"recorded":{"review_outcome":1}}}
---
## Need

Split on 2026-10-22 from `story:an-unfillable-required-field-is-named-by-validate-and-the-view`
(part b; adopter item 5 of 9). Making a list field of an entity non-optional, with no creator
setting it, validates cleanly; the interpreted run then reports unsupported scenarios. The author
should learn about the unset required field from `ess specify validate`, before any suite runs.

Minimal reproduction (brand-free): an entity `Job { id, tags: List<Tag> }` whose only creating
command does not set `tags` and declares no default for it. `ess specify validate` on 0.68.0
exits 0.

Re-measured on 0.92.0 (`ess/35`), on an entity `Item` whose only creator is `CreateItem.created`:

| creator leaves unset | validate | synthesize | interpreted run |
|---|---|---|---|
| `tags: List<String>` | exit 0 | 8 scenarios, 0 refusals | 8 passed |
| `title: String` (projected by a view) | exit 0 | 8 scenarios, 0 refusals | 4 passed, 4 unsupported, exit 1 |

So a required list left unset passes the whole suite silently, and a required scalar shows up only
as unsupported scenarios. The only creator-completeness check today is
`validate_created_invariant_fields` (`crates/specify/ess-domain/src/command.rs`, ess#112), which
fires only for a field an invariant reads.

## Where it lives

- Creating-command checks in `crates/specify/ess-domain/src/command.rs`, beside
  `validate_created_invariant_fields`, and the resolved model in
  `crates/specify/ess-compiler/src/resolve.rs`: `class_of` for the new code, and `advise_locating`,
  which reports `ess-domain` advisories as `Severity::Warning` without changing the exit status.
- A new `ValidationCode` in the `validation_codes!` invocation, which moves the digest pinned in
  `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` and the per-file count in
  `docs/design/review-typed-diagnostics.md` (AGENTS.md, "A new ValidationCode or diagnostic moves
  pins outside the crate that declares it").

## Decisions

2026-10-22: a separate unit from part (a). Validate refuses a required, non-defaulted entity field
that no creating command sets, with a new `ValidationCode`.

2026-11-10 (plan-critic round 1): the refusal is gated on `ess/37`, and below `ess/37` the same
code is reported as a warning through `advise_locating`, because both rows of the table above
validate today and a refusal at `ess/36` would reject a document that is valid under its own
format. The code is `unset_required_field`. The earlier "the implementor records which" is
settled by this choice and dropped.

## Acceptance

Domain cases run as `cargo test -p ess-domain --locked --test created_invariant_fields <case>`,
over the reproduction (`Job { id, tags: List<Tag> }`, one creator `CreateJob.created` that does not
set `tags`).

- `a_required_list_no_creator_sets_is_refused_at_ess_37`: at `format: ess/37` validation fails
  with `[unset_required_field]` naming `Job.tags` and `CreateJob.created`. Failing before: the
  same document validates with no error.
- `a_required_scalar_no_creator_sets_is_refused_at_ess_37`: the same with `title: String` in place
  of `tags`, refused with `[unset_required_field]` naming `Job.title` and `CreateJob.created`.
- `a_required_field_no_creator_sets_is_a_warning_below_ess_37`: the reproduction at `format: ess/36`
  validates with no error, and the advisories hold one `unset_required_field` naming `Job.tags`
  and `CreateJob.created`.
- `a_required_field_a_creator_sets_is_valid`: at `ess/37`, the reproduction with the creator
  setting `tags`, and again with `tags: Optional<List<Tag>>`, validates with no error and no
  advisory. Mutation check: a check that fires on every `List` field fails this case.
- `cargo test -p ess-cli --locked --test validate_unset_required_field`: `ess specify validate` on
  the reproduction at `ess/37` exits 1 and prints an `error[ESS-…]` line naming `Job.tags`; at
  `ess/36` it exits 0 and prints a `warning[ESS-…]` line with the same code and names.
- `task test-xtask` and `cargo test -p ess-compiler --locked --test typed_diagnostics` pass with
  the moved pins in `macro-guards.json` and `review-typed-diagnostics.md`.

## Scope

- cited: `crates/specify/ess-domain/src/command.rs`
- cited: `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json`
- cited: `docs/design/review-typed-diagnostics.md`
- cited: `crates/specify/ess-domain/tests/created_invariant_fields.rs`
- cited: `crates/specify/ess-primitives/src/error.rs` (the `validation_codes!` invocation)
- cited: `crates/specify/ess-compiler/src/resolve.rs` (`class_of`, `advise_locating`)
- inferred: `crates/edge/ess-cli/tests/validate_unset_required_field.rs`
- inferred: `website/docs/reference/diagnostics.md` (rewritten by `cargo xtask diagnostics`)

confidence: medium

Moves the `ess/37` `FORMAT_HISTORY` row and `CHANGELOG.md` sentence (coordinator-reconciled).

## Schedule

Scheduled for release 0.70.0 (defect); still open at 0.92.0. Not part of wave A's no-format
units: gated on `ess/37`, it moves to the wave that carries that version.
