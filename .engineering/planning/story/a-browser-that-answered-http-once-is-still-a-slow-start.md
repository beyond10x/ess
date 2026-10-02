---
format: aep.planning-md/3
id: story:a-browser-that-answered-http-once-is-still-a-slow-start
kind: story
status: active
title: A browser that answered HTTP once is still a slow start
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/browser_startup_refusal_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/browser_startup_slow_serve_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T11:44:58Z", actor: "human:timo", revision: 6, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T11:44:58Z", actor: "human:timo", revision: 7, executor: "agent:codex-ess-backlog", correlation: "consumer-backlog-20261002"}
---
# A browser that answered HTTP once is still a slow start

`crates/edge/ess-cli/tests/support/browser.rs:426` sets `answered` when the browser replies to a
`/session` poll with anything at all, including `404`. It is never cleared. From then on, a deadline
expiry **panics** with `Firefox did not expose BiDi: …` (`:464`) instead of producing a fixture
environment refusal.

`Firefox did not expose BiDi; read firefox.stderr` is the exact message the CI failure that produced
`story:browser-fixture-startup-deadline` carried.

Measured by the wave-24 unit-1 pass-2 adversary at
`browser_startup_slow_serve_boundary.rs:131`, exit 101: a stand-in that announces BiDi, answers `404`
on the first `/session` poll and then stalls produces a panic at 3.010 s against a 3.000 s deadline,
not a refusal.

## The premise nobody has measured

`browser.rs:423`'s branch and its comment — carried unchanged from base — say a booting Firefox
answers `404` on `/session`. The adversary measured **0 of 77 real Firefox starts** writing
`websocket-startup-response.txt` on this machine, so no real browser reached that state here.

So the story's acceptance holds only if a real Firefox on the slow CI runner never 404s while
booting, and that has not been measured on the runner. Somebody has to find out, or the
discriminator has to stop depending on it.

## The discriminator is drawn in the wrong place

The rule currently is *did anything answer HTTP*. A browser that is up and serving a defective
`/session` is genuinely not the runner's fault, and that was the finding this branch answers. But a
browser that 404s once while booting and then never finishes is the runner starving it, which is the
story.

The line the story needs is *did the browser become usable*, not *did it emit bytes*.

## The message carries none of what it promises

`:472`. The panic tells the reader to decide "by the response and the log below" and then prints no
`firefox.stderr` — the 49 bytes the browser actually wrote appear nowhere — no `measured startup:`
and no `deadline:`. It is the one give-up routed away from `startup_refusal`, and it carries none of
the three things the story's acceptance names.

## Acceptance

A start that never becomes usable within its deadline is a fixture environment refusal carrying the
measured startup and `firefox.stderr`, whether or not the browser emitted bytes on the way. A start
that reached a usable browser and then failed is distinguishable from it, and says which by evidence
rather than by assertion.

## History

Wave 24's correction round 1, finding F3, was written by the coordinator and told the unit to stop
reporting a browser that answered HTTP as a runner failure. The unit implemented it and reported, in
the same handback, that the story's own observed input is this code path and that the rule might move
it to the non-refusal side. It declined to pick a side. It was right; the instruction was too broad.

## Scope

- `crates/edge/ess-cli/tests/support/browser.rs` — `cited`
- `crates/edge/ess-cli/tests/browser_startup_slow_serve_boundary.rs` — `cited`, untracked, holds the
  red cases on `impl/browser-fixture-startup-deadline`

## Fresh red evidence 2026-10-02

The two named cases remain explicitly ignored on current source482609 and both fail when executed. The entire ignored startup lane is0passed4failed; browser-startup-existing-defects-red.log retains exact output. support/browser.rs::connect_give_up still panics after an earlier404 and omits the stderr it promises. This is actionable existing harness work, not fixed by the successful late-ready socket-timeout regression. Coordinate the repair with startup-lock and lost-startup-socket-retry; no ignored case is counted as a pass.

## Boundary reconciliation before repair

Current browser_startup_refusal_boundary.rs::a_handshake_answered_404_forever_stays_a_bidi_defect_and_is_not_blamed_on_the_runner encodes the older rule that HTTP404 proves readiness. That expectation conflicts with this accepted story: Firefox can answer404 before registering /session, and no successful upgrade has occurred. The repair must update that old discriminator assertion explicitly, retain its permanently404 stand-in, assert a measured startup refusal with stderr/last response, and separately preserve a genuine malformed/non101 successful-response protocol defect control. Do not delete the permanent404 case or call it passing unchanged. Include browser_startup_refusal_boundary.rs as cited scope alongside slow-serve boundary and support/browser.rs.
