# Concurrent history — wave 1

Epic: `epic:concurrent-history-conformance`, plus its prerequisite
`story:interpreted-command-execution` (`epic:model-driven-interpretation`). Base: `origin/main`
`472d35fbe`. Skill: aep 0.14.17 (`aep:implementing`, wave mode); `aep --version`: `aep 0.61.1`.

Operator goal, 2026-09-27: implement the epic end to end through AEP waves, all eight stories
`implemented`, `task check` green on `main`, an ESS release cut and verified. The goal is the
wave's pre-approval: it waives the stage-1 stop, not this page.

Dispatch types: `aep:story-scoper` (done, 7 runs), `aep:implementor`, `aep:adversary`,
`aep:plan-critic-acceptance`, `aep:plan-critic-design`, `aep:plan-critic-scope`,
`aep:plan-critic-parallel-safety`.

## Commits this approval authorises

Per wave: one commit per unit, the merges of unit branches into `integrate/concurrent-history`, the
store commits (relation repoint, scope, activation, evidence, closing) and the wave page updates.
After the last wave: one PR `integrate/concurrent-history` → `main` through the bot App, its merge,
and the release the goal names. Nothing else.

## Integration

| | |
|---|---|
| branch | `integrate/concurrent-history` |
| base | `472d35fbe` (`origin/main`) |
| worktree | managed `ess-chc-int` |
| build dir | `~/.cache/b10x-target/ess-chc-int` |
| scratch | `~/.cache/ess-chc/scratch` |

## Sequence (all waves)

From `depends_on` and the typed scope written 2026-09-27; every story lands in
`crates/verify/ess-conformance`, so later waves are mostly N = 1.

| wave | units | why together / why alone |
|---|---|---|
| 1 | `interpreted-command-execution`, `concurrent-history-format` | no edge between them; shared only `ess-conformance/src/lib.rs` (one `pub mod` each) — split by symbol |
| 2 | `linearizability-checker-over-the-interpreter` | needs both wave-1 units |
| 3 | `session-and-eventual-view-checks`, `concurrent-explorer-runner` | both need only the checker; shared `lib.rs` only if the runner adds none — split declared at dispatch |
| 4 | `concurrent-history-lanes`, `recorded-history-validation` | both collide on `ess-cli/src/main.rs` `ConformCommand` — split by variant, or sequenced |
| 5 | `declared-fault-injection` | needs the runner and the session checks; shares both explorers and `faulty.rs` |

Replanned from the store before each wave; this table is the plan, not the selection.

## Other work on the same crate

`integrate/the-5-waves` (another session, page written 2026-09-27 20:54, 18 stories, ends in a
release) edits `ess-conformance` `input.rs`, `runner.rs`, `synthesize.rs`, `witness.rs` (its page, lines 55–74). Coordinator decision: this wave does not wait. Before the PR to `main`, run
`git merge-tree --write-tree` against the then-current `origin/main`; whichever integration lands
second merges `main` in and says so in the subject.

## Wave 1 units

Triple per unit: worktree (managed id `ess-chc-<unit>`), build dir `~/.cache/b10x-target/ess-chc-<unit>`,
scratch `~/.cache/ess-chc/<unit>/scratch`.

| unit | story | serves | scope | branch | head | stage |
|---|---|---|---|---|---|---|
| interp | `story:interpreted-command-execution` | `vision:O2` | cited: `ess-conformance`, `src/target.rs`; inferred: `src/input.rs`, `src/interpreted.rs`, `src/lib.rs`, `tests` | `impl/interpreted-command-execution` | `989e811cb` | merged `85e251c20` (2 adversary passes, 8 findings fixed); implemented |
| format | `story:concurrent-history-format` | `vision:O2` | cited: `ess-conformance`, `schemas/`; inferred: `src/history.rs`, `src/lib.rs`, `ess-xtask`, `models/concurrent-history/domains/history.yaml` | `impl/concurrent-history-format` | `64370a752` | merged `7a81ad0ec` (2 adversary passes, 11 findings fixed); implemented |

Split in `crates/verify/ess-conformance/src/lib.rs`: each unit adds exactly one `pub mod` line; no
other edit. `git merge-tree --write-tree --merge-base=<base> impl/interpreted-command-execution
impl/concurrent-history-format` runs before the first unit merges.

## Planning changes made before wave 1

- `depends_on story:external-mutation-explorer-and-toolchain` (archived) replaced on the epic and on
  `story:declared-fault-injection` by `depends_on story:explorer-takes-external-branches`, the one
  successor that supplies what they use (critic round 3 design finding; round 4 approved).
- `story:concurrent-history-format`: the Go/TS equal-bytes acceptance line moved to
  `story:concurrent-explorer-runner`; a model-drift acceptance line added.
- `story:session-and-eventual-view-checks`: acceptance line keeping the new
  `StaleReadUnderReadYourWrites` row separate from the existing `StaleReadYourWrites`
  (`faulty.rs:243-246`).
