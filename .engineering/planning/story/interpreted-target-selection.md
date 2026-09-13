---
format: aep.planning-md/1
id: story:interpreted-target-selection
kind: story
status: implemented
title: An operator can select an interpreter target
summary: --target interpreted is offered, selected and runs a suite, deriving nothing yet
owner: ess
tags:
- priority-high
relations:
- decomposes: epic:model-driven-interpretation
- serves: vision:O2
scope:
- confidence: inferred
  path: changes
- confidence: cited
  path: crates/edge/ess-cli
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: inferred
  path: crates/edge/ess-cli/tests
- confidence: inferred
  path: crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
- confidence: cited
  path: crates/verify/ess-conformance
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/target.rs
revision: 7
---
# Story: an operator can select an interpreter target

## Outcome

`ess conform run --target interpreted` is offered by the CLI's own target list and runs a committed
suite to completion, in the same argument slot that already names `billing` and `oracle`. The target
derives no behaviour yet: every scenario reports an unsatisfied obligation rather than a pass, which
is the honest answer for a target that has been selected and has decided nothing.

This is the seam the rest of the epic fills. It exists separately because a target that cannot be
named cannot be checked, and because everything after it is a claim about behaviour rather than
about wiring.

## Acceptance

`ess conform run --target interpreted` against `examples/billing`'s committed suite exits with a
report whose scenarios are all unsatisfied obligations and none an error, and `ess conform run
--help` lists `interpreted` beside the existing targets.

## Scope

Confirmed by the implementor against the tree, 2026-09-11. Corrections are visible rather than
deleted, because the next wave selects on overlap by reading this.

- `crates/edge/ess-cli/src/main.rs:608-612` (the `--target` value enum) and `:2808-2813` (its
  construction) — **cited, confirmed exact**.
- `crates/verify/ess-conformance/src/target.rs:87`, the `ConformanceTarget` trait — **cited,
  confirmed**; read, not changed.
- `crates/verify/ess-conformance/src/interpret.rs`, 152 lines — the new module. The scoper inferred
  the filename and was right.
- `crates/verify/ess-conformance/src/lib.rs` — **the scope was half wrong.** It said "the `pub mod`
  list *and* `pub use` block". A `pub use` is inventoried as a `…::reexport::<name>` consumer entry
  needing its own classification (`consumer.rs:305-313`), and the siblings are reached by module
  path anyway. Only `pub mod interpret;` was added.
- `crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json` — **confirmed required and
  undercounted.** 14 ids, not the smaller set the pattern suggested: every module file also gets a
  `…::module` entry, of which 258 exist. An unclassified entry bails at `proposal.rs:486-488`.
- `crates/edge/ess-cli/tests/interpreted_target.rs`, 184 lines — the acceptance's own cases.
- **`website/docs/status/where-this-stands.md:68` — missing from the original scope and a gate
  blocker.** Generated from `conform run --help` by `ess-xtask/src/support.rs:334`, rendered at
  `:568`, compared back by `support.rs:31-67`, and run as `cargo xtask support --check` at
  `Taskfile.yml:204`. A package-scoped lane never executes it. Regenerate with `cargo xtask support`;
  never hand-edit.
- **`website/docs/guides/verify-conformance.md:146-149` — also missing.** Prose with no generator
  behind it, which is why it went stale; now covered by a case that parses it against live help.
- `docs/design/review-public-support-claims.md:66` — deliberately left stale. A commit-pinned wave-14
  record whose citations already resolved to unrelated code at this story's base; `docs/design` is
  not served by the Docusaurus site, so no published claim depends on it.
- **Wrong in the original scope:** `report.rs:46-58` only declares `Status`; the mapping from
  `TargetError::unsupported` to an unsatisfied obligation is `runner.rs:2431-2441 target_failure()`.
- `task consumer-check` cannot run on this machine: it pins `RUSTUP_TOOLCHAIN=1.98.1`, absent. The
  classification check above is static only.
