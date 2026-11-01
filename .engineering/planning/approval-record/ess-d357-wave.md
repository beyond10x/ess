---
format: aep.planning-md/3
id: approval-record:ess-d357-wave
kind: approval-record
status: approved
title: Wave D357 runs rows 22, 27+10, 19 and 12 of the decided triage as one wave, one pull request and release 0.82.0
summary: Orchestrator approval D357 of 2026-11-01 under the operator's grant
tags:
- non-interactive
relations:
- decides: story:a-refusal-records-the-document-it-was-read-from
- decides: story:a-marked-region-is-not-a-scan-of-what-runs
- decides: story:concurrent-history-records-inputs
- decides: story:a-no-view-arranged-half-probes-the-row-through-the-command
- decides: story:binding-site-identity-comparisons
revision: 4
transitions:
- {from: "draft", to: "approved", at: "2026-11-01T20:19:26Z", actor: "human:timo", revision: 2}
---
## Decision

D357, 2026-11-01: the orchestrator approved the next ESS wave after 0.81.0 under the operator's
grant (class C: a wave, bot commits, pull request and merge, release through ess's own process),
with the operator's delivery rule of 2026-10-28: "work in waves or batches. So we fix multiple
stories against one integration branch, bigger batches, multiple stories, and then we do one PR. We
do not run expensive full gates after every small change. We run targeted tests only for what we're
currently implementing, and only in the end, when everything ended up in the integration branch, we
run a local full gate and then we create the PR and auto merge it to the default main branch." The
release that follows the merge is `0.82.0`. Units come from the adopted rows of the decided
2026-10-31 findings triage (`approval-record:ess-d338-findings-triage`) that waves D343 and D350 did
not take: rows 4 (narrowed), 8, 10, 11, 12, 19, 21, 22, 27; row 9 is blocked on
`decision-blocker:decimal-canonical-serialization-crosses-repositories` and skipped.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs. Skill: aep `implementing` 0.25.0, wave mode.

## Units

Four, on surfaces the store's `waves` verb finds disjoint. Base `main` 78d58fb47 (0.81.0, CI
green). Integration branch `integration/ess-d357`.

- U1 (row 22, low): `story:a-refusal-records-the-document-it-was-read-from`. A refusal on a
  multi-file load names the file its declaration was read from. Surface (cited):
  `crates/specify/ess-domain/src/spec.rs`, `crates/specify/ess-compiler/src/resolve.rs`,
  `crates/specify/ess-compiler/tests/locator_citations.rs`, `crates/edge/ess-cli/src/load.rs`.
- U2 (rows 27 and 10, low; one unit because both are ess-cli test files):
  `story:a-marked-region-is-not-a-scan-of-what-runs` (the startup panic-site guard scans what a
  start reaches, not a marked region; surface `crates/edge/ess-cli/tests/coverage_browser.rs`,
  `tests/support/browser.rs` cited, `tests/browser_startup_refusal_boundary.rs` inferred) and
  `story:concurrent-history-records-inputs` (the behaviour shipped in beyond10x/ess#576 as
  `ess-history/3`; the unit adds the missing regression case to
  `crates/edge/ess-cli/tests/check_history.rs`, inferred).
- U3 (row 19, low): `story:a-no-view-arranged-half-probes-the-row-through-the-command`. Surface
  (cited): `crates/verify/ess-conformance/src/synthesize/existence.rs` `arranged_refusal`,
  `tests/adversary_arrangement_pass2.rs`, `docs/design/outcome-shapes.md`.
- U4 (row 12, medium): `story:binding-site-identity-comparisons`. Surface (cited):
  `crates/verify/ess-conformance/src/synthesize.rs` binding aspects, `tests/adversary_273_pass1.rs`;
  (inferred) `synthesize/binding_effects.rs`, `bounded_retry.rs`, `delivery_context.rs`.

Every unit gates package-scoped through the host's one build slot (host-monitor D95 and D356:
`CARGO_PROFILE_DEV_DEBUG=line-tables-only`, `CARGO_PROFILE_TEST_DEBUG=line-tables-only`,
`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=8`; start only with / at 80G free or more, at most one
other non-telos cargo (two: jobs 4), the tree's `target/` under 15G; a step stops when / falls under
70G). Any build touching ess-conformance (U3, U4, and the ess-cli targets of U1 and U2) runs only on
a slot the orchestrator relays from host-monitor; never an all-targets ess-conformance build.

## Selection

`aep plan artifact waves --kind story --status draft --format json` (aep 0.79.0), after one
`aep:story-scoper` per candidate with a stale scope (rows 10, 19, 22, 27); rows 8, 11, 12 and 21
carry 2026-11-01 scopes from D343 and D350. Verb output for the candidates, verbatim:

