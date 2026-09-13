# Wave: model-driven interpretation, turn 1

Proposed 2026-09-11 from `epic:model-driven-interpretation`. Skill version **0.9.1**
(`aep-drive/0.9.1/skills/wave`). Selection path: **computed** — `aep plan artifact waves`, binary
`protocol 0.55.0`, exit 0. Not a pairwise reading.

**Status: approved and running.** Decision 43 taken 2026-09-11 12:35 on Timo's recorded words;
N = 2 taken by the coordinator; the module-not-crate architecture taken by the coordinator.
Base branch `plan/model-driven-interpretation` at `0a21a499`, integration branch
`wave/model-driven-interpretation-1`.

## The units

| unit | story | serves | scope confidence | surface |
|---|---|---|---|---|
| 1 | `story:interpreted-target-selection` | `vision:O2` | high | `crates/edge/ess-cli/src/main.rs:608-612,2808-2813`, new module in `crates/verify/ess-conformance` |
| 2 | `story:d2-constraint-has-a-home` | `vision:O2` | high | `docs/plan/ess-wave-6-structural-synthesis.md:73-76`, `docs/design/…-interpretation-design-v0.1.md`, `docs/plan/ess-roadmap.md:307` |

N = 2. Unit 2 is documentation only and collides with no crate.

`subagent_type` for every dispatch: **`aep-drive:implementor`**, then **`aep-drive:adversary`**.
Both with their plugin prefix; no built-in substitution.

## Why only two, from the verb rather than from judgement

The verb places this epic's seven stories in six different waves. Only units 1 and 2 share one:

| verb wave | epic stories in it |
|---|---|
| 2 | `d2-constraint-has-a-home`, `interpreted-target-selection` (+ `enum-variant-in-an-entity-invariant`, `schema-unique-items-signed-zero`, both outside this epic and not proposed) |
| 5 | `interpreted-command-execution` — alone |
| 6 | `interpreted-scenario-supplied-facts` — alone |
| 7 | `interpreted-bindings-and-unmet-obligations` — alone |
| 8 | `interpreted-eventual-views` — alone |
| 9 | `interpreted-trust-gate` — alone |

The six interpreter stories all write one `ConformanceTarget` implementation in one crate. That is
not a wave, it is a sequence, and the verb says so: 63 collisions involve this epic, of which the
intra-epic ones are all on `crates/verify/ess-conformance` (cited) and on
`src/interpret.rs` / `src/lib.rs` / `src/target.rs` (inferred).

Cross-epic collisions the verb excluded, which is why wave 2 is not larger:

```
create-only-command-cannot-refuse x every interpreted-* story  on crates/verify/ess-conformance (cited)
integrate-source-driven-realizations x interpreted-target-selection on crates/edge/ess-cli/src/main.rs (cited)
java-conformance-target x interpreted-target-selection on crates/edge/ess-cli (cited)
the-generated-go-runtime-is-gofmt-clean x every interpreted-* story on crates/verify/ess-conformance (cited)
component-declares-its-settings x interpreted-eventual-views on crates/specify/ess-compiler/src/ir.rs (cited)
enum-variant-in-an-entity-invariant x interpreted-eventual-views on crates/specify/ess-domain/src/view.rs (inferred)
create-only-command-cannot-refuse x d2-constraint-has-a-home on docs/design (inferred)
```

Unassessed after scoping: `story:outcome-decided-by-environment` — outside this epic, not scoped,
not proposed.

## Pre-flight: three refusals, two proceeded over

Refusals 1 and 2 are proceeded over on the coordinator's decision of 2026-09-11, recorded here so a
later reader sees a judgement rather than a gap:

- **18 stale worktrees** — a housekeeping signal, not a correctness one. The disk warden owns that
  surface and is writing a reclaim pass against it. This wave does not touch them.
- **The primary checkout's 12 uncommitted files** — they belong to another session.
  `WORKING-RULES.md` §5 forbids discarding them, and this wave does not read or move them.

Refusal 3 is **not** proceeded over: see *The blocking refusal* below.

## Pre-flight: the three checks

| check | reading | verdict |
|---|---|---|
| previous waves' worktrees gone | 18 live under `trees/b10x/ess/` | **refuse** |
| primary checkout clean | the primary checkout: 12 uncommitted, on `main` | **refuse** |
| coordinator tree clean, on base | this tree: uncommitted, on `publish/ess-pr29-runtime-gaps` | **refuse** |
| free disk | 86 G, `/` at 90%; this tree's `target` is 46 G | pass, thin |
| measured build | warm full gate 5–20 min; cold not measured | partial |
| model budget | not asked | **open** |
| `AGENTS.md` read | yes — byte-pinned generated Rust, explicit fmt package list, `task check` is the gate | pass |

## The blocking refusal

