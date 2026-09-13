---
format: aep.planning-md/1
id: decision-blocker:wave24-unit1-leaves-after-two-passes
kind: decision-blocker
status: open
title: Unit 1 leaves wave 24 after two adversary passes
relations:
- blocks: story:browser-fixture-startup-deadline
revision: 1
---
# Unit 1 leaves wave 24 after two adversary passes

`story:browser-fixture-startup-deadline`, branch `impl/browser-fixture-startup-deadline`, worktree
`wt-1933f8986f66`, uncommitted over base `bd722fa964bd225b9755f272e22b45fab334449f`. The work is not
lost; the branch stays.

| Pass | Record | Outcome |
|---|---|---|
| 1 | `review-result:adversary-wave24-unit1-pass-1` | NEEDS-CHANGE, 8 findings, 3 red |
| correction 1 | `review-result:wave24-unit1-correction-1` | green, 543 cases, exit 0 |
| 2 | `review-result:adversary-wave24-unit1-pass-2` | NEEDS-CHANGE, 8 findings, 5 red, two blockers |

Ledger: `carried 0, new 8, resolved 8`. Nothing pass 1 found was found again.

## Why this is an escalation and unit 3's was not

Unit 3 was also red after two passes and did **not** leave. The discriminator, stated so the two
decisions can be compared:

| | unit 3 | unit 1 |
|---|---|---|
| shipped cases moved | none | none |
| blocker origin | the coordinator's design page, and a pre-existing doc claim | **introduced**, CONFIRMED, in behaviour |
| what the fix needs | four doc sentences and a test that omits a state | a decision resting on a fact nobody has measured, plus a behaviour change |

**G4 is a functional regression, isolated with a control.** The upgrade socket's read and write
timeouts are `deadline - elapsed`, and that socket becomes `Browser::stream` for the browser's whole
life. A browser ready at 2.700 s of a 3.000 s deadline then took 0.800 s over `session.new` and the
start died at `browser.rs:566` on a bare `WouldBlock` — no stage, no measured startup, no
`firefox.stderr`. The **identical** browser ready at 0.05 s passed in 0.85 s.

Base set a flat 20 s regardless. So correction round 1's clamp — which correctly bounded startup, and
took the boundary lane from 22.36 s to 3.13 s — leaked into every later BiDi call, and the harm
scales with slowness. The story's own CI failure exceeded 30 s. This is a regression on precisely the
runner the story was filed about.

**G1 turns on a fact nobody here can measure.** A Firefox that was listening and answered `404` once
while registering `/session`, then missed its deadline, now panics with the story's own headline
`Firefox did not expose BiDi` instead of refusing. `answered` is set at `browser.rs:426` and never
cleared, so one transient 404 decides every later give-up.

The adversary measured 0 of 77 real Firefox starts reaching the 404 state on this machine. The branch
at `:423` and its comment — carried from base — say Firefox does 404 while booting. So the acceptance
statement holds only if a real Firefox on the CI runner never 404s, and nobody has measured that.

## The coordinator caused G1

F3 in correction round 1 was mine. Pass 1 found that a browser answering HTTP 404 forever was being
reported as a runner failure, and I told the unit to stop blaming the runner for a browser that
answered HTTP. The unit implemented it, then reported — before any of this was measured — that the
story's originally-observed input is the same code path and that the rule might move it to the
non-refusal side. It declined to pick a side and said so.

It was right and I did not act on it. Pass 2 built the state and it is exactly what the unit
predicted. The instruction was too broad: it drew the line at *did anything answer HTTP* when the
line the story needs is *did the browser become usable*.

## What was carried out of it

- `story:the-startup-clamp-does-not-outlive-the-startup` — G4
- `story:a-browser-that-answered-http-once-is-still-a-slow-start` — G1, G2, and the unmeasured premise
- `story:the-startup-lock-does-not-cover-the-first-round-trip` — G5
- `story:a-marked-region-is-not-a-scan-of-what-runs` — G6, G7, G8

G3 is pre-existing, INFEASIBLE and reproduces worse at base; it is recorded in the pass-2 result and
not carried as a story.

## Settled in the unit's favour

- The start count is **84**, not pass 1's 81: 92 profile dirs in a package run, 8 the adversary's and
  3 pass 1's. Pass 1 measured before the unit's own diff added three starts.
- The one-file bound holds: `grep -rn -e ESS_FIREFOX -e remote-debugging-port crates/ --include
  '*.rs'` returns exactly 2 matches, both in `support/browser.rs`.
- The serialisation gate's timing assertion cannot flake red — each stand-in sleeps 500 ms, so three
  serialised starts cannot finish under 1.3 s at any load. The coordinator's flake worry was wrong.
- `Stage::ALL` against the declared variants holds, and the five-variant list is complete.
- The panic-site scan's own results are correct today; all 270 region lines were read and the unit's
  hand enumeration of index, slice and arithmetic panics was right.
