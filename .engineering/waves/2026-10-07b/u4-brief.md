# Unit brief U4, wave 2026-10-07b

## Identity

```
story:  story:creating-an-output-root-locks-only-what-it-creates   (beyond10x/ess#485)
branch: unit/ess-wave-20261007b-u4-root-lock   forked from integrate/ess-wave-20261007b
base:   see `git -C <worktree> merge-base HEAD integrate/ess-wave-20261007b`
```

Read the story: `aep plan artifact show story:creating-an-output-root-locks-only-what-it-creates`.
Issue: https://github.com/beyond10x/ess/issues/485.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b-u4
build dir: <worktree>/target   (already created, with Cargo's CACHEDIR.TAG)
scratch:   <worktree>/target/wave-scratch/u4
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/edge/ess-cli/src/output_ownership/filesystem.rs` (`Locks::acquire` and what it calls), one new test file under `crates/edge/ess-cli/tests/` and its `mod` line, the locking paragraph of `docs/design/review-output-ownership.md` |
| **shared, sequenced** | U3 (#484) owns the rest of `crates/edge/ess-cli/src/output_ownership/**` and the rest of the design page and is building now. Before phase 2 you merge the integration branch (which will hold U3) into your branch; until then, edit nothing else under `output_ownership/` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07b/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b`). Read it first.

## Two phases

**Phase 1 (now): no build slot.** Do not run `cargo`, `rustc`, `task` or anything that compiles. The
installed `ess` (0.55.0) is fine to run. Write:

1. the failing tests: N concurrent `generate` runs creating distinct new roots under one shared
   directory get no `busy` refusal; two runs creating the same new root still exclude each other and
   leave nothing half-written; a reserved or enrolled ancestor is still refused;
2. the exact implementation plan, including how a root created by this run is locked without a race
   against a second run creating it, and what happens to the directories created if the run is refused.

Check whether `models/output-ownership` says anything about locking; report it.

Then stop: `verdict: blocked`, `needs-coordinator: build slot after U3 merges`.

**Phase 2 (when resumed):** merge `integrate/ess-wave-20261007b` into your branch (the coordinator
tells you when U3 is in), red run, implement, green run, `cargo fmt -p ess-cli`, clippy for
`ess-cli`, and every ess-cli output-ownership test binary by name.

## Report header

```
unit: story:creating-an-output-root-locks-only-what-it-creates
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