- waves: `story:a-marked-region-is-not-a-scan-of-what-runs` wave 1,
  `story:a-no-view-arranged-half-probes-the-row-through-the-command` wave 1,
  `story:a-refusal-records-the-document-it-was-read-from` wave 1,
  `story:an-optional-element-is-appended-or-refused-consistently` wave 1,
  `story:concurrent-history-records-inputs` wave 1,
  `story:row-set-beside-named-rows-suite-tells-every-order-apart` wave 2,
  `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field` wave 5,
  `story:binding-site-identity-comparisons` wave 7.
- collisions among the picked stories: none. Row 12's collisions are all with stories outside this
  wave, for example `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field x
  story:binding-site-identity-comparisons on crates/verify/ess-conformance/src/synthesize.rs
  (inferred)` and `story:a-binding-trigger-acts-on-the-row-its-setup-arranged x
  story:binding-site-identity-comparisons on .../synthesize/binding_effects.rs (inferred)`.
- unassessed among the candidates: none.

Left out:

- Row 4 (`story:an-authored-text-literal-outside-its-alphabet-is-refused`): already implemented
  (f9fa4d12d, closed at D343); no unit.
- Row 8 (`story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field`) collides
  with U4 on `synthesize.rs` (inferred); next wave.
- Row 11 (`story:row-set-beside-named-rows-suite-tells-every-order-apart`, medium): would be a
  third ess-conformance synthesis unit beside U3 and U4 (`related_guard.rs`, `row_set.rs`); next
  wave.
- Row 21 (`story:an-optional-element-is-appended-or-refused-consistently`, low): wave cap of four
  units.
- Row 9 stays blocked.

## Review scope

Adversary passes review each unit's diff against its acceptance only (operator rule of
2026-10-31). Gaps inside the acceptance are fixed in the wave; anything outside is one
out-of-scope row in the pass's review-result, decided by the coordinator at wave close (adopt as a
story, decline with reason, duplicate) and reported as counts.

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-d357`, the store commits on that branch, the release-prepare commit (version
0.82.0, lock files, licences, CHANGELOG, change record) on that branch before the final gate, one
pull request to `main` from a `queue/` branch through the App and its merge on a green pipeline,
then the release tag and publication through the repository's own process, and the deletion of the
wave's merged remote branches through the bot. Not the next wave.

Decider: operator

## Wave record

Scratch root `~/.cache/ess-d357-wave/`; trees under the worktree manager's `b10x/ess/` root.

| unit | tree id | branch | build dir | scratch | stage |
|---|---|---|---|---|---|
| int | ess-d357-int | integration/ess-d357 | tree `target/` | `~/.cache/ess-d357-wave/` | open |
| U1 | ess-d357-u1 | unit/ess-d357-u1 | tree `target/` | `~/.cache/ess-d357-wave/u1/` | planned |
| U2 | ess-d357-u2 | unit/ess-d357-u2 | tree `target/` | `~/.cache/ess-d357-wave/u2/` | planned |
| U3 | ess-d357-u3 | unit/ess-d357-u3 | tree `target/` | `~/.cache/ess-d357-wave/u3/` | planned |
| U4 | ess-d357-u4 | unit/ess-d357-u4 | tree `target/` | `~/.cache/ess-d357-wave/u4/` | planned |

Pre-flight 2026-11-01: / 102G free of 848G; no other cargo process machine-wide; load1 1.0.

## Build slots

Host build slots the orchestrator relayed for this wave, 2026-11-01:

- Host-monitor D137, relayed as orchestrator D358: "Slot granted (host-monitor D137, my D358). The
  steps you listed for U1 phase 2, U2, U3 and U4 may start now." Requested: U2 ess-cli
  `coverage_browser`, `browser_startup_refusal_boundary`, `browser_startup_slow_serve_boundary`,
  `check_history` and ess-conformance `adversary_w3_389_s2_pass1 --no-run` (~8G); U3 ess-conformance
  `--lib` and five named `--test` targets (~5G); U4 ess-conformance `--lib` and eleven named
  `--test` targets plus up to ten binding targets named first (~8G); U1 phase 2 ess-cli
  `validate_parse_refusals` and one new file (~6G).
- Terms: one cargo at a time through the wave lock and the host's build slot, jobs 8,
  `CARGO_INCREMENTAL=0`, line tables only; before each command / at 80G free or more and at most one
  other cargo (telos and telos-rustgen excluded while load1 < 16 and io avg60 < 5); each tree's
  `target/` at most 15G, discard-cache when its unit merges; a step stops when / falls under 70G; no
  all-targets build; the final gate is its own request. One line per unit to host-monitor: start,
  end, peak `target/`, lowest /.
- Orchestrator D361 (host-monitor), 2026-11-01: slot granted for the four adversary passes under the D137 terms; each pass runs in its unit tree, serially under the wave lock, `adversary_d357_uN_pass1` plus that unit's granted targets, 2-4G per tree and never over 15G; one line per pass to host-monitor.
- Orchestrator D359 (host-monitor D138): hold from 20:34Z while the substrate D354 full gate ran; D360 go at 20:46:07Z with the U4 extension (the `ess` binary and suite regeneration, then 13 pin tests in ess-conformance, ess-cli and ess-diff).
