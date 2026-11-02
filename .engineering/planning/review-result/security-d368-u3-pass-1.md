---
format: aep.planning-md/3
id: review-result:security-d368-u3-pass-1
kind: review-result
status: active
title: 'D368 U3 security pass 1: a refusal witness compares its trigger identities'
relations:
- reviews: story:a-refusal-witness-compares-its-trigger-identities
revision: 1
---
unit: D368 U3 (story:a-refusal-witness-compares-its-trigger-identities), 9bb5460d0 on unit/ess-d368-u3, reviewed against 86d3d194d in tree ess-d368-u3-adv at 7e0d2e880
verdict: NEEDS-CHANGE (1 blocker, 2 warnings); fixed at 11d968fd6 (merged ede765cbe)
cases: executed 225→228, red 3
origin: introduced 0 / pre-existing 3 / undecided 0
wrote-outside-worktree: the wave scratch directory u3/security-scratch (cargo log only)
needs-coordinator: none

**Findings.**
- **F1** (blocker, pre-existing), `refusal_policy.rs:237`.
  - The missing-row witness never compared the fresh literal id in its trigger event.
  - Neither a wrong id nor another record's id failed `missing` or `grant-missing`, on any of the
    five fixtures.
  - The unit's CHANGELOG line claimed this path.
  - Fixed at 11d968fd6.
- **F2** (warning), `tests/adversary_273_pass1.rs:400`: the identity sweep counted only `Instance`
  values. Fixed at 11d968fd6.
- **F3** (warning), `tests/adversary_273_pass1.rs:1066`: the wrong-id case passed when any one
  witness per fixture failed. Fixed at 11d968fd6: every refusal witness must fail a wrong id in each
  identity field it copies.

**Cases ported.** Two security cases were ported into `tests/adversary_d368_u3_pass1.rs`:
- `a_missing_row_witness_compares_the_id_its_trigger_was_sent`;
- `a_refusal_witness_fails_a_target_publishing_another_records_id`.

Its third case duplicated the adversary's case 1.

**Checked, not faulted.**
- `Instance` identities are compared on the forced, swapped-publisher, alternative-trigger and
  arranged-row paths.
- `sent_by` follows the swapped publisher's branch.
- The sweep was not vacuous for the `Instance` kind.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/refusal_policy.rs","line":237,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"pre-existing","message":"a refusal witness that sends its trigger naming no related row never compared that fresh literal id, so a wrong or another record's id passed the missing and grant-missing witnesses; fixed at 11d968fd6"},
{"file":"crates/verify/ess-conformance/tests/adversary_273_pass1.rs","line":400,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"the identity sweep only counted Instance-valued inputs; fixed at 11d968fd6"},
{"file":"crates/verify/ess-conformance/tests/adversary_273_pass1.rs","line":1066,"category":"mutant","severity":"warning","verdict":"CONFIRMED","origin":"pre-existing","message":"the refusal wrong-id case passed when any one witness per fixture failed; fixed at 11d968fd6, every witness must fail"}
]
```
