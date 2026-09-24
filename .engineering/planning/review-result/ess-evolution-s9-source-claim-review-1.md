---
format: aep.planning-md/2
id: review-result:ess-evolution-s9-source-claim-review-1
kind: review-result
status: active
title: Original S9 whole source and claim review pass 1
relations:
- reviews: task:consumer-accounting-main-cli
revision: 1
---
unit: original S9 whole source/claim review pass 1, manifest snapshot on f1af8280338b97d862a6c474ec50f78d5157d71c
verdict: NEEDS-CHANGE
cases: author package 43/43 main and 4/4 Go helper; reviewer ran three focused selections (two red, one green); current main binary contains 45 tests, full target not rerun
origin: introduced 2 / pre-existing 0 / undecided 0
wrote-outside-worktree: assigned target, assigned scratch (five retained files), this report
needs-coordinator: route the two ledger corrections within original S9; common-gate snapshot commit remains refused for the five existing generated-digest exceptions

1. Review delta. The assigned tree was already a dirty, manifest-bound snapshot on the base commit: `git --no-pager diff --stat` at handoff showed 35 inherited tracked paths, including production and generated files, and omitted the untracked S9 files. Those were not reviewer changes. Comparing the author's exact S9 test source with this review tree gives:

```text
 .../ess-cli/tests/evolution_main_cli_accounting.rs | 49 ++++++++++++++++++++++
 1 file changed, 49 insertions(+)
```

The review changed only that Rust integration test. The exact test-only patch is `/var/tmp/ess-evolution-s9-review1-20260918/tmp/review-test-only.patch` (SHA-256 `9d03204efc09a2dc170b1b5542a568d7fc5d3b4d54ddf26f5c6248ff59e9b764`). No production, fixture, ledger, planning, or policy file was changed. The source before review matched the 1908-path complete manifest, and the 155-path author manifest, both `sha256sum --status -c` exit 0. After the review edit, the other 1907 complete-source paths still match; the changed test SHA-256 is `08f66e18914357e82bef369b558279cc9e7906a876b0fa01089bf4ebcf4849c192`.

2. Dedicated cases, written before each first execution. Both cases are at `crates/edge/ess-cli/tests/evolution_main_cli_accounting.rs:836-883`.

`sole_periodic_profile_arms_are_not_claimed_as_go_execution_support` asserts that the eight exact sole-admitted enum arms cannot be marked `Supported` on the strength of fixed-profile Go execution plus an invalid-word refusal. Its first isolated run exited 101. The complete retained output is `/var/tmp/ess-evolution-s9-review1-20260918/tmp/sole-periodic-red.log` (SHA-256 `3173d0715a8df48b2a4c90669ee40c9e7906a876b0fa01089bf4ebcf4849c192`); the decisive output was:

```text
running 1 test
test sole_periodic_profile_arms_are_not_claimed_as_go_execution_support ...
thread 'sole_periodic_profile_arms_are_not_claimed_as_go_execution_support' panicked at crates/edge/ess-cli/tests/evolution_main_cli_accounting.rs:858:9:
assertion `left != right` failed: rust:ess_domain::binding::periodic::Anchor/variant/HostActivation: the only independent word change is refused before Go output
  left: String("Supported")
 right: "Supported"
FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 43 filtered out
```

`supported_go_rows_do_not_retain_missing_receiver_claims` asserts that a supported row does not simultaneously say its exact Go receiver is absent. Its first isolated run exited 101. Complete output: `/var/tmp/ess-evolution-s9-review1-20260918/tmp/stale-supported-red.log` (SHA-256 `39b9ce505b4ec7df499a4e5b8cd04007dd96d4a0c4470e44ca7c22bef2eb87d2`); decisive output:

```text
running 1 test
test supported_go_rows_do_not_retain_missing_receiver_claims ...
thread 'supported_go_rows_do_not_retain_missing_receiver_claims' panicked at crates/edge/ess-cli/tests/evolution_main_cli_accounting.rs:876:13:
"rust:ess_compiler::ir::ResolvedMappingValue/variant/HostContext/field/field" is Supported but still says its Go receiver is missing: rust:ess_compiler::ir::ResolvedMappingValue/variant/HostContext/field/field: current S9 suites name this coordinate, but only the billing view/component target is implemented and executed. A dedicated target for this exact semantic arm remains feasible within the original fixture scope; emitted bytes or unrelated billing execution cannot establish it.
FAILED
test result: FAILED. 0 passed; 1 failed; 0 ignored; 44 filtered out
```

3. Affected checks. Both red runs used `cargo +1.98.1 test -p ess-cli --locked --offline -j2 --test evolution_main_cli_accounting <exact-case> -- --exact --nocapture --test-threads=1`, `CARGO_TARGET_DIR=/var/tmp/ess-evolution-s9-acceptance-20260918/target`, `TMPDIR=/var/tmp/ess-evolution-s9-review1-20260918/tmp`, empty compiler wrappers, debug/incremental 0, native lld, and one test thread. After both cases existed, the original finite-ledger validator `every_s9_profile_has_an_independent_complete_finite_ledger` exited 0 (one passed, 44 filtered; `/var/tmp/ess-evolution-s9-review1-20260918/tmp/ledger-after-both.log`, SHA-256 `abd7d5c5be32c48808dbd7589a5f615c4aea681b8d9e7ce970e5846917937dab`). Scoped `rustfmt +1.98.1 --edition 2021 --check` exited 0. The author's final package log remains the prior whole-suite evidence; rerunning its 184-second main and 142-second Go helper targets would not resolve either ledger contradiction. The 128-file author evidence manifest (SHA-256 `d26b94172a40b4c0ab96722e6874b3567b032bf662a597efb35c3d149042c653`) verified exit 0.

