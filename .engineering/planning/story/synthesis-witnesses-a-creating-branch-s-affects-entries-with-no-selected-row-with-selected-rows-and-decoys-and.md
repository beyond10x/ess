---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-creating-branch-s-affects-entries-with-no-selected-row-with-selected-rows-and-decoys-and
kind: story
status: draft
title: 'Synthesis witnesses a creating branch''s affects: entries with no selected row, with selected rows and decoys, and reads every row back'
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- derived_from: story:create-beside-a-move-on-the-same-key
- depends_on: story:a-branch-that-creates-its-subject-declares-affects-over-rows-that-existed-before-the-outcome-validated-and
- depends_on: story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal
revision: 1
---
## Need

After `g12-affects-beside-create`, a creating branch may declare `affects:`. Synthesis witnesses an
`affects:` entry on a moving, updating or deleting branch: it arranges three rows the filter
selects and one decoy per conjunct, each in a state the move starts from, sends the command, reads
every row back, and sends it again under a witness input that selects none (the zero-match call)
(`docs/design/set-effects-over-filtered-instances.md` § Conformance, 64-80); an entry pinned by
identity uses the pinned arrangement (`crates/verify/ess-conformance/src/synthesize/set_effects/pinned.rs`,
beyond10x/ess#547). Every one of these assumes an existing subject it arranges first. A creating
branch has none, so its scenario must arrange only the affected rows, send a fresh identity, and
read back the created row beside them.

## Outcome

On both reproductions of `g12-affects-beside-create`, `ess verify conform synthesize` writes the
creating branch's scenario with 0 refusals and the suite passes on the interpreted target:

- the zero-match case: the creation with no row selected (the first record on a key) creates only
  and publishes no event of the entry;
- the selected case: for a filter, three selected rows and one decoy per conjunct, as today; for a
  pinned entry, the one named row and a decoy of another identity; then the created row is read back
  in its initial state with what `sets:` wrote, the changed rows in their arrival state, the decoys
  unchanged;
- for an entry over the created row's own entity (the supersede shape), a scenario where a target
  that selects the row it just created (moves it too) fails;
- a target that applies the creation and drops the entry, or applies the entry to every row of the
  entity, fails a scenario.

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/set_effects.rs` (existing binary) gains
  `a_creating_branch_with_affects_is_witnessed_with_and_without_selected_rows`: one synthesis per
  reproduction, then the interpreted run. On the base (after `g12-affects-beside-create`) it fails
  with a refusal or a missing case; after, 0 refusals, every scenario passes, and the suite holds
  the cases above.
- A mutated interpreted target that also selects the created row fails the supersede scenario; one
  that drops the entry fails the selected case. Built as the existing set-effect mutation tests
  build theirs.
- An entry whose rows no declared command can create in a state the move starts from is refused by
  name (`RefusalCause::NoWitness`), the existing message for a moving branch reused, one test.
- Models without the pair keep their suite bytes: repository-wide byte pins unchanged
  (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`; `ess-synth` `go`,
  `served_codec_names`); `synthesis-sources.digest` re-stamped by the task.
- Every new test under 5 s or named with its time; one synthesis per test; no new test binary.

## Fit review

Part of the feature of `g12-affects-beside-create` (report item 12); its fit review applies. This
unit is the synthesis half. No suite major moves: no step vocabulary is added (the read-back and
count checks exist).

## Decisions

None of its own.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/set_effects.rs`,
  `crates/verify/ess-conformance/src/synthesize/set_effects/pinned.rs`,
  `crates/verify/ess-conformance/src/synthesize/set_effects/naming.rs` (the no-witness reasons,
  502)
- inferred: `crates/verify/ess-conformance/src/synthesize/made.rs` or the creating-branch
  identity code (`a-creating-branch-sends-every-row-it-creates-under-its-own-identity`), reused
- cited: `crates/verify/ess-conformance/tests/set_effects.rs` (existing binary)
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- Size: M (about 450 changed lines).
- Overlap: s4 of the wave of 2026-11-24 may change `synthesize/set_effects/pinned.rs`
  (conditional); this unit lands after s4.

## Depends on

- `g12-affects-beside-create`.
- `story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal`
  (s4, `pinned.rs`).

## Source

A downstream application, 2026-11-24, report item 12 (its G2).
