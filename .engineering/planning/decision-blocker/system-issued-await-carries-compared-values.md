---
format: aep.planning-md/3
id: decision-blocker:system-issued-await-carries-compared-values
kind: decision-blocker
status: open
title: Does the system-issued await contract carry the compared input values?
summary: 'Story a-system-issued-awaited-step-honours-its-outcomes-input-guard needs AwaitRequest.compared to carry values; options A (contract carries values, later wave) and B (names only, narrow the story); recommendation A; Decider: operator'
relations:
- blocks: story:a-system-issued-awaited-step-honours-its-outcomes-input-guard
revision: 1
---
# Does the system-issued await contract carry the compared input values?

`story:a-system-issued-awaited-step-honours-its-outcomes-input-guard` needs an accepting branch's
equality input guard to reach the target. Today `AwaitRequest.compared` carries input field names
only, never values (`docs/design/system-issued-commands.md:327`), and the interpreter reads it to
answer unsupported where a scenario compares a value the model does not determine. Writing the
guard's input on the `AwaitOutcome` therefore changes the target contract in Rust, Go and
TypeScript, which is an architectural change. Wave W1 unit SYN1 parked its patch at the stop
(coordinator scratch, outside the repository).

## Options

- **A.** The await contract carries the compared values (`compared` becomes field to value), in
  Rust, Go and TypeScript, with a design-page update; scheduled in a later wave.
- **B.** Keep names only and narrow the story to what synthesis can state without values (for
  example, refuse the scenario by name where the accepting branch has an input guard).

## Recommendation

A: the runners already compare every stated await input (`runner/await_outcome.rs:304-318`,
`go/await.go:172`), so carrying the values closes the gap without withdrawing scenarios.

Decider: operator

Recorded by wave W1 (orchestrator D401, 2026-11-02). The story is not waved until this is answered.
