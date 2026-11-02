---
format: aep.planning-md/3
id: review-result:security-d368-u1-pass-1
kind: review-result
status: active
title: 'D368 U1 security pass 1: an optional element appended as null'
relations:
- reviews: story:an-optional-element-is-appended-or-refused-consistently
revision: 1
---
unit: D368 U1 (story:an-optional-element-is-appended-or-refused-consistently), 295d0b617 on unit/ess-d368-u1, reviewed against 86d3d194d in tree ess-d368-u1-adv at 7e0d2e880
verdict: APPROVE against the Acceptance (no in-scope finding); 1 out-of-scope row
cases: executed 0→8 (review file alone; the suite run was refused by the free-space precheck at 09:02:15Z), red 1
origin: introduced 0 / pre-existing 0 / undecided 1
wrote-outside-worktree: the wave scratch directory u1/security-scratch (cargo log only)
needs-coordinator: none

- **Cases.** `tests/security_d368_u1_pass1.rs`, uncommitted review file. Seven were green:
  - crafted elements of another type are refused, and the list is unchanged;
  - a stored list of another element type is not extended by a null;
  - `Optional<List<Optional<T>>>` held absent appends one null;
  - the domain refuses the four other pairings;
  - a null is appended through an absent Optional struct path;
  - a target that appends nothing fails the suite;
  - a 10,000-element list and an element nested 1,000 deep neither panic nor append an untyped
    value.
- **One red case**, outside the Acceptance: a target that always appends null passes, because no
  scenario sends a present element.
- **Invariants held** (from reading the code):
  - typed values only: determined.rs:550 checks the whole list against its type, and null is
    produced only for an Optional element;
  - the view-row comparison tells null, the text "null" and a missing element apart;
  - nothing is unbounded.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":10542,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"undecided","message":"out-of-scope (outside Acceptance): the synthesized suite for an append over List<Optional<T>> only sends the element absent, so a target that always appends null passes every scenario (red: security_d368_u1_pass1.rs:371); duplicate of the adversary row"}
]
```
