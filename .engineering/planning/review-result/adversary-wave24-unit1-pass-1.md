---
format: aep.planning-md/2
id: review-result:adversary-wave24-unit1-pass-1
kind: review-result
status: active
title: Adversary pass 1 against the browser fixture startup deadline
relations:
- reviews: story:browser-fixture-startup-deadline
revision: 1
---
# Adversary pass 1 against `story:browser-fixture-startup-deadline`

Worktree `wt-1933f8986f66`, branch `impl/browser-fixture-startup-deadline`, uncommitted over base
`bd722fa964bd225b9755f272e22b45fab334449f`. Verdict **NEEDS-CHANGE**. Cases 539 → 542, 3 red.
The adversary wrote one untracked test file and touched no implementation file.

All three red cases drive a stand-in program that announces
`WebDriver BiDi listening on ws://127.0.0.1:<port>` on stderr — the exact shape `assigned_bidi_port`
scrapes — against a Rust listener that misbehaves in one specific way.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| F1 | `support/browser.rs:295` | CONFIRMED / pre-existing / **blocker** | `read_exact().unwrap()` → `ConnectionReset` panic, no stage, no measured startup, no evidence path, no `firefox.stderr`. `browser_startup_adversary_pass1.rs:86`, exit 101 | every one of the 81 starts passes this line. The site is pre-existing — `git show <base>` line 194 is byte-identical — but the unit's claim that the class is closed at five sites is new and false |
| F2 | `support/browser.rs:285` | NEEDS-CHANGE / pre-existing / **blocker** | 20.544 s elapsed against a 1.000 s deadline, then a `WouldBlock` panic. `browser_startup_adversary_pass1.rs:119` | `set_read_timeout(20s)` is not clamped to the remaining budget, so `STARTUP_DEADLINE` bounds the poll loops only. Worst case is deadline + 20 s per iteration |
| F3 | `support/browser.rs:301` | CONFIRMED / introduced / warning | a listening server answering HTTP 404 on `/session` produced `stage: connect`, `measured startup: 2.019s`, and the sentence "not a BiDi protocol defect". `browser_startup_adversary_pass1.rs:167` | at base this same input panicked with `Firefox did not expose BiDi` — a defect signal. The unit converted it into an active denial. The kept-panic boundary catches a `101` with a wrong key and misses a permanent `404` |
| F4 | `coverage_browser.rs:514` | CONFIRMED / introduced / warning | the gate test's three threads pass `Duration::ZERO`, so `assigned_bidi_port` refuses at its first `elapsed >= deadline` check. The lock is held across `spawn()` and nothing else, so `assert_eq!(peak…, 1)` at `:527` never observes a completed startup | a mutant moving `drop(starting)` (`browser.rs:314`) to just after `command.spawn()` — leaving the whole BiDi startup concurrent, which is the story's defect — leaves all 542 cases green. Not demonstrated with a red case; that needs mutating the file under attack |
| F5 | `replay_fidelity_browser.rs:277` | CONFIRMED / introduced / warning | one run left 69 `ess-bidi-<pid>-N` profile dirs for that binary, 10 for `coverage_browser`, 1 + 1 for the writer adversaries — **81 browser starts**, not the 37 the unit reported. `browse()` starts a Firefox per call, not per test | the whole gate. 81 serialised starts at the CI runner's >30 s is a 40-minute worst case where the base arrangement overlapped them |
| F6 | `support/browser.rs:228` | CONFIRMED / pre-existing / warning | 90 profile dirs at ~20 MB each = **1.6 GB** under `$TMPDIR` across two runs. `du -shc …/ess-bidi-*` | every start. Pre-existing; the unit adds 6 per run. Directly relevant to the 25 G `/tmp` quota if a run ever uses the default `TMPDIR` |
| F7 | `coverage_browser.rs:445` | CONFIRMED / introduced / note | `for stage in ["announce", "connect", "exited"]` is a literal derived from nothing. `spawn (<os error>)` ships in the same diff and is not in it; F1's site has no stage at all | the "by construction" claim is false today, not in future |
| F8 | `support/browser.rs:157-159` | CONFIRMED / introduced / note | for `stage: spawn (…)` and `stage: exited`, `started.elapsed()` is sub-millisecond noise printed beside `deadline: 30.000s`. The unit's own case at `coverage_browser.rs:475` pins the noise with `assert!(refusal.contains("measured startup: 0."))` | every spawn/exit refusal. Reads as "the runner was fast" when nothing was measured |

