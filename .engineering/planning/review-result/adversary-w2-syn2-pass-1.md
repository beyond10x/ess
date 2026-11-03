---
format: aep.planning-md/3
id: review-result:adversary-w2-syn2-pass-1
kind: review-result
status: active
title: 'W2 SYN2 adversary pass 1: witnesses read back their outcome effects'
summary: APPROVE with 2 notes, 711 to 725 executed, 1 red; both notes introduced and sent to fix round 1
relations:
- reviews: story:view-asserts-field-set-from-optional-input
- reviews: story:every-witness-reads-back-its-outcome-effect
- reviews: story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists
- reviews: story:a-delete-may-have-a-no-op-branch-that-emits-nothing
revision: 1
---
unit: W2 SYN2 (story:view-asserts-field-set-from-optional-input, story:every-witness-reads-back-its-outcome-effect, story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists, story:a-delete-may-have-a-no-op-branch-that-emits-nothing), e0236a014f merged as a23b3e9e59, reviewed in tree ess-w2-syn2 at e0236a014f
verdict: APPROVE (2 notes)
cases: executed 711→725, red 1
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory syn2/adversary-scratch (cargo logs)
needs-coordinator: none

**F1 (introduced, note, sent to fix round 1).** From `ess/34` a `deletes:` is an admitted change of
a compensating refusal, but the MissingDeclaration refusal of a `compensates: true` branch declaring
no change (`crates/specify/ess-domain/src/command.rs:434`) and `docs/design/refusal-with-effect.md:50`
still name only `moves:`/`updates:`. Red case
`a_compensation_declaring_no_change_is_pointed_at_deletes_from_ess_34`. The lead rules it a drift
the unit introduced, fixed in the unit.

**F2 (introduced, note, sent to fix round 1).** The every-witness story's Acceptance names the case
`older_formats_keep_their_suite_bytes`, and no test has that name; the re-pinned bytes tables cover
the behaviour in practice.

**Attacked, not broken.** Input-guarded refusals beside held-state guards at ess/34 are read back
completely; at ess/33 neither refusal takes the new read-back; a compensating delete is refused from
ess/22 to ess/33 naming ess/34, refused at ess/21, admitted at ess/34 and refused beside `sets:`;
boundary rows of a guarded deleting branch and of one below `unknown_instance:`; the no-op read-back
catches a target that stores a row; a row-set `when_related:` at ess/34 passes on the interpreter;
all bytes tables green. The Go runner's agreement was not checked (no Go-runner target in the slot).
Peak target/ 8G, lowest / 133G.
