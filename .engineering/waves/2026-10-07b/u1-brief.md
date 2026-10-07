# Unit brief U1, wave 2026-10-07b

## Identity

```
story:  story:entity-runtime-lowers-alphabet-and-text-count
branch: unit/ess-wave-20261007b-u1-alphabet-count   forked from integrate/ess-wave-20261007b
base:   see `git -C <worktree> merge-base HEAD integrate/ess-wave-20261007b`
```

Read the story: `aep plan artifact show story:entity-runtime-lowers-alphabet-and-text-count`.
Upstream facts: https://github.com/beyond10x/entity-runtime/issues/54 (last comment) and the
entity-core sources of the pinned release in cargo's git checkout cache once `Cargo.lock` points at
it (`crates/entity-core/src/definition.rs`, `validation.rs`, `tests/text_alphabet.rs`,
`tests/service_values.rs`). Do not run any git command against another repository.

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b-u1
build dir: <worktree>/target
scratch:   <worktree>/target/wave-scratch/u1
```

## The file assignment

| | |
|---|---|
| **yours** | `crates/generate/ess-entity-runtime/**`, `Cargo.toml` (the `entity-core` line only), `Cargo.lock`, `website/docs/reference/entity-runtime-lowering.md` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07b/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b`). Read it before anything else.

## Two phases

**Phase 1 (now): no build slot.** Do not run `cargo`, `rustc`, `task` or anything that compiles.
`cargo metadata --offline` is not allowed either. You may:

1. Move the `entity-core` pin in `Cargo.toml` to tag `0.28.0` (the newest; it adds no lowering
   feature over 0.27.0, so 0.27.0 is the fallback if 0.28.0 breaks something). Do not touch
   `Cargo.lock` in phase 1.
2. Read the entity-core 0.27.0 change from the GitHub release notes and issue #54 with `gh`
   (read-only), and the ESS lowering code.
3. Write the failing tests: re-write
   `tests/lowering.rs::an_alphabet_and_a_text_length_are_refused_by_name_by_the_lowering` into
   cases that assert the two constructs lower (and what the lowered definition carries), plus cases
   that the two still-unsupported shapes (a text length on a quantifier element or as a projection
   key; an alphabet on a declared response field) keep refusing by name. Update the subset entries'
   evidence names in `src/subset.rs` only if the test names change.
4. Write down, in the report, the exact implementation plan: which functions in `src/lib.rs` and
   `src/subset.rs` change and how.

Then stop and report with the header below. `verdict: blocked`, `needs-coordinator: build slot`.

**Phase 2 (later, when the coordinator resumes you):** `cargo update -p entity-core`, run the
tests red and quote the red run, implement, run green, `cargo fmt -p ess-entity-runtime`, clippy and
test for `ess-entity-runtime` only, regenerate whatever the subset table drives (find the task in
`Taskfile.yml`; `cargo xtask` targets are package-scoped builds too).

## Report header

```
unit: story:entity-runtime-lowers-alphabet-and-text-count
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
