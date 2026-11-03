---
format: aep.planning-md/3
id: approval-record:ess-w2-wave
kind: approval-record
status: approved
title: Wave W2 runs units GEN2, EXP1, SYN2 and MUT1 of the 2026-11-02 reconcile roadmap as one wave, one pull request and release 0.85.0
summary: Orchestrator approval D408 of 2026-11-03 under the operator's grant
tags:
- non-interactive
relations:
- decides: story:go-synthesis-repairs-a-served-component-named-server
- decides: story:component-explorer-walks-only-its-component
- decides: story:components-sharing-a-cli-binary-generate-one-tree-with-every-group
- decides: story:go-explorer-compares-direct-response-members
- decides: story:explorer-and-interpreter-agree-on-external-cause-for-stored-record
- decides: story:unknown-instance-and-input-guard-agree-on-order
- decides: story:view-asserts-field-set-from-optional-input
- decides: story:go-synthesis-refuses-an-optional-into-required-assignment
- decides: story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists
- decides: story:a-delete-may-have-a-no-op-branch-that-emits-nothing
- decides: story:every-witness-reads-back-its-outcome-effect
- decides: story:error-swap-mutates-refusals-that-carry-a-payload
- decides: story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock
- decides: story:the-mutation-audit-reports-the-sites-it-could-not-mutate
- decides: story:the-mutation-audit-covers-actor-grants
revision: 3
transitions:
- {from: "draft", to: "approved", at: "2026-11-03T05:57:18Z", actor: "human:timo", revision: 2}
---
## Decision

D408, 2026-11-03: the orchestrator dispatched ESS wave W2 of the 2026-11-02 reconcile roadmap
(section 4, rows W2; D397) under the operator's grant (class C: a wave, bot commits, a bot pull
request and its merge on green, a release through ess's own process): "run ESS wave W2 (release
0.85.0). Approved under the grant ... (decisions.md D408, 2026-11-03)." The delivery rule is the
operator's of 2026-10-28: "work in waves or batches. So we fix multiple stories against one
integration branch, bigger batches, multiple stories, and then we do one PR. We do not run
expensive full gates after every small change. We run targeted tests only for what we're currently
implementing, and only in the end, when everything ended up in the integration branch, we run a
local full gate and then we create the PR and auto merge it to the default main branch." The
release that follows the merge is `0.85.0`.

