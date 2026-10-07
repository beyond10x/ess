# Unit brief U3, wave 2026-10-07b

## Identity

```
story:  story:an-idle-output-record-carries-no-machine-path   (beyond10x/ess#484)
branch: unit/ess-wave-20261007b-u3-idle-record   forked from integrate/ess-wave-20261007b
base:   see `git -C <worktree> merge-base HEAD integrate/ess-wave-20261007b`
```

Read the story, its fit review and the chosen design (option d):
`aep plan artifact show story:an-idle-output-record-carries-no-machine-path`.
Issue: https://github.com/beyond10x/ess/issues/484. Design page:
`docs/design/review-output-ownership.md` (the root-binding section, #306).

## The triple

```
worktree:  ~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b-u3
build dir: <worktree>/target   (already created, with Cargo's CACHEDIR.TAG)
scratch:   <worktree>/target/wave-scratch/u3
```

## The file assignment

| | |
|---|---|
| **yours** | `models/output-ownership/**`, `crates/edge/ess-cli/src/output_ownership/**`, the `ess-cli` tests that cover output ownership (new files are fine), `docs/design/review-output-ownership.md`, any reference or guide page under `website/docs/` that describes `.ess-output/state.json` |
| **not yours** | everything else; `CHANGELOG.md` and `.engineering/**` are the coordinator's |

Invariants: `.engineering/waves/2026-10-07b/invariants.md` in the integration tree
(`~/.local/state/worktree/trees/b10x/ess/ess-wave-20261007b`). Read it first. `AGENTS.md`
"Determinism and formats" applies in full: a new state format version, an old-reader compatibility
test, and ordered collections only.

## Two phases

**Phase 1 (now): no build slot.** Do not run `cargo`, `rustc`, `task` or anything that compiles. You
may run the installed `ess` (0.55.0):

1. Spec first: move `native_root` from `Anchor` to `Transaction` in
   `models/output-ownership/domains/ownership.yaml` (and the README sentence that describes it), then
   `ess specify validate --path models/output-ownership`. Quote the output.
2. Write the failing tests:
   - a write-mode generate leaves an Idle `ess-output-state/3` record with no `root` and no
     `directory`, and no component of the output's absolute path appears anywhere in the file;
   - a record with a transaction keeps both, and a mismatched in-flight binding still refuses;
   - an old `/2` Idle record is admitted, and a write-mode run rewrites it as `/3` even when no
     owned file changed;
   - `--check` on a `/2` Idle record reports no drift and prints one warning naming the fields;
   - the old-reader compatibility case the AGENTS.md format rule asks for.
3. Write the exact implementation plan into your report.

Then stop: `verdict: blocked`, `needs-coordinator: build slot`.

**Phase 2 (when resumed):** red run (quote it), implement, green run, `cargo fmt -p ess-cli`,
`cargo clippy -p ess-cli --all-targets --locked -- -D warnings`, the ess-cli output-ownership test
binaries by name (`cargo test -p ess-cli --locked --test <name>`), and `task` steps that check
models (`Taskfile.yml:174` validates `models/output-ownership`).

## Report header

```
unit: story:an-idle-output-record-carries-no-machine-path
verdict: <green|red|blocked>
cases: executed <before>→<after>, red <n>
origin: n/a
wrote-outside-worktree: <paths, or none>
needs-coordinator: <what, or none>
```
