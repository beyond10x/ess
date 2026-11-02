---
format: aep.planning-md/3
id: story:a-browser-start-bounds-session-new-by-its-deadline
kind: story
status: draft
title: A browser start bounds its first session.new by the startup deadline
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/browser_startup_refusal_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 4
---
## Finding

Out of scope in adversary pass 1 of wave D357 U2 (`review-result:adversary-d357-u2-pass-1`): since that unit, the first `session.new` round trip is part of a browser start in the ess-cli test fixture (`crates/edge/ess-cli/tests/support/browser.rs`), but it is bounded by the session timeout (120 s), not by the startup deadline the caller gave, so a silent `session.new` holds a 1 s start for 120 s.

## Acceptance

- The first `session.new` round trip of a start is bounded by the time left of the caller's startup deadline; when it runs out, the start is a fixture environment refusal at the session stage that carries the measured startup time.
- A boundary case with a stand-in that accepts the upgrade and never answers `session.new` refuses within the startup deadline plus a stated margin.

## Scope

Derived 2026-11-02 by `aep:story-scoper` for wave D368 on 6437975cb. Every line is **cited** (read from the story or the tree) or **inferred**.

- **Primary surface:** `crates/edge/ess-cli/tests/support/browser.rs`, the start `Browser::launch_program` → `reach_bidi` → `exchange` — cited
- **Symbols:** `SESSION_TIMEOUT` `browser.rs:228`, restored onto the socket after upgrade at `:608-609`; `STARTUP_DEADLINE` `:208`; first `session.new` `:492`; `exchange` `:724`; `receive` `:781`; `Stage::Session` `:314` — cited
- **Mechanism:** after a successful upgrade the socket carries `SESSION_TIMEOUT` (120 s) for the browser's whole life, so a silent `session.new` waits 120 s, not the time left of the startup deadline — cited (code path); the 120 s wall time — inferred (not run)
- **Tests that pin today's behaviour:** `browser_startup_refusal_boundary.rs:159`, `:249`, `:308` (session.new lost, close frame, error answer; none is silent); `browser_startup_slow_serve_boundary.rs:244`, `:307` (late-ready and early-ready session.new, 800 ms answer) — cited
- **Test files a unit would add or change:** `browser_startup_refusal_boundary.rs`, one silent-`session.new` case in the `:159` pattern — inferred; no silent `session.new` case exists at origin/main — cited
- **Firefox or stand-in:** both boundary files call `Browser::launch_program` with a `/bin/sh` stand-in; a Rust thread plays the endpoint; `ESS_FIREFOX` is read only by `launch` (`browser.rs:390`) — cited
- **CI:** runs them in the `test` shards (`ci.yml:278-279`, `Taskfile.yml:157`); `.config/nextest.toml:9` places both binaries in the `firefox` group, max-threads 1; not in the release-only lane (`Taskfile.yml:138`) — cited
- **Confidence:** high on the site; medium on the new case's run time (nothing run)
- **Would collide with:** any unit touching `support/browser.rs` (11 ess-cli binaries pull it in), and `no_unaccounted_panic_site_can_end_a_start` (`coverage_browser.rs:1158`), which requires any new panic on the start path to carry a `startup-path:` mark — cited
