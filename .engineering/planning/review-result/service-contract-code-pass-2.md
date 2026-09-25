---
format: aep.planning-md/2
id: review-result:service-contract-code-pass-2
kind: review-result
status: active
title: Final service-contract source examination
relations:
- reviews: story:reusable-service-contract
revision: 1
---
unit: reusable-service-contract final source pass 2 at be604d874ee9e567ae104e565e7cbddf953885a9
verdict: nothing found
cases: executed 7→7, red 0
origin: introduced 0 / pre-existing 0 / undecided 0
wrote-outside-worktree: 9 paths (7 files, 2 directories)
needs-coordinator: none

1. Test-only diff

```text
(no output)
```

`git --no-pager diff --stat` and `git diff --check` both exited 0. The tracked tree is clean,
detached at exact bot submission `be604d874ee9e567ae104e565e7cbddf953885a9`; the unit base is
`f1af8280338b97d862a6c474ec50f78d5157d71c`. No test or fixture was added because the complete
source examination found no additional reachable acceptance break. `test-only.diff` is therefore
an empty patch. No submitted source, test, fixture, manifest, lockfile, design, model, schema or
planning file changed.

The accepted design SHA-256 is
`14ce1677ae0f201d0751e335e69eee91edbe4f8145080d9e4e03987991932159`; the reviewed library SHA-256
is `681039418612ce4baaa359d4ca67416b54573afb7bde19420bd2951efc83e1bb`. Raw identity and scope
receipt: `final-scope.log`.

2. Cases added and targeted correction checks

No case was added. The submitted suite supplied the before-count of seven integration cases. The
prior finding's unchanged observer and the correction's unused-conversion control were run first,
individually, before the full suite.

Both commands ran from
`home-path:sha256:c86366d769d2ac9d37c4a4811b00134d47dd0b82bc5cb29717657220f4fd269b`
with Rust 1.98.1, `--locked --offline`, two jobs, debug and incremental output disabled, `lld`, all
four Rust wrappers empty, `CARGO_TARGET_DIR` and `CARGO_ENCODED_RUSTFLAGS` unset, the tree-local
`target/`, and `TMPDIR=/var/tmp/ess-evolution-service-contract-review-2-20260915`.

The exact prior-finding command was:

```text
env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' TMPDIR=/var/tmp/ess-evolution-service-contract-review-2-20260915 cargo +1.98.1 test -p ess-service-contract --test selection_conversion selected_binding_retains_its_selection_input_preparation_conversion --locked --offline -j 2 -- --exact
```

Its runner output ended:

```text
running 1 test
test selected_binding_retains_its_selection_input_preparation_conversion ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

It exited 0. The verbatim output, including initial dependency compilation, command and cwd, is in
`targeted-selection-conversion.log`.

The exact exclusion-control command was:

```text
env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' TMPDIR=/var/tmp/ess-evolution-service-contract-review-2-20260915 cargo +1.98.1 test -p ess-service-contract --test selection_conversion_control selected_preparation_conversion_excludes_an_unused_competing_conversion --locked --offline -j 2 -- --exact
```

Its complete runner result was:

```text
running 1 test
test selected_preparation_conversion_excludes_an_unused_competing_conversion ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

It exited 0. Verbatim receipt: `targeted-selection-conversion-control.log`.

3. Full crate suite and scoped formatting

The exact full-suite command used the same cwd and environment:

```text
env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' TMPDIR=/var/tmp/ess-evolution-service-contract-review-2-20260915 cargo +1.98.1 test -p ess-service-contract --locked --offline -j 2 --no-fail-fast
```

Its complete Cargo runner output was:

