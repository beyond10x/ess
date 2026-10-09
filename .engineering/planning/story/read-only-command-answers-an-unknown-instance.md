---
format: aep.planning-md/3
id: story:read-only-command-answers-an-unknown-instance
kind: story
status: draft
title: A read-only command declares its answer for an unknown instance
tags:
- adopter-report
- defect
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
revision: 1
---
## Outcome

A read-only command (every outcome `preserves:` its subject; nothing moves, updates, creates or
deletes) can declare an `unknown_instance:` branch, so a query defines its answer for an addressed
record that does not exist and synthesis writes a scenario that fails a server mishandling it.

## Evidence

An adopter on ess 0.56.0 (neutral repro): entity `demo.token.Token {state: Active|Revoked|Expired}`;
command `demo.token.Introspect(token)` whose outcomes only preserve the token and return a
response; an `unknown_instance: true` outcome returning `active: false` is refused
`unreachable_branch` ("answers an identity no record carries, and no branch … acts on an instance
its input names"). On main `names_existing`
(`crates/specify/ess-domain/src/command/outcome_shapes.rs:434-441`) counts only `Moves`, `Updates`
and `Deletes` with a command-input instance, and the refusal is at `outcome_shapes.rs:466-480`; a
`preserves:` branch does not count. The workaround is a fake state change.

## Acceptance

- The repro validates; a command with no branch addressing an instance from its input still
  refuses `unknown_instance:` as today.
- Synthesis writes the absent-record scenario for the read-only command, and the interpreted,
  Rust, Go and TypeScript targets agree on it.
- Separate from `story:refusal-above-held-state-branch-validates` (branch precedence).