## Attacked and could not break

- **The serialisation gate holds for this repository.** `task check` runs `cmds` sequentially,
  `cargo test` runs test targets serially, CI has one `task check` job and one non-browser matrix job
  on separate runners, and there are zero `nextest` references in `Taskfile.yml` or `.github/`.
  `cargo nextest` (process per test) would defeat it — latent, not live.
- **The counter has no window.** `Drop for Starting` decrements before the `MutexGuard` field drops,
  so `STARTING` cannot exceed 1 while the lock is honoured; a panic inside startup releases the lock
  via `PoisonError::into_inner` and still decrements. No leak, no missed decrement, no deadlock.
  The weakness is F4 — what the counter is pointed at, not the counter.
- **No fixed scratch name two processes could collide on.** Every evidence and profile directory is
  `<name>-<pid>` or `ess-bidi-<pid>-<n>`. PID reuse is harmless: `File::create` truncates
  `firefox.stderr` before any read.
- **The cost.** `replay_fidelity_browser` measured 49.40 s here against the unit's reported base of
  45.57 s (+8.4%); the unit's own 54.34 s is within machine noise. The adversary did not measure the
  base itself. F5 is the number that matters, not this one.
- **Zero stray `firefox` processes after a run**, including on the four `Duration::ZERO` spawns the
  unit added.
- **Both of the unit's claimed environmental reds pass under a scratch `TMPDIR`.** Confirmed.

## Store correction the pass settled

The story cited `support/browser.rs:143-151`. At base, `:143` is `Command::new(firefox)`, `:168` is
`Duration::from_secs(30)` and `:177` is the message. The unit's report was right; corrected in the
store at revision 7.

```findings
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 295
  category: acceptance
  severity: blocker
  verdict: CONFIRMED
  origin: pre-existing
  message: "a start lost at the WebSocket upgrade read panics with a bare io error carrying no stage, no measured startup, no evidence path and no firefox.stderr, so the unit's claim that every way Browser can fail now produces a fixture environment refusal is false at a sixth give-up site it never enumerated."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 285
  category: boundary
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: pre-existing
  message: "the 20 s socket read timeout is not clamped to the remaining budget, so STARTUP_DEADLINE is not a bound on startup - a 1.000 s deadline was overrun to 20.544 s before the fixture gave up, and gave up by panicking."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 301
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "a browser that is running, listening and answering HTTP 404 on /session is reported as a fixture environment refusal whose text denies being a BiDi protocol defect, where the base panicked instead, so the unit's kept-panic boundary for a handshake that is not 101 does not hold."
- file: crates/edge/ess-cli/tests/coverage_browser.rs
  line: 514
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the serialisation gate test passes Duration::ZERO so all three threads refuse at the first deadline check, holding the lock only across spawn(), and a mutant moving drop(starting) to just after command.spawn() - leaving the whole BiDi startup concurrent, which is the story's defect - keeps every one of the 542 cases green."
- file: crates/edge/ess-cli/tests/replay_fidelity_browser.rs
  line: 277
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "browse() starts a Firefox per call rather than per test, so one package run makes 81 browser starts and not the 37 the unit reported, and the serialised wall-clock risk on the slow runner the story is about is more than twice what the unit estimated."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 228
  category: judgement
  severity: warning
  verdict: CONFIRMED
  origin: pre-existing
  message: "each start abandons a roughly 20 MB Firefox profile directory under TMPDIR that nothing ever removes, measured at 1.6 GB of profiles across two package runs, and the unit adds six more starts per run."
- file: crates/edge/ess-cli/tests/coverage_browser.rs
  line: 445
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "the stage list is a literal derived from nothing, so the claim that a new give-up site gets the same reporting by construction is already false for the spawn stage shipped in the same diff and for the upgrade-read site that carries no stage at all."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 157
  category: judgement
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "for the spawn and exited stages the measured startup is sub-millisecond noise printed beside a 30.000s deadline it was never compared against, and the unit's own case pins that noise."
```
