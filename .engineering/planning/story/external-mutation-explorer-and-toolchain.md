---
format: aep.planning-md/2
id: story:external-mutation-explorer-and-toolchain
kind: story
status: draft
title: Mutation audits an external target, the explorer takes external branches, ess manages its toolchain
relations:
- decomposes: epic:retrofit-findings-20260927
scope:
- confidence: inferred
  path: crates/edge/ess-cli/Cargo.toml
- confidence: cited
  path: crates/edge/ess-cli/src/input_discovery.rs
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/requires.rs
- confidence: inferred
  path: crates/edge/ess-cli/src/toolchain.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/go/explore.go
- confidence: inferred
  path: crates/verify/ess-conformance/src/go/runtime.go
- confidence: cited
  path: crates/verify/ess-conformance/src/mutate.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/ts/explore.ts
- confidence: inferred
  path: crates/verify/ess-conformance/tests/mutation_audit.rs
revision: 5
---
## Scope

- #153: `ess verify conform mutate` drives an external target (`--emit` mutant suites and
  `--collect` reports, or a target command).
- #156: the model explorer takes `external:` branches as choices a target's double arranges, and
  reports per-branch reach.
- #147: `ess` reads a project toolchain pin, downloads and verifies the named release, caches it
  and delegates; `ess install <version|tag|rev>`; `ess toolchain list|which`.

## Acceptance

- A mutation audit of a non-fixture specification runs against an external runner and scores it.
- An exploration of a spec with external branches reaches them and reports reach per branch.
- In a repository pinning another release, a plain `ess --version` names the dispatcher and the
  delegated release; a missing release is refused non-interactively.

## Derived scope

Derived 2026-09-27 by `aep:story-scoper`. Every line is **cited** or **inferred**.

- **Primary surface:** `crates/verify/ess-conformance` (#153 mutation engine, #156 Go/TS explorers) and `crates/edge/ess-cli` (#153 CLI, #147 toolchain) — cited
- **Files (#153):** `crates/edge/ess-cli/src/main.rs:588` (`ConformCommand::Mutate`), `:693` (`ReferenceTarget`), `:3025` (`conform_mutate`) — cited
- **Files (#153):** `crates/verify/ess-conformance/src/mutate.rs` (`MutateCode::BaselineFailed` = ESS-MUTATE-001, `apply`) — cited
- **Files (#156):** `crates/verify/ess-conformance/src/go/explore.go:518-590` (`explorePlan`, `Exclusion`) — cited
- **Files (#156):** `crates/verify/ess-conformance/src/ts/explore.ts:390-420` — cited
- **Files (#147):** `crates/edge/ess-cli/src/requires.rs`, `src/input_discovery.rs:17-174`, `src/main.rs:34-36,1131,49` — cited
- **Also likely:** `go/runtime.go:1093` and the TS `configureExternalOutcome` (callers) — inferred
- **Also likely:** `go/mod.rs`, `ts/mod.rs` (`EXPLORE_README`) — inferred
- **Also likely:** a new `crates/edge/ess-cli/src/toolchain.rs` and a download dependency in `crates/edge/ess-cli/Cargo.toml` — inferred
- **Also likely:** tests `ess-cli/tests/{mutate_cli,explore_package}.rs`, `ess-conformance/tests/{mutation_audit,explore}.rs` — inferred
- **Documents:** `docs/design/mutation-audit-and-model-runner.md`, `docs/design/specification-requires-release.md`, cli, verify-conformance guide, formats — inferred
- **Confidence:** medium — #153 and #156 sites read; #147 is new code
- **Would collide with:** any unit editing `ess-cli/src/main.rs`, `ess-conformance/src/mutate.rs`, or the Go/TS explorer and runtime templates
- **Not established:** the explorer excludes any condition other than `when`/`otherwise`/`wrong_state` (a new condition kind is needed, reading not run); whether the explorer target exposes external control; which #153 option; where the #147 pin lives; whether `--version` delegation precedes clap; no ESS model covers these commands; which HTTP crate.
