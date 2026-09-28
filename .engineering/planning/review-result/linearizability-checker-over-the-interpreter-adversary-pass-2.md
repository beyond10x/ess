---
format: aep.planning-md/3
id: review-result:linearizability-checker-over-the-interpreter-adversary-pass-2
kind: review-result
status: active
title: Adversary pass 2, story:linearizability-checker-over-the-interpreter
relations:
- reviews: story:linearizability-checker-over-the-interpreter
revision: 1
---
# Adversary pass 2 — story:linearizability-checker-over-the-interpreter

Dispatched as `aep:adversary` after correction round 1, 2026-09-28. Header and findings verbatim.

unit: story:linearizability-checker-over-the-interpreter, uncommitted working tree on base 540d47ad6 (branch impl/linearizability-checker-over-the-interpreter)
verdict: CONFIRMED (3 red cases)
cases: executed 1601→1609 (1601 is derived: the 1609 run minus my 8 new cases; no separate before-run), red 3
origin: introduced 3 / pre-existing 0 / undecided 0
wrote-outside-worktree: 2 logs and about 2,830 test temp dirs (2.1G) in the assigned scratch, all deleted; scratch is now empty
needs-coordinator: none. Disk: `df -h /` showed 43G free (95%) before the suite run and 41G free after cleanup.

Cases: tests/linearizability_adversary_pass2.rs (every lost-update race shrinks to 2 clients / 4
operations — RED at seeds 16 and 19; identity and budget cases green) and
crates/edge/ess-cli/tests/check_history_adversary.rs (missing history file and missing spec path
exit 2 — both RED, exit 1; report round-trip and key set green).

Suite: ess-conformance and ess-cli with --no-fail-fast: 1606 passed, 3 failed (the three new), 6
ignored (pre-existing).

Coordinator routing: findings 1–3 `introduced` → correction round 2. Finding 4 is marked
`pre-existing` (the wave-1 format records no inputs); the doc sentence is corrected in this unit and
the format gap is filed as its own story.

```findings
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 556
  category: property
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "The shrinker keeps a removal only when every operation of the original first-found longest linearization is still ordered, which depends on search order, so a three-client lost update (seeds 16 and 19) stays at 3 clients and 5 operations although create, issue and the two settled payments are a 4-operation violation."
- file: crates/edge/ess-cli/src/main.rs
  line: 3038
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "An unreadable --history file fails through `?` and exits 1, the Violation code, where the subcommand's help promises exit 2 for a refused history."
- file: crates/edge/ess-cli/src/main.rs
  line: 3035
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "A --path that does not exist fails through `resolved(...)?` and exits 1, the Violation code, where the help promises exit 2 when the specification did not load."
- file: crates/verify/ess-conformance/src/linearize.rs
  line: 42
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: pre-existing
  message: "The module doc says the check is sound in one direction only, but because ess-history/1 records no inputs a Linearizable verdict can also be wrong for any input-dependent fault; inferred from the code, not run."
```