- `## Scope` sections and typed scope written for all seven concurrent-history stories.

## Pre-flight (2026-09-27 21:00)

- `/` 111G free; `/dev/shm` 32G free (`df -h`); sccache wired (`~/.cargo/config.toml:60`).
- Load average 29.78 on 20 cores (`uptime`): every build `nice -n 19`, `CARGO_BUILD_JOBS=4`, `RUST_TEST_THREADS=4`.
- One integration build dir of this repository measured 2.5G (`du -sh ~/.cache/b10x-target/ess-5w-int`, another session's).
- Primary checkout carries another session's untracked `docs/reviews/2026-09-25-consumer-request-input-constraints.md`; no unit touches it.
- `TMPDIR=~/.cache/claude-tmp`, not inside a Git checkout (`git -C ~/.cache rev-parse` fails).
- Model budget: none stated; N ≤ 4.

## Deviations

- Units forked from `472d35fbe` and dispatched before the opening store commit: each `aep` write took 3–4 min at load average 43 (`uptime` 21:08), and the 33-write batch would have held dispatch about 2 h. Story moves to `active` follow the batch; unit branches merge into `integrate/concurrent-history` after it.
- Store writes of 22:30–23:40 were rolled back and replayed: the parallel-safety round-3 critic quoted an absolute home path, `validate` refused it (S9, 6 problems), and `review-result` bodies are immutable. The rolled-back writes are kept in `git stash` on this tree ("chc: store writes … (S9); replayed clean"); the replay records the same verdicts with the path replaced by "the integration worktree".
- The store takes one `review_outcome` per (artifact, review): four repeat `fixed` records for format adversary pass 1 were refused `semantic_mismatch`, so one outcome stands for its five findings.
- Critic rounds 3–4 (acceptance, design, scope, parallel-safety): round 3 two `needs-revision` (3 findings, all fixed); round 4 four `approve`. The design critic's round-4 empty findings fence is recorded as `[]`.

## Close (2026-09-28)

Gate on `integrate/concurrent-history` `85e251c20`, one exit status per step, own build dir `b10x-target/ess-chc-int`: `task fmt-check` 0; clippy `-p ess-conformance -p ess-cli -p ess-xtask -p ess-domain` 0; doc 0; `cargo test -p ess-conformance` 0 (832 passed, 98 binaries); `-p ess-xtask` 0 (313); `-p ess-domain` 0 (756); `-p ess-cli` 0 (750); `specify validate models/concurrent-history` 0. Full `task check` runs in CI on the final PR and locally before the release tag (ESS `AGENTS.md` § Gate).

Build dirs `ess-chc-format` (17G) and `ess-chc-interp` (23G) deleted after merge; `/` had filled to ENOSPC once during the interp gate.

## Wave 2 (2026-09-28)

| unit | story | branch head | stage |
|---|---|---|---|
| checker | `story:linearizability-checker-over-the-interpreter` | `0e8cd39fb` | merged `b08ed292d`; fix `57b8cb105`; implemented |

Gate on `b08ed292d`: every step exit 0 except `cargo test -p ess-xtask` exit 101 (3 failed). `tests/layout.rs` rejected two literal missing paths in the adversary test `crates/edge/ess-cli/tests/check_history_adversary.rs`, fixed in `57b8cb105`. Two `consumer_coverage` unit tests failed with `No such file` and `authority changed` while `/` was at 98%; alone they passed (2 passed), and the full `ess-xtask` rerun on `57b8cb105` passed (313, exit 0). Cause not established; recorded, not dismissed.

Follow-up filed: `story:concurrent-history-records-inputs` (adversary pass 2 finding 4, pre-existing: `ess-history/1` records no inputs).


## Wave 3 (2026-09-28)

| unit | story | branch head | stage |
|---|---|---|---|
| session | `story:session-and-eventual-view-checks` | `639f90c08` | merged `c589fddb0`; implemented |
| runner | `story:concurrent-explorer-runner` | `76931b63a` | merged `acd6a0862`; implemented |

Gate on `acd6a0862`, own build dir: fmt, clippy, doc 0; `ess-conformance` 874 passed; `ess-domain` 756; `ess-cli` 785 (Go and Node lanes included); model valid; `ess-xtask` 101 in the gate run, 313 passed on a rerun.

The two `consumer_coverage` failures seen in waves 2 and 3 are explained: `source_files` (`crates/edge/ess-xtask/src/consumer_coverage/mod.rs:323`) hashes every tracked and untracked file, `.engineering/` included, and compares it with the snapshot taken at build time. Coordinator store writes into this tree ran during both gates. Rule from here: no store write into the integration tree while a gate runs.

The installed `aep` became 0.62.0 at 04:08 (not this session), which refuses this `aep.project/3` store; this branch writes through a private 0.61.1 built from tag `0.61.1` (`b6213e1fc`).
