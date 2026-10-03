---
format: aep.planning-md/3
id: story:ess-protospec-foundation
kind: story
status: implemented
title: Execute and check bounded communicating protocol models
summary: Experimental protocol documents, deterministic execution, trace checking and bounded exploration.
relations:
- derived_from: design:ess-protospec
- serves: vision:O2
scope:
- confidence: cited
  path: CHANGELOG.md
- confidence: cited
  path: crates/edge/ess-cli/src/main.rs
- confidence: cited
  path: crates/edge/ess-cli/src/protocol.rs
- confidence: cited
  path: crates/edge/ess-cli/tests/protocol_cli.rs
- confidence: cited
  path: crates/edge/ess-xtask/src/docs.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/lib.rs
- confidence: cited
  path: crates/specify/ess-compiler/src/protocol.rs
- confidence: cited
  path: crates/specify/ess-compiler/tests/protocol.rs
- confidence: cited
  path: crates/specify/ess-domain/src/lib.rs
- confidence: cited
  path: crates/specify/ess-domain/src/protocol.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/lib.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/protocol
- confidence: cited
  path: docs/design/protocol-specification.md
- confidence: cited
  path: examples/protocols
- confidence: cited
  path: models/protospec
- confidence: cited
  path: website/docs/reference/cli.md
- confidence: cited
  path: website/docs/reference/spec-versions.md
revision: 13
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T00:29:48Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-protospec", correlation: "ess-protospec-build-20261003"}
- {from: "proposed", to: "active", at: "2026-10-03T00:29:48Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-protospec", correlation: "ess-protospec-build-20261003"}
- {from: "active", to: "implemented", at: "2026-10-03T02:08:09Z", actor: "human:timo", revision: 9, decided_on: {"recorded":{"test_result":1}}, executor: "agent:codex-ess-protospec", correlation: "ess-protospec-build-20261003"}
---
## Goal

Make protocol behavior executable as typed participants, messages, channels and timers, so independently observed conversations can be checked against allowed behavior.

## Fit review

1. Need: ordinary command histories cannot represent exact message delivery, timer generations or cross-peer obligations.
2. Existing surface: ESS predicates/types, canonical digests and native conformance provide precedents; history linearization omits input payloads and cannot serve as this oracle.
3. Ownership: authored contracts in ess-domain, admission in ess-compiler, execution and observation checking in ess-conformance, file I/O in ess-cli.
4. Generality: terminal response flushing and lost acknowledgements are two distinct bounded protocol cases.
5. Compatibility: use an explicit experimental ess-protospec/1 sidecar, not silently extended ordinary ESS or conformance-suite formats. Existing projections do not consume this format.
6. Smallest useful change: finite machines and bounded message/timer execution with explicit failed/inconclusive outcomes, native target seam, replay and exploration.
7. Evidence: reference-model tests plus independent handwritten target fixtures and deliberate wrong-output/timer mutants; no claim of downstream implementation coverage.

## Decisions

Accept the need, redesign the first integration as a separately versioned protocol document rather than introducing declarations ignored by current schema/OpenAPI/code projections. User authorized implementation on 2026-10-03. The broader design remains a roadmap; arbitrary expressions, binary codecs, durable recovery and universal clock synchronization remain deferred. The declared nouns are modeled in models/protospec.

## Acceptance

An authored bounded two-peer protocol can be validated and compiled, executed through explicit send/deliver/timer actions, explored within declared limits and checked against complete observed traces with reproducible evidence, and the terminal-response and lost-acknowledgement examples detect independently introduced behavior faults while existing formats remain unchanged.

## Verification

Run the affected Rust crates' tests, formatting and strict Clippy, task ci-lint for the public API and task site-build for documentation. Preserve actual red and green output. No downstream adapter or full gate claim is implied.

## Conformance scenarios

- `terminal_and_lost_ack_examples_replay_and_mutants_fail`: both authored examples replay, and an extra application notification is rejected.
- `independent_target_checks_actual_flush_order_and_always_cleans_up`: an independent terminal fixture passes; closing before flush fails; adapter cleanup is observed after failure.
- `independent_ack_target_detects_duplicate_application_notification`: a handwritten bounded ACK fixture passes; duplicate application delivery fails.
- `retransmission_backoff_caps_and_ack_cancels_both_server_timers`: retransmission backoff reaches the cap, ACK cancels G/H, and I/D remain independently owned.
- `replay_keeps_both_nondeterministic_states_until_observation_disambiguates`: equal observations preserve both compatible states.
- `trace_digest_completeness_unknown_fields_and_duplicates_are_checked`: stale digests and malformed trace vocabulary are refused; incomplete capture is inconclusive.
- `cli_simulation_replay_and_mutation_have_distinct_exit_codes`: end-to-end simulation/replay reports passed, failed and inconclusive distinctly.

These exercise the experimental protocol runtime. They do not claim ordinary ESS suite synthesis or execution against a downstream stack.

## Delivery evidence

Implemented the experimental sidecar, strict compiler admission, pure protocol runtime, bounded exploration, exact trace checker, native adapter interface and five CLI commands. Terminal-response and lost-ACK examples run end to end. The broad design remains a roadmap; no downstream implementation adapter or complete RFC conformance is claimed.

Verification on 2026-10-03: all 546 integration targets across ess-domain, ess-compiler, ess-conformance and ess-cli passed. Unit suites passed (361, 29, 109, CLI library 6 and binary 47), as did their doc tests. The protocol unit lane is 0 to 17 passing tests, with additional compiler and CLI acceptance cases. Behavioral red evidence covers missing commands, initial safety, flush obligations, invalid witnesses, future timer cancellation and adapter admission classification. Final strict package Clippy, task ci-lint, task site-build, source formatting, CLI-reference drift and diff whitespace checks passed. The full release gate was not run locally.

Retained evidence: target/protospec-evidence/README.md and its selected logs and exact example traces in the managed worktree ess-protospec-aep-20261003. Helpers stopped at an execution-service usage limit; the coordinator completed implementation and review locally. No independent final agent-review claim is made. Work remains uncommitted and unpublished for operator review, with recovery through the managed-worktree archive.

## PR preparation

The operator authorized committing, pushing and opening a pull request against remote main on 2026-10-03, superseding the initial local-only handoff. The branch was updated to main at e68684efb. Integration retained the publisher commands, adjusted the CLI leaf count to 80 and placed the new changelog entry under Unreleased; protocol semantics did not change.

On the updated tree, protocol compiler tests (3), the complete conformance unit suite (109), the CLI binary and protocol/command-surface tests passed. The CLI reference was regenerated; task ci-lint and task site-build passed again. Logs are retained under target/protospec-evidence/pr. The earlier 546 integration-target result applies to the original implementation base; CI owns the full combined-tree gate.

## CI correction

PR #411's required Checks job exposed a missing integration obligation: the generated CLI documentation names ess-protospec/1 and ess-prototrace/1, but FORMAT_RELEASES does not register either family. Both projection-check and the existing published-family xtask regression correctly refuse the omission. Register both as unreleased experimental formats and document them in the version reference, without claiming a released version or weakening the checker. Integrate current remote main and verify the affected documentation/projection and xtask lanes before updating the bot-authored PR.
