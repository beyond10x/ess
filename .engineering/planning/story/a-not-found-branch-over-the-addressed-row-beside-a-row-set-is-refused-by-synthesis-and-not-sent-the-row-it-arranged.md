---
format: aep.planning-md/3
id: story:a-not-found-branch-over-the-addressed-row-beside-a-row-set-is-refused-by-synthesis-and-not-sent-the-row-it-arranged
kind: story
status: draft
title: A not-found branch over the addressed row beside a row set is refused by synthesis as ESS-SYNTH-008 and not sent the row it arranged
summary: Synthesis wrote the not-found scenario of an addressed row beside a row-set guard by sending the arranged row, so the interpreted run failed it; the row-set path now refuses it as the related-guard path does
tags:
- defect
- issues-2026-11-26
relations:
- serves: vision:O2
- informed_by: story:an-identity-guard-beside-a-row-set-validates
- informed_by: story:an-unknown-identity-beside-a-related-guard-is-synthesized
revision: 1
---
## Need

A downstream application reported on 0.99.5 (received 2026-11-26) a conformance suite that fails against the interpreted target on a scenario synthesis wrote: a command with a not-found branch over the addressed row, beside a row-set guard. Synthesis wrote the branch's scenario by sending the not-found input the identity of the row it had just arranged, so the send named an existing row and the command answered its accepting branch instead.

Cause (`crates/verify/ess-conformance/src/synthesize/row_set.rs`, `prepare_with`): the missing row's input gets a fresh identity (`missing_named`, `ess/32`), then `subject_setup` arranges the subject that the sibling branches address through the same input field, and `supply()` binds that input to the arranged subject. The related-guard path already refuses this case (`related_guard.rs`, `arranged_at`: a name that is the subject's or is bound by the setup is `unarranged()`); the row-set path has no such check. Under the order of `docs/design/generic-language-features.md` § G4 a missing addressed row is answered before any related `exists: false`, so no arrangement reaches the branch: it must be refused, not written.

Reproduction, generic, on 0.99.5 and on the 0.99.7 preparation (`row_set.rs` identical): a `Shelf` and an `Item` row set, a command `CloseShelf` with a not-found branch over the shelf and a row-set guard on the items. Synthesis writes `CloseShelf/outcome/no-shelf`; the interpreted run passes 4 of 5 and fails that scenario (observed outcome `closed`).

## Outcome

Where the missing row's input is the one that addresses the subject, `prepare_with` refuses with the related-guard refusal, `ESS-SYNTH-008` (`arrange_related_row`), and no scenario is written for the branch. A suite never holds a scenario its own model answers differently.

## Acceptance

- The red test `related_guard::a_missing_row_branch_over_the_addressed_row_beside_a_row_set_is_not_sent_that_row` fails on 0.99.5 (the `no-shelf` scenario is written and its observed outcome is `closed`) and passes after the fix.
- On the reported model the interpreted run passes 4 of 4 (it passed 4 of 5), and the missing-row scenario is refused as `ESS-SYNTH-008`.
- A not-found branch over a row the input does not address keeps its scenario; no committed byte row moves.

## Fit review

1. **Need.** A suite written by synthesis must pass the model it was written from. Reproduction above.
2. **Class.** *Defect*: synthesis writes a scenario its own interpreter fails, and the sibling path (`related_guard.rs`, `arranged_at`) already refuses the same case.
3. **Already expressible?** No author action avoids it; the author drops the not-found branch or the row set.
4. **Fit.** The row-set path adopts the check the related-guard path has. The code is the existing `ESS-SYNTH-008`.
5. **Second adopter.** Any command that guards on a set of child rows and answers an unknown parent.
6. **Cost.** One refusal in synthesis, a scenario the suite no longer holds (each of them failed before); no format, no key.
7. **Considered.** (a) Change nothing. (b) Arrange the not-found scenario with an identity no row carries: contradicts the arrangement, since the siblings need the subject under that input. (c) Refuse (chosen).

## Decisions

- None open. Taken as a defect, a missing check adopted from the sibling path (the coordinator, 2026-11-26).
- D157: the fix goes into the open-issues wave, not into the patch release prepared from tag 0.99.6.

## Delivery

Branch `fix/g20b-a-missing-addressed-row-beside-a-row-set-is-refused` at `bf6b1ac226` (red `2200cc5fb6`, fix `bf6b1ac226`, from `main` at `c0b5f560cb`), pushed, merged into the open-issues wave; not released when written. Gate: stamp, `task fmt-check`, clippy, `related_guard`, `related_guard_moves`, the row-set and identity-beside-row-set adversary targets, the bytes table with its slow lanes, `cli_read`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `tutorial_page`, every step exit 0; no committed byte row moved. `tutorial_page` ran its one test in 60.48 s, already so before this change.

## Scope

Cited: `crates/verify/ess-conformance/src/synthesize/row_set.rs` (`prepare_with`), `crates/verify/ess-conformance/src/synthesize/related_guard.rs` (`arranged_at`, `unarranged`), `crates/verify/ess-conformance/tests/related_guard.rs`.
