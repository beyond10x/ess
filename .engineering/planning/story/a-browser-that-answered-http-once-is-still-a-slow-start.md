---
format: aep.planning-md/2
id: story:a-browser-that-answered-http-once-is-still-a-slow-start
kind: story
status: draft
title: A browser that answered HTTP once is still a slow start
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 2
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
