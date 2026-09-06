---
format: aep.planning-md/1
id: review-result:composition-contract-adversary-pass-2
kind: review-result
status: active
title: Composition contract final source attack
relations:
- reviews: story:review-composition-contract
revision: 1
---
unit: story:review-composition-contract correction at 75df9e6d41202dd007c78c0dd116bda371451b04
verdict: nothing found
cases: executed 0→0, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: none
needs-coordinator: record final source attack, run changed-input integration checks and deliver
git --no-pager diff --stat
```text
```

The working-tree diff is empty. This is the second and final source attack, covering only the one-line addition at website/docs/reference/cli.md:73 after d65b6281339c2ba257d726c16a24dbb77d9def77. It adds mkdir -p target/composition-example before the existing compose invocation. No source, documentation, test, fixture, store or Git state was changed by this pass.

Read the full pass-2 brief, installed aep-drive 0.8.0 adversary charter, repository AGENTS, story acceptance, correction brief/report and immutable first attack report. Read the complete one-line diff and actual CLI preflight functions. Both bot identities and frozen HEAD were observed. Verified the supplied report/document hashes: correction report aa7d340e5d6bd0f47d7338d2620ac164e698965660662d534fd65b3cca5b00c6; corrected cli.md 9a8cff80f8d7dc611d7b01c65987015888f0f944d5ab5d24bf0574be30911452; unchanged first report 73f2c626cbb62cbc262d7ff4c0b675935f7a1f1401411b8b8c8168601ab160d1.

1. Probes declared before execution

probes.md was written and hashed in probes-before.sha256 before either CLI execution. It declares fresh-output success and an existing incompatible-destination refusal, plus exact-byte comparisons and input preservation. There are no new permanent tests. These are two actual CLI invocations and filesystem/byte checks, not Rust runtime test cases; the header therefore records 0→0.

Used the already-gated binary read-only at /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/debug/ess, SHA256 58736100f486d77e86925ff837a4930b9ce35d09d4ab79e7bc069a5d1696787c, owning source d565eb0206e0139da9e3ab6ad89aacac3c373927. Before executing it, this comparison exited 0 with empty output in decisive-source-comparison.log/.exit:

```text
git --no-optional-locks diff --exit-code d565eb0206e0139da9e3ab6ad89aacac3c373927 HEAD -- Cargo.lock crates/edge/ess-cli/src crates/specify/ess-composition/src crates/specify/ess-compiler crates/specify/ess-domain crates/specify/ess-primitives
```

Working directory was /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract. TMPDIR was /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-2/tmp. No CLI/package build or Cargo invocation was necessary. The binary and decisive source/input hashes remained unchanged after execution.

First probe: test ! -e target/review-boundaries-9/adversary-pass-2/fresh-output exited 0. Then the documented mkdir and compose argument shape executed with only the exact binary location and output base substituted. mkdir exited 0; compose exited 0. Full expanded commands and raw output from fresh-command.log follow; individual statuses are fresh-absent.exit, fresh-mkdir.exit and fresh-compose.exit:

```text
+ fixture=crates/specify/ess-composition/tests/fixtures
+ mkdir -p target/review-boundaries-9/adversary-pass-2/fresh-output
+ mkdir_exit=0
+ set +x
+ /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/debug/ess specify compose --path crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml --service todo=crates/specify/ess-composition/tests/fixtures/two-components --service usage=crates/specify/ess-composition/tests/fixtures/two-components --out target/review-boundaries-9/adversary-pass-2/fresh-output/composition.json --client-plan-out target/review-boundaries-9/adversary-pass-2/fresh-output/client-plan.json --client-rust-out target/review-boundaries-9/adversary-pass-2/fresh-output/rust-client
devcenter — 2 exact component surface(s), 2 semantic reference(s), compiled to target/review-boundaries-9/adversary-pass-2/fresh-output/composition.json; client plan written to target/review-boundaries-9/adversary-pass-2/fresh-output/client-plan.json; 3 Rust client artifact(s) written to target/review-boundaries-9/adversary-pass-2/fresh-output/rust-client
+ compose_exit=0
+ set +x
```

All three generated client artifacts were compared with the retained corpus, and the companion client-plan.json was compared with the retained client plan. Each cmp exited 0; corpus-comparison.exit is 0. Exact paths and bytes are fixed by the corpus/output manifests:

```text
Cargo.toml: exit 0
ess-client-plan.json: exit 0
src/lib.rs: exit 0
companion client-plan.json: exit 0
```

Second probe: test ! -e conflict-output succeeded in assigned scratch. Created that base and its composition.json directory, containing a marker whose hash was saved before execution. Then executed the same compose argument shape against that destination. The expected actual compose exit was 1, retained in conflict-compose.exit. Full expanded command and raw output:

```text
+ fixture=crates/specify/ess-composition/tests/fixtures
+ /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/debug/ess specify compose --path crates/specify/ess-composition/tests/fixtures/compositions/workbench.yaml --service todo=crates/specify/ess-composition/tests/fixtures/two-components --service usage=crates/specify/ess-composition/tests/fixtures/two-components --out target/review-boundaries-9/adversary-pass-2/conflict-output/composition.json --client-plan-out target/review-boundaries-9/adversary-pass-2/conflict-output/client-plan.json --client-rust-out target/review-boundaries-9/adversary-pass-2/conflict-output/rust-client
error: output path has an incompatible file type or symlink: /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-2/conflict-output/composition.json
+ compose_exit=1
+ set +x
```

