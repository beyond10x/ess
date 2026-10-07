---
format: aep.planning-md/3
id: specification:cli-binding-paths-are-independent-of-the-cli-block
kind: specification
status: draft
title: 'A CLI binding''s command paths are independent of a component''s cli: block'
refs:
- provider: github
  reference: beyond10x/ess#482
relations:
- serves: vision:O2
revision: 1
---
## What was found

https://github.com/beyond10x/ess/issues/482 asks that `ess specify cli` refuse a binding command
path missing from the component's `cli:` block. Measured on 0.55.0: a binding presenting
`[no-such, word]` validates (`… — valid CLI presentation binding`).

## Why it is not built

The two are independent routes. A component's `cli:` block drives the Clap grammar emitter
(`website/docs/concepts/ess.md`, CLI-only example); an `ess-cli/1` binding is "an independently
authored" presentation for `ess generate cli` (`website/docs/reference/cli.md`, CLI presentation
bindings), and `compile` in `crates/specify/ess-cli-contract/src/resolve.rs:462` validates it
against the model's operations and types, never against `cli:` groups. A binding may present a
path no group declares, and a `local` callable has no group at all.

## Decision

**decline, with the idiom**: a binding is its own grammar. The reference page now says so in one
sentence. A team that wants the two to agree generates from one route only.
