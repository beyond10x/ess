---
format: aep.planning-md/3
id: review-result:adversary-d343-u1-pass-1
kind: review-result
status: active
title: 'D343 U1 adversary pass 1: bound command input holds, coverage gaps closed'
relations:
- reviews: story:a-bound-command-input-decodes-the-published-request-body
revision: 1
---
unit: story:a-bound-command-input-decodes-the-published-request-body, worktree ess-d343-u1 at 7978d1b5d, plus 2 untracked adversary test files
verdict: CONFIRMED (test-suite gaps only; no implementation defect found, every new case green)
cases: executed 28→33, red 0
origin: introduced 3 / pre-existing 0 / undecided 1
wrote-outside-worktree: 6 (scratch logs, part 6)
needs-coordinator: none

**1. Diff.** `git diff --stat` empty; untracked only `crates/generate/ess-gen/tests/adversary_d343_u1_pass1.rs` and `crates/edge/ess-cli/tests/adversary_d343_u1_pass1.rs`.

**2. Cases, each run alone first.**
- ess-gen first run exit 101 on the adversary's own fixture: the form-encoded variant of A1 bound a command with a nested struct and the binding compile refused it (`ESS-HTTP-021 … has no flat spelling in a form request`). Fixture fixed only: a scalar-only command `demo.board.Rename` (title renamed on the wire `cardTitle`) carries the form check as A3; A2's expected roots include it.
- ess-gen rerun exit 0, 3 passed: A1 renamed path fields, nested and optional members, JSON body; A2 a command with no input fields refused for having none; A3 a form-encoded body.
- ess-cli exit 0, 2 passed: B1 `go test` decodes the published body and refuses `note_id`; B2 `tsc` accepts the published body and refuses `note_id`.

**3. Suite (exit 0 both).** `-p ess-gen --test bound_command_input_roots --test command_view_roots --test adversary_d343_u1_pass1`: 4 + 5 + 3 passed. `-p ess-cli --test model_type_events --test adversary_d343_u1_pass1`: 19 + 2 passed. Before count 28 is the same runs without the two adversary binaries.

**4. Findings.**
- F1 (in scope): `without_a_binding_every_root_keeps_its_bytes` cannot fail; it compares `select` with `select_bound(None)`, which `select` now delegates to. Unbound byte stability rests on the pre-existing tests.
- F2 (in scope): no unit case reached the "declares no input field" refusal; A2 covers it.
- F3 (in scope): the unit only searched generated Go and TypeScript source text; B1 and B2 show both decode the published body and refuse `note_id`.
- F4 (out of scope): under a binding the `.response` root is selected even when the bound document publishes no `{command}.Result`.

**5. Could not break**: bound input equals the published schema under JSON and form; renamed and path-carried fields leave the body; nested types and optional fields unchanged; field order kept; unpublished bodies refused and left out of `--all-commands`; Go and TypeScript decode the body and refuse path fields.

**6. Written outside the worktree**: six logs in the wave scratch directory `u1/adv-scratch/`; the ess-cli tests also wrote to the Go build cache and to temporary directories they delete.

```findings
[
  {"file": "crates/generate/ess-gen/tests/bound_command_input_roots.rs", "line": 248, "category": "judgement", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F1 in: without_a_binding_every_root_keeps_its_bytes compares select with select_bound(None), which select now delegates to, so it cannot fail; unbound byte stability rests on the unchanged pre-existing tests"},
  {"file": "crates/generate/ess-gen/src/model_types.rs", "line": 444, "category": "mutant", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F2 in: no unit case reaches the 'declares no input field' refusal, and the ess-gen reason check contains(\"path\") also matches the query message; adversary case A2 covers it and passes"},
  {"file": "crates/edge/ess-cli/tests/model_type_events.rs", "line": 732, "category": "acceptance", "severity": "note", "verdict": "CONFIRMED", "origin": "introduced", "message": "F3 in: the unit only searched the generated Go and TypeScript source text; adversary cases B1 (go test) and B2 (tsc) show both decode the published body and refuse note_id, green on 7978d1b5d"},
  {"file": "crates/generate/ess-gen/src/model_types.rs", "line": 73, "category": "judgement", "severity": "note", "verdict": "INFEASIBLE", "origin": "undecided", "message": "F4 out: under a binding whose outcomes answer response.<field>, error or none, the bound document publishes no {command}.Result, yet the .response root is selected unchanged; outside this unit's acceptance, not tested"}
]
```
