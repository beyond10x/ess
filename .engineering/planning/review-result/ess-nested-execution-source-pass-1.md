---
format: aep.planning-md/1
id: review-result:ess-nested-execution-source-pass-1
kind: review-result
status: active
title: Nested execution whole-source examination pass 1
relations:
- reviews: task:consumer-accounting-nested-execution
revision: 1
---
unit: M9 whole measured nested execution working source on base f1af8280338b97d862a6c474ec50f78d5157d71c, complete manifest e99a625f1414c7030e6321aac89baec39296af0b04d1a18d204e428462802641
verdict: NEEDS-CHANGE
cases: executed 216 author-baseline unit cases→1 bounded reviewer selection, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 3 path roots
needs-coordinator: route the introduced outer-native verification defect to the implementor; preserve the separate unresolved 8-finding common-admission refusal

1. `git --no-pager diff --stat`

```text
 .../consumer_coverage/nested_execution_tests.rs    | 23 ++++++++++++++++++++++
 1 file changed, 23 insertions(+)
```

The index remains the frozen 278-path candidate. The only unstaged change is the reviewer test above.

2. Case added before execution

`crates/edge/ess-xtask/src/consumer_coverage/nested_execution_tests.rs:547` adds
`nested_execution_rejects_a_coordinated_outer_native_substitution`. It starts with a valid candidate,
plan, journal, and execution, then substitutes the outer native digest and copy path consistently in
the receipt, journal invocation, every observation, and journal digest. It asserts that final
execution verification refuses the substituted native. The case is red now because verification
returns success.

The first and only execution of the case was:

```console
env -u CARGO_TARGET_DIR CARGO_BUILD_JOBS=1 CARGO_NET_OFFLINE=true TMPDIR='~/beyond10x/.ess-evolution/waves/0010-opus-accounting/nested-execution/source-review-1/scratch/tmp' RUST_TEST_THREADS=1 RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' cargo +1.98.1 test --target-dir '~/beyond10x/.ess-evolution/waves/0010-opus-accounting/nested-execution/target' --locked --offline -p ess-xtask -j 1 'consumer_coverage::nested_execution_tests::nested_execution_rejects_a_coordinated_outer_native_substitution' -- --exact
```

Verbatim result:

```text
   Compiling ess-primitives v0.24.0 (~/.local/state/worktree/trees/b10x/ess/ess-evolution-nested-review-1-20260916/crates/specify/ess-primitives)
   Compiling ess-domain v0.24.0 (~/.local/state/worktree/trees/b10x/ess/ess-evolution-nested-review-1-20260916/crates/specify/ess-domain)
   Compiling ess-compiler v0.24.0 (~/.local/state/worktree/trees/b10x/ess/ess-evolution-nested-review-1-20260916/crates/specify/ess-compiler)
   Compiling ess-xtask v0.24.0 (~/.local/state/worktree/trees/b10x/ess/ess-evolution-nested-review-1-20260916/crates/edge/ess-xtask)
   Compiling ess-gen v0.24.0 (~/.local/state/worktree/trees/b10x/ess/ess-evolution-nested-review-1-20260916/crates/generate/ess-gen)
    Finished `test` profile [unoptimized] target(s) in 36.63s
     Running unittests src/main.rs (~/beyond10x/.ess-evolution/waves/0010-opus-accounting/nested-execution/target/debug/deps/ess_xtask-933e37d58b3a5a7c)

running 1 test
test consumer_coverage::nested_execution_tests::nested_execution_rejects_a_coordinated_outer_native_substitution ... FAILED

failures:

---- consumer_coverage::nested_execution_tests::nested_execution_rejects_a_coordinated_outer_native_substitution stdout ----

thread 'consumer_coverage::nested_execution_tests::nested_execution_rejects_a_coordinated_outer_native_substitution' (3468648) panicked at crates/edge/ess-xtask/src/consumer_coverage/nested_execution_tests.rs:563:5:
a receipt must not choose its own outer native identity
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    consumer_coverage::nested_execution_tests::nested_execution_rejects_a_coordinated_outer_native_substitution

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 219 filtered out; finished in 1.11s

error: test failed, to rerun pass `-p ess-xtask --bin ess-xtask`
```

Exit status: `101`.

3. Suite run

No blanket suite was run. The assignment reserved the heavy lane for the SDK full gate and granted
exactly the bounded command above. The frozen author evidence reports 216 unit tests passed and 3
ignored in log 71; the bounded reviewer run registered 220 unit cases after this addition, selected
one, filtered 219, and failed the selected case. Preflight before compilation observed
22,345,158,656 bytes available disk and 49,904,857,088 bytes available RAM, above the required
20 GiB/16 GiB floors. Post-run disk remained 22,343,372,800 bytes available.

