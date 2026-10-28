---
format: aep.planning-md/3
id: story:the-workspace-compiles-on-its-declared-rust-version
kind: story
status: active
title: The workspace compiles on its declared rust-version
summary: ess-conformance compiles on rustc 1.85, the declared rust-version, and an MSRV CI job holds every pull request to it
tags:
- ess-0.75.0
relations:
- serves: vision:O2
revision: 3
transitions:
- {from: "draft", to: "proposed", at: "2026-10-28T08:42:09Z", actor: "human:timo", revision: 2, executor: "agent:b10x-chef-orchestrator-ess-0720"}
- {from: "proposed", to: "active", at: "2026-10-28T08:42:10Z", actor: "human:timo", revision: 3, executor: "agent:b10x-chef-orchestrator-ess-0720"}
---
## Problem

A downstream repository's CI found this while moving to ESS 0.72.0. The repository has an MSRV 1.85 job that compiles `ess-conformance`.

- The workspace declares `rust-version = "1.85"` (`Cargo.toml:57`).
- `ess-conformance` 0.72.0 does not compile on rustc 1.85.0.
- Location: `crates/verify/ess-conformance/src/synthesize/subject_fact.rs:6537`, `Some(&declared.state_field().type_ref)` inside an if/else.
- Error: `state_field()` returns an owned `ResolvedField`, and rustc 1.85 drops that temporary at the end of the statement: error[E0716], temporary value dropped while borrowed. Newer compilers extend the temporary's lifetime.
- The line came in with 032576ec1d (2026-10-10).

Nothing in ESS CI checks the declared rust-version, so this could ship. Other lines may break on 1.85 too; this one is only the first error rustc reported.

## Acceptance

- `cargo +1.85.0 check --workspace --all-targets --locked` passes on the release head. Alternatively, the declared `rust-version` is raised to the oldest toolchain that passes, with a CHANGELOG entry naming the move.
- ESS CI has an MSRV job that runs that check on every pull request against the declared `rust-version`, taken from `Cargo.toml` rather than a second literal.
- The fix for `subject_fact.rs:6537` binds the state field to a local that outlives the borrow. It changes no synthesized bytes, and the suite-bytes tables do not move.
