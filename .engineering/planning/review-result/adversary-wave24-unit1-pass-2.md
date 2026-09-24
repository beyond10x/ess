---
format: aep.planning-md/2
id: review-result:adversary-wave24-unit1-pass-2
kind: review-result
status: active
title: Adversary pass 2 against the browser fixture startup deadline
relations:
- reviews: story:browser-fixture-startup-deadline
revision: 1
---
# Adversary pass 2 against `story:browser-fixture-startup-deadline`

Worktree `wt-1933f8986f66`, uncommitted over base `bd722fa964bd225b9755f272e22b45fab334449f`.
Verdict **NEEDS-CHANGE**. Cases 543 → 549, 5 red, 1 green control. Budget exhausted.

`TMPDIR=<scratch> cargo test --locked -p ess-cli --no-fail-fast` → 544 passed / 5 failed, exit 101.
544 is the unit's 543 plus the adversary's control case. No pre-existing case regressed.

## Findings

| # | Where | Verdict / origin / severity | Measured | Reaches it |
|---|---|---|---|---|
| G1 | `support/browser.rs:464` | NEEDS-CHANGE / introduced / **blocker** | a Firefox that was listening, answered `404` once while registering `/session`, then missed a 3.000 s deadline **panicked** — `Firefox did not expose BiDi: …` at `:468` — rather than refusing. `browser_startup_slow_serve_boundary.rs:131`, exit 101 | `answered` is set at `:426` and never cleared. One transient 404 moves the start to the non-refusal side permanently. **The state is constructed.** In the same run, 0 of 77 real Firefox starts wrote `websocket-startup-response.txt`, so no real browser reached it on this machine. The branch at `:423` and its comment are deliberate and say Firefox does 404 while booting; base carried the same comment |
| G2 | `support/browser.rs:472` | CONFIRMED / introduced / warning | the panic says the reader decides "by the response and the log below" and prints no `firefox.stderr`; the 49 bytes the browser wrote appear nowhere. It also prints no `measured startup:` and no `deadline:`. `:158` | same path as G1. The one give-up routed away from `startup_refusal` carries none of the three things the story's acceptance names |
| G3 | `support/browser.rs:413` | INFEASIBLE / pre-existing / note | 1 connection attempt against a 2.000 s deadline with the child still alive, then refusal. `:186` | a refused connect retries (`:382`) and a 404 retries (`:427`); an upgrade io error does not. Constructed; no real browser shown to produce a socket error mid-boot with the child alive. Reproduces at base, worse — base `unwrap()`ed it |
| G4 | `support/browser.rs:436` | CONFIRMED / introduced / **blocker** | the upgrade socket's read/write timeout is `deadline - elapsed`, and that socket becomes `Browser::stream`. A browser ready at 2.700 s of 3.000 s then took 0.800 s over `session.new` and the start died at `:566` on a bare `WouldBlock` — no stage, no measured startup, no `firefox.stderr`. The **identical** browser ready at 0.05 s passed in 0.85 s. `:222` red, `:281` green | every one of the 84 starts sets it. The harm scales with slowness, which is the story's own condition: the CI run the story is about exceeded 30 s, so a start that just makes it leaves near-zero budget for `session.new` and every later `open`/`evaluate`. Base set a flat 20 s regardless. A regression on precisely the slow runner the story is about |
| G5 | `support/browser.rs:338` | CONFIRMED / introduced / warning | 3 of 3 fixtures inside `session.new` simultaneously; three 0.500 s calls finished in 0.65 s against ≥1.5 s serialised. `:326` | `drop(starting)` at `:338` is eight lines before `browser.call("session.new")` at `:346`, still inside the constructor and the first BiDi round trip of a cold start. `cargo test` runs the four `Browser::new` callers on parallel threads, so this is the production arrangement every run. `peak_concurrent_startups()` cannot see it: `InStartup` is scoped to `reach_bidi` at `:326` |
| G6 | `coverage_browser.rs:546` | CONFIRMED / introduced / warning | `support/browser.rs:346` is inside the marked region, matches none of the eight forms, and can end the process one call below at `:566` — demonstrated by G4's red. The scan reports 15 sites / 0 unaccounted and misses it | every start. The bound the unit stated — "explicit panic macros and methods only" — is not the bound that matters. The one that matters is that the scan reads lines, and a panic one call deep is not a line |
| G7 | `coverage_browser.rs:556` | CONFIRMED / introduced / warning | the region ends at the first line whose **tail** is `// startup-path: end`. A forged `end` at line 333 leaves `region.len() = 82` (guard wants > 50) and `accounted = 8` (guard wants ≥ 8) — **both floors still pass** — while 188 lines go unscanned, including all of `reach_bidi`, `upgrade`, `connect_give_up` and `assigned_bidi_port` | the next person to edit the file. The two floors were chosen to detect a missing region; neither detects a truncated one |
| G8 | `support/browser.rs:403` | CONFIRMED / introduced / note | **reasoned from the assertions, not executed** — demonstrating it needs mutating the file under attack. Deleting the `elapsed >= deadline` check in the upgrade-error branch turns the silent-socket case from `stage: connect` into `stage: upgrade (…)` and nothing notices: the only case on that path asserts the `fixture environment refusal:` prefix and a 5 s slack on a 1 s deadline, both of which survive | the discriminator between "the deadline expired" and "the socket broke" is the unit's own new distinction and no case pins it. One line closes it |

