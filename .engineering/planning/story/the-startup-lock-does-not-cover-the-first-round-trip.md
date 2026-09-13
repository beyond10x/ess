---
format: aep.planning-md/1
id: story:the-startup-lock-does-not-cover-the-first-round-trip
kind: story
status: draft
title: The startup lock does not cover the first round trip
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 3
---
# The startup lock does not cover the first round trip

`crates/edge/ess-cli/tests/support/browser.rs:338` releases the startup lock eight lines before
`browser.call("session.new")` at `:346` — still inside `Browser::new`, and the first BiDi round trip
of a cold start.

Measured by the wave-24 unit-1 pass-2 adversary at `:326` of its own file: **3 of 3 fixtures were
inside `session.new` at the same instant**. Three 0.500 s calls finished in 0.65 s; serialised they
could not finish under 1.5 s.

`cargo test` runs the four `Browser::new` callers in `coverage_browser.rs` on parallel threads, so
this is the production arrangement on every run, not a constructed state.

`peak_concurrent_startups()` cannot see it. `InStartup` is scoped to `reach_bidi` at `:326`, so the
gate at `coverage_browser.rs:695` is blind to this window by construction — it measures the part of
startup the lock covers, which is the part that is already serialised.

`story:browser-fixture-startup-deadline`'s acceptance says the fixtures do not compete for CPU during
startup. Whether `session.new` counts as startup is the question: it is inside the constructor, it is
the first thing a cold browser is asked to do, and a browser that has not completed it is not usable.

What is not measured: `session.new`'s share of a cold start on the slow runner. That is why the
adversary graded this a warning rather than a blocker, and it is the number that decides how much
this costs.

## Acceptance

Either the startup lock covers every step a cold browser takes before it is usable, or the gate
measures the window the lock actually covers and the module says which steps are outside it and why.

Named fix from the adversary, not applied: move `drop(starting)` below `:346`.

## Scope

- `crates/edge/ess-cli/tests/support/browser.rs` — `cited`
- `crates/edge/ess-cli/tests/coverage_browser.rs` — `cited`
