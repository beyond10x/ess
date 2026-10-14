---
format: aep.planning-md/3
id: story:ess-yaml-owns-its-parser-and-emitter
kind: story
status: draft
title: ess-yaml owns its parser and emitter, without serde_norway
relations:
- depends_on: story:yaml-parser-without-unsafe-libyaml
- serves: vision:O2
revision: 1
---
## Summary

`ess-yaml` is backed by `serde_norway` since the release that closed https://github.com/beyond10x/ess/issues/541, a stopgap that keeps libyaml's C-to-Rust translation under the parser. This story replaces that backend with the one `docs/design/yaml-parser-replacement.md` § Decision chose: `saphyr-parser` (pinned exactly) for events, a port of `serde_yaml` 0.9.34's serde layer, and an ESS-owned emitter for the events `serde_yaml` sends. `ess-yaml` then drops `serde_norway`.

## Acceptance

- The design page's units n6 (emitter) and n7 (reader), each with its differential test in `ess-xtask` against `serde_yaml` as a dev-dependency oracle, over every committed YAML file, the generated documents and the page's scalar sample.
- A differential fuzz target in `fuzz/` comparing reads, refusals and writes.
- `cargo xtask generate --check`, `cargo xtask site-data --check` and every existing test pass unchanged; no format version moves.
- `cargo metadata` shows neither `serde_norway` nor `unsafe-libyaml-norway` in the graph of a published crate.

## Scope

`crates/specify/ess-yaml` only, plus `ess-xtask` tests, `fuzz/` and its lock file. Adversary: yes, on source positions, duplicate-key refusals and emitted bytes.
