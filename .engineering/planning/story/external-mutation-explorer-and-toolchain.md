---
format: aep.planning-md/2
id: story:external-mutation-explorer-and-toolchain
kind: story
status: draft
title: Mutation audits an external target, the explorer takes external branches, ess manages its toolchain
relations:
- decomposes: epic:retrofit-findings-20260927
revision: 2
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
