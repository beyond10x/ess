---
format: aep.planning-md/3
id: story:a-rust-runner-compares-a-decimal-by-exact-value
kind: story
status: draft
title: The Rust runner compares a Decimal by exact value
summary: Rust reads a Decimal through binary64 before the runner sees it, so it compares values Go and TypeScript keep apart; deferred from unit E of 0.70.0.
tags:
- defect
- ess-0.71.0
- severity-high
- zero-open-issues
relations:
- serves: vision:O2
- informed_by: decision-blocker:decimal-canonical-serialization-crosses-repositories
- decomposes: epic:zero-open-issues-2026-11
scope:
- confidence: inferred
  path: crates/specify/ess-primitives/src/facts.rs
- confidence: cited
  path: crates/specify/ess-primitives/src/node.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/runner.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/retained_replay.rs
- confidence: cited
  path: docs/design/retained-command-results.md
- confidence: cited
  path: website/docs/reference/formats.md
revision: 5
---
## Problem

Unit E of 0.70.0 made a `Decimal` comparable in both exact observers (complete subject, retained
result). Go (`exactEqual`, `big.Rat` over the `json.Number` tokens) and TypeScript (`compareExact`)
compare by exact value. The Rust runner cannot: `ConformanceTarget::execute_command` returns `Node`,
and `Node::from_value` (`crates/specify/ess-primitives/src/node.rs:55-75`) reads every non-integer
and every integer past u64 through binary64. So `0.1` and `0.10000000000000001`, `1` and
`1.0000000000000000001`, and two 30-digit integers one apart compare as one value, and the three
runners disagree on one suite: a target whose retry changes a retained `Decimal` result fails under
Go and TypeScript and passes under Rust.

Reproduction (in-process; a CLI run cannot make a built-in target answer two different decimals):
`crates/verify/ess-conformance/tests/retained_replay.rs::adversary_e_two_decimals_inside_the_exact_range_replay_unequal`
drives a `DecimalBackend` that answers `original` and then `retry` for the `/outcome/replayed`
scenario of a model whose retained field is a `Decimal`. On main it asserts that no pair fails,
i.e. it pins the defect. The documented divergence is in
`docs/design/retained-command-results.md:114-117` and the `ess-conformance/58` row of
`website/docs/reference/formats.md`.

The fix is wider than the observation path: either `Number`/`Node`/`CanonicalValue` keep the
authored digits and write them (every `Node` user and every suite and report byte pin), or the public
`ConformanceTarget` result carries raw number tokens (a documented adopter contract). Both cross the
parked decision `decision-blocker:decimal-canonical-serialization-crosses-repositories`.

## Acceptance

- `adversary_e_two_decimals_inside_the_exact_range_replay_unequal` asserts that each of the three
  pairs (`0.1`/`0.10000000000000001`, `1`/`1.0000000000000000001`,
  `123456789012345678901234567890`/`123456789012345678901234567891`) ends `Status::Failed` under the
  Rust runner, and passes.
- `adversary_e_decimal_spellings_of_one_value_replay_equal` still passes (`1.5e0`/`1.5`, `0`/`-0.0`
  replay equal).
- `docs/design/retained-command-results.md` and `website/docs/reference/formats.md` state one rule
  for all three runners, without the Rust exception.

## Scope

- cited: crates/specify/ess-primitives/src/node.rs
- cited: crates/verify/ess-conformance/tests/retained_replay.rs
- cited: docs/design/retained-command-results.md
- cited: website/docs/reference/formats.md
- inferred: crates/verify/ess-conformance/src/runner.rs
- inferred: crates/specify/ess-primitives/src/facts.rs

confidence: low

## Also open

From the same review: a `Binary64` declared directly as a field of a binding delivery context is
not in `binary64.rs` `uses_of` (the actor-attribute arm was added in 0.70.0).
