---
format: aep.planning-md/3
id: story:the-startup-clamp-does-not-outlive-the-startup
kind: story
status: implemented
title: The startup clamp does not outlive the startup
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T11:18:00Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T11:18:00Z", actor: "human:timo", revision: 5, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "active", to: "implemented", at: "2026-10-02T11:18:00Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
# The startup clamp does not outlive the startup

`crates/edge/ess-cli/tests/support/browser.rs:436` sets the upgrade socket's read and write timeouts
to `deadline.saturating_sub(elapsed)`. That socket becomes `Browser::stream` and is the browser's
transport for its whole life.

So a browser that becomes ready late carries **the leftover startup budget** as the timeout on
`session.new` and on every later `open`, `evaluate` and `receive`.

Measured by the wave-24 unit-1 pass-2 adversary, with a control:

| Stand-in | Ready at | `session.new` takes | Deadline | Result |
|---|---|---|---|---|
| A | 2.700 s | 0.800 s | 3.000 s | dies at `browser.rs:566`, bare `WouldBlock`, no stage, no measured startup, no `firefox.stderr` |
| B (control) | 0.05 s | 0.800 s | 3.000 s | passes in 0.85 s |

Identical browser, identical call, identical deadline. The only difference is how much budget was
left, which is what isolates the cause.

Base set a flat 20 s regardless of elapsed time. The clamp was added in wave 24 to stop
`STARTUP_DEADLINE` being overrun by 20 s per loop iteration, and it did — the boundary lane went
22.36 s to 3.13 s. It then leaked into the session.

**The harm scales with slowness, which is the condition the story was filed about.** The CI run that
produced `story:browser-fixture-startup-deadline` exceeded 30 s. A start that only just makes its
deadline leaves near-zero budget for the first BiDi round trip.

## Acceptance

A browser that became ready inside its deadline serves BiDi calls with a timeout that does not
depend on how long its startup took.

Named fix from the adversary, not applied: restore the socket's timeouts after a successful upgrade
rather than leaving the startup clamp on the session.

## Scope

- `crates/edge/ess-cli/tests/support/browser.rs` — `cited`
- `crates/edge/ess-cli/tests/browser_startup_slow_serve_boundary.rs` — `cited`, untracked, holds the
  red case and its green control, on `impl/browser-fixture-startup-deadline`

## Verified reconciliation 2026-10-02

Already fixed by5c8fc4ecd, contained in released0.51.0. Current support/browser.rs:464-478 restores SESSION_TIMEOUT for both read and write after successful upgrade; the startup clamp is not left on the session socket. Fresh execution at482609 of browser_startup_slow_serve_boundary passed the late-ready regression and same-speed early-ready control:2passed0failed4ignored, captured in server target/backlog-input/browser-startup-reconciliation.log. This matches the story's specific acceptance. The four ignored neighboring cases concern separate outstanding startup classification/retry/serialization defects and do not qualify as passing coverage; they are not closed by this move.
