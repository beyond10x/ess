# Unit brief c1, wave 2026-10-07c

## Identity

```
story:  story:cli-binding-globals-config-and-output-are-optional   (https://github.com/beyond10x/ess/issues/481)
branch: unit/ess-wave-20261007c-c1   forked from integrate/ess-wave-20261007c
```

Read the story: `aep plan artifact show story:cli-binding-globals-config-and-output-are-optional` (from the integration tree), and the issue.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c-c1
build dir: <worktree>/target   (created by the coordinator with Cargo's CACHEDIR.TAG)
scratch:   <worktree>/target/wave-scratch/c1
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/specify/ess-cli-contract/**`, `crates/generate/ess-cli-project/**`, the ess-cli tests that cover `specify cli` and `generate cli` (new files fine), `crates/edge/ess-xtask/src/docs.rs` (format-history rows for the two new formats only), the reference pages that describe `ess-cli/*` and `ess-cli-plan/*` (`website/docs/reference/formats.md`, `spec-versions.md`, the CLI presentation bindings section of `website/docs/reference/cli.md`), `docs/design/cli-presentation-binding.md` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07c/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007c`). Read it first.

Decided: `ess-cli/2` and `ess-cli-plan/2`; `config` and `output` optional, `state` required; the reader keeps `ess-cli/1`; an `ess-cli/1` binding compiles to the same plan bytes as today; a generated CLI without `output` prints JSON only; `null`, `~` and `""` refuse naming the global. Spec first: declare the formats where the repository declares formats before code. Release notes must name both formats.

## Two phases

**Phase 1 (now): no build.** Another build holds the machine. Do not run `cargo`, `rustc`, `task` or
anything that compiles. Run the installed `ess` 0.55.0 to reproduce the issue (scratch only). Write
the failing tests and the exact implementation plan, then stop: `verdict: blocked`,
`needs-coordinator: build slot`. If the issue does not reproduce, stop and report that.

**Phase 2 (when resumed):** red run (quote it), implement, green run, `cargo fmt -p <crate>`,
clippy and tests for the crates you touched, by package.

## Report header

```
unit: story:cli-binding-globals-config-and-output-are-optional
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
