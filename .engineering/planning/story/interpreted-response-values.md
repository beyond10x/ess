---
format: aep.planning-md/3
id: story:interpreted-response-values
kind: story
status: active
title: Execute native response mappings with one transactional response
relations:
- decomposes: epic:downstream-reported-gaps
- serves: vision:O2
scope:
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/command.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/execute/values.rs
- confidence: cited
  path: crates/verify/ess-conformance/src/interpret/protected.rs
- confidence: inferred
  path: crates/verify/ess-conformance/src/interpret/response.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/interpreted_response_values.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_payload.rs
- confidence: cited
  path: crates/verify/ess-conformance/tests/response_union.rs
revision: 6
transitions:
- {from: "draft", to: "proposed", at: "2026-10-03T00:02:39Z", actor: "human:timo", revision: 4, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
- {from: "proposed", to: "active", at: "2026-10-03T00:02:40Z", actor: "human:timo", revision: 5, executor: "agent:codex-ess-backlog", correlation: "consumer-runtime-20261002"}
---
## Outcome

The native Interpreted conformance target executes every admitted ResponseField mapping by reading the single actual response produced for that invocation, including legacy mappings without returns:true and nested destination leaves. Complete response generation, row/event publication and private one-time issuance state commit together after successful candidate execution.

## Fit review

Internal gap under the operator's standing all-features instruction. Existing binding designs typed-response-outcome-payloads.md, direct-library-returns.md, one-time-response-values.md and retained-command-results.md already define authority. No new source syntax or domain entity is introduced. Scope report d829db314ae24ea766ab8bd1de3da2cf9d8c5d8ad6396453c6e568c507ab5a9c, private interpreted-response-next-scope.md, identifies actual admitted fixtures and current missing evaluator/ordering. This extends historical interpreted-command-execution without rewriting its earlier completion.

## Decisions

Prepare one private invocation-owned response for the actually selected successful outcome when returns:true or a recursively nested event ResponseField requires it. Selection, missing-instance and wrong-state outcomes remain authoritative. Refusals issue nothing. Legacy source4 response mappings do not require returns:true. Copy from that response into event leaves and return the identical response, never mint a second value or use runner expectations. Validate both declared source and target types; preserve optional absence/null, exact integers, complete aggregate shape and existing resource limits. Opaque conversions remain explicit refusals.

Stage a clone of private Issued before generation; a failed value, invariant, at-rest check or non-unique candidate discards all staged issuance/state/events. complete_command commits the prepared response and staged Issued once. Keep protected plaintext outside public Step/Store/history serialization and generic Debug output. Marked values still cannot flow into events or unrelated response positions. An unmarked response field mapped beside a marked field must execute correctly. Existing dispatch occurs after commit and is not promised rollback by this unit.

Public execute/execute_generating retain their current no-response-authority behavior; no new public provided-response API or Generated policy is added in this bounded unit. The complete native Interpreted target supplies private actual response authority. Retained replay is subsequent work: never implement it by generating a fresh response. This explicit boundary does not remove replay or observer gaps from the full backlog.

## Acceptance

Actual source4 response-payload.yaml and response-union.yaml suites run on Interpreted without dropping assertions. Direct returns with no event mapping remain green. Add named controls for nested destination leaves; whole record/list/map/tagged-union values; independent generated receipt; absent/null/present Optional and declared presence; exact large integers; repeated fields/events and successive invocations; independent response/event mutation; refusal with no response; failure after preparation leaves rows/events/issuance unchanged against a fresh control; protected origin with unmarked event mapping remains fresh without disclosure; marked-flow and marked-replay source refusals remain intact. Measure the same admitted tests red before implementation and green after.

Native nested execution is not full nested conformance coverage: story:nested-response-observations separately requires the missing synthesized relationship witness and native/Go/TypeScript/WASM mutant parity. Do not claim that work from direct native assertions.

## Scope

Cited: crates/verify/ess-conformance/src/interpret.rs; interpret/command.rs; interpret/protected.rs; interpret/execute.rs; interpret/execute/values.rs. Inferred: optional private interpret/response.rs. Cited tests: new interpreted_response_values.rs plus response_payload.rs and response_union.rs. Existing interpreted_responses, one_time_generation/execution/contract and response mutation suites are validation neighbors, not permission to weaken them.

## Delivery

One managed tree based on reviewed runtime integration617cfa2248, own lease and owned servers cache; jobs2/debug0/incremental0/external TMPDIR,8GiB floor. Rust-only committed executable code. Root owns AEP, synthesis/observers and publication; other session owns318/282/304/319 and third owns390–395. No shared synthesis edits. Freeze exact source/report for independent review before bot commit. No component PR/full remote gate. Current standing backlog authorization covers this bounded continuation.

## Recovered worker checkpoint, 2026-10-03

Worker stopped after the service reported its usage limit; root inspected actual Git and processes. Nine scoped files remain uncommitted in ess-interpreted-response-values-20261003 at base617cfa2248. Initial private unit controls4/0 and changed integration tests17/0 passed. The original strict lint exited101 on function length and single-match style; root extracted an ordinary-response-field helper and shortened the response binding, then reran scoped strict Clippy successfully. No authority or source scope broadened.

Recovered full library plus focused response/generation run:96library +7interpreted_response_values +1interpreted_responses +17one_time_generation +8response_payload +2response_union =131passed0failed, exit0. Evidence target/backlog-input/response-values-recovered-tests.log/.exit and response-values-clippy-recovered.log/.exit in the managed response tree. Exact nine-file candidate patch831fc5b434664ccfe839d3822bf8c885335448a4ef7f419c1c582f2f83c5ecb9. Root changes are part of that frozen candidate and must be reviewed too.

On the older base, one_time_execution and three one_time_contract cases fail before execution because canonical fixtures lack the new initial-state metadata; root already migrated those fixtures in8b3f128a0. Preserve that evidence and run both full binaries after reviewed import. Do not weaken tests or temporarily alter the worker fixture authority. Independent review, integrated neighbor verification and bot commit remain pending. This is not a completed story or release claim. Retained replay and nested/browser observer gaps remain separate required work.
