---
format: aep.planning-md/1
id: review-result:service-contract-code-pass-1
kind: review-result
status: active
title: First independent source examination of the service contract
relations:
- reviews: story:reusable-service-contract
revision: 1
---
unit: reusable-service-contract source pass 1 at 68f68ed172b65a1c70bb03ca1175e402edb0f37b
verdict: NEEDS-CHANGE
cases: executed 5→6, red 1
origin: introduced 1 / pre-existing 0 / undecided 0
wrote-outside-worktree: 12 paths (10 files, 2 directories)
needs-coordinator: repair the introduced binding selection-conversion closure omission, then dispatch the final bounded source pass

1. Test-only diff

```text
 .../selection-conversion-fixtures/domain.yaml      |  32 +++++++
 .../selection-conversion-fixtures/system.yaml      |   5 +
 .../selection-conversion-fixtures/wiring.yaml      |  33 +++++++
 .../tests/selection_conversion.rs                  | 105 +++++++++++++++++++++
 4 files changed, 175 insertions(+)
```

`git diff --check` exited 0. HEAD remained detached at exact submission
`68f68ed172b65a1c70bb03ca1175e402edb0f37b`; the base is
`f1af8280338b97d862a6c474ec50f78d5157d71c`. Every changed path is a new Rust
integration test or its dedicated fixture. No submitted source, original assertion, original
fixture, manifest, lockfile, design, model, schema, or planning file changed.

2. Added case

`crates/specify/ess-service-contract/tests/selection_conversion.rs:59` adds
`selected_binding_retains_its_selection_input_preparation_conversion`. Its dedicated three-file
fixture places a declared `String -> List<selection.core.Leg>` conversion in the exact compiler IR.
The selected component accepts `selection.core.Receive`; binding `choose` invokes that accepted
operation and prepares its binding-local `legs` selection input through the declared conversion.
The case independently establishes all of these facts before comparing the selected capability
with the exact `PlannedCapability` from `SynthesisPlan::of(&ir)`.

The case is red now. The final exact targeted run was:

```text
Script started on 2026-09-15 21:11:43+02:00 [COMMAND="env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' TMPDIR=/var/tmp/ess-evolution-service-contract-review-1-20260915 cargo +1.98.1 test -p ess-service-contract --test selection_conversion selected_binding_retains_its_selection_input_preparation_conversion --locked --offline -j 2 -- --exact" <not executed on terminal>]
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running tests/selection_conversion.rs (target/debug/deps/selection_conversion-d3c22695078093dc)

running 1 test
test selected_binding_retains_its_selection_input_preparation_conversion ... FAILED

failures:

---- selected_binding_retains_its_selection_input_preparation_conversion stdout ----

thread 'selected_binding_retains_its_selection_input_preparation_conversion' (3332786) panicked at crates/specify/ess-service-contract/tests/selection_conversion.rs:98:5:
assertion `left == right` failed: a conversion used by an included binding must retain its original plan disposition
  left: None
 right: Some(PlannedCapability { capability: Capability { kind: Conversion, source: "String -> List<selection.core.Leg>" }, disposition: Obligation(ImplementationObligation { reason: UnspecifiedAlgorithm, contract: "a function from `String` to `List<selection.core.Leg>` — the crossing is permitted (the host decodes the ordered legs), the computation is not declared" }) })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    selected_binding_retains_its_selection_input_preparation_conversion

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-service-contract --test selection_conversion`

