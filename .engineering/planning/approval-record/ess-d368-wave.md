---
format: aep.planning-md/3
id: approval-record:ess-d368-wave
kind: approval-record
status: approved
title: Wave D368 runs rows 21 and 11 of the decided triage and three D357 adoptions as one wave, one pull request and release 0.83.0
summary: Orchestrator approval D368 of 2026-11-02 under the operator's grant
tags:
- non-interactive
relations:
- decides: story:an-optional-element-is-appended-or-refused-consistently
- decides: story:a-no-view-held-state-refusal-probes-the-record
- decides: story:a-no-view-probe-tells-from-states-apart
- decides: story:a-refusal-witness-compares-its-trigger-identities
- decides: story:row-set-beside-named-rows-suite-tells-every-order-apart
revision: 8
transitions:
- {from: "draft", to: "approved", at: "2026-11-02T07:12:04Z", actor: "human:timo", revision: 2}
---
## Decision

D368, 2026-11-02: the orchestrator approved the next ESS wave after 0.82.0 under the operator's
grant (class C: a wave, bot commits, pull request and merge, release through ess's own process),
with the operator's delivery rule of 2026-10-28: "work in waves or batches. So we fix multiple
stories against one integration branch ... We run targeted tests only for what we're currently
implementing, and only in the end, when everything ended up in the integration branch, we run a
local full gate and then we create the PR and auto merge it to the default main branch." The
release that follows the merge is `0.83.0`. Units come from the adopted rows of the decided
2026-10-31 findings triage (`approval-record:ess-d338-findings-triage`) still open (rows 8, 11, 21;
row 4 was closed as implemented at D343; row 9 stays blocked on
`decision-blocker:decimal-canonical-serialization-crosses-repositories`) and the five draft stories
adopted at the close of wave D357 (`approval-record:ess-d357-wave` § Close). Every candidate was
`draft` on `origin/main` 6437975cb.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs. Skill: aep `implementing` 0.25.0, wave mode.
Agents: `aep:story-scoper`, `aep:implementor`, `aep:adversary`, `aep:security-reviewer` (no unit
touches credentials, auth or confinement, so no security pass is planned).

## Units

Four units, five stories. Base `main` 6437975cb (0.82.0, CI 41375102591 green). Integration branch
`integration/ess-d368`.

- U1 (row 21, low): `story:an-optional-element-is-appended-or-refused-consistently`. Fit decided
  here: option B, the interpreter appends `null` for an absent or null element of an Optional
  element type, and synthesis expects it. Reason: the domain has admitted the read since ess/31
  (U10) and the generated Go and Rust already append `nil`/`None` (scope, cited lines), so B moves
  the interpreter to shipped behaviour with no format major; option A refuses a specification that
  validates today, which by the `stricter:` precedent in `system.rs` needs a new major. Surface:
  `interpret/execute/determined.rs` `append`, `synthesize.rs` `appended_at`, the comments in
  `command/append_list.rs` and `system.rs`, `tests/append_set.rs`.
- U2 (two stories on one surface, low): `story:a-no-view-held-state-refusal-probes-the-record` and
  `story:a-no-view-probe-tells-from-states-apart`. One unit because both rewrite `probe` in
  `synthesize/existence.rs` and the same texts (`docs/design/outcome-shapes.md` § Conformance), so
  they cannot run in parallel. Decided here: the held-state story's third acceptance line holds for
  every model with an identity view and every committed suite; the test fixture
  `occurrence-clock-offers.yaml` (no views) gains the probe, so its pinned rows are re-pinned and the
  model is named in the CHANGELOG line, as the tells-apart story's third line allows.
- U3 (low): `story:a-refusal-witness-compares-its-trigger-identities`. Surface:
  `synthesize/refusal_policy.rs` `witness`, `synthesize/bounded_retry.rs` `clean_trigger` /
  `clean_publisher`, `tests/adversary_273_pass1.rs`; five synthesis bytes rows expected to move, no
  committed suite.
- U4 (row 11, medium): `story:row-set-beside-named-rows-suite-tells-every-order-apart`. Surface:
  `synthesize/row_set.rs`, `synthesize/related_guard.rs`, possibly `synthesize/precedence.rs`,
  `tests/adversary_ess0700a_identity_beside_row_set_2.rs`. Both shapes reproduce on 0.82.0; the four
  named cases were never committed, so the unit writes them.

Every unit gates package-scoped through the host's one build slot
(`CARGO_PROFILE_DEV_DEBUG=line-tables-only`, `CARGO_PROFILE_TEST_DEBUG=line-tables-only`,
`CARGO_INCREMENTAL=0`, `CARGO_BUILD_JOBS=8`; start only with / at 80G free or more and at most one
other cargo, telos and telos-rustgen excluded while load1 < 16 and io avg60 < 5; the tree's
`target/` under 15G; a step stops when / falls under 70G). Every unit builds ess-conformance, so
each runs only on a slot the orchestrator relays from host-monitor, named targets only, never an
all-targets ess-conformance build.

## Selection

One `aep:story-scoper` per candidate except row 8 (its D343 scope of 2026-11-01 stands); the scope
sections and typed entries are written to each story. Then `aep plan artifact waves --kind story
--status draft --format json` (aep 0.79.0).

The first run reported the candidates colliding on `CHANGELOG.md`, the synthesis bytes tables and
`synthesize.rs` (inferred). The coordinator removed those typed entries where they did not name an
edited surface: `CHANGELOG.md` and the bytes tables are shared files every synthesis unit adds a line
to or re-pins its own rows in, merged by the coordinator (the convention of earlier waves'
scopes); `synthesize.rs` was removed where the scoper named it only as the home of helpers the unit
calls (held-state, tells-apart, refusal witness). Row 21 keeps `synthesize.rs` (`appended_at` is
edited). The prose Scope sections keep every line. Second run, verbatim for the candidates:

- waves: `story:a-browser-start-bounds-session-new-by-its-deadline` wave 1,
  `story:a-no-view-held-state-refusal-probes-the-record` wave 1,
  `story:a-refusal-witness-compares-its-trigger-identities` wave 1,
  `story:a-no-view-probe-tells-from-states-apart` wave 2,
  `story:row-set-beside-named-rows-suite-tells-every-order-apart` wave 2,
  `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field` wave 5,
  `story:an-eventual-event-compares-a-captured-identity` wave 7,
  `story:an-optional-element-is-appended-or-refused-consistently` wave 8.
- collisions among the candidates:
  `story:a-no-view-held-state-refusal-probes-the-record x story:a-no-view-probe-tells-from-states-apart on crates/verify/ess-conformance/src/synthesize/existence.rs (cited)`;
  the same pair on `docs/design/outcome-shapes.md (cited)`;
  `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field x story:an-eventual-event-compares-a-captured-identity on crates/verify/ess-conformance/src/synthesize.rs (inferred)`;
  `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field x story:an-optional-element-is-appended-or-refused-consistently on crates/verify/ess-conformance/src/synthesize.rs (inferred)`;
  `story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field x story:row-set-beside-named-rows-suite-tells-every-order-apart on crates/verify/ess-conformance/src/synthesize/row_set.rs (cited)`;
  `story:a-refusal-witness-compares-its-trigger-identities x story:an-eventual-event-compares-a-captured-identity on crates/verify/ess-conformance/tests/adversary_273_pass1.rs (cited)`;
  `story:an-eventual-event-compares-a-captured-identity x story:an-optional-element-is-appended-or-refused-consistently on crates/verify/ess-conformance/src/synthesize.rs (inferred)`.
- unassessed among the candidates: none. Cycles: none.

The wave numbers differ because each story collides with others outside this set; among the four
units the only collision is the held-state x tells-apart pair, which is one unit by design.

Left out:

- Row 8 (`story:a-parent-tenant-guard-holds-across-every-related-row-and-absent-field`) collides with
  U4 on `row_set.rs` (cited) and with U1 on `synthesize.rs` (inferred); next wave.
- `story:an-eventual-event-compares-a-captured-identity`: large (a new suite format major across the
  Rust, Go and TypeScript runners, about 700-1200 lines) and collides with U3 on
  `adversary_273_pass1.rs`; next wave, as its own unit.
- `story:a-browser-start-bounds-session-new-by-its-deadline` (small, ess-cli test fixture only): the
  cap of four units.

## Review scope

Adversary passes review each unit's diff against its acceptance only (operator rule of
2026-10-31). Gaps inside the acceptance are fixed in the wave; anything outside is one out-of-scope
row in the pass's review-result, decided by the coordinator at wave close (adopt as a story,
decline with reason, duplicate) and reported as counts.

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-d368`, the store commits on that branch, the release-prepare commit (version
0.83.0, lock files, licences, CHANGELOG, change record) on that branch before the final gate, one
pull request to `main` from a `queue/` branch through the App and its merge on a green pipeline,
then the release tag and publication through the repository's own process, and the deletion of the
wave's merged remote branches through the bot. Not the next wave.

Decider: operator

## Wave record

Scratch root `~/.cache/ess-d368-wave/`; trees under the worktree manager's `b10x/ess/` root.

| unit | tree id | branch | build dir | scratch | stage |
|---|---|---|---|---|---|
| int | ess-d368-int | integration/ess-d368 | tree `target/` | `~/.cache/ess-d368-wave/` | open |
| U1 | ess-d368-u1 | unit/ess-d368-u1 | tree `target/` | `~/.cache/ess-d368-wave/u1/` | planned |
| U2 | ess-d368-u2 | unit/ess-d368-u2 | tree `target/` | `~/.cache/ess-d368-wave/u2/` | planned |
| U3 | ess-d368-u3 | unit/ess-d368-u3 | tree `target/` | `~/.cache/ess-d368-wave/u3/` | planned |
| U4 | ess-d368-u4 | unit/ess-d368-u4 | tree `target/` | `~/.cache/ess-d368-wave/u4/` | planned |

Pre-flight 2026-11-02 07:10Z: / 83G free of 848G; no cargo process machine-wide; load1 1.1. The
trees of earlier waves (D325 to D357, issues-1 to issues-3) are still registered with the worktree
manager, finished or idle, each `target/` at 4K or absent.

## Build slots

Host build slots the orchestrator relayed for this wave, 2026-11-02:

- Orchestrator D369 (host-monitor grant): "Go (D369, host-monitor grant): ess-conformance named
  targets for U1-U4 as requested, serial under one wave lock, jobs 8, CARGO_INCREMENTAL=0 +
  line-tables-only. Before each command: / >= 80G and <= 1 other cargo ... with 2 others start
  nothing. Stop under 70G. Tree <= 15G. discard-cache at each unit merge." Requested: U1
  ess-conformance `--lib`, `append_set`, `synthesis_suite_bytes_table` and ess-domain `append_set`;
  U2 `--lib` and nine named targets; U3 `--lib` and eleven; U4 `--lib` and seven; clippy on those
  targets and `task fmt-check`. Each tree 3-6G expected.
- Orchestrator, 07:3xZ: "U4's 2 extra ess-conformance targets (row_set_many_selectors_work,
  synthesis attempt counters) are approved under D369's terms." U4 adds `row_set_many_selectors_work`
  and `synthesis` to its seven, because its fix adds a `row_set.rs` attempt those two count.
- Holds: PRECHECK-FAIL at 79G (U1 07:20:53Z, U3 07:21:44Z, U4 07:28:31Z) and 78G (U1 07:30:50Z,
  resumed at 80G); the second refusal on one reason stopped the wave until the orchestrator resumes it
  at / >= 80G with margin.
- Orchestrator, 07:4xZ: "history_format2 for U2 is approved under D369's terms." U2 adds
  `history_format2`, which builds the `occurrence-clock-offers` suite whose bytes U2 moves.
- Orchestrator D373 (host-monitor grant): "6 passes (adversary + security for U1, U3, U4), serial
  under the wave lock behind U2, named ess-conformance targets only, tree <= 15G, discard-cache
  after each pass." The passes run in one tree per unit (`ess-d368-<unit>-adv`) off the integration
  head 7e0d2e880; the named targets are in each unit's review brief.
- Orchestrator D381 (host-monitor grant), 09:2xZ: unit builds and review passes pre-check at
  / >= 75G and stop under 65G, one cargo at a time and only with 0 other cargo on the host (no telos
  exemption); tree <= 15G, discard at merge; the final gate stays a separate request at 80G. The
  wave wrapper was changed to these terms (backups `cargo-in.sh.d369`, `recheck.sh.d369`).
- Holds: PRECHECK-FAIL at 75-77G at 09:02:15-20Z stopped every pass and U2; reported once.
- D387 (orchestrator, 11:14Z): / >= 80G allows 1 other cargo (stop under 70G); 75-80G allows
  none (stop under 65G); under 75G nothing starts. The aap gate hold (11:20Z to 11:32Z host clock)
  stopped all wave cargo.
- D388 (host-monitor grant, 12:21Z; basis: Timo 2026-10-29, "dips to ~60G during gates
  acceptable"): one targeted run at a time, 0 other cargo, / >= 68G before each run, stop below
  60G. Given for U2 first, U4 on a separate go. Wrapper backups `cargo-in.sh.d387`, `recheck.sh.d387`.
