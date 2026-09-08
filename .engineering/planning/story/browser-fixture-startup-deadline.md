---
format: aep.planning-md/1
id: story:browser-fixture-startup-deadline
kind: story
status: draft
title: The Firefox BiDi fixture assumes a 30-second startup on a shared runner
summary: Three ess-cli browser tests fail on a slow runner at a constant 30 s deadline; report the environment refusal and stop the three starts competing.
tags:
- flake
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 2
---
# The Firefox BiDi fixture assumes a 30-second startup on a shared runner

## Finding

Three `ess-cli` browser tests share one startup deadline in
`crates/edge/ess-cli/tests/support/browser.rs:143` (`Duration::from_secs(30)`) and fail with
`Firefox did not expose BiDi; read firefox.stderr` (`browser.rs:151`) when the runner is slow.
Observed on CI run 34288754559 (PR #17, `fix/fuzz-story-closure` at 954fb43): `retained_legacy_player_bytes_still_replay_in_actual_firefox`,
`actual_browser_admits_the_pair_before_creating_replay_state` and
`actual_browser_and_rust_refuse_every_closed_model_field_boundary` all panicked at that
assertion; `test result: FAILED. 1 passed; 3 failed`, lane finished in 46.10 s. The same tests
passed on main runs 34234834142 (wave 19) and 34280793309 (51dd8a7). The change under test
touched `fuzz/`, `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` and
`docs/design/review-specification-fuzzing.md` only; nothing on the browser path. Filed as a
pre-existing fixture defect by the wave-21 coordinator on 2026-09-09; the run was re-run.

## Acceptance

A slow Firefox start on a shared runner is reported as a fixture environment refusal with the
measured startup time and `firefox.stderr` attached, distinguishable in the runner output from a
BiDi protocol defect, and the three tests do not compete for CPU during startup (serialized
startup or a deadline derived from a measured baseline rather than a constant).

## Scope

- `crates/edge/ess-cli/tests/support/browser.rs:143-151` — cited.
- The three tests above in `crates/edge/ess-cli/tests/` — inferred; they share the fixture.
- Would collide with: any unit touching `ess-cli/tests/support/browser.rs` — cited.
