---
format: aep.planning-md/3
id: review-result:concurrent-history-lanes-adversary-pass-1
kind: review-result
status: active
title: Adversary pass 1, story:concurrent-history-lanes
relations:
- reviews: story:concurrent-history-lanes
revision: 1
---
# Adversary pass 1 — story:concurrent-history-lanes

Dispatched as `aep:adversary` against `ess-chc-lanes` (uncommitted tree on
`impl/concurrent-history-lanes`, base `c589fddb0`), 2026-09-28. Header and findings verbatim.

unit: story:concurrent-history-lanes, uncommitted working tree on impl/concurrent-history-lanes (base c589fddb0)
verdict: NEEDS-CHANGE
cases: executed 881→883, red 2
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: 1 (scratch log), plus temp dirs that were created and then deleted
needs-coordinator: yes. Clippy was not run on the new test file because / is at 7G. I also broke the disk rule once (see part 3).

Cases (tests/lanes_adversary.rs), both RED: a client that made no calls still has its lane (2 lanes
drawn for clients: 3); a read violation marks the read that decided it (no call marked).

Suite: `cargo test -p ess-conformance --locked --no-fail-fast` EXIT=101, 881 passed, 2 failed (the
two new). The run started with 8G free, below the 12G floor.

Not broken: escaping in text, attributes and SVG title; no external fetch; determinism; Indeterminate
drawn to its AfterEveryOther bound; conflict() cannot change a verdict; no false conflict pair found;
exit 0 after rendering.

Coordinator routing: both `introduced` → same implementor.

```findings
- file: crates/verify/ess-conformance/src/lanes.rs
  line: 469
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "lanes are drawn only for clients that appear on an operation, so an admitted history with clients: 3 and calls from two clients gets 2 lanes under a header saying 3 client(s), against the outcome's one lane per client"
- file: crates/verify/ess-conformance/src/lanes.rs
  line: 252
  category: acceptance
  severity: warning
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a read violation (StaleReadUnderReadYourWrites) renders with no operation marked failing because the failing mark is computed only when checked.read is none, contrary to the guide's statement that the page marks the call where the search failed"
```