The fourteen draft stories below are accepted by this record (draft -> proposed -> active, run by
the coordinator on D408's word); `story:every-witness-reads-back-its-outcome-effect` was already
active.

This run is non-interactive: the coordinator is a sub-agent with no operator turn, so the stage-1
proposal stop is recorded here and the wave runs. Skill: aep `implementing` 0.26.0, wave mode.
Agents: `aep:story-scoper`, `aep:implementor`, `aep:adversary`, `aep:security-reviewer`.

## Units

Base `main` b89faaa41 (0.84.0). Integration branch `integration/ess-w2`. Wave cap four units.

- GEN2: `story:go-synthesis-refuses-an-optional-into-required-assignment` (P0),
  `story:go-synthesis-repairs-a-served-component-named-server` (P0),
  `story:components-sharing-a-cli-binary-generate-one-tree-with-every-group` (P0).
- EXP1: `story:explorer-and-interpreter-agree-on-external-cause-for-stored-record` (P0),
  `story:go-explorer-compares-direct-response-members`,
  `story:unknown-instance-and-input-guard-agree-on-order`,
  `story:component-explorer-walks-only-its-component`.
- SYN2: `story:view-asserts-field-set-from-optional-input` (P0),
  `story:every-witness-reads-back-its-outcome-effect`,
  `story:a-boundary-row-of-a-deleting-branch-is-sent-for-a-row-that-exists`,
  `story:a-delete-may-have-a-no-op-branch-that-emits-nothing`.
- MUT1: `story:error-swap-mutates-refusals-that-carry-a-payload` (beyond10x/ess#709),
  `story:the-mutation-audit-reports-the-sites-it-could-not-mutate` (beyond10x/ess#711),
  `story:mutate-interpreted-synthesized-scenarios-run-on-the-scenario-clock`,
  `story:the-mutation-audit-covers-actor-grants` (beyond10x/ess#710).

Exit criterion (roadmap section 4): every P0 implemented; #709, #710, #711 closed.

## Selection (2026-11-03)

One `aep:story-scoper` per story wrote the `## Scope` sections and typed scope entries (cited or
inferred). `aep plan artifact waves --kind story` (aep 0.79.0) placed the fifteen in its waves 1,
2, 3, 5, 8, 15, 28, 54, 55 and 60; unassessed: none; cycles: none. Collisions between two units
(every other collision lies inside one unit):

- MUT1 (`the-mutation-audit-covers-actor-grants`, `the-mutation-audit-reports-the-sites-it-could-not-mutate`)
  x SYN2 (all four) on `crates/verify/ess-conformance/src/synthesize.rs` (inferred on the MUT1 side).
- EXP1 (`component-explorer-walks-only-its-component`) x MUT1 (`error-swap-...`, `...covers-actor-grants`,
  `...reports-the-sites-...`) on `crates/edge/ess-cli/src/main.rs` (inferred on the EXP1 side).
- GEN2 (`components-sharing-a-cli-binary-...`) x SYN2 (`every-witness-...`) on
  `crates/specify/ess-domain/src/system.rs` (inferred, both sides: a possible `ess/34`).
- SYN2 (`a-delete-may-have-...`, `every-witness-...`) x EXP1 (`unknown-instance-and-input-guard-agree-on-order`)
  on `crates/specify/ess-domain/src/command.rs` (inferred on the EXP1 side).

Decision: all four units run, with each shared file split by symbol (the skill's fallback), diff
hunk headers reported, and `git merge-tree --write-tree` dry runs before each merge:

- `synthesize.rs` is SYN2's. MUT1 puts any synthesis hook in `synthesize/grant.rs` or a new
  submodule and edits `synthesize.rs` only at the `with_dropped_write` hook (about :12283) or the
  sibling-retarget handling (:12426-12476).
- `ess-cli/src/main.rs`: MUT1 owns the `mutate` flags and class mirror (:927-1113) and the mutate
  dispatch (:4439-4732); EXP1 owns `write_suite`/`synthesize_suite` (:4800-5100).
- `system.rs`: SYN2 owns `ess/34` if it adds it. GEN2 adds no format; if its refusal needs a format
  gate it reads the `V34` SYN2 lands, after SYN2's merge.
- `command.rs`: SYN2 owns `validate_compensation` (:359-440) and the ESS-COMMAND-005 text
  (:8027-8037); EXP1 owns the step-order validators (:3095-3104, :3902).

Scoper notes that bear on the close: three stories are partly implemented on main
(`unknown-instance-and-input-guard-agree-on-order`: ess/29+ already refuses the reverse order;
`every-witness-...`: #587 shipped; `view-asserts-...`: the no-conversion variant is asserted); none
is wholly implemented. `view-asserts-...` conflicts with
`docs/design/binding-mapping-bounded-accessor.md:259-263`, which the unit amends first.

## Build terms

D394 as modified by D404/D407 (host-monitor): every cargo through `cargo-in.sh` (wave lock, nice
and ionice, jobs and test threads from `terms.env`, `CARGO_INCREMENTAL=0`,
`CARGO_PROFILE_DEV_DEBUG=line-tables-only`); pre-check / >= 55G, at most one other cargo, io
pressure avg60 < 15, no HOLD; in-step kill under 45G or MemAvailable under 4G; tree target/ <= 15G.
Every ess-conformance build or test runs only after the orchestrator relays host-monitor's slot,
named targets only, never an all-targets local ess-conformance build.

## Review scope

Adversary and security passes review each unit's diff against its acceptance only (operator rule of
2026-10-31). Introduced findings and gaps inside the acceptance go back to the implementor.
Findings outside a unit's acceptance become one out-of-scope row in the wave review-result record,
or a backlog draft tagged `from-finding`; never accepted by the coordinator and never added to
this wave (operator, 2026-11-02: waves take only accepted stories; findings wait until every
planned wave is delivered). A story found already implemented or impossible is recorded and
reported; no story replaces it.

## Commits this approval authorises

One or more commits per unit on its unit branch, the merges of green units into
`integration/ess-w2`, the store commits on that branch (this opening record and the acceptances,
evidence, close), fix commits for gate findings, the release-prepare commit (version 0.85.0, lock
files, licences, CHANGELOG, change record) on that branch before the final gate, the push of
`queue/ess-0.85.0` and one pull request to `main` through the App. Not the merge, the tag or the
release before the orchestrator's go; not the next wave.

Decider: operator

## Wave record

Scratch root `~/.cache/ess-w2-wave/`; trees under the worktree manager's `b10x/ess/` root, each
building into its own `target/`.

| unit | tree id | branch | build dir | scratch | stage |
|---|---|---|---|---|---|
| int | ess-w2-int | integration/ess-w2 | tree `target/` | `~/.cache/ess-w2-wave/` | open |
| GEN2 | ess-w2-gen2 | unit/ess-w2-gen2 | tree `target/` | `~/.cache/ess-w2-wave/gen2/` | planned |
| EXP1 | ess-w2-exp1 | unit/ess-w2-exp1 | tree `target/` | `~/.cache/ess-w2-wave/exp1/` | planned |
| SYN2 | ess-w2-syn2 | unit/ess-w2-syn2 | tree `target/` | `~/.cache/ess-w2-wave/syn2/` | planned |
| MUT1 | ess-w2-mut1 | unit/ess-w2-mut1 | tree `target/` | `~/.cache/ess-w2-wave/mut1/` | planned |

Pre-flight 2026-11-03: / 195G free of 848G. The trees of earlier waves (d325 to d368, issues-1 to
-3, 0680, 0690, 0750) are still registered with the worktree manager; none is this wave's and none
is touched.

## Retraction (D410, 2026-11-03)

The orchestrator's D410 retracts acceptance bullet 1 of
`story:unknown-instance-and-input-guard-agree-on-order`. That bullet asks for a refusal of an input
guard declared above an unknown-instance branch. The precedence section of
`docs/design/cross-record-and-stored-field-guards.md` makes that order the correct one, and
`ess/29` already refuses the reverse order. The precedence code is not changed. Bullet 2 stays as
the story's acceptance, and the story moves to implemented at the wave's close.
