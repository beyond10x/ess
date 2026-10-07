# Unit brief U2, wave 2026-10-07b

## Identity

```
story:  story:generated-rust-types-make-exact-numbers-a-crate-feature   (beyond10x/ess#483)
branch: unit/ess-wave-20261007b-u2-json-numbers   forked from integrate/ess-wave-20261007b
base:   see `git -C <worktree> merge-base HEAD integrate/ess-wave-20261007b`
```

Read the story, including its fit review and the design it chose (option d):
`aep plan artifact show story:generated-rust-types-make-exact-numbers-a-crate-feature`.
Issue: https://github.com/beyond10x/ess/issues/483.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b-u2
build dir: <worktree>/target
scratch:   <worktree>/target/wave-scratch/u2
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/generate/schema-contract/**`; the reference page for `ess generate types` under `website/docs/` if it describes the generated manifest |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's; `Cargo.toml` and `Cargo.lock` at the root |

Invariants: `.engineering/waves/2026-10-07b/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b`). Read it before anything else.

## The design (from the story)

- The generated `Cargo.toml` names `arbitrary_precision` only when the selected types realize a
  `serde_json::Number` or `serde_json::Value` (`realize/rust.rs:289`, `:308`, `:314`; check
  `Shape::Literal` too).
- When they do, the manifest declares a default-on crate feature (name it; `exact-numbers` is the
  suggestion) that enables `serde_json/arbitrary_precision`. The `serde_json` dependency line itself
  does not name the feature. `raw_value` for binary64 plans is unchanged.
- The types report (`types-report.json`) names, for that crate, that turning the feature off
  realizes numbers as binary64 and lists the fields affected.

## Two phases

**Phase 1 (now): no build slot.** Do not run `cargo`, `rustc`, `task` or anything that compiles.
You may run the installed `ess` 0.55.0 binary (it is not a build) to look at today's output. Write:

1. the failing tests in `crates/generate/schema-contract/tests/` (a new file is fine): the
   manifest for a model with no number shape carries no `arbitrary_precision`; a model with one
   gets the default-on crate feature; and a case that builds a generated crate with and without
   default features and asserts `serde_json::to_vec` of `{"a":1.50,"b":1e2}` gives
   `{"a":1.5,"b":100.0}` without it and `{"a":1.50,"b":1e+2}` with it. Follow how the existing
   `normalization_*_targets.rs` tests build generated crates.
2. the exact implementation plan in your report.

Then stop and report with the header below. `verdict: blocked`, `needs-coordinator: build slot`.

**Phase 2 (later, when the coordinator resumes you):** run the tests red and quote the red run,
implement, run green, `cargo fmt -p schema-contract`, clippy and test for `schema-contract`, then
`ess-cli`'s tests that cover `generate types` output (find them; package-scoped).

## Report header

```
unit: story:generated-rust-types-make-exact-numbers-a-crate-feature
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
