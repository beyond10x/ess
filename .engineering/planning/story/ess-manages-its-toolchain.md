---
format: aep.planning-md/2
id: story:ess-manages-its-toolchain
kind: story
status: implemented
title: Any ess runs the release a repository pins
relations:
- supersedes: story:external-mutation-explorer-and-toolchain
- decomposes: epic:retrofit-findings-20260927
- serves: vision:O2
revision: 4
---
## Scope

- #147: `ess` reads a project toolchain pin, downloads the named release, verifies it against the
  release's `SHA256SUMS`, caches it per version and delegates; `ess install <version|tag|rev>`,
  `ess toolchain list|which`.

Split from `story:external-mutation-explorer-and-toolchain` (archived). Cited sites:
`crates/edge/ess-cli/src/requires.rs`, `src/input_discovery.rs:17-174`, `src/main.rs:49`.
Open: where the pin lives (`.ess.toml` or `requires:`), which HTTP crate.

## Acceptance

In a repository pinning another release, `ess --version` names the dispatcher and the delegated
release; a release that is already cached runs offline; a missing release is refused
non-interactively with the nearest downloadable release named.
