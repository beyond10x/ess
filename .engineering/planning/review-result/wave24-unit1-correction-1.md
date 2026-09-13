---
format: aep.planning-md/1
id: review-result:wave24-unit1-correction-1
kind: review-result
status: active
title: 'Unit 1 correction round 1: green, with the refusal boundary left open'
relations:
- reviews: story:browser-fixture-startup-deadline
revision: 1
---
# Unit 1, correction round 1: green, with one tension left open on purpose

Worktree `wt-1933f8986f66`, branch `impl/browser-fixture-startup-deadline`, uncommitted over base
`bd722fa964bd225b9755f272e22b45fab334449f`.

```
 crates/edge/ess-cli/tests/coverage_browser.rs | 334 +++++++++++++++-
 crates/edge/ess-cli/tests/support/browser.rs  | 435 ++++++++++++++++++----
 2 files changed, 701 insertions(+), 68 deletions(-)
```

Plus the adopted pass-1 file, renamed `tests/browser_startup_refusal_boundary.rs` (180 lines).

## Gate

```
TMPDIR=<scratch> cargo test --locked -p ess-cli --no-fail-fast   exit 0   543 passed, 0 failed
cargo clippy --locked --package ess-cli --all-targets -- -D warnings      exit 0
cargo fmt --package ess-cli -- --check                                    exit 0
```

56 `test result:` lines, 0 occurrences of `FAILED`.

| Lane | Cases | Wall clock |
|---|---|---|
| whole package | 542 → 543 | — |
| `browser_startup_refusal_boundary` | 3 red → 3 green | 22.36 s → 3.13 s |
| `coverage_browser` | 9 → 10 | — |
| `replay_fidelity_browser` | 30 → 30 | 152.08 → 64.73 s, not attributed (see F5) |
| `coverage_writer_adversary_pass1` / `pass2` | 2, 1 | — |

## The tension the unit reported rather than resolved

**F3 may have moved the story's own evidence onto the wrong side of the boundary.**

The story exists because CI failed with `Firefox did not expose BiDi; read firefox.stderr` at base
`browser.rs:151`. I told the unit to stop reporting a browser that answered HTTP as a runner
failure. It implemented that, and then said the story's originally-observed input **is this same
code path**, so the new rule may push it onto the non-refusal side.

It did not pick a side and did not restore the old denial. The give-up now states that the runner did
start a browser, and that the response and the log decide whether `/session` is defective or the
runner never gave it CPU.

Three states, and the third is the gap:

| State | Should be |
|---|---|
| Firefox slow to start, nothing listening at the deadline | fixture environment refusal — the story |
| Firefox listening, answers HTTP 404 on `/session` forever | not the runner's fault |
| Firefox listening but slow to serve `/session`, deadline expires mid-request | **unknown** |

Adversary pass 2 was asked to build the third row and settle whether "did anything answer HTTP" is
the right discriminator for the defect the story was filed about.

## Findings, each answered as a class

**F1 — the sixth give-up site, and the class made machine-checkable.** The upgrade read, write and
socket-option calls go through `Browser::upgrade -> io::Result`; a failure is `stage: exited` (child
gone) or `stage: upgrade (<io error>)`. `String::from_utf8().unwrap()` became `from_utf8_lossy`.

The startup path is bracketed by `// startup-path: begin` / `// startup-path: end`, and
`no_unaccounted_panic_site_can_end_a_start` scans that region of `include_str!("support/browser.rs")`
for `.unwrap()`, `.expect(`, `panic!`, `assert*!`, `unreachable!`, `todo!`, failing unless the line
or the one above ends with `// startup-path: harness` or `// startup-path: defect`. 15 sites, 0
unaccounted: 11 `harness`, 3 `defect`, plus the receipt file.

The unit stated the check's bound rather than leaving it implied: explicit panic macros and methods
only. It enumerated index, slice and arithmetic panics in the region separately and reports the only
candidate is `byte[0]` on a 1-element array. Two sites are exempt on purpose — `Browser::new`'s
`panic!("{refusal}")`, which is how a refusal reaches the runner, and everything after
`startup-path: end`, which is post-readiness BiDi traffic where a panic is the right signal.
`grep -rn -e ESS_FIREFOX -e remote-debugging-port crates/ --include '*.rs'` finds nothing outside
this file, bounding the class to one file.

