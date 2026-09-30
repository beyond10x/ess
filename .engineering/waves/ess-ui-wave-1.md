# ess-ui wave 1 — renderer-neutral UI document

Epic: `epic:ess-ui-renderer-neutral-ui` (serves `vision:O2`). Skill: aep:implementing 0.16.0, wave mode.
Integration branch: `integrate/ess-ui-1`, forked from the 0.46.0 release commit `285910a79`
(PR #252; rebased onto `main` once it merges). Managed integration tree id: `ess-ui-int`.

## Selection

`aep plan artifact waves --kind story --status draft` (aep 0.65.0), lines for these stories verbatim:

```
wave 1
  story:ui-spec-schema (inferred)
wave 2
  story:ui-spec-checks (inferred)
  story:ui-spec-docs-generator (inferred)
  story:ui-spec-react-renderer (inferred)
  story:ui-spec-tui-renderer (inferred)
wave 3
  story:ui-spec-test-language (inferred)
```

Collisions involving these stories: none. Unassessed among these stories: none.
Every scope entry is `inferred`: the crates do not exist yet; each story owns one new directory.

## Shared files (coordinator-owned)

- workspace `Cargo.toml` members for `crates/ui/*`;
- `crates/edge/ess-cli/src/main.rs`: the `ess ui check|docs|run|test` and `ess generate ui` subcommands,
  wired by the coordinator at merge from each unit's library entry point.

## Units

| Wave | Story | Branch | Managed tree id | Build dir | Scratch | Stage |
|---|---|---|---|---|---|---|
| 1 | ui-spec-schema | impl/ui-schema | ess-ui-schema | b10x-target/ess-ui-schema | ess-ui-wave/schema | dispatched |
| 2 | ui-spec-checks | impl/ui-checks | ess-ui-checks | b10x-target/ess-ui-checks | ess-ui-wave/checks | approved |
| 2 | ui-spec-docs-generator | impl/ui-docs | ess-ui-docs | b10x-target/ess-ui-docs | ess-ui-wave/docs | approved |
| 2 | ui-spec-tui-renderer | impl/ui-tui | ess-ui-tui | b10x-target/ess-ui-tui | ess-ui-wave/tui | approved |
| 2 | ui-spec-react-renderer | impl/ui-react | ess-ui-react | b10x-target/ess-ui-react | ess-ui-wave/react | approved |
| 3 | ui-spec-test-language | impl/ui-test | ess-ui-test | b10x-target/ess-ui-test | ess-ui-wave/test | approved |

Build dirs are under `$HOME/.cache/`, scratch under `$HOME/.cache/`; worktrees under the managed
worktree root.

## Commits approval authorises

Six unit commits, their merges into `integrate/ess-ui-1`, the coordinator's CLI-wiring and store
commits on that branch, and the merge into `main` once the gate is green. No push beyond publishing
the integration branch for its pull request, no tag, no release.

## Pre-flight

- Free disk at proposal: 24G (`df -h /`); floor 10G per build; waves 2 runs four builds, so it runs
  two at a time unless disk allows more.
- Model budget: not stated; default N = 4.
- Primary checkout: on `main`, untracked `.agents/` (not this wave's; left alone).

## Approval

Operator approved all three waves in sequence, 2026-09-30.
