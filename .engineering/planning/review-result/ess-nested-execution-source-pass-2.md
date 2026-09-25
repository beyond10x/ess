---
format: aep.planning-md/2
id: review-result:ess-nested-execution-source-pass-2
kind: review-result
status: active
title: Original nested execution source review final pass2
relations:
- reviews: task:consumer-accounting-nested-execution
revision: 1
---
unit: M9 whole measured nested execution final composed working source on base f1af8280338b97d862a6c474ec50f78d5157d71c, manifest 2374a74a619ac9e792cb95933e75fc663d23724d61be63a06d65d6a068a05048
verdict: NEEDS-CHANGE
cases: executed 218 author-baseline unit cases→1 bounded reviewer selection, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 5 path roots
needs-coordinator: route the introduced unchecked nested-claim entrypoint binding to the implementor; preserve the separate unresolved common-admission refusal and provider-administration restriction

```text
 .../consumer_coverage/nested_execution_tests.rs    | 54 ++++++++++++++++++++++
 1 file changed, 54 insertions(+)
```

The cached candidate remains exactly 278 paths with cached-diff SHA-256
`011c88182443cf862096e1701bc49261edc2c3295acf27a36196a8040954eeb8`. The only unstaged
worktree path is the tests-only file above.

## 2. Case added before execution

`crates/edge/ess-xtask/src/consumer_coverage/nested_execution_tests.rs:731` adds
`nested_execution_rejects_a_stale_claim_entrypoint_binding`. It builds a claim whose entrypoint
source, declaration digest, AST digest, and source-file digest agree with a supplied source profile
and consumer inventory and first proves that baseline candidate construction succeeds. It then
changes only `entrypoint_source_file_sha256` and requires candidate construction to refuse it.

The exact bounded command was:

```console
env -u CARGO_TARGET_DIR CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true TMPDIR='home-path:sha256:c54df90ced2da6a21aa18dd4dd2a92e4eb1cc52b7c4578276a1d2dc4fca19c55' RUST_TEST_THREADS=1 RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' cargo +1.98.1 test --target-dir 'home-path:sha256:6ed1633f7c51d9df60d6e6555d12fa80168e9f40edd2cd8208683adb9aef8eae' --locked --offline -p ess-xtask -j 1 'consumer_coverage::nested_execution_tests::nested_execution_rejects_a_stale_claim_entrypoint_binding' -- --exact
```

The first invocation reached compilation but the orchestration call yielded without retaining its
session handle; its captured text ended after the five package compilation lines and carried no
exit status. An immediate process check found no remaining Cargo, rustc, or selected-test process.
The identical second invocation produced the first retained case result below. This missing first
terminal transcript is a review limitation; the retained rerun is the reproducible red result.

