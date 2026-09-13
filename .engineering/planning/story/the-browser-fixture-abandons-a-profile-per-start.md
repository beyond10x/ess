---
format: aep.planning-md/1
id: story:the-browser-fixture-abandons-a-profile-per-start
kind: story
status: draft
title: The browser fixture abandons a profile per start
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/replay_fidelity_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 3
---
# The browser fixture abandons a profile per start

`crates/edge/ess-cli/tests/support/browser.rs:228` creates a Firefox profile directory per browser
start under `$TMPDIR` and nothing ever removes it.

Measured by the wave-24 unit-1 adversary: 90 profile directories at roughly 20 MB each — **1.6 GB**
— left behind across two `-p ess-cli` package runs, about 1.6 GB per run.

The count is larger than it looks. `browse()` in `replay_fidelity_browser.rs:277` starts a Firefox
per **call**, not per test, so one package run makes **81 browser starts**: 69 in
`replay_fidelity_browser`, 10 in `coverage_browser`, 1 each in the two writer adversaries.

`~/.claude/settings.json` sets `TMPDIR` away from `/tmp` for agent sessions, but a run made with the
default `TMPDIR` puts 1.6 GB per run against the 25 G per-user `/tmp` quota, where an over-quota
write fails with `EDQUOT` while `df` still shows free space.

## The distinction that has to survive the fix

The evidence directories a startup refusal points at are **deliberately kept** — they are the
evidence. The profile directories are not evidence of anything once the browser has exited. Whoever
takes this separates the two rather than cleaning up both.

## Acceptance

A package run leaves no profile directory behind for a browser that started and exited cleanly, and
a startup refusal's evidence directory still exists after the run that wrote it.

## Scope

- `crates/edge/ess-cli/tests/support/browser.rs` — `cited`
- `crates/edge/ess-cli/tests/replay_fidelity_browser.rs` — `inferred`, if the per-call start is
  changed to a per-test one
