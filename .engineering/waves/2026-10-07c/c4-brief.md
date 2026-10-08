# Unit brief c4, wave 2026-10-07c

## Identity

```
story:  story:interpreter-default-outcome-beside-a-conjunction-guard   (https://github.com/beyond10x/ess/issues/471)
branch: unit/ess-wave-20261007c-c4   forked from integrate/ess-wave-20261007c
```

Read the story: `aep plan artifact show story:interpreter-default-outcome-beside-a-conjunction-guard` (from the integration tree), and the issue.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c-c4
build dir: <worktree>/target   (created by the coordinator with Cargo's CACHEDIR.TAG)
scratch:   <worktree>/target/wave-scratch/c4
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/verify/ess-conformance/src/interpret/**`, the report field that carries an error diagnostic if the fix needs it (name the file in your plan), new test files under `crates/verify/ess-conformance/tests/` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07c/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c`). Read it first.

Reported on 0.53.0: build the reproducer from the issue (it describes the entity, enums, event, error, actor and view in prose) and run it with `ess verify conform run --target interpreted` on 0.55.0 first.

## Two phases

**Phase 1 (now): no build.** Another build holds the machine. Do not run `cargo`, `rustc`, `task` or
anything that compiles. Run the installed `ess` 0.55.0 to reproduce the issue (scratch only). Write
the failing tests and the exact implementation plan, then stop: `verdict: blocked`,
`needs-coordinator: build slot`. If the issue does not reproduce, stop and report that.

**Phase 2 (when resumed):** red run (quote it), implement, green run, `cargo fmt -p <crate>`,
clippy and tests for the crates you touched, by package.

## Report header

```
unit: story:interpreter-default-outcome-beside-a-conjunction-guard
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
