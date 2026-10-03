---
format: aep.planning-md/3
id: review-result:legacy-runner-entry-migration-312-20261003-r1
kind: review-result
status: active
title: Legacy runner entry migration independent whole-unit review
relations:
- reviews: story:feature-request-312
revision: 1
---
approve

# Story 312 legacy runner entry-point migration — independent whole-unit review round 1

Candidate `6d5886b1df775386592f0afc2c02aedc390ab041` (tree `109fb29a430f43a72bee3dfb0dbbb5262bb48cee`) is approved over base `5d3de744f6c59e40f6dadea36ac7aad7b99d40cd` for the bounded `execution.rs` and `faults.rs` migration. The review found no correctness, compatibility, acceptance, provenance, or scope finding.

The governing story requires current synthesized and deliberately mutated suites in these two binaries to be admitted and executed through `run_admitted`, while preserving custom clocks/configuration, all 32 tests, exact diagnostic and faulty-target assertions, legacy refusal behavior, and deterministic report equality (`.engineering/planning/story/feature-request-312.md:69-75`). The candidate satisfies that scope without touching production runner code or legacy format policy.

The only changed paths are `crates/verify/ess-conformance/tests/execution.rs` and `crates/verify/ess-conformance/tests/faults.rs`. Their SHA-256 digests are respectively `1581fb0e90e45c0c54943fcbc6ad1b87ff97cfc81cee497e86cf10ae52e63a1a` and `017d0ea95bf3759ec2e6c2d2ec75be4e39e5a466e29d21050a7009a0fe1aefe8`. A reviewer-regenerated binary diff is byte-identical to the author patch, SHA-256 `735af560af714e1761cc9549c8cb766f5953870a76eb6622e9168a208809274d`.

`execution.rs` admits the exact canonical current suite, constructs the runner from the admitted suite, executes through `run_admitted`, and retains the diagnostic report (`execution.rs:88-94`). Both narrowed custom-runner tests admit before runner construction, seed IDs from the admitted suite, and retain the exact `RunnerConfig` and `AdvancingClock` values (`execution.rs:534-647`).

`faults.rs` now sends Billing, Oracle, and Retry through one admitted helper (`faults.rs:123-146`). All healthy-system counts, `Fault::ALL` matrices, exact diagnostics, blast-radius limits, concurrent and injected history controls, and canonical report-equality assertions remain intact (`faults.rs:168-195,1224-1253`). No legacy `.run(...)` or `.try_run(...)` call remains in either file.

The candidate retains 12 `execution` tests and 20 `faults` tests, with no ignored or `should_panic` test. No assertion, expected count/status, diagnostic text, target/fault selection, seed count, clock budget, or determinism comparison changed.

The reviewer directly executed the exact retained binaries without Cargo or compilation:

- `execution-430c867f011cc109`, SHA-256 `08898a837d5e7a52c4c1a96eb5b434781000b5dc5a1b66388a7cf50752a3363f`: 12/12 passed, zero ignored/filtered, exit 0. Log SHA-256 `55125b4b6ed7277e77e94aed0be27c9ed897eec761078f73ad10a98b3ea7f1f3`.
- `faults-2d768c688660e46a`, SHA-256 `b4f35cdc00a747e727f24ebedb0b23485a8e21cc46df2aafc897c83b21fa414c`: 20/20 passed, zero ignored/filtered, exit 0. Log SHA-256 `b8d217b37dd4555dd752e3fe8d5953af78810a63e8eb625485a6656ce7099168`.

The reviewer also hash-verified, but did not rerun, the author's 32/32 compile-and-test log (`50946790b028693ea6c92bde5da86a0d917dcbed19a8538e7012aca710bbeb3f`), strict Clippy log (`8141ae10e81e907aa3ffe4d1b6033afe2d46ba8ba5ce2f77a9285c04e04bf123`), package-format log (`7431a0e3a6740a94c5802db6c6e69743628077c1a655320a186f354bc884a1b3`), and final audit (`a33931b1c191717813937b4027bd4463429d71fe861f7c54d8725b75699aa966`).

```findings
[]
```
