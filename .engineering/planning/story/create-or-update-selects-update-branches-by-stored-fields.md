---
format: aep.planning-md/3
id: story:create-or-update-selects-update-branches-by-stored-fields
kind: story
status: draft
title: A create-or-update command selects its update branches by stored fields
revision: 1
---
## Outcome

A create-or-update command can select among its update branches with `when_subject:` predicates
over the stored row's fields and held state, while an `unknown_instance:` branch creates the row
for an identity no row carries, as `when_subject_state:` already allows.

Closes https://github.com/beyond10x/ess/issues/530.

## Fit review

1. **Need.** A command addressed by one identity creates the row when none exists and otherwise
   moves it; which move and which event depend on stored fields (a busy seat with a call reference
   emits `Left`, one without does not). Minimal reproduction: the `demo.desk.SignIn` command of the
   issue, ess/24: three `when_subject:` branches (`state == Busy and call_ref != ""`, ...) beside
   `unknown_instance: true` with `creates:`. Requester's syntax: the same document, unchanged; no
   new key is asked for.
2. **Class: gap.** The same create-or-update shape validates with `when_subject_state:`
   (`crates/specify/ess-domain/src/command/subject_state.rs:74,159` admit `UnknownInstance` beside
   held-state branches), and is refused with `when_subject:`:
   `crates/specify/ess-domain/src/command/subject_fact.rs:220-233` sets `shared = false` for any
   branch whose subject effect is `Creates` and pushes `conflicting_declaration` "subject fact
   selection requires one existing subject identity". Stored fields beside a creating branch are
   not expressible at all.
3. **Already expressible?** No. `when_subject_state:` reads the state only; the fields cannot be
   read. Splitting into two commands changes the protocol. Inferred from the two files above; no
   idiom found in `docs/design/cross-record-and-stored-field-guards.md`.
4. **Fit.** No new key. The creating branch answers at step 3 of the precedence order (existence,
   `docs/design/cross-record-and-stored-field-guards.md` "The precedence order"), before any
   step-4 `when_subject` branch reads the row, so the order already says what happens: the
   predicate branches are decided only on a stored row. The partition check
   (`validate_partition`, `subject_fact.rs:236`) runs over the existing-row branches only. Targets:
   interpreter, Rust, Go, TypeScript synthesis and Entity Runtime lowering each already handle
   `unknown_instance` + `creates:` beside `when_subject_state:`; each must accept or refuse by name
   the `when_subject` variant.
5. **Second adopter.** A shopping cart `AddItem` creates the cart on first use and otherwise
   updates it, emitting `CartReopened` only when the stored `checked_out_at` is set.
6. **Cost.** A relaxation of an existing refusal. Admitting at ess/24 would change what an ess/24
   document means (a refused document becomes valid), so by `story:format-rule-for-relaxations`
   it rides the next source format (`ess/25`). No new diagnostic; the refusal stays below ess/25.
7. **Considered.** (a) change nothing: the adopter keeps `when_subject_state:` and loses the field
   distinction; (b) relax only `subject_fact.rs` (chosen); (c) a dedicated upsert construct: new
   surface for a shape the order already covers. Requester's proposal taken unchanged.

## Decisions

Accept as proposed: no new key; the creating branch is admitted beside `when_subject:` branches
from `ess/25`.

## Acceptance

Each line names the failing test written first.

- `crates/specify/ess-domain/tests/subject_fact_create_or_update.rs`
  `create_or_update_with_stored_field_branches_validates_at_ess_25`: the issue's `SignIn` at
  ess/25 validates.
- same file, `create_or_update_with_stored_field_branches_is_refused_below_ess_25`: at ess/24 the
  existing refusal is unchanged, naming ess/25 in its hint.
- same file, `stored_field_branches_still_partition_existing_rows`: a gap in the predicates over
  stored rows is still refused by `validate_partition`.
- `crates/verify/ess-conformance/tests/synthesis_subject_fact_plan.rs`
  `create_or_update_synthesizes_creation_on_an_unknown_identity_and_each_stored_field_branch`:
  synthesis writes the `first` scenario on an identity nothing stores and one scenario per
  `when_subject` branch on an arranged row, and the interpreted target passes all of them.
- `crates/verify/ess-conformance/tests/subject_fact_complete_refusal.rs`
  `create_or_update_stored_field_runs_alike_on_rust_go_and_typescript`: the generated targets
  answer the suite alike, or the target refuses by name (`MissingRepresentation`).

## Scope

- cited: `crates/specify/ess-domain/src/command/subject_fact.rs:203-240` (the refusal and
  partition).
- inferred: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (arrangement of the
  creating branch), `crates/verify/ess-conformance/src/interpret/execute.rs` (step 3 before step 4),
  `crates/generate/ess-entity-runtime/src/subset.rs` (lowering accepts or refuses by name).
- format: `ess/25`, shared with every other ess/25 construct; no design page beyond a section in
  `docs/design/cross-record-and-stored-field-guards.md`.

## Plan

- depends_on: the unit that opens `ess/25` (first ess/25 story of the release).
- du estimate: 8G (ess-domain plus ess-cli and one `--test` of ess-conformance at a time).
- adversary: yes (relaxes a refusal on the precedence order).
- shared surfaces: `command/subject_fact.rs` and `interpret/execute.rs` with
  `story:refusal-above-held-state-branch-validates` (inferred); the two land one after the other
  in the wave, this one first (no new key).
