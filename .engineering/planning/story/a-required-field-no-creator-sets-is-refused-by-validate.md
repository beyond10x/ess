---
format: aep.planning-md/3
id: story:a-required-field-no-creator-sets-is-refused-by-validate
kind: story
status: draft
title: Validate refuses a required entity field that no creating command sets
summary: 'Part b of the unfillable-required-field report (adopter item 5): validate exits 0 for a required field no creator sets; a new ValidationCode refuses it; defect, 0.70.0.'
tags:
- adopter-report
- ess-0.70.0
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
- informed_by: story:an-unfillable-required-field-is-named-by-validate-and-the-view
scope:
- confidence: cited
  path: crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json
- confidence: inferred
  path: crates/specify/ess-compiler/src/resolve.rs
- confidence: inferred
  path: crates/specify/ess-domain/src/command.rs
- confidence: cited
  path: docs/design/review-typed-diagnostics.md
revision: 3
---
## Need

Split on 2026-10-22 from `story:an-unfillable-required-field-is-named-by-validate-and-the-view`
(part b; adopter item 5 of 9, beyond10x/a2a). Making `Task.artifacts: List<Artifact>`
non-optional, with no creator setting it, validates cleanly; the interpreted run then reports 71
unsupported scenarios. The author should learn about the unset required field from `ess specify
validate`, before any suite runs.

Minimal reproduction (brand-free): an entity `Job { id, tags: List<Tag> }` whose only creating
command does not set `tags` and declares no default for it. `ess specify validate` on 0.68.0
exits 0.

## Where it lives

- Creating-command checks in `crates/specify/ess-domain/src/command.rs` and the resolved model in
  `crates/specify/ess-compiler/src/resolve.rs` (inferred: whether a creator-completeness check
  exists for scalar fields was not traced; the implementor cites it or adds the check beside the
  other creator checks).
- A new `ValidationCode` in the `validation_codes!` invocation, which moves the digest pinned in
  `crates/edge/ess-xtask/src/consumer_coverage/macro-guards.json` and the per-file count in
  `docs/design/review-typed-diagnostics.md` (AGENTS.md, "A new ValidationCode or diagnostic moves
  pins outside the crate that declares it").

## Decisions

2026-10-22: a separate unit from part (a). Validate refuses a required, non-defaulted entity field
that no creating command sets, with a new `ValidationCode`. If the refusal would reject a document
valid today under the current format, it is gated on the next format version and the current
format gets a warning naming the same field and creator instead; the implementor records which,
with the count of currently valid fixtures in the repository it would reject.

## Acceptance

- **Validate refuses the unset field.** Transition: `ess specify validate` on the reproduction.
  Before: exit 0. After: refused with the new code, naming `Job.tags` and the creating command that
  does not set it (or, if gated, a warning naming both under the current format and the refusal
  under the next). Test: `crates/specify/ess-domain` tests (the file holding the other
  creating-command refusals), case `a_required_field_no_creator_sets_is_refused`; and
  `crates/edge/ess-cli` validate output pinned in its existing validate CLI test file. Failing
  before: validate exits 0 on the current tree.
- **A field every creator sets passes.** Transition: the same model with the creator setting
  `tags`, and with `tags` Optional. After: valid, no diagnostic. Same test file, case
  `a_required_field_a_creator_sets_is_valid`. Mutation check: a check that fires on every List
  field fails this case.
- **Pins moved.** `task test-xtask` and `cargo test -p ess-compiler --test typed_diagnostics` pass
  with the moved pins in `macro-guards.json` and `review-typed-diagnostics.md`.

## Schedule

Release 0.70.0 (defect). Not part of wave A's no-format units: if gated on a format version, it
moves to the wave that carries that version.
