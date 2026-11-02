---
format: aep.planning-md/3
id: approval-record:ess-w1-wave
kind: approval-record
status: approved
title: Wave W1 runs units GEN1, UI1, SYN1 and RUN1 of the 2026-11-02 reconcile roadmap with the H0 store hygiene as one wave, one pull request and release 0.84.0
summary: Orchestrator approval D394 of 2026-11-02 under the operator's grant
tags:
- non-interactive
relations:
- decides: story:the-generated-rust-server-refuses-what-its-openapi-refuses
- decides: story:a-json-command-body-refuses-an-unknown-member
- decides: story:a-generated-ui-calls-the-routes-its-http-binding-serves
- decides: story:account-menu-entries-keep-sets-and-navigate
- decides: story:a-record-action-in-the-terminal-renderer-sees-its-row
- decides: story:a-row-actions-confirm-is-addressed-by-the-views-key
- decides: story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain
- decides: story:grant-denied-is-synthesized-beside-a-ranked-view
- decides: story:a-system-issued-awaited-step-honours-its-outcomes-input-guard
- decides: story:a-union-variant-scenario-reads-each-row-under-its-own-identity
- decides: story:expect-error-checks-generated-field-presence
- decides: story:a-view-read-checks-every-rows-shape
- decides: story:view-and-error-assertions-carry-their-value-shape
- decides: story:authored-success-step-asserts-no-error
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-11-02T20:39:26Z", actor: "human:timo", revision: 2}
---
## Decision

D394, 2026-11-02 20:40Z: the orchestrator dispatched ESS wave W1 of the 2026-11-02 reconcile
roadmap (section 4, rows W1) under the operator's grant (class C: a wave, bot commits, a bot pull
request and its merge on green, a release through ess's own process), with the operator's delivery
rule of 2026-10-28: "work in waves or batches. So we fix multiple stories against one integration
branch ... We run targeted tests only for what we're currently implementing, and only in the end,
when everything ended up in the integration branch, we run a local full gate and then we create the
PR and auto merge it to the default main branch." The release that follows the merge is `0.84.0`.

The store hygiene H0 of the same roadmap (section 2, 2a-2i) rides on the W1 integration branch as one
store-only commit, with the roadmap's decisions D1-D11 taken as recommended (orchestrator D376,
2026-11-02 09:50Z: "decisions taken as recommended: D1 A digest-only + name override as P2; D2 A
archive after one #268 check; D3 B retained compat rule, archive; D4 A red-first test; D5 A uphold
#234; D6 A archive interpreted-absent-optional-comparison-is-false; D7 A decompose typed-open-questions
before W6; D8 B leave in P2; D9 A keep stale active with dated note; D10 decided by W3 lead from the
story options before dispatch; D11 A Decimal writer in W5"). Issue closes in 2c go through the bot,
after the merge.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs. Skill: aep `implementing`, wave mode. Agents:
`aep:story-scoper`, `aep:implementor`, `aep:adversary`, `aep:security-reviewer`.

## Units (as dispatched by D394; the selection below may move a unit to W2)

Base `main` b81deac78 (0.83.0, CI run 41508375860 green). Integration branch `integration/ess-w1`.
Wave cap four units.

