---
format: aep.planning-md/2
id: story:the-startup-clamp-does-not-outlive-the-startup
kind: story
status: draft
title: The startup clamp does not outlive the startup
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 2
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