4. Findings

| Location | Verdict | Origin | Finding and reachability |
|---|---|---|---|
| `crates/edge/ess-xtask/src/consumer_coverage/nested_execution.rs:1214` | `NEEDS-CHANGE` | `introduced` | Final verification validates only the shape of `receipt.native`, then at lines 1218–1233 constructs the expected journal binding from that same receipt's self-selected SHA. It never opens or hashes `native.copy`. A coordinated replacement of native SHA/copy plus the journal's native bindings and journal hash is accepted, contradicting the design's “rechecks every receipt” rule and the required wrong-native causal refusal. The regression measures `verify_execution` returning success, so its refusal assertion fails at test line 563. The normal production caller is `consumer_coverage::check_at` at `mod.rs:805–816`; the diagnostic caller is `nested_behavior` at `mod.rs:552–560`. Both obtain a filesystem-backed receipt from `native::execute_nested` and then call this verifier, so replacement or removal of the retained outer image between production and verification is reachable and remains undetected. `model_behavior::valid_native` at `model_behavior.rs:655–684` is structural only. The verifier must independently bind and remeasure the gate-owned outer native rather than deriving the expected identity solely from the receipt being checked. |

The finding covers the frozen working source and the reviewer test only. It does not assert that the
separate common commit admission succeeded. That admission remains refused for the eight exact
findings in `candidate-freeze/common-findings.json` (three workstation-path findings and five
source-shape findings); this review neither retries nor waives them.

5. What was attacked and did not produce another finding

- Verified the exact 21-path unit manifest SHA-256
  `d15aa96e42926d3867cd0bec884fa0d9caf00f2de80aaf46541f15e88f18aa9d`, all 2,134 complete-source
  path hashes, complete manifest SHA-256
  `e99a625f1414c7030e6321aac89baec39296af0b04d1a18d204e428462802641`, and canonical source digest
  `2431ccd9f14a5f8d6d7a9f0d6667c43a7231bd3aac05aa53e17dcd7699218494`.
- Compared the unit paths against the 265-file inherited overlay and base
  `f1af8280338b97d862a6c474ec50f78d5157d71c`; inspected the complete nested authority/candidate/plan,
  journal/receipt verifier, native executor, harness, accounting/6 integration, CLI wiring, old-reader
  fixtures, S6/S8 plumbing, actual Go/Node fixtures, binding design, and their direct callers.
- Attacked empty/unknown/duplicate authority, source/profile/case/plan freshness, ordered
  step/ordinal completeness, exact argv normalization, installed/Cargo/generated executable
  identities, producer/consumer artifact continuity, result/scenario/count matching, exact outer
  listing/execution, accounting/6 legacy/direct/nested partitioning, and old accounting /1–/5 and
  direct-model /1 reader boundaries. No additional reachable defect was established.
- Checked the retained evidence claims and hashes in `report.md` SHA-256
  `71b888cab5f61cd7dbd3db58fb40f307fc0c48112f0ed91ab60346e495464def`, `checks.json` SHA-256
  `59f81be62c635a1ee8a1415ec3ff579e1382984bff18e3c541608ac6edbd3277`, and `evidence.sha256`
  SHA-256 `411e6d77a27cb6580628e3b2f96c50adc835703f96e23dc50e7414f0d02961a9`.
- Confirmed the canonical `reviewed-nested-execution.json` is the deliberate empty pending authority
  and is rejected before candidate construction; diagnostic executions qualify zero production
  cells. No diagnostic claim was copied into production authority.
- Preserved the unrelated Eventlog administration restriction and performed no provider work,
  AEP writes, policy changes, admission retry, commit, publication, deployment, or source repair.

6. Paths written outside the worktree

- `~/beyond10x/.ess-evolution/waves/0010-opus-accounting/nested-execution/source-review-1/report.md`
- `~/beyond10x/.ess-evolution/waves/0010-opus-accounting/nested-execution/source-review-1/scratch/tmp/`
- `~/beyond10x/.ess-evolution/waves/0010-opus-accounting/nested-execution/target/` (pre-existing, root-reassigned compiler cache used by the bounded test)

```findings
- file: crates/edge/ess-xtask/src/consumer_coverage/nested_execution.rs
  line: 1214
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: final execution verification accepts a coordinated outer-native substitution because it structurally validates receipt.native and derives the expected journal native digest from that same unmeasured receipt
```
