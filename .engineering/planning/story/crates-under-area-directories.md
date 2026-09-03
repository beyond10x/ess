---
format: aep.planning-md/1
id: story:crates-under-area-directories
kind: story
status: implemented
title: Crates move under area directories; names and consumers unchanged
summary: git mv the 20 crates into crates/{specify,generate,verify,infra,edge}; fix member paths, workspace dependency paths and the seven Taskfile fixture paths; no crate renamed.
relations:
- decomposes: epic:area-layout
scope:
- confidence: cited
  path: AGENTS.md
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: README.md
- confidence: cited
  path: Taskfile.yml
- confidence: cited
  path: crates
revision: 8
---
# Story: Crates move under area directories; names and consumers unchanged

## Context

Target layout:

```
crates/specify/   ess-primitives ess-domain ess-compiler ess-composition ess-realization
crates/generate/  ess-gen ess-synth ess-openapi schema-contract ess-deployment
crates/verify/    ess-conformance ess-diff
crates/infra/     infra-domain infra-compiler infra-analyze infra-spec infra-project ess-kubernetes
crates/edge/      ess-cli ess-xtask
```

`ess-deployment` sits under `generate/` because its dependencies are `ess-compiler` and
`ess-realization`, not `infra-*`. Known literal paths: `Cargo.toml` `[workspace] members` and
`[workspace.dependencies]` (the `examples/*` members and the `exclude` list stay);
`Taskfile.yml:55-72` names `crates/ess-composition/tests/fixtures/...` and
`crates/ess-cli/tests/fixtures/schema-contract/...`. Anything else `rg 'crates/'` finds outside
`target/` and `CHANGELOG.md` is in scope, including path dependencies from `examples/` or
`generated/` workspaces if any exist.

## Acceptance

`git mv` has placed every crate under its area directory, `task check` exits 0 from a clean
checkout, and every literal `crates/…` path that names a workspace crate resolves on disk in every
tracked file, with these exclusions only: `target/`, `CHANGELOG.md`, `.engineering/` (append-only
journal and CLI-owned artifacts), and the dated record `docs/design/`, `docs/reviews/`,
`docs/extraction.md`. `docs/plan/` is live and is rewritten; `website/docs/` is published source and
is in scope. The check is the test in `crates/edge/ess-xtask/tests/layout.rs`: it collects
area-qualified paths `crates/<area>/<crate>/…`, asserts it collected at least one, and matches
excluded paths root-relative.

## Notes

No crate is renamed and no `Cargo.toml` `name` changes. `CHANGELOG.md` gains an Unreleased entry.
The README gains a short area list. Atlas records components as `crates/<name>/Cargo.toml`; the
catalog update is the coordinator's step after this lands.
