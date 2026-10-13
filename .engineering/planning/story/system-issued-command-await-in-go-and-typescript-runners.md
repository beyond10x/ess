---
format: aep.planning-md/3
id: story:system-issued-command-await-in-go-and-typescript-runners
kind: story
status: draft
title: The Go and TypeScript runners await a system-issued command
revision: 1
---
## Outcome

The generated Go and TypeScript conformance runners execute the await step of a system-issued
command (`AwaitOutcome(command, instance, bound)`), with the same answers as the Rust runner and
the interpreter, so a Go or TypeScript implementation can be held to a system-issued command.

Stage 2 of https://github.com/beyond10x/ess/issues/542 (stage 1:
`story:system-issued-command-is-awaited-not-driven`).

## Decisions

- Same interface and semantics as stage 1; a target that does not implement `AwaitOutcome` answers
  unsupported, so existing Go and TypeScript targets keep their results.

## Acceptance

Each line names the failing test written first.

- `crates/verify/ess-conformance/tests/system_issued_command.rs`
  `the_go_runner_awaits_a_system_issued_command`, `the_typescript_runner_awaits_a_system_issued_command`,
  `all_runners_agree_on_an_await_refusal_within_the_bound`.

## Scope

- inferred: `crates/verify/ess-conformance/src/{go,ts}/` runner templates, emitted READMEs.

## Plan

- depends_on: `story:system-issued-command-is-awaited-not-driven`.
- du estimate: 8G.
- adversary: yes (parity across three runners).
