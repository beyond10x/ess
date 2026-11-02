---
format: aep.planning-md/3
id: review-result:security-d368-u2-pass-1
kind: review-result
status: active
title: 'D368 U2 security pass 1: no-view refusals that tell states apart'
relations:
- reviews: story:a-no-view-held-state-refusal-probes-the-record
- reviews: story:a-no-view-probe-tells-from-states-apart
revision: 1
---
unit: D368 U2 (story:a-no-view-held-state-refusal-probes-the-record, story:a-no-view-probe-tells-from-states-apart), e266be529 merged as 550377957, reviewed in tree ess-d368-u2-adv at 0bd706d7d
verdict: NEEDS-CHANGE (2 warnings); fixed at c5eddc4a2 (merged 7cdd82c68)
cases: executed 246→250, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: the wave scratch directory u2/security-scratch (cargo log)
needs-coordinator: none

**Findings.**
- **F1** (warning), `existence.rs:614` (`MAX_PROBE_SEARCHES = 64`).
  - The search expands at most 64 partial sequences, then reports `UNTOLD` for the best one.
  - On a 7-place model where each place is told apart by one send of its own, a 7-send sequence
    exists. Reaching it would take 120 expansions, so the scenario is withdrawn naming `P5, P6, P7`
    as states no send tells apart.
  - That contradicts the cause text and the "shortest sequence" promise in `outcome-shapes.md:286`
    and the CHANGELOG.
  - Red case: `untold_names_no_state_a_single_send_tells_apart`. It fails safe: no weak probe is
    filed.
- **F2** (warning), `existence.rs:641`.
  - When valid sends exist but none narrows the states, the scenario is refused as `UNPROBED` with
    an empty path that claims no valid send.
  - Story 2's Acceptance requires the refusal to name the states the probe cannot tell apart.
  - Red case: `a_valid_send_that_tells_nothing_apart_is_refused_naming_the_states`.

**Checked, not faulted.**
- Bounded: at most 64 searches, and the sends per resting state are cached. Memory is polynomial.
- Deterministic: on wide(40), two runs gave identical suite bytes (2.4 s each).
- Shortest below the cap: the search is breadth-first and returns at generation.
- The actor-parity filter holds.
- Models with a view keep their bytes.

```findings
[
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":614,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"the 64-expansion search cap stops before a 7-send sequence that exists, and the UNTOLD withdrawal names states one send each tells apart, against the cause text and the shortest-sequence promise"},
{"file":"crates/verify/ess-conformance/src/synthesize/existence.rs","line":641,"category":"acceptance","severity":"warning","verdict":"NEEDS-CHANGE","origin":"introduced","message":"where valid sends exist but none narrows the states, the scenario is refused as UNPROBED with an empty path claiming no valid send, instead of naming the states it cannot tell apart"}
]
```
