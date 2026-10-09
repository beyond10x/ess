# Unit brief e2, wave 2026-10-08e

## Identity

```
story:  story:a-scalar-to-record-input-change-is-breaking-for-callers
branch: unit/ess-wave-20261008e-e2   forked from integrate/ess-wave-20261008e
```

Read the story (`aep plan artifact show story:a-scalar-to-record-input-change-is-breaking-for-callers`
from the integration tree). Its Decisions and Acceptance sections are the contract. Reproducer:
`.engineering/repro/upcast/` (`before.yaml`, `after.yaml`, `after-additive.yaml`).

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261008e-e2
build dir: <worktree>/target
scratch:   the TMPDIR named in the go message (outside every Git checkout)
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/verify/ess-diff/**` (source and tests); new test files under `crates/edge/ess-cli/tests/`; existing ess-cli diff tests only to re-pin an expectation the story changes (name each); the format tables and reference pages a new `ess-diff/N` must be registered in (find them where `ess-diff/15` is named; `cargo xtask docs` regenerates generated pages); the diagnostics catalogue if a code changes |
| **not yours** | everything else. `CHANGELOG.md` and `.engineering/**` are the coordinator's. Held by another session: `crates/verify/ess-conformance/src/synthesize.rs`, `src/synthesize/**`, `src/interpret/**`, `src/authored.rs`, `src/mutate.rs`, `crates/specify/ess-domain/src/command.rs`, `command/**`, `crates/specify/ess-compiler/src/ir.rs`, `ir/**` |

Invariants: `.engineering/waves/2026-10-08e/invariants.md` in the integration tree. Read it first.

Decided by the coordinator: the verdict is decided where the change kind is classified
(`ess-diff/src/compatibility.rs`), from the before and after declared types of the input field;
whether the written delta moves to `ess-diff/16` follows `AGENTS.md` ("Determinism and formats"),
and the report cites the code that decides it.

Spec first: model the change in this repository's ESS specification, validate it with the
newest `ess`, regenerate, then implement against the generated code. If the specification cannot
express it, stop and report that; do not hand-write a parallel model.

(No authored-language change is expected. Say in the report whether `models/` declares anything
this touches.)

Reproduce first on the installed `ess` 0.56.0 (`ess verify diff` on the reproducer; quote the
verdict). Then the red test run (quote it), implement, green run, `cargo fmt -p ess-diff`, then
`CARGO_INCREMENTAL=0 cargo clippy -p ess-diff --all-targets --locked -- -D warnings`, and the
`ess-diff` tests plus the `ess-cli` diff test binaries, by package, one `--test` at a time.
Nothing in this unit runs `ess-conformance` tests.

## Report header

```
unit: story:a-scalar-to-record-input-change-is-breaking-for-callers
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
changelog: <the [Unreleased] lines you would write>
format: <ess-diff/16 or none, with the cited reason>
```
