# Unit brief c3, wave 2026-10-07c

## Identity

```
story:  story:synthesis-survives-two-identity-guards-on-a-self-reference   (https://github.com/beyond10x/ess/issues/474)
branch: unit/ess-wave-20261007c-c3   forked from integrate/ess-wave-20261007c
```

Read the story: `aep plan artifact show story:synthesis-survives-two-identity-guards-on-a-self-reference` (from the integration tree), and the issue.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c-c3
build dir: <worktree>/target   (created by the coordinator with Cargo's CACHEDIR.TAG)
scratch:   <worktree>/target/wave-scratch/c3
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/verify/ess-conformance/src/synthesize*` (synthesis sources only), new test files under `crates/verify/ess-conformance/tests/` or `crates/edge/ess-cli/tests/` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07c/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c`). Read it first.

Sequenced after c2 on the same synthesis files: phase 1 writes a new test file only; phase 2 starts after c2 merges, with the integration branch merged into this branch first. Reported on 0.53.0: reproduce on 0.55.0 first (a stack overflow aborts the process; capture the exit status and stderr).

## Two phases

**Phase 1 (now): no build.** Another build holds the machine. Do not run `cargo`, `rustc`, `task` or
anything that compiles. Run the installed `ess` 0.55.0 to reproduce the issue (scratch only). Write
the failing tests and the exact implementation plan, then stop: `verdict: blocked`,
`needs-coordinator: build slot`. If the issue does not reproduce, stop and report that.

**Phase 2 (when resumed):** red run (quote it), implement, green run, `cargo fmt -p <crate>`,
clippy and tests for the crates you touched, by package.

## Report header

```
unit: story:synthesis-survives-two-identity-guards-on-a-self-reference
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