This coordinator's tree is dirty and sits on `publish/ess-pr29-runtime-gaps`, not on the base
branch. That is a correctness refusal, not tidiness, for one reason: **the artifacts this wave runs
on are uncommitted.** `epic:model-driven-interpretation`, its seven stories, their typed scope
entries and this page exist only in this working tree. A worktree forked now would not contain them,
so an implementor dispatched into one would be pointed at a story id its checkout does not have, and
the wave would fork from PR #29's head rather than from `main`.

It clears when the planning artifacts and the design pages are committed and the wave forks from a
base that carries them — which is decision 43, not something this wave can take for itself.

## Open decisions this wave rests on

1. **Crate or module.** The design page addresses "implementors of `ess-conformance` and of a new
   `ess-interpret`"; both stories and the epic say a module inside `crates/verify/ess-conformance`.
   No `ess-interpret` exists in the workspace members. If the crate route is taken, the epic's
   internal collisions mostly disappear and this wave could be larger. Two scopers raised it
   independently.
2. **How far `d2-constraint-has-a-home` reaches.** Its own Scope says documentation only; its
   acceptance, read literally, reaches 25 files under `crates/`, `examples/` and `generated/` that
   restate D-2. Narrow reading keeps it parallel-safe; wide reading does not.

## Corrections this wave found in its own inputs

- `crates/generate/ess-synth/src/rust/system.rs:23-27` is cited in the epic, in
  `story:interpreted-bindings-and-unmet-obligations` and in the design page. The quoted passage is at
  **`:31-35`**; `:23-27` is the `on_failure` policy paragraph. Verified by reading the file.
- The design page cites `configure_external_outcome` at `:115`, `mark_instant` at `:164`,
  `observe_elapsed` at `:194`. They are at `:178`, `:235`, `:265`.

## Commits approval would authorise

Two unit commits, the two merges into the integration branch, the closing store commit, and the
merge of the integration branch into `main`. **Nothing else** — no push, no tag, no release, no
second wave. The grant ends when this wave closes.

## Per-unit record

Empty until stage 2 starts. Each unit's row carries branch, head commit, worktree path, build
directory path, scratch root, and stage.

| unit | branch | head | worktree | build dir | scratch | stage |
|---|---|---|---|---|---|---|
| 1 | `impl/interpreted-target-selection` | `d13c152c` | `trees/b10x/ess/mdi1-interpreted-target-selection` | `~/.cache/ess-wave-mdi1/unit1/target` | `~/.cache/ess-wave-mdi1/unit1/scratch` | merged 436fcbfa, 2 adversary passes, implemented |
| 2 | `impl/d2-constraint-has-a-home` | `cd74b055` | `trees/b10x/ess/mdi1-d2-constraint-has-a-home` | `~/.cache/ess-wave-mdi1/unit2/target` | `~/.cache/ess-wave-mdi1/unit2/scratch` | merged 1268e06f, 2 adversary passes, implemented |

Managed worktree ids: `mdi1-interpreted-target-selection`, `mdi1-d2-constraint-has-a-home`.
Briefs: `~/.cache/ess-wave-mdi1/brief-unit1.md`, `brief-unit2.md`.

**Incident, 2026-09-11 ~12:55.** Both implementors were killed mid-flight by HTTP 429, session
limit, resets 15:20 Europe/Berlin; the account was re-logged at 13:00 and the limit cleared. Both
unit branches were verified at `1a2effd6` with zero commits ahead and clean working trees before
re-dispatch, so nothing was paid for twice — this is a restart, not a resume, and it is recorded as
such because the distinction is what the skill's rate-limit rule turns on.


## Closing record

Gate on the integration branch, per step, after both merges:

| step | exit |
|---|---|
| fmt-check | 0 (201 first pass — coordinator's own `items_after_statements` and an unwrapped line in files added after each unit's lane) |
| clippy | 0 (201 first pass, same cause) |
| test | 0 — 288 suites, 0 failed |
| doc-check · example-check · projection-check · support-check | 0 |
| fuzz-check · release-check · action-check | 0 |
| consumer-check | **not run** — pins `RUSTUP_TOOLCHAIN=1.98.1`, absent on this machine. CI disables it by operator request. Unverified, not passing |

Cost: roughly 1.32 M sub-agent tokens over 9 dispatches — 7 scopers, 2 implementors with 4
correction rounds, 2 adversaries with 4 passes. Two agents ran ~5 hours wall each.

Findings ledger across both units: pass 1 → pass 2 was **0 carried, 7 new, 6 resolved** for
`d2-constraint-has-a-home`, and 4 → 1 for `interpreted-target-selection`. Carried zero in both means
every specific correction held; the new findings were fresh claims the corrections themselves
introduced.

**What the wave taught, for the next one:** every blocker either adversary found was invisible to a
package-scoped gate — a generated support row that `task check` compares back, and a claim matcher
that missed a second copy by one word. Running a unit's own tests harder would have caught neither.
A documentation-only unit should not be dispatched into a Rust worktree at all: giving it a build
directory is what made a workspace build reachable, twice, and it cost the machine's free disk both
times.
