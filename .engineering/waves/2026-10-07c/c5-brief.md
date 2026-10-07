# Unit brief c5, wave 2026-10-07c

## Identity

```
story:  story:diff-classifies-an-added-domain-entry   (https://github.com/beyond10x/ess/issues/469)
branch: unit/ess-wave-20261007c-c5   forked from integrate/ess-wave-20261007c
```

Read the story: `aep plan artifact show story:diff-classifies-an-added-domain-entry` (from the integration tree), and the issue.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c-c5
build dir: <worktree>/target   (created by the coordinator with Cargo's CACHEDIR.TAG)
scratch:   <worktree>/target/wave-scratch/c5
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/verify/ess-diff/**`, new tests in that crate or under `crates/edge/ess-cli/tests/` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07c/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c`). Read it first.

Measured on 0.55.0 by the coordinator: the issue reproducer still fails `--fail-on breaking-or-unknown` with `system/warehouse/unclassified-changed`.

## Two phases

**Phase 1 (now): no build.** Another build holds the machine. Do not run `cargo`, `rustc`, `task` or
anything that compiles. Run the installed `ess` 0.55.0 to reproduce the issue (scratch only). Write
the failing tests and the exact implementation plan, then stop: `verdict: blocked`,
`needs-coordinator: build slot`. If the issue does not reproduce, stop and report that.

**Phase 2 (when resumed):** red run (quote it), implement, green run, `cargo fmt -p <crate>`,
clippy and tests for the crates you touched, by package.

## Report header

```
unit: story:diff-classifies-an-added-domain-entry
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