Checks after refusal (0 denotes each successful assertion):

```text
expected compose exit 1: 0
client-plan.json absent: 0
rust-client absent: 0
target/review-boundaries-9/adversary-pass-2/conflict-output/composition.json/marker: OK
```

This is the reachable existing CLI preflight route: compose preflights all named/generated destinations before write_preflighted_files; preflight_named_output at crates/edge/ess-cli/src/main.rs:2159 calls inspect_output_entry, which refuses a directory where a file is selected. Neither companion output was created, and the original directory marker was unchanged. The refusal is an expected control, not a frozen-source defect or a rollback guarantee for later I/O failures.

2. Suites and inherited results

No package, child, workspace, site-build, formatter or Clippy suite ran in this turn. The supplied baseline of 12 package cases and 3 distinct generated-client child cases remains inherited. Root's 1948 workspace cases and all 10 gate lanes, including site-build, at d565eb0206e0139da9e3ab6ad89aacac3c373927 are also inherited, not reruns against this corrected document. The brief explicitly forbids an unchanged-suite rerun solely to repeat that result. Root owns changed-input integration checks after this pass.

Root's earlier example without mkdir exited 1 because the fresh output parent did not exist. The correction report retains that original log/result; they were read here and not overwritten or relabeled as this pass's execution. This final pass found no remaining issue in the corrected setup. There were no unexpected setup or command failures.

3. Findings

No finding against frozen 75df9e6d41202dd007c78c0dd116bda371451b04. No judgement findings or origin assignments.

4. What this pass did not break

- The corrected copyable mkdir/compose sequence succeeded from an absent output base and emitted the retained client corpus unchanged.
- A conflicting directory at the named composition output still refused before creating the companion plan/client tree, preserving the marker.

5. Preservation, manifests and outside writes

source-input-preservation.log/.exit records 17 successful checks: six source/document/test/lock owners, the exact read-only binary, the predeclared probes, and nine original fixtures/golden files. git diff --check exited 0. Final HEAD remains the frozen subject, and final-status.txt and test-only-diff-stat.txt are empty. The prior private-descriptor and byte-mutation cases, all original assertions and all generated golden bytes remain unchanged.

source-before.sha256, rechecked after execution:

```text
9a8cff80f8d7dc611d7b01c65987015888f0f944d5ab5d24bf0574be30911452  website/docs/reference/cli.md
cee8bed7df2bcf936a482ad8c4c79c7045b803ece402590ac73c9308e25860f3  website/docs/reference/formats.md
f468192fd7aa10ca4b84a9615ef52e086ca5a649b0c355011209c3d1c8937cc6  crates/edge/ess-cli/src/main.rs
db71d32fd00681c3047b1efd41759898ee4e3923ba3373b0580430a81c309afb  crates/specify/ess-composition/src/lib.rs
a2448d93c3a03a374cf29cc83cbb4b290796471beb86bc8ae6e36544e75f7e5b  crates/specify/ess-composition/tests/composition.rs
48292c59c73e812efabbd03dee3fb4bf2edcbc145cecc1c0c90fd724ea27969e  Cargo.lock
```

outputs.sha256:

```text
d3c6f57e34c12ef5db76854b829a386ca38743c0200040901618073961ac55d9  target/review-boundaries-9/adversary-pass-2/fresh-output/composition.json
ea9b489fd2675780903118625c6a7ad1c1d71d92f9120863e8c0ba92922c0a2b  target/review-boundaries-9/adversary-pass-2/fresh-output/client-plan.json
5934f005d385a43e896c9104ca5098bb383c80be202b138f95b6e4838d3eac0a  target/review-boundaries-9/adversary-pass-2/fresh-output/rust-client/Cargo.toml
ea9b489fd2675780903118625c6a7ad1c1d71d92f9120863e8c0ba92922c0a2b  target/review-boundaries-9/adversary-pass-2/fresh-output/rust-client/ess-client-plan.json
52041795e7601764300c900244edca77b08a73062a1c8b20f97a640916f9e312  target/review-boundaries-9/adversary-pass-2/fresh-output/rust-client/src/lib.rs
```

evidence.sha256 hashes the declared probes, all raw command/check logs and exits, final Git observations, and the source, binary, output and marker manifests. The source fixture manifest used is target/review-boundaries-9/correction-pass-1/original-inputs.sha256. All other named evidence files are under /home/timo/.local/state/worktree/trees/b10x/ess/ess-composition-contract/target/review-boundaries-9/adversary-pass-2.

Outside-worktree writes: none. All generated outputs, marker, TMPDIR and reports are inside assigned scratch. The coordinator binary and correction/first-pass records were only read. No production/doc/test/store/Git changes, cleanup, AEP helper action, browser or network operation occurred. No third attack is planned or requested.

This report is immutable after return. I relinquish all source, build and scratch writes. The coordinator owns records, integration decisions and public delivery; this report makes no approval, verifier-independence or publication claim.

```findings
[]
```

