---
format: aep.planning-md/3
id: story:a-rust-runner-compares-a-decimal-by-exact-value
kind: story
status: draft
title: The Rust runner compares a Decimal by exact value
summary: Rust reads a Decimal through binary64 before the runner sees it, so it compares values Go and TypeScript keep apart; deferred from unit E of 0.70.0.
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- informed_by: decision-blocker:decimal-canonical-serialization-crosses-repositories
revision: 1
---
## Problem

Unit E of 0.70.0 made a `Decimal` comparable in both exact observers (complete subject, retained result). Go (`exactEqual`, `big.Rat` over the `json.Number` tokens) and TypeScript compare by exact value. The Rust runner cannot: `ConformanceTarget::execute_command` returns `Node`, and `Node::from_value` (`crates/specify/ess-primitives/src/node.rs` about :56) reads every non-integer and every integer past u64 through binary64, so `0.1` and `0.10000000000000001`, `1` and `1.0000000000000000001`, and two 30-digit integers one apart compare as one value. The three runners can therefore disagree on one suite.

Adversary case `adversary_e_two_decimals_inside_the_exact_range_replay_unequal` (`crates/verify/ess-conformance/tests/retained_replay.rs`) is committed ignored as this finding.

The fix is wider than the observation path: either `Number`/`Node`/`CanonicalValue` keep the authored digits and write them (every `Node` user and every suite and report byte pin), or the public `ConformanceTarget` result carries raw number tokens (a documented adopter contract). Both cross the parked canonical-serialization decision.

## Acceptance

- The ignored adversary case passes un-ignored: the Rust runner fails each of the three pairs, as Go and TypeScript do.
- `docs/design/retained-command-results.md` and `website/docs/reference/formats.md` state one rule for all three runners.

Also open from the same review: a `Binary64` declared directly as a field of a binding delivery context is not in `binary64.rs` `uses_of` (the actor-attribute arm was added in 0.70.0).
