---
format: aep.planning-md/3
id: story:synthesis-witnesses-a-related-predicate-that-reads-the-addressed-row-on-rows-that-separate-the-held-state-from
kind: story
status: draft
title: Synthesis witnesses a related predicate that reads the addressed row, on rows that separate the held state from the related row
tags:
- gap-p2
- gap-reports-2026-11-24
relations:
- serves: vision:O2
- depends_on: story:a-when-related-predicate-reads-the-addressed-row-as-subject-field-and-subject-state-so-one-branch-is-selected
- depends_on: story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal
revision: 1
---
## Need

`g11-related-reads-subject` lets a `when_related` predicate read `subject.<field>` and
`subject.state`. The one-row related-guard strategy arranges the related row for a predicate over
that row and the input (`crates/verify/ess-conformance/src/synthesize/related_guard.rs`, with
`related_guard/stored.rs` for a stored-field `via`); it holds the addressed subject in whatever
state its own branch needs and does not steer it by the predicate. A branch whose predicate reads
the subject would get a witness only by accident, and a target that ignores the subject half (or
the related half) of the predicate would pass.

## Outcome

On the reproduction of `g11-related-reads-subject` (`held`, `recalled`, `loaded`, `wrong-state`
over `Shipment` and `Carrier`), `ess verify conform synthesize` writes a scenario for every branch
with 0 refusals, and the suite passes on the interpreted target. For each branch whose predicate
reads the subject, the suite holds:

- the witness: the subject in a state (or with a field value) the predicate needs and the related
  row satisfying its own part, the branch answering;
- a subject decoy: the same related row, the subject in a state the predicate refuses and the move
  still starts from, another branch answering (`held` vs `recalled` vs `loaded`);
- a related decoy: the subject as in the witness, the related row failing its part (`Active`), the
  next branch answering.

A target that drops `subject.state == …` from the predicate, or drops `state == Suspended`, fails a
scenario. Arranging the subject's held state uses the declared commands that reach it; a state no
declared command reaches is a named gap (`RefusalCause::NoWitness`), never a skipped half.

## Acceptance

- Red first: `crates/verify/ess-conformance/tests/interpreted_related_guards.rs` (existing binary)
  gains `a_related_predicate_reading_the_subjects_state_is_witnessed_with_both_decoys`: one
  synthesis of the reproduction, then the interpreted run; it fails on the base (after
  `g11-related-reads-subject`) with refusals or with a suite that lacks a decoy, and passes after:
  0 refusals, every scenario passes, and the suite holds the three scenarios per branch above.
- Two mutated interpreted targets (`subject.state` part dropped; `state` part dropped), built the
  way the existing mutation tests build them (`ess_conformance::mutate`), each fail at least one
  scenario. If `mutate` gains no operator for it, the test edits the IR of a second compile.
- A predicate reading `subject.<field>` (a stored field, not the state) is witnessed the same way,
  one test.
- A held state no declared command reaches is refused by name, one test.
- Models without a subject read in a related predicate keep their suite bytes: the repository-wide
  byte pins (`synthesis_suite_bytes_table`, `synthesis_suite_bytes_table_cli_read`,
  `adversary_related_values_all_bytes`, `external_beside_held_guard`; `ess-synth` `go`,
  `served_codec_names`, `adversary_related_guard_behaviour_pass1`) are unchanged;
  `synthesis-sources.digest` is re-stamped by the task.
- Every new test under 5 s or named with its time; one synthesis per test; no new test binary.
- `website/docs/reference/predicates.md`, the related guard's synthesis paragraph, names the two
  decoys.

## Fit review

Part of the feature of `g11-related-reads-subject` (report item 11); its fit review applies. This
unit is the synthesis half. No suite major moves: no step vocabulary is added.

## Decisions

None of its own.

## Scope

- cited: `crates/verify/ess-conformance/src/synthesize/related_guard.rs`,
  `crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs`
- inferred: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs` (steering the held
  state and stored fields of the subject, reused)
- inferred: `crates/verify/ess-conformance/src/synthesize/precedence.rs`
- cited: `crates/verify/ess-conformance/tests/interpreted_related_guards.rs` (existing binary)
- inferred: `crates/verify/ess-conformance/tests/fixtures/synthesis-sources.digest` (re-stamped)
- Size: M (about 400 changed lines).
- Overlap: `g10-list-beside-synthesis` also changes `related_guard.rs`; land one after the other
  (report order: g10 first).

## Depends on

- `g11-related-reads-subject`.
- `story:synthesis-witnesses-an-instance-and-an-each-in-that-name-a-member-of-a-struct-input-on-the-parent-s-minimal`
  (s4, shared synthesis files).

## Source

A downstream application, 2026-11-24, report item 11 (its G3).