**F2.** `set_read_timeout`/`set_write_timeout` take `deadline.saturating_sub(elapsed).max(1ms)`;
expiry returns a refusal. The 1.000 s-deadline case now returns inside its deadline; the lane went
22.36 s → 3.13 s.

**F4.** `Starting` holds only the mutex; a separate `InStartup` guard spans announce+connect inside
`reach_bidi`, so releasing `STARTUP` early leaves real startups overlapping and the peak says so.
The case drives three 0.5 s stand-in startups behind a barrier and asserts `peak == 1` **and**
`elapsed >= 3×window − 200 ms`. The adversary's mutant is red at `left: 5, right: 1`. Three extra
runs for flakiness: 5.02 / 4.66 / 4.62 s.

**F5 — restated with a measured number.** **84 launch attempts per package run**, counted by diffing
the profile-directory inventory across a run: 69 `replay_fidelity_browser`, 10 `coverage_browser`,
3 the boundary lane, 1 + 1 the writer adversaries. 75 populate a real profile; the correction
converted the 3 serialisation-gate starts into shell stand-ins. Uncontended
`spawn → "WebDriver BiDi listening"`: 0.295 / 0.295 / 0.339 / 0.408 / 0.432 s, 5 samples at load
8.4 — about 27 s of serialised startup in a run whose wall clock is about 1,290 s.

The unit's answer on the acceptance OR: serialisation is still the right branch **on this evidence**,
because a start costs about 0.35 s and the deadline is per-start rather than a shared budget. What it
says it cannot close is the number that decides it — the cost of an uncontended start on the CI
runner, unmeasured, with a worst case linear in it (at 30 s/start, about 38 minutes). It names the
lever that removes the linearity as the start count, not serialisation:
`replay_fidelity_browser.rs:275-277` takes 69 starts for 30 tests because `browse()` calls
`Browser::new` per call. It declined to change that — not its file — and called it a separate story.

It also declined to attribute `replay_fidelity_browser`'s 152.08 → 64.73 s to its own change: the
lane measured 38.34 / 42.55 / 49.28 / 64.73 s across four runs under varying load on a machine
carrying other agents.

**F7.** The stage literal is gone. `Stage<'_>` is an enum; the case walks `Stage::ALL` and checks it
against the variants parsed out of the enum's own source via `include_str!`.

**F8.** `spawn (…)` prints `measured startup: not timed; the browser was never spawned` and no
`deadline:` line. Every other stage prints `measured startup: X (spawn to give-up)` and
`deadline: Y (expired)` or `(not reached; this start ended for the reason above)`. The assertion that
pinned the noise is replaced by one holding the label.

## Three mutants the unit ran against itself, each red

| Mutant | Case | Result |
|---|---|---|
| `drop(starting)` moved to just after `command.spawn()` — the adversary's own, which left 542 green | `fixtures_never_start_more_than_one_firefox_at_a_time` | FAILED, `left: 5, right: 1` |
| an unmarked `.unwrap()` added inside the startup region | `no_unaccounted_panic_site_can_end_a_start` | FAILED, `browser.rs:381 can end a start …and names no reason` |
| a sixth `Stage` variant added, not listed in `ALL` | `a_startup_refusal_attaches_the_stderr_firefox_actually_wrote` | FAILED, `Stage::ALL names 5 stages and the fixture declares 6` |

## Scope confirmed

The story's one `inferred` scope line — "the three tests above … they share the fixture" — is
correct. All three are in `coverage_browser.rs` and use `browser::Browser::new`:
`actual_browser_and_rust_refuse_every_closed_model_field_boundary:15`,
`retained_legacy_player_bytes_still_replay_in_actual_firefox:132`,
`actual_browser_admits_the_pair_before_creating_replay_state:223`.

## Reported against itself

One command wrote `/tmp/x` (33 kB, a stray redirect in a counting pipeline), against the standing
rule and the brief. The unit deleted it and confirmed removal. Its scratch directory reached 11 GB,
almost all of it the abandoned Firefox profiles of `story:the-browser-fixture-abandons-a-profile-per-start`;
it left it rather than clean up, as instructed, and said so. The coordinator reclaimed it.

## Start count: 81 or 84

Pass 1 measured 81 launch attempts; the correction measures 84 and attributes 3 of the difference to
the serialisation-gate stand-ins it added. Pass 2 was asked to settle it.
