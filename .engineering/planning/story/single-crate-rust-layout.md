---
format: aep.planning-md/3
id: story:single-crate-rust-layout
kind: story
status: implemented
title: The rust target can emit one crate instead of a workspace
relations:
- serves: vision:O2
- decomposes: epic:generated-determined-behaviour
revision: 4
transitions:
- {from: "draft", to: "proposed", at: "2026-09-29T20:42:52Z", actor: "human:timo", revision: 2}
- {from: "proposed", to: "active", at: "2026-09-29T20:42:52Z", actor: "human:timo", revision: 3}
- {from: "active", to: "implemented", at: "2026-09-29T21:45:33Z", actor: "human:timo", revision: 4, decided_on: {"recorded":{"test_result":1}}}
---
## Outcome

`ess generate synthesize --target rust` has an opt-in single-crate layout for an adopter that
never deploys one component alone: one crate at the `--out` root instead of a workspace of types,
system, server and one crate per component. The multi-crate layout stays the default, so no other
adopter's bytes change.

## Acceptance

- `--layout crate` (default `--layout workspace`) writes `Cargo.toml` and `src/lib.rs` at the root,
  at most two levels deep: one module file per bounded context (as the types crate has today), the
  component ports in `src/ports.rs` (or one module per component under `src/ports/`), and
  `src/system.rs`.
- The HTTP surface (http, json wire codecs, routes, the transport-free entry point) sits behind a
  `server` Cargo feature, off by default; built without it, the crate uses no `std::net` (a test
  greps the feature-off build's sources and builds it with `-D warnings`).
- The same model's single-crate output passes its synthesized conformance suite with the `server`
  feature on, as the workspace layout does.
- `--layout workspace` output is byte-identical to today's.
- The plan and `synthesize.md` document the option.

## Origin

A downstream specification's generated tree is 14 crates, about 50,000 lines, four directories
deep; its adopter never deploys a component alone and finds the nesting hard to work with.
