---
format: aep.planning-md/3
id: review-result:adversary-d368-u2-pass-1
kind: review-result
status: active
title: 'D368 U2 adversary pass 1: no-view refusals that tell states apart'
relations:
- reviews: story:a-no-view-held-state-refusal-probes-the-record
- reviews: story:a-no-view-probe-tells-from-states-apart
revision: 1
---
unit: D368 U2 (story:a-no-view-held-state-refusal-probes-the-record, story:a-no-view-probe-tells-from-states-apart), e266be529 merged as 550377957, reviewed in tree ess-d368-u2-adv at 0bd706d7d
verdict: NEEDS-CHANGE (2 warnings, 1 note); fixed at c5eddc4a2 (merged 7cdd82c68)
cases: executed 0→5, red 3
origin: introduced 3 / pre-existing 0 / undecided 0 (plus 1 out-of-scope row)
wrote-outside-worktree: the wave scratch directory u2/adversary-scratch (cargo logs)
needs-coordinator: none

**Findings.** All three are in the arranged half, under story 2's Acceptance: "or the scenario is
refused with a cause naming the states it cannot tell apart".
- **F1** (warning), `existence.rs:641`.
  - `best` is `None` both when no send is valid and when every valid send answers alike in every
    state. In the second case the scenario is refused as `UNPROBED` with an empty path.
  - Red case: `a1_withdrawal_where_every_send_is_valid_but_alike_names_the_states`.
- **F2** (warning), `existence.rs:654`.
  - `MAX_PROBE_SEARCHES = 64` stops the breadth-first search before depth 7: it would need 120
    tries.
  - The scenario is then withdrawn as `UNTOLD`, naming states one send tells apart.
  - Red case: `a2_seven_side_states_each_told_apart_by_one_send_keep_the_scenario`. Its six-state
    control passes.
- **F3** (note), `existence.rs:1110`.
  - `reads_plainly` excludes a command with a free-text `String` input where identities are plain
    `String`. The D357 swap case with `Stage(reason: String)` is then withdrawn as untellable.
  - Red case: `a3_a_free_text_field_beside_the_identity_does_not_disqualify_stage`. Its control
    passes.
- **Out of scope:** on the held-state half, where an identity view exists but
  `observe_selection_subject` fails for a held state, nothing reads the record back. Inferred from the
  code; the row is decided at wave close.

**Attacked, not broken.**
- Bytes: only `occurrence-clock-offers` moved.
- All named targets are green.
- Probe sends changing the record.
- The held-state guard.
- Determinism.
- The bounded search: `gate(7)` takes 0.14 s.
- `now` handling.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":641,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"best None conflates no valid send with valid sends that tell nothing apart; the latter is withdrawn as UNPROBED with an empty path instead of UNTOLD naming the states"},
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":654,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"MAX_PROBE_SEARCHES 64 caps the breadth-first search; with 7 states each told apart by one non-moving send the scenario is withdrawn as UNTOLD naming states one send tells apart; the cap is undocumented"},
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":1110,"category":"acceptance","severity":"note","verdict":"NEEDS-CHANGE","origin":"introduced","message":"reads_plainly treats an input whose type equals an entity identity type as naming another record, so a free-text String field excludes a command where identities are plain String and the D357 swap case is withdrawn as untellable"},
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":1275,"category":"coverage","severity":"note","verdict":"INFEASIBLE","origin":"pre-existing","message":"out of scope: on the held-state half, where an identity view exists but observe_selection_subject fails for a held state, nothing reads the record back; inferred from code, decided at wave close"}
]
```
