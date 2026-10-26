---
format: aep.planning-md/3
id: story:a-create-or-update-writer-is-no-proof-a-stored-reference-is-present
kind: story
status: draft
title: A create-or-update writer is no proof that a stored reference is present
summary: The ess/30 enforced-reference proof and synthesis count an unknown_instance creator as refusing a missing related row first, although under the one rule it stores one; found by the 0.70.0 unit I adversary
tags:
- ess-0.71.0
relations:
- serves: vision:O2
- decomposes: epic:downstream-reported-gaps
revision: 1
---
## Problem

The `ess/30` proof that a stored reference names a present row (`crates/specify/ess-domain/src/command/reference_integrity.rs`, `checked`) counts a creating command as "refusing an identity no row carries before storing it" when the command has a `when_related: {via: input.<field>, exists: false}` branch. A create-or-update command with an `unknown_instance:` branch, `bound`, answers before that refusal under the one precedence rule and stores the identity anyway. So from `ess/31` (where the proof drops the requirement) a later command reading the stored reference may leave out its `exists: false` branch although the row can be missing, and synthesis (`crates/verify/ess-conformance/src/synthesize/related_guard/stored.rs`, `checks`) calls a declared `exists: false` branch unreachable.

Found by the 0.70.0 unit I adversary (`review-result:unit-i-pass-1`), pre-existing in the domain code.

## Acceptance

- `a_later_command_reading_the_stored_agent` in `crates/verify/ess-conformance/tests/adversary_i_create_or_update.rs` passes: a `Ping(seat_id)` with `agent-gone` and `pinged` synthesizes both branches and both pass.
- `a_stored_agent_written_by_the_creator_is_not_proven_present` passes: the same `Ping` without an `exists: false` branch is refused as `non_exhaustive_branches` at `ess/29` and at `ess/31`, its hint naming the create-or-update writer.
- A model whose writers all refuse first keeps its IR bytes and suite bytes.
