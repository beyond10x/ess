---
format: aep.planning-md/3
id: review-result:security-d368-u4-pass-1
kind: review-result
status: active
title: 'D368 U4 security pass 1: a row-set suite beside named rows'
relations:
- reviews: story:row-set-beside-named-rows-suite-tells-every-order-apart
revision: 1
---
unit: D368 U4 (story:row-set-beside-named-rows-suite-tells-every-order-apart), 3e665f021 on unit/ess-d368-u4, reviewed against 86d3d194d in tree ess-d368-u4-adv at 7e0d2e880
verdict: NEEDS-CHANGE (2 blockers, 1 note); fixed at 7510ae01f (merged 21ef8a7fa)
cases: executed 0→6 (review file alone; the rerun was refused by the free-space precheck at 09:02:19Z), red 2 (+4 fixture typos)
origin: introduced 0 / pre-existing 0 / undecided 3
wrote-outside-worktree: the wave scratch directory u4/security-scratch (cargo log, a backup of the test file)
needs-coordinator: none

**Findings.**
- **F1** (blocker), `related_guard.rs:1235`.
  - A named-row refusal beside a row set carrying `when: input.strict == false` got a send that did
    not hold the `when:`, so the emitted `sink-not-input` and `source-not-output` scenarios failed
    the interpreter.
  - Fixed at 7510ae01f.
- **F2** (blocker), `row_set.rs:3192`.
  - The `Ok(false)` fallback never tried `SEND_ROWS`. With `forall: to_port == input.to_port`, the
    accepting branch was witnessed only on an empty row set, and a target refusing on any wire from
    the source passed all 7 scenarios.
  - Fixed at 7510ae01f: an arrangement selecting no row is the last resort.
- **F3** (note, INFEASIBLE), `related_guard.rs:770`.
  - The `named_refusal` doc claimed a `when:` for missing rows, which the validator forbids.
  - Doc corrected at 7510ae01f.

**Checked, not faulted.**
- Bounded work: the extra `prepare_with` calls are capped by `ARRANGEMENTS = 4096`.
- `named_refusal` has a single caller.
- The `by_rows` skip, with the creator-beside-input-refusal case. That case led to
  `refutes_whens_first` in the fix round.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":1235,"category":"integrity","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"undecided","message":"a named-row refusal beside a row set carrying when: input.strict == false got a send that did not hold the when, so the emitted scenarios failed the interpreter; fixed at 7510ae01f"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":3192,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"undecided","message":"the Ok(false) fallback never tried SEND_ROWS, so with forall to_port == input.to_port a target refusing on any wire passed all 7 scenarios; fixed at 7510ae01f"},
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":770,"category":"contract-drift","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"the named_refusal doc claimed a when for missing rows, which the validator forbids; doc corrected at 7510ae01f"}
]
```
