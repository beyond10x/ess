# Unit brief d4, wave 2026-10-08d

## Identity

```
story:  story:test-scratch-directories-are-removed-on-drop
branch: unit/ess-wave-20261008d-d4   forked from integrate/ess-wave-20261008d
```

Read the story: `aep plan artifact show story:test-scratch-directories-are-removed-on-drop` (from the integration tree).

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008d-d4
build dir: <worktree>/target
scratch:   <worktree>/target/wave-scratch/d4
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/edge/ess-xtask/src/**`, `crates/edge/ess-xtask/tests/**`, `crates/generate/ess-synth/src/go/selection.rs` (test module), `crates/generate/ess-synth/tests/nested_response_wasm.rs`, `crates/generate/ess-gen/tests/asyncapi_transport.rs`, `crates/specify/ess-composition/tests/composition.rs`, `crates/infra/infra-analyze/tests/analysis.rs`, `crates/ui/ess-ui-tui/tests/read_filter.rs` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-08d/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008d`). Read it first.

Your part: every crate except ess-cli and ess-conformance, plus the leak check the story asks for: an `ess-xtask` test (not a new CI job) that runs a representative set of test binaries from the crates you own with `TMPDIR` set to an empty directory and fails naming the leaking prefix. Do not run `ess-conformance` tests.

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
