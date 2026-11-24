---
format: aep.planning-md/3
id: story:the-slow-nextest-profile-kills-a-test-at-1200-s-while-adv-e-u6-byte-identity-probe-is-marked-over-1318-s-and
kind: story
status: draft
title: 'The slow nextest profile kills a test at 1200 s while adv_e_u6_byte_identity_probe is marked over 1318 s and its sibling 729 s: the two probes are split per model so no slow-lane test runs over 60 s'
summary: Two ess-synth byte-identity probes loop over every model in one test; one is marked over the profile kill time, the other twelve times the 60 s bound, and neither asserts a byte
tags:
- issues-2026-11-24
- test-budget
- wave-2026-11-23
relations:
- serves: vision:O2
- derived_from: story:the-test-step-meets-the-test-budget
revision: 2
---
## Outcome

No test of the `slow` lane is killed by its profile's timeout, and none runs over 60 s: the two
byte-identity probes of `ess-synth` are split into cases of one model (or one small group of
models) each, or read one shared fixture, so each case ends well inside the 60 s of the
organization's test budget. The kill time of the profile is not raised.

## Need

**Reading.** Verified at the wave head `b2cdf564b` (`origin/integration/wave-2026-11-23`); the
slow lane is not on `main` yet.

- `.config/nextest.toml:26-27`: `[profile.slow]` has `slow-timeout = { period = "120s",
  terminate-after = 10 }`, which kills a test at 1200 s.
- `crates/generate/ess-synth/tests/adversary_e_u6_bytes.rs:161`: the test
  `adv_e_u6_byte_identity_probe` is marked `#[ignore = "slow lane: byte identity of every model's
  generated Rust and Go, over 1318 s at load 89"]`. 1318 s is over the 1200 s kill: run in the
  `slow` profile the test is terminated before it ends. The adversary review of the wave close
  recorded it as finding F8 ("read, not run"); the figure is the marker's own, measured at load 89,
  not a CI time.
- The sibling `adv_e_u2_byte_identity_probe` (`adversary_e_u2_bytes.rs:109`) is marked 729 s at the
  same load: under the kill, and over twelve times the 60 s bound.
- Both are one test that loops over every YAML file of the repository whose first line is
  `format: ess/<n>`, as written and relabelled `ess/22`, and hashes several products of each
  (`adversary_e_u6_bytes.rs:161` on). The model set is the shared one of
  `adversary_selection_plan_w3_u2_pass1.rs:266`. The test asserts only that it read more than 50
  models; the hashes are written to the file named by `ADV_E_U6_PROBE_OUT` for a comparison of two
  trees that no file of the repository at this head runs (only the probes read the variable). So the cost is a loop of independent, equal-shaped
  items inside one test, which is the shape the rule "an expensive product is made once or split"
  names.
- The `slow` job has 30 minutes including a cold compile (`ci.yml`, job `slow`,
  `timeout-minutes: 30`); the probes' CI times were not measured.

## Acceptance

`task test-slow PACKAGES=ess-synth` on the build node, in a fresh target directory, reports no
test over 60 s in its ten slowest and no test killed by `terminate-after`, with
`adv_e_u6_byte_identity_probe` and `adv_e_u2_byte_identity_probe` replaced by per-model cases whose
list `cargo xtask slow-list` still names.

## Decisions

- **No longer timeout.** The organization's rule gives no test over 60 s; the slow lane keeps its
  profile as it is.
- **Split or fixture, by measurement.** The first step is the time of one model's line
  on the build node: if the mean is a few seconds, one case per model (a macro or a
  list of `#[test]` functions) is the form; if the shared work dominates, one
  fixture built once per run under `target/`, keyed by the digest of its sources, read by every
  case. A process-global `OnceLock` is not the fixture under nextest.
- **Whether the probes stay.** They assert no bytes; the repository-wide byte pins are the
  `synthesis_suite_bytes_table` family and the committed trees. Retiring the two probes is the
  third option, left open here.

## Out of scope

- The other marked tests of `slow_lane.rs`'s `MARKED` list: their times are not measured here.
- The `slow` job's `timeout-minutes` and the skip decision of the job.

## Scope

- `crates/generate/ess-synth/tests/adversary_e_u6_bytes.rs`
- `crates/generate/ess-synth/tests/adversary_e_u2_bytes.rs`
- `crates/generate/ess-synth/tests/adversary_selection_plan_w3_u2_pass1.rs` (the shared model set)
- `.config/nextest.toml` (unchanged; named so a change is visible)
