---
format: aep.planning-md/3
id: review-result:adversary-d368-u4-pass-1
kind: review-result
status: active
title: 'D368 U4 adversary pass 1: a row-set suite beside named rows'
relations:
- reviews: story:row-set-beside-named-rows-suite-tells-every-order-apart
revision: 1
---
unit: D368 U4 (story:row-set-beside-named-rows-suite-tells-every-order-apart), 3e665f021 on unit/ess-d368-u4, reviewed against 86d3d194d in tree ess-d368-u4-adv at 7e0d2e880
verdict: NEEDS-CHANGE (1 blocker, 1 warning); fixed at 7510ae01f (merged 21ef8a7fa)
cases: executed 0→12 (review file alone; the rerun was refused by the free-space precheck at 09:02:19Z), red 6 (2 findings, 2 out of scope, 2 invalid fixtures)
origin: introduced 0 / pre-existing 0 / undecided 3
wrote-outside-worktree: the wave scratch directory u4/adversary-scratch (cargo log only)
needs-coordinator: none

**Findings.**
- **F1** (blocker), `related_guard.rs:1235`.
  - With `when: input.from_port == input.to_port` on `sink-not-input`, `named_rows` arranged two
    separate ports, so the emitted `Connect/outcome/sink-not-input` scenario failed the unmodified
    interpreter.
  - Fixed at 7510ae01f: the send holds the `when:` (one row named twice), else the refusal is
    witnessed on its related row alone.
- **F2** (warning), `related_guard.rs:1350`.
  - With `when:` on both `source-not-output` and `sink-not-input` and the row set declared first,
    the order answering `source-not-output` before `source-taken` passed the whole suite.
    `Connect(p,p)` on an input port makes that pair reachable.
  - Fixed at 7510ae01f: the creator may be sent the linked row twice (`Others::Linked`).
- **Out of scope:** a selector with only `!=`, or an `any:` selector, beside named rows refuses
  every branch. That refusal is honest; the row is decided at wave close.

**Attacked, not broken.**
- A `forall` by the sink beside named rows.
- `exists: false` on the accepting branch.
- A `when:`-apart source refusal in the default order.
- The `synthesize.rs` skip, with a creator that never selects.
- Determinism.
- A fixed attempt list, so synthesis cannot loop.

**Fix-round run (7510ae01f).**
- Review file `adversary_d368_u4_pass1`: 9 passed and 8 failed before the fix, 17 passed after.
- lib 172, `identity_beside_row_set_2` 11, `synthesis` 65, the row-set targets and both bytes tables
  are green, with no bytes moved.
- clippy -D warnings and fmt-check pass.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":1235,"category":"acceptance","severity":"blocker","verdict":"NEEDS-CHANGE","origin":"undecided","message":"with when: input.from_port == input.to_port on sink-not-input, named_rows arranged two ports so the emitted sink-not-input scenario failed the interpreter; fixed at 7510ae01f"},
{"file":"crates/verify/ess-conformance/src/synthesize/related_guard.rs","line":1350,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"undecided","message":"with when: on both port refusals and the row set first, the order answering source-not-output before source-taken passed the whole suite; fixed at 7510ae01f (Others::Linked)"},
{"file":"crates/verify/ess-conformance/src/synthesize/row_set.rs","line":null,"category":"judgement","severity":"note","verdict":"INFEASIBLE","origin":"undecided","message":"out of scope: a row-set selector with no equality conjunct (!=-only or any:) beside named rows refuses every branch; honest, decided at wave close"}
]
```