4. Findings on the exact snapshot and test-only delta:

| Location | Verdict / origin | Measured claim and reachability |
| --- | --- | --- |
| `crates/edge/ess-cli/tests/fixtures/evolution_main_cli_accounting/non-authority-conformance-go-runner.json:40433` | NEEDS-CHANGE / introduced | Twenty-four Go rows covering eight sole-admitted periodic enum arms and their associated field/parent identities cite the same invalid-word refusal as independent variation and say `Supported`. The exemplar at :40383-40436 states its only alternate is `unadmitted_anchor`; `go_runner_refuses_each_nonadmitted_periodic_profile_word_before_output` at test :2662-2700 confirms exit 1 before a package exists. `profile_word!` in `crates/specify/ess-domain/src/binding/periodic.rs:89-142` gives each enum exactly one admitted arm. The fixed-profile Go run at test :2589-2660 reaches four periodic scenario outcomes (12 others skip) and establishes that fixed profile, but never independently varies those eight arm values. This is a structurally single-admitted-arm attribution limit, not a request for another Go target or a claim that the existing fixed-profile run did not execute. The original unit requires variation at the exact claimed occurrence; pre-output refusal is control-only. Reattribute the finite rows to their honest structural/parent limit unless a valid exact receiving contrast is demonstrated. |
| `crates/edge/ess-cli/tests/fixtures/evolution_main_cli_accounting/non-authority-conformance-go-runner.json:40430` | CONFIRMED / introduced | Forty-eight `Supported` rows retain nonempty `entrypoint_observation.missing_observation`; 35 still say only billing/component targets exist. The representative HostContext row's positive case is `go_runner_executes_workbench_periodic_and_host_variants_on_a_fixed_target` at test :2589-2660, which runs the generated Go package and checks four changed outcomes. Its retained negative text instead says that receiver remains to be built. Other rows say no Supported claim is made while their disposition is Supported. This is stale, contradictory source/claim metadata. It does not by itself show missing actual-Go evidence for those rows; reconcile each retained statement with its current cited case and disposition. |

5. Attacked without another finding: all three exact ledgers had their asserted 686/2416/686 identities and canonical inventory/profile pins checked by the passing finite-ledger validator; the browser route emits legacy `model.json` and `suite.json`, loads both in Firefox and steps the authored scenario; the release route's positive selection and stale/wrong-suite controls bind model provenance, selected digest, and report status at `release_evidence.rs:132-185`; actual generated Go packages and report counts are checked in the main and helper targets. The 33 outside-entrypoint candidates, five aggregate candidates per profile, 71-tuple policy, and generated-digest common exceptions remain non-authority and were not admitted here. No unsupported inference is made from the `outcome-idempotent` case name.

6. Outside-worktree custody. Reviewer writes: `/var/tmp/ess-evolution-s9-review1-20260918/tmp/sole-periodic-red.log`, `/var/tmp/ess-evolution-s9-review1-20260918/tmp/stale-supported-red.log`, `/var/tmp/ess-evolution-s9-review1-20260918/tmp/ledger-baseline.log` (earlier green filter, SHA-256 `055d45ceddc00f26e66619f149faab7cc03a69dc7f842cd14f39673974182a8a`), `/var/tmp/ess-evolution-s9-review1-20260918/tmp/ledger-after-both.log`, `/var/tmp/ess-evolution-s9-review1-20260918/tmp/review-test-only.patch`, and this report. The reviewer also used the exclusively assigned `/var/tmp/ess-evolution-s9-acceptance-20260918/target` and scratch directory `/var/tmp/ess-evolution-s9-review1-20260918/tmp`; neither was cleaned. No worktree finish, publication, planning write, or common-gate retry occurred. The base commit remains `f1af8280338b97d862a6c474ec50f78d5157d71c`; there is no committed S9 snapshot because the normal bot commit was refused by the five previously proposed generated-digest exceptions, as recorded in `coordinator-freeze-20260918/receipt.md`.

```findings
- file: crates/edge/ess-cli/tests/fixtures/evolution_main_cli_accounting/non-authority-conformance-go-runner.json
  line: 40433
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: Twenty-four Go Supported rows promote fixed periodic singleton arms using only a pre-output invalid-word refusal, which cannot establish independent exact-arm execution variation.
- file: crates/edge/ess-cli/tests/fixtures/evolution_main_cli_accounting/non-authority-conformance-go-runner.json
  line: 40430
  category: contract-drift
  severity: warning
  verdict: CONFIRMED
  origin: introduced
  message: Forty-eight Supported Go rows retain missing-receiver assertions that contradict their current dispositions and cited actual-Go cases.
```