Script done on 2026-09-15 21:11:43+02:00 [COMMAND_EXIT_CODE="101"]
```

Raw receipt:
`home-path:sha256:be082b26a9a0a3c65a9c1154d6bea61c99d5b693cc31ea5bf04215677c62a483`.

3. Full crate suite and scoped formatting

The first default full-suite command stopped after the new failing integration binary, so it did
not establish the after-count. The required final run used Cargo's `--no-fail-fast` only to execute
every selected test binary. Its command, output, and exit were:

```text
Script started on 2026-09-15 21:12:35+02:00 [COMMAND="env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_JOBS=2 CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 RUSTFLAGS='-C link-arg=-fuse-ld=lld' TMPDIR=/var/tmp/ess-evolution-service-contract-review-1-20260915 cargo +1.98.1 test -p ess-service-contract --locked --offline -j 2 --no-fail-fast" <not executed on terminal>]
    Finished `test` profile [unoptimized] target(s) in 0.06s
     Running unittests src/lib.rs (target/debug/deps/ess_service_contract-d5c29f269d257460)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/selection_conversion.rs (target/debug/deps/selection_conversion-d3c22695078093dc)

running 1 test
test selected_binding_retains_its_selection_input_preparation_conversion ... FAILED

failures:

---- selected_binding_retains_its_selection_input_preparation_conversion stdout ----

thread 'selected_binding_retains_its_selection_input_preparation_conversion' (3339308) panicked at crates/specify/ess-service-contract/tests/selection_conversion.rs:98:5:
assertion `left == right` failed: a conversion used by an included binding must retain its original plan disposition
  left: None
 right: Some(PlannedCapability { capability: Capability { kind: Conversion, source: "String -> List<selection.core.Leg>" }, disposition: Obligation(ImplementationObligation { reason: UnspecifiedAlgorithm, contract: "a function from `String` to `List<selection.core.Leg>` — the crossing is permitted (the host decodes the ordered legs), the computation is not declared" }) })
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    selected_binding_retains_its_selection_input_preparation_conversion

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p ess-service-contract --test selection_conversion`
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

error: 1 target failed:
    `-p ess-service-contract --test selection_conversion`

Script done on 2026-09-15 21:12:36+02:00 [COMMAND_EXIT_CODE="101"]
```

The authored suite supplied the before-count of 5. The final full run executed all 6 integration
cases: the submitted 5 passed and the new case failed. Unit and doc lanes each executed 0. Raw
receipt:
`home-path:sha256:759ba2c9c1b8816ab15404ff6d04927205dbab20546e1f356b0f365fcbc81fbf`.

Scoped formatting was applied only to the new Rust integration test, then checked with:

```text
env -u CARGO_TARGET_DIR -u CARGO_ENCODED_RUSTFLAGS RUSTC_WRAPPER= RUSTC_WORKSPACE_WRAPPER= CARGO_BUILD_RUSTC_WRAPPER= CARGO_BUILD_RUSTC_WORKSPACE_WRAPPER= rustfmt +1.98.1 --edition 2021 --check crates/specify/ess-service-contract/tests/selection_conversion.rs
```

It exited 0. Raw receipt:
`home-path:sha256:052b960d5dad54b435fd9b82bdafa769ee11813a9c3afcb2254019aa51c33b7f`.

4. Finding

| File:line | Verdict | Origin | Measured and reachable evidence |
| --- | --- | --- | --- |
| `crates/specify/ess-service-contract/src/lib.rs:491` | `NEEDS-CHANGE` | `introduced` | `include_binding` adds the resolved selection's type handles but never adds conversions declared on `selection.plan.inputs`. The case at `tests/selection_conversion.rs:98` measured selected `None` against the exact planned conversion obligation and exited 101. The fixture reaches this path through selected component `selection-service`, its accepted `selection.core.Receive` operation, included invoking binding `choose`, and the binding's compiler-admitted preparation conversion. The base has no `ess-service-contract` crate, so the omission originates in this submission. |

The accepted design requires `Conversion` entries for declared conversions used by included
bindings and requires obligations/refusals to be filtered from those same selected plan entries.
Dropping this conversion also drops its `UnspecifiedAlgorithm` obligation from
`ServiceIr::obligations()`. The bounded repair is to include each admitted selection input's exact
declared preparation conversion in the existing conversion selection; no ER or runtime semantics
are implicated.

5. Attacks that did not break