## Attacked and could not break

- **The 84.** 92 profile dirs in one package run, 8 the adversary's and 3 pass 1's: **84 is right**, and
  pass 1's 81 was measured before the unit's diff added three starts. Settled in the unit's favour.
- **The one-file bound.** `grep -rn -e ESS_FIREFOX -e remote-debugging-port crates/ --include '*.rs'`
  returns exactly 2 matches, both in `support/browser.rs`. Verified.
- **F4's timing assertion cannot flake red.** Each stand-in sleeps 500 ms, so three serialised starts
  cannot finish under 1.3 s at any load; the 200 ms slack is all in the lenient direction. The
  coordinator's flake worry was unfounded for this assertion.
- **`Stage::ALL` against the declared variants** holds; the parser handles the doc comments and the
  `#[allow]`, and the five-variant list is complete.
- **The panic-site scan's own results are correct today.** All 270 region lines read for index, slice
  and arithmetic panics; none live. The unit's hand enumeration was right. G6 and G7 are about what
  the scan cannot see, not a site it missed.
- **The `Browser::new` and post-`end` exemptions are right** — `:243` turns a refusal into a test
  failure with the text intact, and everything after `:521` is genuinely post-startup, except `:346`,
  which is G6.
- **No stray `firefox` processes**, and no fixed scratch name two processes could collide on.

## Named fixes, not applied

- G1/G2 — give up through `startup_refusal` whenever `elapsed >= deadline` regardless of `answered`,
  and carry the last response into the refusal as evidence rather than as a verdict.
- G4 — restore the socket's timeouts after a successful upgrade instead of leaving the startup clamp
  on the session.
- G5 — move `drop(starting)` below `:346`.

```findings
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 464
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: "a Firefox that was listening and answered 404 once while registering /session, then missed its deadline, panics with the story's own Firefox did not expose BiDi headline instead of refusing, because the answered discriminator is sticky and nobody has measured whether a real Firefox on the slow runner reaches the 404 state - 0 of 77 real starts did here."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 472
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the one give-up routed away from startup_refusal tells the reader to decide by the response and the log below and then prints no firefox.stderr, no measured startup and no deadline, so it carries none of the three things the story's acceptance names."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 413
  category: boundary
  severity: note
  verdict: INFEASIBLE
  origin: pre-existing
  message: "a refused connect and a 404 are both retried within the deadline but an upgrade socket error is not, so the fixture made one connection attempt against a 2.000s deadline with the child still alive; no real browser was shown to produce that error mid-boot, and it reproduces at base."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 436
  category: property
  severity: blocker
  verdict: CONFIRMED
  origin: introduced
  message: "the remaining-budget read and write timeouts are set on the socket the Browser keeps for life, so a browser that became ready late carries the leftover deadline as the timeout on session.new and every later BiDi call and dies on a bare WouldBlock with no stage, no measured startup and no firefox.stderr - the identical browser ready early succeeds, and base set a flat 20s, making this a regression on precisely the slow runner the story is about."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 338
  category: acceptance
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the startup lock is released eight lines before browser.call session.new, so three fixtures were measured creating their BiDi session at the same instant and three 0.500s calls finished in 0.65s rather than 1.5s, and peak_concurrent_startups cannot see it because InStartup is scoped to reach_bidi alone."
- file: crates/edge/ess-cli/tests/coverage_browser.rs
  line: 546
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the panic-site guard's class claim is already false at a live line - support/browser.rs:346 is inside the marked region, matches none of the eight forms and ends the process one call below, which the guard reports as 0 unaccounted."
- file: crates/edge/ess-cli/tests/coverage_browser.rs
  line: 556
  category: mutant
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: "the scanned region ends at the first line whose tail is the end marker, and a forged one at line 333 leaves both floors passing while 188 unscanned lines include the whole of reach_bidi, upgrade, connect_give_up and assigned_bidi_port."
- file: crates/edge/ess-cli/tests/support/browser.rs
  line: 403
  category: mutant
  severity: note
  verdict: CONFIRMED
  origin: introduced
  message: "deleting the deadline check in the upgrade-error branch changes the silent-socket give-up from stage connect to stage upgrade and no case notices; reasoned from the assertions, not executed, since demonstrating it requires mutating the file under attack."
```
