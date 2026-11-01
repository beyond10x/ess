---
format: aep.planning-md/3
id: review-result:adversary-d343-u3-pass-1
kind: review-result
status: active
title: 'D343 U3 adversary pass 1: compound identity guards and boundary runs below unknown_instance'
relations:
- reviews: story:an-identity-guard-below-unknown-instance-is-decided-on-the-row-sent
revision: 1
---
unit: D343 U3, `unit/ess-d343-u3` at ef2cc60f1, plus the untracked `crates/verify/ess-conformance/tests/adversary_d343_u3_pass1.rs`
verdict: NEEDS-CHANGE
cases: executed 23→32, red 3
origin: introduced 1 / pre-existing 0 / undecided 3
wrote-outside-worktree: 5 paths, all in the wave scratch directory `u3/adv-scratch/`
needs-coordinator: none

Run window 09:42:20–09:44:21 UTC; `/` lowest 82G; final `target/` 765M; every pre-check held (80G, 15G, at most one other cargo). Worktree diff empty; nothing committed.

**1. Fixture fix.** The first run had one invalid model: a deleting branch guarded on its own identity is rejected by the compiler (`[unobservable_fact] … an identity is opaque, so no branch may be chosen by reading one`), so the "subject branch's own guard" case is a shape no valid model has; removed. The compound and Integer cases now check the witness before the all-pass assertion.

**2. Cases (`--test adversary_d343_u3_pass1`, run alone): 9 run, 3 red.**
- `a_compound_identity_guard_below_unknown_is_sent_the_row_of_that_identity` (:385) red: `[ESS-SYNTH-003] no candidate of the 1 tried satisfies `demo.notes.Note` stored row, its identity decided on the row sent under `(note_id == keep and not (defined(why)))`, selecting this branch, over the rows 1 bounded arrangements left`.
- `a_compound_enum_identity_guard_below_unknown_…` (:395) red the same way, naming `(note_id == b and not (defined(why)))`. In the first run both compound models also had `closed` failing on the target (`note_id = "keep", why = "why"` expecting `outcome = closed`, observing `outcome = unknown`; the same for `b`).
- `an_integer_identity_range_below_unknown_is_sent_a_row_inside_it` (:429) red: `dropped` is witnessed on a note above 5, but `closed` fails: `demo.notes.CloseNote(note_id = 5.0)` expects `outcome = closed`, observes `outcome = unknown`.
- Green: enum membership; the refusal arm where no row can meet the guard; the moved related row with enum and with Integer identity; the String guard held to its control; determinism.

**3. Suite run (`--test adversary_u11_batch3_pass1`):** `test result: ok. 23 passed; 0 failed`.

**4. Findings**
- F1 (in scope, introduced): the row's creator is steered only by guards that read the identity input alone (`onto_identity`, called at `synthesize.rs:5571`); a compound guard that also reads another input gets no hint, so the search tries one row and refuses although a note `"keep"` (or `b`) can be opened. Reached by any accepting branch below `unknown_instance:` with such a compound guard. Fix: use the identity parts of a compound guard as hints.
- F4 (in scope, origin undecided): the `boundaries` family (`synthesize.rs:17552`, `boundary_inputs`) sends the boundary runs of an identity guard as literal identities without arranging a row (the neighbour 5 of `note_id > 5`, the per-part runs of an `all:` guard). Below `unknown_instance:` the target answers `unknown`, so the written `closed` scenario fails. The diff does not touch this family; the base was not run.
- F2 (out of scope, judgement, inferred): in a related-boundary run the first answering branch wins (`synthesize.rs:4068`); under `kind == "x"` that is `refused-b` on slot `"x"`, isolating no conjunct.
- F3 (out of scope): the stored-field path `prepare_state_input` (`synthesize.rs:5858`) and `addressed_row` (`synthesize.rs:5413`) do not decide an identity guard on the row sent.

**5. Attacked and not broken:** Integer, enum and membership guards on the identity alone (the `dropped` witness); the ESS-SYNTH-003 refusal where no row exists; the moved related row with enum and Integer identities; no "creates a second row" refusal anywhere; determinism.

**6. Paths outside the worktree:** `a1.log`, `a2.log`, `b1.log`, `low1.txt`, `watch.sh` in the wave scratch directory.

```findings
[
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5571, "category": "acceptance", "severity": "blocker", "verdict": "CONFIRMED", "origin": "introduced", "message": "F1 in: identity_steered hints only from guards reading the identity input alone, so a compound guard such as {all: [note_id == \"keep\", not defined(why)]} is refused ESS-SYNTH-003 where a note \"keep\" meets it (adversary_d343_u3_pass1.rs:385, :395 red)"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 17552, "category": "acceptance", "severity": "blocker", "verdict": "CONFIRMED", "origin": "undecided", "message": "F4 in: boundary runs of an identity guard below unknown_instance (note_id = 5.0 for note_id > 5, per-part runs of an all: guard) send a literal identity with no row arranged, so the written closed scenario fails with outcome = unknown (adversary_d343_u3_pass1.rs:429 red)"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 4068, "category": "mutant", "severity": "note", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "F2 out: the related-boundary run for a conjunct takes the first answering branch, so under an identity guard refused-b answers it on the guarded identity and it isolates nothing (inferred from the pre-fix probe log and code reading)"},
  {"file": "crates/verify/ess-conformance/src/synthesize.rs", "line": 5858, "category": "judgement", "severity": "note", "verdict": "NEEDS-CHANGE", "origin": "undecided", "message": "F3 out: an identity guard is not decided on the row sent on the stored-field path prepare_state_input or on addressed_row (synthesize.rs:5413), which passes no input"}
]
```
