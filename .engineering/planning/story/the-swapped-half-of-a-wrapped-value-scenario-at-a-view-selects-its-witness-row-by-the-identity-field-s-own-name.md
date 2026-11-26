---
format: aep.planning-md/3
id: story:the-swapped-half-of-a-wrapped-value-scenario-at-a-view-selects-its-witness-row-by-the-identity-field-s-own-name
kind: story
status: draft
title: The swapped half of a wrapped-value scenario at a view selects its witness row by the identity field's own name
summary: 'Two synthesized invariant-wrapped scenarios at a view fail on their own model''s interpreter (851 others pass): the caller-swap redraw rewrites the identity name inside a value_invariants witness selector; red 0a4db7255e, fix ad49428e1c'
tags:
- conformance
- defect
- external-request
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- informed_by: story:a-swapped-caller-run-renames-only-the-names-it-binds
- informed_by: story:a-caller-swap-redraw-keeps-unread-members-and-related-ties
revision: 4
---
## Finding

A downstream application's conformance suite, synthesized by 0.99.1, holds two scenarios of the kind
`invariant-wrapped` at a view that fail on the declared interpreter of their own model; 851 other
scenarios of the suite pass. Both fail at the second view check of the scenario, the check of the
swapped half: `no selected row holds an actual occurrence ...`. The first half of each scenario
passes. The model holds two newtypes over a string with a length bound; the failing scenarios are
the ones that witness those bounds through a view.

## Verdict

Synthesis. The interpreter is right and the suite is wrong. The swapped half's query step is right;
its `expect_view` names the wrong row. The `value_invariants` witness selector (`projected_identity`)
carries the identity field's declaration, `{name: <identity>, type}`. The caller-swap redraw
(`redraw` and `redraw_under` in `crates/verify/ess-conformance/src/synthesize/caller.rs`, called
where the swapped run is built) rewrites every string under a key in `Identities.keys` that equals a
drawn identity. The first half sent the identity as the literal of its own field name, and `name` is
in `keys` when the model has a String-typed identity input (every String field name is then a key),
so `identity.name` becomes `<identity>-<n>`. The runner then selects rows by a column of that name,
finds none, and fails.

## Reduction

A model with a String-typed identity input on the parent entity; a child entity whose identity is a
newtype over a string (`shelf_id`) minted from a create input through a conversion; a list of
structs whose member is a newtype over a string with a length bound (at most 256); a create guard
refusing a list of more than 50 items, which is what gives the scenario a swapped half; an update
command; a view selected by the parent identity and the child identity; one caller attribute on the
actor. Of seven scenarios, the `invariant-wrapped` one fails, and its swapped selector names an
identity field `shelf_id-525248`. The same model without the count guard writes no swapped half and
passes; the same model with a newtype for the parent identity passes, because `name` is then not a
key.

## Since

0.63.0: `e81131c92d` serialized the selector into the expectation. The redraw is older (`8aaa71961e`,
0.49.0) and did not meet a selector before.

## Red test

`a_swapped_half_selects_its_witness_row_by_the_identity_field_and_passes` in the target
`wrapped_value_invariants` of `crates/verify/ess-conformance`, commit `0a4db7255e` on the branch
`fix/n91-an-invariant-wrapped-scenario-at-a-view-passes-its-own-model`. It fails at
`wrapped_value_invariants.rs:949` with the selected identity fields `[shelf_id, shelf_id-525248]`
(the target ran 19 tests in 8.6 s).

## Fix

Commit `ad49428e1c` on the same branch, head `ad49428e1cda258780a5e3fc3eb320f5eed9c29b`, pushed:
`redraw_under` leaves the names of a `value_invariants` expectation alone (root, position,
declarations, selector entity and identity) and redraws only the selector's `expected` values. The
commit carries the `CHANGELOG.md` entry, the stamp and the raw-string form of the test's model
constant (a Clippy refusal of the first form).

## Gate

First run: Clippy refused the first form of the test constant (`needless_raw_string_hashes`, fixed);
`wrapped_value_invariants_runtimes` failed with `TS2688` because `ESS_TYPES_NODE` was unset (the
environment, not the change); every other step exit 0: stamp, `task fmt-check`,
`wrapped_value_invariants`, the bytes table with its slow models and slow components lanes,
`cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`,
`caller_struct_identity`, `caller_swap_authored_field_names`,
`a_caller_swap_redraw_keeps_unread_members_and_related_ties`, `tutorial_page`; no byte row moved.
Second run: stamp, `task fmt-check`, `clippy -p ess-conformance`, `wrapped_value_invariants`
(19 passed, 17 s) and `wrapped_value_invariants_runtimes` with `ESS_TYPES_NODE` set (4 passed, 8 s),
every step exit 0; no byte row moved.

## Confirmation

The reported suite, as written by 0.99.1, run with the 0.99.4 binary through `--format json`: both scenarios fail at their second view check (the first
passed, the swapped failed) with `no selected row holds an actual occurrence ...`, and the swapped
selectors name identity fields with a numeric suffix. That is the verdict's cause, seen on the
reported suite.

## Acceptance

- The red test is green on the fix and red without it.
- On the reduction's model every `invariant-wrapped` scenario passes its own model's interpreter.
- A suite synthesized from the reported model again, with the fix, passes both scenarios; the other
  851 keep their result.
- No committed byte table moves.

## Reason not shown

The text run did not say why either scenario failed: see
`story:the-text-output-of-a-failed-scenario-names-the-failed-check`, which holds the second report.

## Fit review

1. **Class.** A defect of synthesis: a scenario that its own model's interpreter refuses, from the
   caller-swap redraw rewriting a name that is data of the selector.
2. **Release.** A patch release, 0.99.5 (see Decisions).
3. **Borders.** `story:a-swapped-caller-run-renames-only-the-names-it-binds` (the rename of names a
   swapped run binds; this one is a redraw of a name it must not touch) and
   `story:a-caller-swap-redraw-keeps-unread-members-and-related-ties`.
4. **Evidence form.** Times are measured on one node, debug test builds, the target alone.

## Decisions

- None open. Ships as 0.99.5 with a full gate of its own (the operator session, 2026-11-25, D144
  part 1 = B), because the build node was down when 0.99.4 was tagged.

## Delivery

Carried by the branch `hotfix/0.99.5`, in preparation on 2026-11-25 when this was written.

0.99.5 is released (2026-11-25 23:23 CET): tag on `3cdf39f157` (branch `hotfix/0.99.5`, the release this story's fix was planned for); the tag's release run 45682187587 took the full gate; the release is public at https://github.com/beyond10x/ess/releases/tag/0.99.5. Pull request 837 brings 0.99.5 to `main`; it was open when this was written.

Pull request 837 brings 0.99.5 to `main`, open when written.
