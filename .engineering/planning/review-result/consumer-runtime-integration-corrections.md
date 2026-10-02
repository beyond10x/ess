---
format: aep.planning-md/3
id: review-result:consumer-runtime-integration-corrections
kind: review-result
status: active
title: Independent review of combined runtime corrections
relations:
- reviews: story:feature-request-389
revision: 1
---
approve

```findings
[]
```

Bounded read-only review of ess-backlog-next-20261002 at d170c67f6 plus integration-expectations.patch SHA256 0a3eabfc64f7a12d1d53209ba75d9f98ef3ea8ef8648d85dfaa4bbaed4751465 and the untracked crates/verify/ess-conformance/tests/one_time_event_batch.rs SHA256 d10a05622cd835fc232be595467a22e1da36c9d562dff802bc81dff6b09d8435. Own test/build executions: 0. No implementation, test or planning edits. No concrete defect found in this correction scope.

The direct-response and structured-reference changes replace obsolete language-capability refusals at the same emit and emit_input/model entry points. They do not erase malformed/older-version controls: adversary_instlist_pass1.rs still rejects extra keys, unavailable vocabulary, invalid nested shapes and the /33 document relabelled /31; its new positive emit calls use the original admitted /33 input. The nearby native actual-execution assertions remain, and the separate live Go/TS parity suites supply runtime evidence. Emission success alone is not claimed here as execution proof.

Future-version rejection remains negative on both targets. adversary_generated_docs_pass2.rs now exercises /36 and /37 above the supported /35 ceiling and requires UnsupportedTarget, the exact provenance path, actual requested version, supported maximum and target name. ts/mod.rs returns those details; both language emitters remove advice to use a Rust runner that also lacks the future vocabulary. Existing emitted-runtime ceiling comparison remains intact. The two added TS package paths and three new field-name grammar sites update exact artifact/grammar inventories rather than removing checks.

Report/2 expectations preserve the new five-category contract. periodic.rs still executes both the healthy and unsupported Go cases; the latter now runs in its separately selected subprocess, where nonzero strict exit is expected. Both report files are read through CountReport with passed/unsupported/skipped counts checked. The Go fixture checks unsupported=4, skipped=0, passed=0 and failed execution/conformance, while the healthy case checks passed=4. This avoids making the initial healthy command fail merely because its old loop also executed the now-nonzero unsupported producer. pre_execution_fixtures.rs continues all eight fixture modes in Go and Node, retains exact begin/end callback expectations and value-free fixture refusal text, and now accepts successful exit only for the valid case; unsupported is not normalized to skipped/success.

The held-state correction strengthens behavior checks: refusal_beside_state.rs executes unconfigured, configured and revoked stores; only the configured store may rotate and store the supplied secret. Both denied states require unchanged store, no event and the exact NotConfigured error. The removed expectation was the obsolete interpreter capability refusal, not a declared business refusal.

The native polling guards preserve an already-recorded disclosure/resource result. runner.rs:983 stops before event matching, timeout diagnostics or another callback when remember marks the run stopped. runner.rs:1454 returns None after the same condition; both count_before and nothing_published_after already turn None into Flow::Stop, so no count-based success/failure is fabricated afterward. The new actual target regression returns an aggregate two-by-600000-byte ordinary event batch on the second observation, requires Unsupported and exactly two observations, and checks returned plaintext is absent from scenario diagnostics. It would fail both the original late timeout override and an extra poll. Other shared resource controls provide the healthy bounded cases; this review adds no new execution claim.

The coordinator's integration-corrections-2.log now records 3/0 and 13/0 for its two selected binaries. Those are coordinator executions, not mine. The earlier full-package failure counts and native full-capability freeze remain separate evidence; this review does not certify the still-pending combined suite or interpreter trust matrix.
