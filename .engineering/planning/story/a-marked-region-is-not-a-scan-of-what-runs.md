---
format: aep.planning-md/2
id: story:a-marked-region-is-not-a-scan-of-what-runs
kind: story
status: draft
title: A marked region is not a scan of what runs
scope:
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 3
---
# A marked region is not a scan of what runs

`crates/edge/ess-cli/tests/coverage_browser.rs:546`,
`no_unaccounted_panic_site_can_end_a_start`, reads `support/browser.rs` through `include_str!`,
takes the lines between `// startup-path: begin` and `// startup-path: end`, and fails on any
`.unwrap()`, `.expect(`, `panic!`, `assert*!`, `unreachable!` or `todo!` not marked
`// startup-path: harness` or `// startup-path: defect`. It reports 15 sites, 0 unaccounted.

Two ways it does not do what it claims, both measured by the wave-24 unit-1 pass-2 adversary.

## It cannot see one call down

`support/browser.rs:346` is inside the marked region, matches none of the eight forms, and ends the
process one call below at `:566` — demonstrated live by
`story:the-startup-clamp-does-not-outlive-the-startup`'s red case.

The unit stated the check's bound as "explicit panic macros and methods only". That is not the bound
that matters. The one that matters is that the scan reads **lines**, and a panic one call deep is not
a line.

## The region can be truncated and both floors still pass

`:556`. The region ends at the first line whose **tail** is `// startup-path: end`, so a doc comment,
a prose line or a string literal ending in that text closes it.

Measured on the real file: a forged `end` at line 333 leaves `region.len() = 82` — the guard wants
more than 50 — and `accounted = 8` — the guard wants at least 8. **Both floors pass.** 188 lines go
unscanned, including all of `reach_bidi`, `upgrade`, `connect_give_up` and `assigned_bidi_port`: every
line the story is about.

The two floors were chosen to detect a region that went missing. Neither detects one that got
shorter.

## A third, reasoned rather than run

`support/browser.rs:403`. Deleting the `elapsed >= deadline` check in the upgrade-error branch turns
the silent-socket give-up from `stage: connect` into `stage: upgrade (…)`, and nothing notices: the
only case on that path asserts the `fixture environment refusal:` prefix and a 5 s slack on a 1 s
deadline, both of which survive. The adversary reasoned this from the assertions rather than running
it, because demonstrating it requires mutating the file under attack. One line closes it: assert the
stage.

## Acceptance

The guard fails when a panic site is added anywhere a start can reach, including one call below a
line in the region, and fails when the region is made shorter than the code it is supposed to cover.
Every claim in its doc comment is true of what it does.

## Scope

- `crates/edge/ess-cli/tests/coverage_browser.rs` — `cited`
- `crates/edge/ess-cli/tests/support/browser.rs` — `cited`

A general note this story is one instance of: wave 24 caught four separate "by construction" claims
that rested on a hand-written literal or on a line scan. See
`review-result:adversary-wave24-unit2-pass-2` and `review-result:adversary-wave24-unit3-pass-2`.
