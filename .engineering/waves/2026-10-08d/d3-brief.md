# Unit brief d3, wave 2026-10-08d

## Identity

```
story:  story:test-scratch-directories-are-removed-on-drop
branch: unit/ess-wave-20261008d-d3   forked from integrate/ess-wave-20261008d
```

Read the story: `aep plan artifact show story:test-scratch-directories-are-removed-on-drop` (from the integration tree).

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008d-d3
build dir: <worktree>/target
scratch:   <worktree>/target/wave-scratch/d3
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/verify/ess-conformance/tests/**` (including `tests/support*/`) |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-08d/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008d`). Read it first.

Your part: ess-conformance integration tests only; `src/` is not yours. Every directory a test creates under `std::env::temp_dir()` is removed by a drop guard, also when the test panics. Write one guard in a test support module and use it everywhere. Phase 1 needs no build: edit, then stop with `verdict: blocked`, `needs-coordinator: build slot`, naming the test binaries you changed. Phase 2 (when the coordinator resumes you) runs them one at a time with `TMPDIR` set to an empty directory under your scratch, and shows it empty afterwards.

Spec first: model the change in this repository's ESS specification, validate it with the
newest `ess`, regenerate, then implement against the generated code. If the specification cannot
express it, stop and report that; do not hand-write a parallel model.

Red run first (quote it), implement, green run, `cargo fmt -p <crate>`, then
`CARGO_INCREMENTAL=0 cargo clippy -p <crate> --all-targets --locked -- -D warnings` and the tests of
the crates you touched, by package, one `--test` at a time.

## Report header

```
unit: story:test-scratch-directories-are-removed-on-drop
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
