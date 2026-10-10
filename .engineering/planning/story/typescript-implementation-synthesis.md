---
format: aep.planning-md/3
id: story:typescript-implementation-synthesis
kind: story
status: draft
title: ess generate synthesize has a TypeScript target
tags:
- adopter-report
- deployment-chain
- design-first
- feature-request
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

`ess generate synthesize` has a TypeScript target that writes the implementation skeleton (types,
behaviour ports, server) the Rust and Go targets write.

## Evidence

An adopter on ess 0.55.0 building a Node service. On main `SynthesisTarget` is
`Rust | Go | Web | Clap` (`crates/edge/ess-cli/src/main.rs:563-569`); TypeScript exists only as a
conformance-suite target (`SuiteTarget::Typescript`, `main.rs:1086-1087`).

## Acceptance

- A design page decides the TypeScript target's scope against the Rust and Go targets.
- `ess generate synthesize --target typescript` writes a project that type-checks and passes the
  synthesized conformance suite on an example, in a test.
