---
format: aep.planning-md/3
id: story:component-release-check-toolchains
kind: story
status: implemented
title: Provision component-owned release check toolchains
summary: Let an ESS component release select its Rust, Node, and pnpm check toolchains without forking the OCI release pipeline.
relations:
- decomposes: epic:oci-component-delivery
scope:
- confidence: cited
  path: .github/actions/release-component/action.yml
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: Cargo.lock
- confidence: cited
  path: Cargo.toml
- confidence: cited
  path: Taskfile.yml
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-09-04T04:36:44Z", actor: "human:timo", revision: 3, imported: true}
- {from: "proposed", to: "active", at: "2026-09-04T04:36:44Z", actor: "human:timo", revision: 4, imported: true}
- {from: "active", to: "implemented", at: "2026-09-04T04:42:56Z", actor: "human:timo", revision: 6, decided_on: {"recorded":{"test_result":2}}, imported: true}
---
## Outcome

The reusable ESS component release action provisions an exact component-selected Rust toolchain and optional Node/pnpm toolchains before running the repository-owned check command.

## Acceptance

- Rust toolchain selection is an explicit reusable-action input with the existing baseline as its default.
- Node and pnpm setup is opt-in and version-pinned for components whose checks include generated browser surfaces.
- Components that do not need Node retain the current release path and cost.
- The action remains registry-, service-, and Service SDK-neutral.
- Repository checks cover the new action inputs and conditional setup.

## Scope

- `.github/actions/release-component/action.yml`
- `Taskfile.yml`
- `CHANGELOG.md`
