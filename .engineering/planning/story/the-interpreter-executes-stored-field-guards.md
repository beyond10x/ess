---
format: aep.planning-md/3
id: story:the-interpreter-executes-stored-field-guards
kind: story
status: active
title: The interpreter executes guards over the subject's stored fields
relations:
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/adversary_mixed_guard_pass1.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_stored_guards.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/mixed_guard_wrong_state.rs
revision: 5
transitions:
- {from: "draft", to: "proposed", at: "2026-10-02T19:04:59Z", actor: "human:timo", revision: 3, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-02T19:05:28Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-takeover", correlation: "consumer-runtime-20261002"}
---
# Story: the interpreter executes guards over the subject's stored fields

## Outcome

`ess verify conform run --target interpreted` executes `when_subject` / subject-field guards instead of refusing the command as not interpreted, so the interpreter can act as the model authority for stored-field semantics (e.g. which branch applies in a state the command cannot run from).

## Why

Found while fixing #192 (2026-09-28): `crates/verify/ess-conformance/src/interpret/execute.rs:456` refuses every `SubjectField`/`SubjectPredicate` guard (`NotInterpreted { construct: "the guard over the subject's stored fields of ..." }`), so the ruling that stored fields select a branch before wrong_state rests on the Entity Runtime ordering (`ess-entity-runtime/src/lib.rs:1450`) only.

## Acceptance

- The #192 shapes (`mixed_guard_wrong_state.rs`, `adversary_mixed_guard_pass1.rs`) run on the interpreted target and give the synthesized verdicts.

## Scope

- `crates/verify/ess-conformance/src/interpret/execute.rs` — cited
