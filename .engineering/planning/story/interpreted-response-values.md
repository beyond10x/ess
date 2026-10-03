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
revision: 8
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

## Independent review and committed correction, 2026-10-03

Pass1 review found that a source-admitted creation identity sourced from response.id was evaluated before private response preparation. Two compiled regression controls both failed before correction: the successful creation could not read the actual response, and the rollback control stopped before its intended invariant. The fix prepares the selected creation candidate before identity evaluation, reuses that same private response for stored identity, event and returned response, and discards staged issuance on failure. Noncreation selection retains its previous missing-instance/wrong-state ordering. No public response injection or expectation authority was added.

Actual corrected interpreted_response_values binary:9passed0failed, including successive distinct creation identities and failed creation versus a fresh control. Strict scoped Clippy passed. Genuine red log SHA256 d212ab98262c3b3030bfd292e904671c4a53497f9754c0340fcb06e91bc1d2b8; green cbd23e54bca5057e12fa7f054fb4a21735bb9f5167580d1a54c3b81a414bade5. Earlier harness compile errors are not the red evidence. Independent pass2 approves, findings empty, reviewer executions0; review-result:consumer-native-response-pass2 retains its exact scope and evidence.

Bot source commit e9d543ac4bb5fd5d2a58d51b7fab8d60a62073af on fix/interpreted-response-values-20261003 contains the exact reviewed candidate. Applying reviewed patch f1c4794577fc935fec30ed9f7a72c2a3a0a685a478389735c7ab21f735262126 to an independent index reconstructed tree e1296613dd3022df77b94b5fcbfc1cc3553685e9, identical to the commit tree. The staged patch has different ordering, not different content. Author and committer are b10x-bot[bot]. No push, PR or remote gate was triggered.

Root integration preserves the newer support_versions::legacy_json fixture helper instead of the worker's older inline metadata normalization. All other source edits applied unchanged. Integrated library/response/generation/one-time/existence checks are running; results and integrated source commit will be recorded separately. Story remains active, with retained replay and nested/browser observations still required elsewhere.

## Integrated native response addendum, 2026-10-03

Append source commit046db8a6805154aa5cafc63b0a4b741bc26e954d after the frozen57-commit source list, yielding58 selected source commits. The original frozen list and its hashes remain unchanged. This commit imports independently reviewed worker e9d543ac4bb5fd5d2a58d51b7fab8d60a62073af, preserving the existing support_versions::legacy_json helper in response_payload.rs. Both author and committer are b10x-bot[bot]. Nine source/test files only; no planning, generated artifact, delivery-control or unrelated documentation edit.

Integrated actual results:97library,9interpreted_existence,9interpreted_response_values,1interpreted_responses,7one_time_contract,21one_time_execution,17one_time_generation,8response_payload,2response_union,21upsert_by_existence:192passed0failed0ignored0filtered. Scoped strict Clippy covering that same library/test selection exits0. Repository task fmt-check exits0. The initial cargo fmt --all --check also reached byte-pinned generated Rust and reported formatting differences; the Taskfile explicitly excludes generated projections from formatter input. No generated bytes were rewritten. The scoped and repository-defined formatting checks both pass; package-wide/full release gates remain pending.

Carrier evidence SHA256: response-integrated-tests.log a8cfdc77f76745efc3baf11fe5e3f8cbe37021509ad619ae22d87fc3345a810b; response-integrated-clippy.log 5a1e3af04e934337e7ca5670f07289d6f796d4d37d654abacd54d8ae50794f9a; response-repository-fmt.log b1b92dbfb232bd4c5604171da6ab65153fb809d08740f37d511dce0c9d1fe085. Logs and terminal exit files are retained under carrier target/backlog-input.

Source tree ess-interpreted-response-values-20261003 remains retained on fix/interpreted-response-values-20261003 with local-only bot commit and review evidence. Root carrier ess-backlog-next-20261002 holds the integrated source on batch/consumer-runtime-20261002. The shared coordinator remains sole owner of batch/ui-live-apps-complete-20261003 and its eventual PR. The source handoff remains local and unacknowledged: cross-session MCP delivery still fails at its endpoint, so no receipt is inferred. No remote gate or component PR was started.

Full ess21 bundle hold remains. Nested response observation probe is independently recorded: healthy37/37 and unequal37/38 and38/37 all pass, while receipt=null fails. Native response execution fixes do not close that observer defect, browser product support, retained replay, or the full consumer backlog. PR402 remains open at9e15e08e9f0201e06bddc268934c3f96a92e5482; refreshed read shows CI progressing, not final Gate success, and unchanged source still lacks the reviewed contradictory-bound correction. Release owner retains that blocker.
