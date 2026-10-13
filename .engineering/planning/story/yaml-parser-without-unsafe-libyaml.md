---
format: aep.planning-md/3
id: story:yaml-parser-without-unsafe-libyaml
kind: story
status: draft
title: No published crate depends on serde_yaml or unsafe-libyaml
revision: 1
---
## Summary

Every ESS crate that reads YAML depends on `serde_yaml` 0.9.34+deprecated, which pulls in `unsafe-libyaml` 0.2.11. A consumer's `cargo deny check advisories` therefore reports RUSTSEC-2024-0449. This story replaces the dependency in the workspace `Cargo.toml` and 23 crate manifests, and in the 211 Rust files that name it, so no published ESS crate depends on either package. Closes https://github.com/beyond10x/ess/issues/541.

## Design first

A design page under `docs/design/` chooses the replacement before any code changes. For each candidate (a maintained fork with the same serde API, or a pure-Rust YAML parser behind a small ESS adapter), the page states:

- which source positions it reports, because the compiler's `SourceMap` and every located diagnostic depend on them;
- how it handles duplicate keys, anchors and aliases, tags and merge keys, compared with today's behaviour;
- whether any canonical byte changes. A change needs a format consequence decided under the repository's format rules.

## Acceptance

- A new test, `crates/edge/ess-xtask/tests/layout.rs::no_published_crate_depends_on_serde_yaml_or_unsafe_libyaml`, fails today. It reads `cargo metadata --locked --offline` and refuses either package in the dependency graph of any published crate.
- Every existing test that pins a diagnostic position or a canonical byte passes unchanged. Where one moves, the move is named in the design page with its format consequence.
- `cargo deny check advisories` over a crate that depends on `ess-domain` and `ess-compiler` reports no RUSTSEC-2024-0449.

## Scope

Inferred, not yet read file by file: the workspace `Cargo.toml` and `Cargo.lock`, `fuzz/Cargo.lock` (AGENTS.md: the second lock file), the 23 crate manifests that `git grep -l serde_yaml -- '*Cargo.toml'` lists, and the Rust files that call it. The load-bearing ones are in `ess-primitives`, `ess-domain` (`wire.rs`) and `ess-compiler` (`source`). This change touches every crate, so it is one wave on its own, after wave m. Its units are split by crate area once the design page has chosen the parser.

## Estimate

About 14G for the workspace build of the gate. Adversary: yes, on source positions and duplicate-key refusals.