```text
   Compiling ess-service-contract v0.24.0 (home-path:sha256:3a39772933e9127d707a475f4785fef963ca45613c2d94e44bcc5bffd647d186)
    Finished `test` profile [unoptimized] target(s) in 0.33s
     Running unittests src/lib.rs (target/debug/deps/ess_service_contract-d5c29f269d257460)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/selection_conversion.rs (target/debug/deps/selection_conversion-d3c22695078093dc)

running 1 test
test selected_binding_retains_its_selection_input_preparation_conversion ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/selection_conversion_control.rs (target/debug/deps/selection_conversion_control-ab417f8db953e7e7)

running 1 test
test selected_preparation_conversion_excludes_an_unused_competing_conversion ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/service_contract.rs (target/debug/deps/service_contract-a7202836c380549e)

running 5 tests
test selected_obligations_and_refusals_retain_their_complete_plan_values ... ok
test contextual_closure_and_publication_select_exact_capabilities_in_plan_order ... ok
test billing_invoice_service_exposes_its_declared_surface ... ok
test billing_and_gatepass_keep_complete_compiler_values_and_source_bytes ... ok
test input_diagnostics_accumulate_in_stable_order_and_admit_only_the_exact_plan ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

   Doc-tests ess_service_contract

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

The command exited 0: seven integration cases passed, with zero unit and zero doc cases. Verbatim
receipt: `full-crate-suite.log`.

Scoped formatting covered all Rust integration tests added by this unit and correction:

```text
env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= TMPDIR=/var/tmp/ess-evolution-service-contract-review-2-20260915 rustfmt +1.98.1 --edition 2021 --check crates/specify/ess-service-contract/tests/service_contract.rs crates/specify/ess-service-contract/tests/selection_conversion.rs crates/specify/ess-service-contract/tests/selection_conversion_control.rs
```

It emitted no formatter diagnostics and exited 0. Verbatim receipt: `scoped-test-format.log`.

4. Judgement findings

None. This report covers exact submission `be604d874ee9e567ae104e565e7cbddf953885a9`.

5. What was attacked and did not break

- The prior blocker is closed at `crates/specify/ess-service-contract/src/lib.rs:491`: each included
  binding selection input with a conversion is matched against compiler-resolved conversions by
  complete typed source, complete typed target and the admitted reason; the matched conversion's
  exact plan entry, disposition, obligation and plan order survive extraction.
- The same path excludes an admitted but unused competing conversion. ConversionRegistry refuses
  duplicate source/target crossings, so the correction cannot ambiguously select two distinct
  decisions with one typed crossing.
- Primitive, declared, optional, list and map selection-input shapes are compared structurally;
  named leaves resolve through the compiler-minted selection handle table rather than display text.
- Exact full-plan admission, stable simultaneous diagnostics, complete borrowed selected surfaces,
  contextual entity/type/event/error closure, binding inclusion rules, and plan-order filtering
  remain consistent with the accepted design and existing public tests.
- The complete unit diff from base, accepted design, all submitted tests/fixtures, prior review,
  correction handoff, compiler selection input representation, synthesis conversion planner, and
  public callers were examined. The only public callers in this submission are the three crate
  integration binaries; no runtime, ER or SDK contract was inferred.

6. Outside-worktree writes

- `home-path:sha256:f67fe5a7f560ad403009f818d766f5f7a62d5947716353ff1bf2ca561e696ce2`
- `home-path:sha256:93799a2f2604536b668a053001682a00681b0adce78cddb4cf430c205fc6034d`
- `home-path:sha256:acdda0b8a344557c863a9fd775fee5699c06e61070e08f2da72877e9b2ac883a` — empty, because the tracked test-only diff is empty.
- `home-path:sha256:2e00ce7f24774f17fd9440ebc09c3aeba5fa10f3f6f8dfb00296d0475653202f` — prior-finding targeted case, exit 0.
- `home-path:sha256:c6916b40c43c470e6f82a7787c4e6ec4fc639a74fba529164e252ebf0f7aa890` — unused-conversion control, exit 0.
- `home-path:sha256:b8e66a35ca7ebd9a778736c6f17f39c58638ea5b7996d402a85b0877e49e8f32` — full Rust 1.98.1 crate suite, exit 0.
- `home-path:sha256:3af7010f162548ab36a1726a86b1dbb788ce85403cd0de36952b6dfb9d2b1983` — scoped Rust integration test format check, exit 0.
- `home-path:sha256:f23de682627075e46922b01e889b95ef0406ab4ed0e7725641364e27d42de138` — frozen identity, hashes, clean tracked status and final free-space receipt, exit 0.
- `/var/tmp/ess-evolution-service-contract-review-2-20260915/` — assigned scratch directory, empty.

The tree-local ignored `target/` contains this review's reproducible build output. Available space
was 45 GiB before and after the bounded checks, above the 8 GiB stop threshold. No network,
service, planning, source repair, cleanup, commit, publication or external integration command ran.

7. Machine-readable findings

```findings
[]
```
