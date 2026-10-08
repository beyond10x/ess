# Unit brief d1, wave 2026-10-08d

## Identity

```
story:  story:a-wire-name-in-a-path-segment-refuses-slash-and-dot-segments
branch: unit/ess-wave-20261008d-d1   forked from integrate/ess-wave-20261008d
```

Read the story: `aep plan artifact show story:a-wire-name-in-a-path-segment-refuses-slash-and-dot-segments` (from the integration tree).

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008d-d1
build dir: <worktree>/target
scratch:   <worktree>/target/wave-scratch/d1
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/specify/ess-domain/src/wire.rs` (and a new test file in `ess-domain/tests/`), `crates/specify/ess-primitives/src/error.rs` only if a new validation code is needed, new test files under `crates/generate/ess-gen/tests/`, `crates/generate/ess-synth/tests/` and `crates/edge/ess-cli/tests/`, the diagnostics catalogue entry and generated reference page if a code is added (`cargo xtask diagnostics`), the reference page that documents `naming.wire` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-08d/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008d`). Read it first.

Decided: the refusal lives in validation (`wire::validate`, called from `Specification::validate_after`), because every generated HTTP path comes from `ess_gen::http::routes`. Refuse a wire name of a domain, command or view that contains `/`, or is `.` or `..`, naming the declaration and the wire name. Decide yourself whether views are refused always or only when network-reached, and say which in the report. Examples and committed suites must validate unchanged.

Spec first: model the change in this repository's ESS specification, validate it with the
newest `ess`, regenerate, then implement against the generated code. If the specification cannot
express it, stop and report that; do not hand-write a parallel model.

Red run first (quote it), implement, green run, `cargo fmt -p <crate>`, then
`CARGO_INCREMENTAL=0 cargo clippy -p <crate> --all-targets --locked -- -D warnings` and the tests of
the crates you touched, by package, one `--test` at a time.

## Report header

```
unit: story:a-wire-name-in-a-path-segment-refuses-slash-and-dot-segments
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
