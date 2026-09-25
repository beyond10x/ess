---
format: aep.planning-md/2
id: story:a-multi-paragraph-summary-emits-valid-rust
kind: story
status: archived
title: A multi-paragraph summary emits valid Rust
relations:
- decomposes: epic:specification-runs-as-a-fake-backend
revision: 3
---
## The defect

A domain `summary:` holding more than one paragraph validates, and then emits a module doc comment
in which only the first paragraph carries its `//!` prefix:

```rust
//! Manager bootstrap — `adopter.fe.bootstrap`.
//!
//! The four reads the manager makes before it shows anything. …
None of these views can be scoped to the caller. An ESS view takes no parameters, …
//!
```

```console
$ ess specify validate --path docs
adopter v1 — 25 file(s), valid

$ cargo build -p adopter-realization
error: expected one of `!` or `::`, found `of`
 --> generated/rust/adopter/crates/adopter-types/src/bootstrap.rs:9:6
```

## Reproduction

Any domain whose `summary:` is a YAML folded block containing a blank line. It reproduced on both
domains written that way and on neither single-paragraph one, against ESS 0.23.0. `--target go` was
not checked.

## Why it matters more than the one-line fix

Nothing between the author and `cargo` catches it. `ess specify validate` is the gate an adopter is
told to trust, and it passes a specification that cannot be compiled.

## Acceptance

The emitter prefixes every line of a multi-line summary, and a test covers a summary containing a
blank line.

## Retirement under the revised ESS evolution scope

The operator explicitly excluded this epic and all twelve dedicated stories on 2026-09-15 in approved plan ess-evolution-20260915 revision 1. Retire this draft through AEP without deleting its original content, relations or journal history. Its acceptance is no longer a requirement of the current initiative. Generic future protocol/UI work is recorded separately in task:deferred-protocol-ui-bindings and does not reopen this artifact.
