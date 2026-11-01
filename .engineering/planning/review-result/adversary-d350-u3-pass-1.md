---
format: aep.planning-md/3
id: review-result:adversary-d350-u3-pass-1
kind: review-result
status: active
title: 'D350 U3 adversary pass 1: the wrong-state witness for unknown and own stored guards'
relations:
- reviews: story:wrong-state-witness-unknown-and-own-stored-guards
revision: 1
---
unit: D350 U3, story:wrong-state-witness-unknown-and-own-stored-guards; covers a886ef467 on unit/ess-d350-u3 plus the adversary's uncommitted test file
verdict: CONFIRMED (two red cases, one cause)
cases: executed 0→4, red 2 (only `--test adversary_d350_u3_pass1` ran; the file is new, so the before count is 0)
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 path (the wave scratch directory u3/adv-scratch/cargo.log)
needs-coordinator: none

**1. Diff.** Only an untracked test file, `crates/verify/ess-conformance/tests/adversary_d350_u3_pass1.rs`. No implementation edits.

**2. Cases.** The test target answers as Entity Runtime does: guarded branches in declaration order, then the default, then `wrong_state`; three-valued guards, an Unknown stops selection. The model's `Place` sets `code: {generated: true}`; no command writes the optional field `note`; only `Rushed` and `Closed` are judged.
- `adversary_d350_u3_an_exempt_siblings_absent_field_masked_by_a_generated_field`: a moving default plus an exempt moving sibling guarded `any: [code == "zzz", note == "x"]`. Red.
- `adversary_d350_u3_the_own_guards_absent_field_masked_by_a_generated_field`: the moving branch's own guard is that `any`, the default moves too. Red.
- Two controls with the guard `note == "x"` alone: green, so the red comes from the generated leaf only.
- Output for `Rushed` (identical for `Closed`): `outcome demo.orders.Confirm/gone Failed ... expected: ["outcome = gone"], observed: ["no declared outcome was reached; the target refused for a reason the specification does not model"]`.

**3. Run** through the wave wrapper: `test -p ess-conformance --locked --test adversary_d350_u3_pass1`: 2 passed, 2 failed; peak target 3G, lowest / 83G, rc 101.

**4. Findings.**
- F1 (CONFIRMED, introduced, warning), `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:3323`, the `absent` closure in `selection`: a guard counts as absent only if it is Unknown read plainly and False read as a guard, but `evaluate_row` (:593-596) returns Unknown before any absence check whenever a read field has no literal value, as a creator's generated value has. A guard that also reads an empty optional field is then taken as an Unknown the arrangement caused; the exemption test and the own-guard test (`Truth::Unknown => !moves_on || by_absence`) accept the row, the scenario is written, and a correct target fails it on the empty `note`. The design text at `docs/design/cross-record-and-stored-field-guards.md:327` ("either way the move answers") and the comment in `refusal_input_from` are false for this case. Suggested fix: count a guard as stopping selection whenever it reads a field the row leaves empty (`arrangement.unwritten`) unless the known leaves decide it. Reach: any model whose guard reads a creator-generated field and an optional field nothing fills; no committed model found (search not exhaustive).
- F2 (CONFIRMED, introduced, note), `crates/verify/ess-conformance/tests/adversary_mixed_guard_pass2.rs:623`: the port of case 3 dropped the archived `assert!(!results.is_empty())`; the port documents why (no correct synthesis can satisfy it on that model) and checks the acceptance's ESS-SYNTH-003 clause instead.

**5. Attacked, not broken (reasoned, not run).** An own guard False beside a moving default or a true exempt sibling; a later sibling skipped once `outcome` is selected first; rows from the search fallback and the held-state overlap sends going through `selection`; the retry path's first refusal; the own guard not observed before the command. Outside the acceptance, not filed: an earlier exempt sibling True and a later one Unknown is refused though a target would pick the earlier one (too cautious, not wrong).

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/subject_fact.rs","line":3323,"category":"acceptance","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"A stored guard that reads a creator-generated field next to an optional field no creator fills is never counted as absent, because evaluate_row returns Unknown before any absence check; the wrong-state witness is then written to a row where an exempt sibling's guard, or the moving branch's own guard, is Unknown, and an Entity-Runtime-ordered target fails it (adversary_d350_u3_pass1, two red cases, controls green)."},
{"file":"crates/verify/ess-conformance/tests/adversary_mixed_guard_pass2.rs","line":623,"category":"acceptance","severity":"note","verdict":"CONFIRMED","origin":"introduced","message":"The ported case 3 dropped the archived assert!(!results.is_empty()), so the case as archived does not pass as the acceptance asks; the port documents why (no correct synthesis can satisfy it on that model) and checks the ESS-SYNTH-003 clause instead."}
]
```
