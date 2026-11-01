---
format: aep.planning-md/3
id: story:a-marked-region-is-not-a-scan-of-what-runs
kind: story
status: active
title: A marked region is not a scan of what runs
relations:
- serves: vision:O2
scope:
- confidence: inferred
  path: crates/edge/ess-cli/tests/browser_startup_refusal_boundary.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/coverage_browser.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/support/browser.rs
revision: 7
transitions:
- {from: "draft", to: "proposed", at: "2026-11-01T20:19:28Z", actor: "human:timo", revision: 6}
- {from: "proposed", to: "active", at: "2026-11-01T20:19:28Z", actor: "human:timo", revision: 7}
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

Derived 2026-11-01 by `story-scoper` against `integration/ess-d357` at 78d58fb47 (ESS 0.81.0). Every
line is **cited** (read from the story or the tree) or **inferred** (a reading that could be wrong).
The story's line numbers are from an older file; the current ones are below.

- **Primary surface:** `crates/edge/ess-cli/tests/coverage_browser.rs`, the guard — cited
- **Files:** `crates/edge/ess-cli/tests/coverage_browser.rs:551-612`: doc comment :551-556, `no_unaccounted_panic_site_can_end_a_start` :558, `marks` tail match :561, begin/end scan :565-573, floor `region.len() > 50` :575, floor `accounted >= 8` :608 — cited
- **Files:** `crates/edge/ess-cli/tests/support/browser.rs:388-400` (`// startup-path: begin` and the region's prose) and `:693` (`// startup-path: end`) — cited
- **Files:** `support/browser.rs:403-692`, the region today: `Browser::launch_program` :403, `reach_bidi` :494, `upgrade` :605, `connect_give_up` :631, `assigned_bidi_port` :650 — cited
- **Files:** `support/browser.rs`, callees the region reaches that the scan never reads: `ProfileScratch::create` :160 (`.unwrap()` :168), `Starting::begin` :273, `InStartup::begin` :283 and its `Drop` :289, `OwnedChild::stop` :183, `startup_refusal` :344, `Stage::label` :323 / `timed_a_startup` :333, and via `browser.call("session.new", …)` at :486: `Browser::call` :695, `send` :715, `receive` :736 — cited
- **Callers into the region:** `Browser::new` :381 → `launch` :384 → `launch_program`; called from `coverage_browser.rs:619,707,740`, `browser_startup_refusal_boundary.rs:61`, `browser_startup_slow_serve_boundary.rs:83,409`; `launch` at `coverage_browser.rs:412` — cited
- **Defect 1 today (one call down):** callees hold unscanned, unmarked panic sites at :698, 706, 707, 708, 710 (`call`), :718, 721, 724, 734 (`send`), :738, 739, 740, 744, 749, 754, 755, 756 (`receive`), plus :168 under the harness mark at :414 — inferred (counted by reading)
- **Defect 2 today (truncation):** the eighth accounted site is :474, so a forged `// startup-path: end` anywhere in :477-692 still passes both floors — inferred
- **Defect 3 (stage not asserted):** `browser.rs:556-567`; after the retry rewrite the mutation no longer changes the stage; asserting `stage: connect` in the silent-socket case is still a true one-line hardening — inferred
- **Also likely:** `crates/edge/ess-cli/tests/browser_startup_refusal_boundary.rs:134-142` (add the stage assert) — inferred
- **Also compiled:** `crates/verify/ess-conformance/tests/adversary_w3_389_s2_pass1.rs:6` includes `support/browser.rs` via `#[path]` and calls `Browser::new` at :142; must still compile, no edit — cited
- **Design (guard):** drop the markers; a syn-free scanner in `coverage_browser.rs` finds same-file `fn` items (body cut at the first later line at the `fn` line's own indentation followed by `}`), takes the closure over same-file calls from `launch_program` (incl. `Drop` impls of named types), applies the existing site forms and marks to every reached line; floors become: root found, and synthetic sources (a panic one call down, a forged end marker) go red — inferred (proposal)
- **Design (forced by the guard):** ~18 existing sites get flagged: `send`/`receive` return `io::Result`, a fallible exchange carries `session.new`, `launch_program` turns its I/O error into `startup_refusal`; protocol asserts marked `defect`, receipt/profile filesystem sites `harness` — inferred
- **Confidence:** high for the surface; medium for the site count and defect-3 staleness
- **Would collide with:** units touching `crates/edge/ess-cli/tests/support/browser.rs`, `coverage_browser.rs`, or `browser_startup_refusal_boundary.rs`
- **Safety fact:** `support/browser.rs` is compiled only into test targets (`no_test_builds.rs:124` lists it `NEVER_ALLOWED`), so no shipped binary or format changes — inferred
