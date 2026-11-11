---
format: aep.planning-md/3
id: review-result:zi-t2-lane2-adv-1
kind: review-result
status: active
title: Turn 2 adversary pass 1 on story:an-absent-optional-under-false-and-unknown-is-arranged
relations:
- reviews: story:an-absent-optional-under-false-and-unknown-is-arranged
revision: 1
---
unit: story:an-absent-optional-under-false-and-unknown-is-arranged (commit e7d7777aeb), findings cover tree zi-t2-0 at acc5129912 plus one untracked test file
verdict: CONFIRMED (warning, beside the acceptance)
cases: executed 103→108, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 paths (part 6)
needs-coordinator: whether a refusal that changed from ESS-SYNTH-002 to ESS-SYNTH-003 needs a correction now or a backlog draft

**1. Diff.** `git --no-pager diff --stat` is empty because my only change is a new, untracked file. `git status --short`:
```
?? crates/verify/ess-conformance/tests/adversary_absent_optional_arrangement.rs
```
It is a test file. No implementation file was touched.

**2. Cases added** (`tests/adversary_absent_optional_arrangement.rs`). Each one checks that every emitted scenario passes on the interpreted target.
- `two_optional_inputs_each_read_by_a_refusal_are_arranged`: two Optional inputs, each read by its own refusal. Green.
- `two_optional_inputs_in_one_refusal_are_arranged`: two Optional inputs inside one `any`. Green.
- `a_member_of_an_optional_struct_is_arranged`: the guard reads `info.check` on an `Optional<Info>`. Green.
- `a_guarded_move_beside_an_optional_refusal_is_arranged`: the story's second adopter (Invoice, `Pay` moves the row, `when_subject tier != Basic`). Green.
- `a_row_only_a_left_out_optional_could_create_refuses_naming_it`: **red.** An `Erratum` row is refused for every `check` value, and every candidate that leaves `check` out is Unknown.

Red output from run 1, the new file alone (`build.sh adv/run1.txt <tree> test -p ess-conformance --locked --test adversary_absent_optional_arrangement`):
```
panicked at crates/verify/ess-conformance/tests/adversary_absent_optional_arrangement.rs:225:5:
assertion `left == right` failed: no candidate of the 1 tried satisfies ``example.review.Decision` stored kind selecting this branch, over the rows 1 bounded arrangements left`
  left: "ESS-SYNTH-003"
 right: "ESS-SYNTH-002"
exit 101
slot B waited 41s ran 121s
```
Run 1 had a sixth case, a guard on an undeclared member `info.vat`. It failed only because the compiler rejects the fixture (`unobservable_fact`), so it tested nothing and I deleted it.

**3. Suite run 2.** I ran the gate's targets plus my file: `adversary_absent_optional_arrangement`, `arranged_row_beside_absent_optional_guard`, `adversary_related_values_all_bytes`, `external_beside_held_guard`, `mutation_audit`, `synthesis`, `synthesis_suite_bytes_table` and `synthesis_suite_bytes_table_cli_read`, with `--no-fail-fast`. Output is in `adv/run2.txt`.
- My file: 4 passed, 1 failed (the same assertion at :225).
- Everything else passed: 1, 1, 17, 16, 65, 2 and 1.
- All four byte pins stayed unchanged.
- Run ended with `exit 101`, `slot B waited 339s ran 272s`.

The 103 "before" figure comes from `gate.txt`: the same seven targets, counting the byte table after its re-pin.

**4. Findings**

| file:line | verdict | origin | finding |
|---|---|---|---|
| crates/verify/ess-conformance/src/synthesize/subject_fact.rs:5206 | CONFIRMED | introduced | The left-out cause is returned only when no candidate built any row (`out.len() == 1`). If some candidate builds a row that does not serve the goal (here every `kind: Accept` row), the cause is dropped. `not-accepted` is then refused with ESS-SYNTH-003, which no longer names the left-out `check`. |

- **What was measured:** the assertion at test :225, exit 101, in run 1 and run 2.
- **Origin:** the base source at 93ec319d3e:5179 (`input_selects(...)?`) returned the first Unknown, which is ESS-SYNTH-002. I worked this out from the base code and did not run the base.
- **What reaches it:** any model where only a candidate that leaves the Optional out could create the row. I built this one; I found no repository model like it. Common shapes do get the input supplied (cases 1 to 4 are green).
- **Acceptance:** it sits beside acceptance bullet 3, which covers only "every candidate Unknown", so this is a backlog item rather than a blocker. It does contradict the commit message's "where no candidate serves the refusal is that first ESS-SYNTH-002".
- **Fix:** keep `left_out` when none of the rows the candidates built differ from the plain row in the fields the hints read, or let the ESS-SYNTH-003 that `search_rows` gives prefer a remembered fixable cause.

**5. Attacked and could not break**
- Two Optional inputs, in separate refusals and in one refusal: arranged and passing.
- A member of an Optional struct: arranged and passing.
- A move guarded on a stored field (the "toward" candidates): arranged and passing.
- Byte pins: unchanged.
- Looping or a silently dropped scenario: not found. `fixable_by_another_candidate` is true only for `ValueAbsent`, and the loop is bounded by the candidate list.
- An Unknown that is not fixable still returns at once: I could not cover this with a model the compiler accepts, because an undeclared path is rejected at compile time.

**6. Paths written outside the worktree**
- ~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/an-absent-optional-under-false-and-unknown-is-arranged/adv/invoice-part.rs
- ~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/an-absent-optional-under-false-and-unknown-is-arranged/adv/run1.txt
- ~/.cache/scratch/e368f6dc-c92b-425c-8fdf-280e82792de9/ess/waves/t2/an-absent-optional-under-false-and-unknown-is-arranged/adv/run2.txt

I used 2 of the 3 cargo runs and released the lease (`session-end`).

**7. Findings block**
```findings
- file: crates/verify/ess-conformance/src/synthesize/subject_fact.rs
  line: 5206
  category: boundary
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "when a candidate creates a row that does not serve the goal, the remembered left-out ESS-SYNTH-002 is dropped and the scenario is refused with ESS-SYNTH-003, which no longer names the Optional input"
```