```text
   Compiling ess-xtask v0.24.0 (home-path:sha256:c30b9559b552847605564ddaf60347a0cdb64a6f86e9eb755d7c746ff348d4ae)
    Finished `test` profile [unoptimized] target(s) in 29.03s
     Running unittests src/main.rs (home-path:sha256:d5bef2647eec51b9df98e3fc806630302824a49e6e5550d969b1b3611de87113)

running 1 test
test consumer_coverage::nested_execution_tests::nested_execution_rejects_a_stale_claim_entrypoint_binding ... FAILED

failures:

---- consumer_coverage::nested_execution_tests::nested_execution_rejects_a_stale_claim_entrypoint_binding stdout ----

thread 'consumer_coverage::nested_execution_tests::nested_execution_rejects_a_stale_claim_entrypoint_binding' (3943666) panicked at crates/edge/ess-xtask/src/consumer_coverage/nested_execution_tests.rs:778:5:
a stale nested claim entrypoint must not reach accounting/6
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::nested_execution_tests::nested_execution_rejects_a_stale_claim_entrypoint_binding

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 221 filtered out; finished in 0.28s

error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

Retained run exit status: `101`. Before the first launch, available disk was `31,952,171,008`
bytes and available RAM was `49,759,002,624` bytes. After it, they were `31,773,249,536` and
`49,064,357,888` bytes. Final observation after review was `30,140,567,552` bytes disk and
`49,495,584,768` bytes RAM, above the 20 GiB/16 GiB floors.

## 3. Suite and other commands

No blanket suite or gate was run. The coordinator allocated only the exact case above. The final
author package evidence reports 218 unit tests passed and 3 native acceptance tests ignored; its
focused corrected-native controls, full package, strict Clippy, format check, native-20 preflight,
native-20 execution, and Go/Node execution exit files all contain `0`.

Source/evidence commands and outcomes:

- `sha256sum -c correction-1/complete-source-composed.sha256` checked all 2,134 paths and exited
  `0`; output is retained in `source-review-2/complete-source-verification.log`.
- A `jq` structural audit of the terminal execution envelopes exited `0`: native-20 has 20 selected
  cases, 20 receipts, outer counts 20/20/0/0, 429 observations
  (`generated_rust` 360, `child_ess_cli` 36, `firefox` 33), 168 exact scenarios passed, and one
  expected typed exit-1 refusal. Go/Node has 2 selected cases, 2 receipts, outer counts 2/2/0/0,
  2 observations and 2 scenarios passed. The output is retained in
  `source-review-2/evidence-structure-audit.log`.
- A four-document authority/candidate/plan/execution equality query initially exited `5` because
  the reviewer's `jq` expression had incorrect pipe precedence. The corrected query exited `0` and
  returned `true`: all four carry identical cases, every selected-case list equals its case keys,
  and execution receipt keys equal the planned case keys.
- Rehashing the three distinct retained outer-native images named by the 22 terminal receipts
  exited `0`. Their observed SHA-256/size pairs are
  `604609564cabb3f56cafaae8f7e677443725e4d9eb0742cf390f65da2eb74b8a/17351392`,
  `85a53f6b814391f70f4c25427e46f6e94d22b6194fdb86d9ec685ad828c5821d/18018896`, and
  `0722a50253a0257e5f05fc8ef346ee323a4af35752aa13b2c8a5c71c07acfa68/2894328`.
- `connectors --version`, `connectors --help`, and `connectors --output json inspect doctor`
  succeeded. The local configuration/state/keyring were healthy; the unused daemon was not running.
- `worktree hook session-start` and the subsequent heartbeat succeeded. Final Git checks before
  this report showed 278 staged paths, the unchanged cached digest above, and one unstaged test file.

## 4. Findings

| Location | Verdict | Origin | Finding and reachability |
|---|---|---|---|
| `crates/edge/ess-xtask/src/consumer_coverage/nested_execution.rs:721` | `NEEDS-CHANGE` | `introduced` | `candidate` validates case target/source/harness/step sources and case inventory at lines 683–720, then clones every claim into the candidate without comparing `entrypoint_source`, `entrypoint_sha256`, `entrypoint_ast_sha256`, or `entrypoint_source_file_sha256` with the supplied source profile and consumer inventory. The regression measures the mutant candidate returning success, so the refusal assertion fails at test line 778 with exit 101. The normal production path reaches this code through `prepare_nested_execution` at `mod.rs:701–720`, which supplies both current inputs; `plan_v6` and `qualify_v6` later match claim identity, cases and behavior but never the entrypoint fields (`enforce.rs:2626–2764`). Thus a stale or fabricated entrypoint binding can reach accounting/6 qualification. The sibling direct-model path already enforces the required binding at `model_behavior.rs:392–408`. The terminal diagnostic authorities also demonstrate present acceptance by carrying placeholder declaration/AST digests, though diagnostics themselves qualify zero cells. |

The finding covers the final composed working source plus this tests-only regression. The nested
module and its candidate path do not exist at base `f1af8280338b97d862a6c474ec50f78d5157d71c`, so the
origin is introduced. A correction must resolve the claim entrypoint uniquely in the supplied
package inventory and compare its source, declaration digest, source-item digest, and source-file
digest before candidate construction can succeed.

## 5. Exact source examined and attacks that did not establish another finding

The original 21-path mechanism manifest was examined against
`source-review-inherited-baseline/` and base `f1af8280338b97d862a6c474ec50f78d5157d71c`:

```text
crates/edge/ess-cli/tests/evolution_coverage_accounting.rs
crates/edge/ess-xtask/src/consumer_coverage/account.rs
crates/edge/ess-xtask/src/consumer_coverage/enforce.rs
crates/edge/ess-xtask/src/consumer_coverage/entry-classifications.json
crates/edge/ess-xtask/src/consumer_coverage/fixtures/nested_execution/accounting-v5-added-nested-negative.json
crates/edge/ess-xtask/src/consumer_coverage/fixtures/nested_execution/accounting-v5-valid.json
crates/edge/ess-xtask/src/consumer_coverage/fixtures/nested_execution/model-behavior-authority-v1.json
crates/edge/ess-xtask/src/consumer_coverage/fixtures/nested_execution/model-behavior-case-v1.json
crates/edge/ess-xtask/src/consumer_coverage/mod.rs
crates/edge/ess-xtask/src/consumer_coverage/model_behavior.rs
crates/edge/ess-xtask/src/consumer_coverage/native.rs
crates/edge/ess-xtask/src/consumer_coverage/nested_acceptance_tests.rs
crates/edge/ess-xtask/src/consumer_coverage/nested_execution.rs
crates/edge/ess-xtask/src/consumer_coverage/nested_execution_tests.rs
crates/edge/ess-xtask/src/consumer_coverage/nested_harness.rs
crates/edge/ess-xtask/src/consumer_coverage/reviewed-candidates.json
crates/edge/ess-xtask/src/consumer_coverage/reviewed-nested-execution.json
crates/edge/ess-xtask/src/main.rs
crates/edge/ess-xtask/tests/nested_runtime_accounting.rs
crates/specify/ess-composition/tests/evolution_composition_accounting.rs
docs/design/consumer-nested-execution.md
```

The complete correction delta was also examined, including its modifications to `mod.rs`,
`native.rs`, `nested_acceptance_tests.rs`, `nested_execution.rs`, and
`nested_execution_tests.rs`, plus the three composed provenance-only files
`non-authority-browser-paired-replay.json`, `non-authority-coverage-execution.json`, and
`non-authority-coverage-selection.json`. Their only change is the already-recorded relative
`obligations_source`; digest and ledger values are unchanged. Direct caller sections in the S6
composition test, S8 CLI/browser test, browser helper, executor, model-behavior verifier,
accounting readers, aggregate input path, CLI command surface, normal gate, and diagnostic path
were followed. The inherited S6/S8 source judgements were not reopened.

- The pass-1 outer-native defect is coherently corrected in production. `execute_nested` returns a
  nonserialized per-case map captured from the gate-owned `Native`; normal and diagnostic callers
  pass tuple member `.1` directly; final verification requires exact case keys, exact receipt/map
  equality, and reopens the retained copy to check digest and size. It does not fall back to the
  receipt. Positive, substitution, absent, changed and restored controls use real readable bytes.
- Authority/candidate/plan/execution stage and format tags, unknown-field refusal, unique JSON
  keys, nonempty case/claim coverage, exact case identity/AST/source/profile bindings, case-key
  equality, exact ordered step/ordinal observations, nonce/journal/native bindings, tool versions
  and executable bytes, producer/consumer artifact continuity, typed refusal diagnostics, result
  counts, and outer exact-libtest 1/1/0/0 checks were traced. No second reachable failure was
  established.
- All five finite runtime kinds are present in source and terminal evidence. The final native-20
  run covers 18 complete S6 generated-Rust cases and 2 S8 cases with child ESS CLI and 33 Firefox
  sessions. The separate gate-owned run covers actual Go and Node/TypeScript fixtures. Observed
  source/profile/authority/candidate/plan/case/native/tool/launch/result fields propagate through
  their receipt and envelope paths; the unchecked claim entrypoint fields are the exception
  reported above.
- Accounting/6 creates an explicit version boundary, first validates the inherited accounting/5
  plan, partitions inherited legacy cases into disjoint legacy/nested sets, reuses the unchanged
  accounting/5 qualifier only after combining verified tokens, and retains aggregate,
  reconciliation, acquisition, metadata and outside-boundary checks. Fixtures preserve direct
  model-behavior /1 bytes and accounting /1–/5 reader refusal boundaries.
- Canonical `reviewed-nested-execution.json` remains the deliberate empty pending authority and is
  refused before candidate construction. Diagnostic executions record `qualified_cells: 0` and
  do not grant production authority. Candidate authority population, aggregate/reconciliation,
  full native/G4 qualification and consumer-enabled ESS/site gates remain coordinator duties.
- Final author artifacts checked were report SHA-256
  `469887ed48ce500119ff30a9a91124e7a0c3017a0e40644b3bc24c0693cba80c`, complete-source manifest
  SHA-256 `2374a74a619ac9e792cb95933e75fc663d23724d61be63a06d65d6a068a05048`, source digest
  `cc954f0203531f162348b8e4964ae589b014ea2657889f589d538e8c03cb7742`, native-20 envelope
  `11f92ac7cf6ba2317b28ad628603a3f6e5cfcc0fc609b50edf6e3879d6ce4d0b`, and Go/Node envelope
  `ca8c648203fc6092f4ac4585956260a0412c6332610a7589bcb03c61e3bfd63a`.

Limitations: no blanket package suite, repository gate, site gate, production authority promotion,
common-admission retry, provider-administration material, policy change, AEP mutation, commit,
publication or deployment was performed. The retained red proves the unchecked source-file digest;
the source trace establishes that the other claim entrypoint members follow the same missing loop.
The initial compile invocation's terminal output was not retained, as disclosed in section 2.

## 6. Paths written outside the worktree

- `home-path:sha256:62dbed7864b918cc83ae19309b701686da7340054e75c086b11ea4c1831330c0`
- `home-path:sha256:5fe3a4fe283fa35aae67e6ac6fd769c8f06a106b358e6db1f7c95c4f5badd69a`
- `home-path:sha256:6bdf1a6d601bb8d607f981db36085f23cceef39c302f429a21549844716b625e`
- `home-path:sha256:9429c8c7b65968e74f2c862546b10bf1739c72923597e39f747159af0b5aa160`
- `home-path:sha256:c418b633ba1cb0d2cf3baa7168330366eb7428d3798a8351ad275fdff04a6c4e` (pre-existing root-assigned compiler cache used and updated by the bounded case)

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/nested_execution.rs
  line: 721
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: nested candidate construction copies claim entrypoint source and digest bindings without checking them against the supplied source profile and consumer inventory, allowing a stale entrypoint to reach accounting/6 qualification
```
