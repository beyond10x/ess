---
format: aep.planning-md/3
id: story:a-row-arranged-for-a-later-command-names-arranged-referenced-rows
kind: story
status: draft
title: A row arranged for a later command names the referenced rows it points at
summary: 'Adversary finding on story:accepted-input-arranges-referenced-list-rows: a subject arranged through its creating command for another command''s scenario (synthesize.rs created()) sends a references input naming no created row (turn-3 split, backlog)'
tags:
- backlog
- defect
- from-finding
- review-finding
- severity-medium
- zero-open-issues
relations:
- decomposes: epic:zero-open-issues-2026-11
- serves: vision:O2
revision: 1
---
## Need

`story:accepted-input-arranges-referenced-list-rows` makes the scenario of a command that writes an
input into a `references` field arrange the referenced rows first. It does so in `run_as`, for the
branch under test only. A scenario for another command on the same entity (a move, an update, a
delete, a view read) arranges its subject through `created()` and `invoke()`
(`crates/verify/ess-conformance/src/synthesize.rs`, `created`, near line 7576), which sends the
creating command the plain witness input. When that creating command copies an input into a
`references` field, the subject is created naming an identity no step created, so a target that
resolves the references it stores refuses the arrangement and the scenario fails before it tests
anything.

## Problem

- `demo.reg.Retire/outcome/retired` (moves `Server` from `Live` to `Retired`) arranges its `Server`
  through `demo.reg.Register`, which writes `sources` from its input. The `Register` step in that
  scenario sends a literal identity, and no `AddSource` step precedes it.
- Repository model
  `crates/verify/ess-conformance/tests/fixtures/related-guard-unknown-instance.yaml`:
  `demo.seats.ChangeTier/outcome/changed` arranges its `Seat` through `demo.seats.Issue`, which
  writes `tier_id` from its input into the `tier` reference. The `Issue` step sends a literal
  `Tier` identity, and no `DefineTier` step precedes it.
- Every scenario arranged this way is a scenario the interpreter passes, because it never checks
  that a referenced row exists. Only a target that does will fail it.

## Design rule

An input a creating step sends into a field carrying a `references` relation names rows an earlier
step of the same scenario created, wherever the creating step sits: the branch under test, an
arrangement of its subject, an arrangement of a related row. The rule of
`docs/design/referenced-input-arrangement.md` applies to every creating step synthesis emits, not
only to the one a scenario is about.

## Acceptance

1. `cargo test -p ess-conformance --locked --test referenced_list_input
   adversary_a_server_arranged_for_a_later_command_names_arranged_sources` passes (the case is
   `#[ignore = "backlog: later-command-rows"]` today): the `Register` step of
   `demo.reg.Retire/outcome/retired` sends a non-empty `sources` whose every identity is the `id`
   of an earlier `execute_command demo.reg.AddSource` step.
2. Same target,
   `adversary_a_repository_seat_arranged_for_an_update_names_an_arranged_tier` passes: the `Issue`
   step of `demo.seats.ChangeTier/outcome/changed` sends a `tier_id` naming an instance an earlier
   `demo.seats.DefineTier` step captured.
3. The structural check of `every_referenced_identity_names_a_row_an_earlier_step_of_its_scenario_created`
   is extended to the `Retire` model and holds over every scenario of its suite.
4. The pinned tables move only in rows whose model has a creating command that writes a
   `references` field and a scenario arranging a row through it; each moved row is named with its
   cause.

## The case that shows it

`adversary_a_server_arranged_for_a_later_command_names_arranged_sources` and
`adversary_a_repository_seat_arranged_for_an_update_names_an_arranged_tier` in
`crates/verify/ess-conformance/tests/referenced_list_input.rs`. The first builds the reproduction
with a `Server` lifecycle `Live -> Retired` and a `Retire` move; the second reads the repository
model above.

## Notes for whoever picks it up

- The natural home is `invoke`/`invoke_created_with` for a `Creates` driver, so one place covers
  every arrangement. `referenced_input::row` already builds a referenced row through the creating
  branch with its own references arranged first (depth bound 3); a row arranged for a later
  command is the same operation.
- Distinctions: `referenced_input.rs` numbers its rows from `27_720 * 60` in blocks of 4096 per
  run; arrangements for a later command run at other distinctions and need a block of their own.
- The related-guard and row-set arrangements create their own rows through `related_guard::drive`
  and `row_set`; check them with the same structural assertion.