- GEN1: `story:the-generated-rust-server-refuses-what-its-openapi-refuses` (beyond10x/ess#707, P0),
  `story:a-json-command-body-refuses-an-unknown-member`.
- UI1: `story:a-generated-ui-calls-the-routes-its-http-binding-serves` (beyond10x/ess#713, P0),
  `story:account-menu-entries-keep-sets-and-navigate`,
  `story:a-record-action-in-the-terminal-renderer-sees-its-row`,
  `story:a-row-actions-confirm-is-addressed-by-the-views-key`.
- SYN1: `story:a-denied-scenario-covers-every-ungranted-actor-of-the-domain`,
  `story:grant-denied-is-synthesized-beside-a-ranked-view` (beyond10x/ess#712),
  `story:a-system-issued-awaited-step-honours-its-outcomes-input-guard`,
  `story:a-union-variant-scenario-reads-each-row-under-its-own-identity`.
- RUN1: `story:expect-error-checks-generated-field-presence`,
  `story:a-view-read-checks-every-rows-shape` (beyond10x/ess#680),
  `story:view-and-error-assertions-carry-their-value-shape`,
  `story:authored-success-step-asserts-no-error`.

Exit criterion (roadmap section 4): #707, #713, #712, #680 closed; 9 of 14 P0 implemented; a
generated Rust server refuses every body its OpenAPI refuses.

## Review scope

Adversary and security passes review each unit's diff against its acceptance only (operator rule of
2026-10-31). Gaps inside the acceptance are fixed in the wave; anything outside is one out-of-scope
row in the pass's review-result, decided by the coordinator at wave close (adopt as a story,
decline with reason, duplicate) and reported as counts.

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-w1`, the store commits on that branch (the opening record, H0, evidence, close), the
release-prepare commit (version 0.84.0, lock files, licences, CHANGELOG, change record) on that
branch before the final gate, one pull request to `main` from a `queue/` branch through the App and
its merge on a green pipeline, the release tag and publication through the repository's own process,
the H0 2c issue comments and closes and the exit-criterion issue closes through the App after the
merge, and the deletion of the wave's merged remote branches through the bot. Not the next wave.

Decider: operator

## Wave record

Scratch root `~/.cache/ess-w1-wave/`; trees under the worktree manager's `b10x/ess/` root, each
building into its own `target/`.

| unit | tree id | branch | build dir | scratch | stage |
|---|---|---|---|---|---|
| int | ess-w1-int | integration/ess-w1 | tree `target/` | `~/.cache/ess-w1-wave/` | open |
| GEN1 | ess-w1-gen1 | unit/ess-w1-gen1 | tree `target/` | `~/.cache/ess-w1-wave/gen1/` | planned |
| UI1 | ess-w1-ui1 | unit/ess-w1-ui1 | tree `target/` | `~/.cache/ess-w1-wave/ui1/` | planned |
| SYN1 | ess-w1-syn1 | unit/ess-w1-syn1 | tree `target/` | `~/.cache/ess-w1-wave/syn1/` | planned |
| RUN1 | ess-w1-run1 | unit/ess-w1-run1 | tree `target/` | `~/.cache/ess-w1-wave/run1/` | planned |

Pre-flight 2026-11-02: / 117G free of 848G; one other cargo machine-wide (outside this repository).
The trees of earlier waves are still registered with the worktree manager, finished or idle.

## Build slots

D387 bands before every cargo command (orchestrator, 2026-11-02 11:14Z): / >= 80G with at most one
other cargo, stop under 70G; 75-80G with no other cargo, stop under 65G; under 75G nothing starts.
`CARGO_BUILD_JOBS=8`, `CARGO_PROFILE_DEV_DEBUG=line-tables-only`,
`CARGO_PROFILE_TEST_DEBUG=line-tables-only`, `CARGO_INCREMENTAL=0`, the shared build slot, tree
`target/` under 15G. Every ess-conformance build or test needs host-monitor's slot first, named
targets only, never an all-targets local ess-conformance build. The final gate runs in stages.

## Selection (2026-11-02)

`aep plan artifact scope` was written for all 14 stories by one `aep:story-scoper` each (Scope
sections and typed scope entries, cited or inferred). `aep plan artifact waves --kind story` over
the store, filtered to the 14, placed them in waves 1, 2, 6, 7, 10 and 12. Every collision between
two of the 14:

- the suite-major trio (`story:a-view-read-checks-every-rows-shape`,
  `story:expect-error-checks-generated-field-presence`,
  `story:view-and-error-assertions-carry-their-value-shape`) collide with each other and with
  `story:a-union-variant-scenario-reads-each-row-under-its-own-identity` on `src/runner.rs`,
  `src/go/runtime.go`, `src/ts/runtime.ts` and `src/synthesize.rs` (cited and inferred), with
  `story:a-system-issued-awaited-step-honours-its-outcomes-input-guard` on `src/scenario.rs`
  (cited), and with `story:authored-success-step-asserts-no-error` on `src/authored.rs` (inferred);
- `story:a-generated-ui-calls-the-routes-its-http-binding-serves` x
  `story:account-menu-entries-keep-sets-and-navigate` on `crates/ui/ess-ui-check/src/lib.rs` and
  `src/model.rs` (inferred), both in unit UI1;
- `story:a-json-command-body-refuses-an-unknown-member` x
  `story:the-generated-rust-server-refuses-what-its-openapi-refuses` on
  `crates/generate/ess-synth/src/rust/http/bound.rs` (inferred), both in unit GEN1;
- `story:a-record-action-in-the-terminal-renderer-sees-its-row` x
  `story:a-row-actions-confirm-is-addressed-by-the-views-key` on `crates/ui/ess-ui-tui/src/app.rs`
  (cited), both in unit UI1.

Unassessed: none. Cycles: none.

Decision: the three suite-major stories move to W2 as one shared unit. They need a new suite format
pair (64/65) across the Rust, Go and TypeScript runners and a design page, about 1,500-2,000
lines, and they collide with SYN1's union story and RUN1's authored story. RUN1 is narrowed to
`story:authored-success-step-asserts-no-error`. The union story stays in SYN1 on a synthesis-side
fix inside suite major 63; if only a runner change satisfies it, it moves to W2 too. Accepted by
the orchestrator (host-monitor grant relay, 2026-11-02 21:04Z). After the move, every remaining
collision lies inside one unit.

Consequence: beyond10x/ess#680 is not closed in W1, and W1 implements 8 of the 14 P0 stories, not
9.

Store finding (out of scope, for the close): `story:a-view-read-checks-its-rows-shape` is
implemented, but only its item 1 exists in the tree
(`crates/verify/ess-conformance/tests/author_generated_value.rs:255`).

## Build slots (host-monitor terms, 2026-11-02 20:45Z)

Verbatim: "D394 GRANTED (ESS W1, 4 trees, serial under one wave lock), on D387 bands plus the IO
terms in force since 20:15Z." Every cargo call runs as `nice -n 10 ionice -c3` with
`CARGO_BUILD_JOBS=8` and `RUST_TEST_THREADS=4`. Only `--lib` plus named test targets build until
the integration gate. Before each step: / >= 80G, at most one other cargo machine-wide, io pressure
`some avg60` under 15; when one fails, wait. Each tree stays at or under 15G and is discarded at
merge. The wave wrapper waits on a failed check (re-checks every 30 s, up to 45 minutes).
