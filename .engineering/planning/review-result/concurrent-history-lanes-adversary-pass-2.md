---
format: aep.planning-md/2
id: review-result:concurrent-history-lanes-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:concurrent-history-lanes
relations:
- reviews: story:concurrent-history-lanes
revision: 1
---
# Adversary pass 2 — story:concurrent-history-lanes

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:concurrent-history-lanes, uncommitted working tree on impl/concurrent-history-lanes (base c589fddb0), including the pass-1 correction
verdict: NEEDS-CHANGE
cases: executed 1642→1644 (ess-conformance 883→884, ess-cli 759→760; each "before" is the after-run with my one file per crate deselected), red 2
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 logs in the unit scratch dir (listed in part 6); probe and test temp dirs created there were deleted
needs-coordinator: yes. I broke the disk rule once: the ess-cli suite started with 11G free, under the 12G floor. Clippy was not run on the two new test files because free space fell to 7G.

Cases: tests/lanes_adversary_pass2.rs — points of one subject stay in their order when calls share an
instant (RED: points 1–4 all at x=102); crates/edge/ess-cli/tests/conform_web_history_adversary.rs —
the largest admitted client count renders as it checks (RED: abort, 2^56-byte allocation).

Suites: ess-conformance 883 passed, 1 failed; ess-cli 759 passed, 1 failed (the two new).

Not broken: client ids ≥ clients refused by the reader; a read violation and a command conflict
cannot co-occur; byte stability across two processes; the pass-1 correction.

Coordinator routing: all three `introduced` → correction round 2. Decided: points at strictly
increasing x per subject; idle clients drawn individually up to a threshold and summarised above it;
`--out` keeps the shared family and its replacement behaviour is documented.

```findings
- file: crates/verify/ess-conformance/src/lanes.rs
  line: 432
  category: boundary
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "calls that share an instant, which history.rs admits for millisecond clocks, get linearization points clamped to one x (points 1-4 all at x=102), so the lanes no longer show the order found, contrary to lanes.rs's promise that each point is drawn after the one ordered before it"
- file: crates/verify/ess-conformance/src/lanes.rs
  line: 450
  category: boundary
  severity: warning
  verdict: INFEASIBLE
  origin: introduced
  message: "the pass-1 correction materializes 0..history.clients, so an admitted history declaring clients 9007199254740991 aborts web --history with a 2^56-byte allocation failure (signal 6) while check-history judges it with exit 0; no writer of such a count was found"
- file: crates/edge/ess-cli/src/main.rs
  line: 3696
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "web --history --out publishes under the scenario player's conformance-browser family, so pointing it at a player directory silently deletes the player's other artifacts (player.js, model.json, suite.json, README.md, assets/vue.*) with exit 0, and neither the guide nor the CLI row says --out replaces the directory"
```