- Exact full-plan admission and stable simultaneous input diagnostics remained supported by the
  implementation and submitted cases.
- Component-owned domains/entities, accepted operations, owned-domain views, and exact declared
  publications follow compiler handles rather than inferred SDK identity or domain membership.
- Complete borrowed command values preserve ordered outcomes, response fields, subject/effect,
  zero/many emissions, payload mappings, assignments, exact literals, errors, summaries, and
  references through the source graph.
- Published-but-not-emitted and emitted-but-unpublished events select binding capabilities by the
  accepted publication rule; an invoking binding independently enters through the accepted
  operation rule.
- Incoming `owns` sources, outgoing relation targets, contextual entity lifecycle entries, entity
  state/field types, deep named type bodies, and cycles use terminating closure sets without
  changing `owned_entities()`.
- Selected actor grants, mapping conversions, escalation events, capability order, original
  dispositions, filtered obligations/refusals, repeatability, canonical source bytes, and source
  digest were covered without another reachable break.
- The new crate has no production caller in the submitted workspace; its public callers are its two
  integration test binaries. No caller contract beyond the accepted design was inferred.
- No additional judgement finding remained after the executable selection-conversion case.

6. Outside-worktree writes

- `home-path:sha256:6e5b2c8d16b7bffa86663d6d50aca8bb27ea95f41c3ab3a3dfef38fd6dbd5e15`
- `home-path:sha256:e95a67b7529bd0dac87b09f2847bde1718dbbf67f517c974e8d9a167c8c2901e`
- `home-path:sha256:841264f199d80a44ebb76ec291546ff0d4ca31811ae6668eac6d469bd63918ec` — initial exact targeted red before fixture relocation; exit 101.
- `home-path:sha256:82bc79a08a9a7d811cf5a5c74606b52e76fb0744a683975ac1db0cfad54f0fe9` — default fail-fast full selection stopped at the new red binary; exit 101.
- `home-path:sha256:587e8b4d48345877a546957054118e8768c2bfefe064d804cc3fa2547bb82d2b` — exposed contamination from initially nesting the new fixture under the original recursive fixture root; exit 101, retained separately and not classified as a product finding.
- `home-path:sha256:2b39a61fee5f8689e6219a782d0096013bdc8204fe6d383121a209f726a50a38` — post-relocation redundant red; exit 101, superseded because one Cargo wrapper environment key was misspelled in that command.
- `home-path:sha256:be082b26a9a0a3c65a9c1154d6bea61c99d5b693cc31ea5bf04215677c62a483` — final exact targeted source red; exit 101.
- `home-path:sha256:759ba2c9c1b8816ab15404ff6d04927205dbab20546e1f356b0f365fcbc81fbf` — final exact no-fail-fast full crate run; exit 101 with submitted 5 green and new 1 red.
- `home-path:sha256:bfce9b4647a2ddebc09b539ec771c5fbe05b8e17ab667a2530b3a427f55568d7` — initial test-only formatting check; exit 1 with one line-wrap diff.
- `home-path:sha256:829eee4f906122f09b178c9575fbec64aa95669667c6180c83ed3d667a0f6bf9` — test-only formatting application; exit 0.
- `home-path:sha256:052b960d5dad54b435fd9b82bdafa769ee11813a9c3afcb2254019aa51c33b7f` — final test-only formatting check; exit 0.
- `/var/tmp/ess-evolution-service-contract-review-1-20260915/` — assigned scratch directory, empty.

No service, network, SQL/database, planning, source repair, cleanup, commit, publication, or
external integration command ran. Available disk was 14 GiB before and after the bounded runs.

7. Machine-readable findings

```findings
- file: crates/specify/ess-service-contract/src/lib.rs
  line: 491
  category: acceptance
  severity: blocker
  verdict: NEEDS-CHANGE
  origin: introduced
  message: included bindings drop a declared selection-input preparation Conversion capability and its original obligation because include_binding records only selection type handles
```
