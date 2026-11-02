---
format: aep.planning-md/3
id: review-result:adversary-d368-u3-pass-1
kind: review-result
status: active
title: 'D368 U3 adversary pass 1: a refusal witness compares its trigger identities'
relations:
- reviews: story:a-refusal-witness-compares-its-trigger-identities
revision: 1
---
unit: D368 U3 (story:a-refusal-witness-compares-its-trigger-identities), 9bb5460d0 on unit/ess-d368-u3, reviewed against 86d3d194d in tree ess-d368-u3-adv at 7e0d2e880
verdict: NEEDS-CHANGE (1 blocker, 1 warning); fixed at 11d968fd6 (merged ede765cbe)
cases: executed 225→4 (review file alone; the suite run was refused by the free-space precheck at 09:02:20Z), red 1
origin: introduced 1 / pre-existing 1 / undecided 0
wrote-outside-worktree: the wave scratch directory u3/adversary-scratch (cargo log only)
needs-coordinator: none

**Findings.**
- **F1** (blocker, pre-existing), `synthesize.rs:9296`, called from `refusal_policy.rs:237`.
  - A refusal witness whose trigger is sent a fresh literal id (`related_missing`, an `exists: false`
    refusal) compared nothing in the trigger event. `give-on-touched/binding/refusal/missing`
    expected `{}`, so a target publishing a wrong `grant_id` passed on all five refusal fixtures.
  - Inside Acceptance bullet 1.
  - Fixed at 11d968fd6: `expect_sent_trigger_event` adds each literal sent for an identity-typed
    event field the branch copies unconverted.
- **F2** (warning, introduced), `tests/adversary_273_pass1.rs:400`.
  - The rewritten sweep counted only captured-instance values, so literal identities were
    invisible to it.
  - Fixed at 11d968fd6: the sweep counts identity-typed literals.
  - Fifteen non-refusal binding scenarios that still miss a literal id are pinned in
    `literal_trigger_id_residue()`. That is an out-of-scope row.

**Attacked, not broken.**
- A swapped-in publisher copying its id from an input under another name is compared.
- The unguarded-publisher path compares the instance id.
- A forced refusal on an event with two ids of two real rows compares both.

**Fix-round run (11d968fd6).**
- Red before the fix: `adversary_273_pass1` 7 passed and 2 failed; `adversary_d368_u3_pass1` 3 passed
  and 3 failed.
- Green after it: lib 172, `adversary_273_pass1` 9, `adversary_d368_u3_pass1` 6, `refusal_policy` 19,
  `bounded_retry` 13, and both bytes tables with seven models re-pinned.
- clippy -D warnings and fmt-check pass.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize.rs","line":9296,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"binding refusal witnesses whose trigger is sent a new literal id (related_missing) compared no identity of it, so a target publishing a wrong grant_id passed; fixed at 11d968fd6 (expect_sent_trigger_event)"},
{"file":"crates/verify/ess-conformance/tests/adversary_273_pass1.rs","line":400,"category":"judgement","severity":"warning","verdict":"CONFIRMED","origin":"introduced","message":"the rewritten sweep counted only captured-instance values; fixed at 11d968fd6, which counts identity-typed literals; 15 non-refusal binding misses pinned in literal_trigger_id_residue() are an out-of-scope row"}
]
```
