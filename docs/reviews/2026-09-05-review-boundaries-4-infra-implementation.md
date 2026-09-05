unit:                   story:review-infra-ir-invariants — Keep resolved infrastructure handles valid for the IR lifetime
verdict:                green
cases:                  executed 244→265, red 9
origin:                 n/a
wrote-outside-worktree: none
needs-coordinator:      no

## 1. Unit, acceptance and scope confirmation

Public consumers cannot invalidate a resolved infrastructure handle by mutating its owning IR collections after compilation.

Base: 28e97095d9e06c8b4585876a681a5eda5278c1ab. All changes remain uncommitted in the assigned managed worktree. No AEP command, planning mutation, Git state mutation, lifecycle action, outside scratch or external message was performed.

The full story and implementor charter were read. The owning story has no unlanded depends_on edge. The brief explicitly prohibited AEP invocation, so its complete checked-in body and relations were inspected directly rather than executing the charter's usual artifact graph command.

Confirmed every assigned surface against actual source: infra-compiler's public owner and six handle classes; existing compile/read owner construction and private strict reader mirrors; infra-analyze and infra-spec read sites; infra-project's mutable working owner, four Change variants and record/apply ordering; and the actual Kubernetes import model query in ess-cli. The CLI project caller is also required by the deliberately changed Result API. No loader, infra-domain, scanner, manifest/lockfile, example, public guide or shared file edit was needed.

The inferred design file was absent and was written before the API change. It binds a crate-private model, shared model() query, checked detached try_transform using the existing reader, and Result<Projection, ValidationErrors>. The coordinator inspected and confirmed this choice. Inferred API/tests were grounded as follows: the original public mutation routes really compiled; reuse of read_document passed all six lookup/admission tests and actual four-change projection round trips; explicit transaction ordering was independently mutation-tested.

Implementation:
- InfraIr.model is crate-private. model() only returns &InfraModel. Detached model fields remain public; no mutable owner accessor, unchecked public construction/replacement or Deserialize was introduced.
- try_transform clones detached data, preserves provenance, uses the existing canonical document/digest writer and read_document, and returns an admitted new owner. Source owner and obtained source handles remain usable on failure.
- Projector candidate admission precedes artifact recording, generated dispositions, slots and progress. project returns Result and aborts without a partial projection on admission failure. CLI propagates before writing projection files.
- All downstream model queries use model(). Existing persisted fields, format identifiers and algorithms are unchanged.

## 2. Actual change shape

`git --no-pager diff --stat`:

```text
 crates/edge/ess-cli/src/main.rs                  |   5 +-
 crates/infra/infra-analyze/src/diagnose.rs       |  44 +++----
 crates/infra/infra-analyze/src/graph.rs          |  48 +++----
 crates/infra/infra-analyze/src/invariants.rs     |   8 +-
 crates/infra/infra-analyze/src/properties.rs     |  27 ++--
 crates/infra/infra-compiler/src/ir.rs            | 140 +++++++++++++++++++-
 crates/infra/infra-compiler/tests/determinism.rs |   2 +-
 crates/infra/infra-compiler/tests/read.rs        | 158 +++++++++++++++++++++++
 crates/infra/infra-compiler/tests/resolution.rs  |  12 +-
 crates/infra/infra-project/src/project.rs        |  96 +++++++++++---
 crates/infra/infra-project/tests/determinism.rs  |  17 ++-
 crates/infra/infra-project/tests/projection.rs   |  28 ++--
 crates/infra/infra-project/tests/round_trip.rs   |  74 ++++++++++-
 crates/infra/infra-project/tests/secrets.rs      |   8 +-
 crates/infra/infra-spec/src/drift.rs             |  78 +++++------
 crates/infra/infra-spec/src/facts.rs             |   8 +-
 crates/infra/infra-spec/src/simulate.rs          |  16 +--
 17 files changed, 609 insertions(+), 160 deletions(-)
```

Tracked numstat:

```text
3	2	crates/edge/ess-cli/src/main.rs
22	22	crates/infra/infra-analyze/src/diagnose.rs
24	24	crates/infra/infra-analyze/src/graph.rs
4	4	crates/infra/infra-analyze/src/invariants.rs
14	13	crates/infra/infra-analyze/src/properties.rs
139	1	crates/infra/infra-compiler/src/ir.rs
1	1	crates/infra/infra-compiler/tests/determinism.rs
158	0	crates/infra/infra-compiler/tests/read.rs
6	6	crates/infra/infra-compiler/tests/resolution.rs
80	16	crates/infra/infra-project/src/project.rs
12	5	crates/infra/infra-project/tests/determinism.rs
19	9	crates/infra/infra-project/tests/projection.rs
71	3	crates/infra/infra-project/tests/round_trip.rs
5	3	crates/infra/infra-project/tests/secrets.rs
39	39	crates/infra/infra-spec/src/drift.rs
4	4	crates/infra/infra-spec/src/facts.rs
8	8	crates/infra/infra-spec/src/simulate.rs
```

Untracked assigned design is additional to Git's unstaged tracked diff:

```text
81 docs/design/review-infra-ir-invariants.md
```

Actual status:

```text
 M crates/edge/ess-cli/src/main.rs
 M crates/infra/infra-analyze/src/diagnose.rs
 M crates/infra/infra-analyze/src/graph.rs
 M crates/infra/infra-analyze/src/invariants.rs
 M crates/infra/infra-analyze/src/properties.rs
 M crates/infra/infra-compiler/src/ir.rs
 M crates/infra/infra-compiler/tests/determinism.rs
 M crates/infra/infra-compiler/tests/read.rs
 M crates/infra/infra-compiler/tests/resolution.rs
 M crates/infra/infra-project/src/project.rs
 M crates/infra/infra-project/tests/determinism.rs
 M crates/infra/infra-project/tests/projection.rs
 M crates/infra/infra-project/tests/round_trip.rs
 M crates/infra/infra-project/tests/secrets.rs
 M crates/infra/infra-spec/src/drift.rs
 M crates/infra/infra-spec/src/facts.rs
 M crates/infra/infra-spec/src/simulate.rs
?? docs/design/review-infra-ir-invariants.md
```

## 3. Red evidence

The meaningful original red is the native Rustdoc lane: 14 executed, 5 passed, 9 failed. Six map clears, owner replacement, nested mutation and raw owner construction all compiled successfully on the old public API. Its positive compile controls passed. The existing borrowed-document and deserialization boundaries already held.

The initial ad hoc rustc harness executed one failing harness case but stopped on ambiguous dependency rlibs before it tested privacy. That setup failure is preserved below and is not counted among the nine meaningful red probes. Native Cargo doctests replaced the harness and delegate exact dependency selection to Cargo/Rustdoc; their cases preserve the same mutations and positive controls, without a new dependency.

The two new-API compilation failures executed no runtime tests: they establish absent methods and the old mutable projector route, not behavioral regression counts.

### red-public-owner

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test ownership public_owner_rejects_collection_replacement_and_nested_mutation -- --exact --nocapture
```

Exit: 101. Raw combined output (`target/review-boundaries-4/red-public-owner.log`):

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling memchr v2.8.3
   Compiling syn v3.0.4
   Compiling serde_derive v1.0.229
   Compiling infra-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-domain)
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Finished `test` profile [unoptimized] target(s) in 7.85s
     Running tests/ownership.rs (target/debug/deps/ownership-f8b12e714c3b88d0)

running 1 test

thread 'public_owner_rejects_collection_replacement_and_nested_mutation' (947833) panicked at crates/infra/infra-compiler/tests/ownership.rs:18:5:
assertion `left == right` failed: ambiguous infra_compiler dependency: ["/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/target/debug/deps/libinfra_compiler-d57328d556ea533c.rlib", "/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/target/debug/deps/libinfra_compiler-f1a3518dfd146321.rlib"]
  left: 2
 right: 1
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace
test public_owner_rejects_collection_replacement_and_nested_mutation ... FAILED

failures:

failures:
    public_owner_rejects_collection_replacement_and_nested_mutation

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 1 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p infra-compiler --test ownership`
```

### red-owner-doctests

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --doc
```

Exit: 101. Raw combined output (`target/review-boundaries-4/red-owner-doctests.log`):

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Finished `test` profile [unoptimized] target(s) in 1.68s
   Doc-tests infra_compiler

running 14 tests
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 598) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 623) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 619) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 546) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 552) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 558) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 564) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 576) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 570) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 615) ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 584) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 537) ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 590) - compile fail ... FAILED
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 604) - compile fail ... FAILED

failures:

---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 546) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 552) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 558) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 564) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 576) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 570) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 584) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 590) stdout ----
Test compiled successfully, but it's marked `compile_fail`.
---- crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 604) stdout ----
Test compiled successfully, but it's marked `compile_fail`.

failures:
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 546)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 552)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 558)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 564)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 570)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 576)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 584)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 590)
    crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 604)

test result: FAILED. 5 passed; 9 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

error: doctest failed, to rerun pass `-p infra-compiler --doc`
```

### red-transform-api

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test read deleting_any_referenced_target_is_refused_without_changing_the_source_owner -- --exact
```

Exit: 101. Raw combined output (`target/review-boundaries-4/red-transform-api.log`):

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
error[E0599]: no method named `try_transform` found for struct `InfraIr` in the current scope
   --> crates/infra/infra-compiler/tests/read.rs:283:32
    |
283 |     let transformed = original.try_transform(|model| {
    |                       ---------^^^^^^^^^^^^^ method not found in `InfraIr`

error[E0599]: no method named `try_transform` found for struct `InfraIr` in the current scope
   --> crates/infra/infra-compiler/tests/read.rs:314:31
    |
314 |         let errors = original.try_transform(delete).expect_err("a live reference cannot lose its target");
    |                               ^^^^^^^^^^^^^ method not found in `InfraIr`

error[E0599]: no method named `try_transform` found for struct `InfraIr` in the current scope
   --> crates/infra/infra-compiler/tests/read.rs:332:32
    |
332 |     let transformed = original.try_transform(|_| {}).expect("a no-op preserves validity");
    |                                ^^^^^^^^^^^^^ method not found in `InfraIr`

For more information about this error, try `rustc --explain E0599`.
error: could not compile `infra-compiler` (test "read") due to 3 previous errors
```

### red-project-transaction-api

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --lib failed_candidate_admission_records_nothing_and_preserves_the_working_owner
```

Exit: 101. Raw combined output (`target/review-boundaries-4/red-project-transaction-api.log`):

```text
   Compiling syn v3.0.4
   Compiling hashbrown v0.17.1
   Compiling indexmap v2.14.1
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling thiserror v2.0.20
   Compiling serde v1.0.229
   Compiling infra-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-domain)
   Compiling schemars v0.8.22
   Compiling serde_yaml v0.9.34+deprecated
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-primitives)
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
   Compiling infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-analyze)
   Compiling infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-spec)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
error[E0599]: no method named `record_admitted` found for struct `project::Workbench` in the current scope
    --> crates/infra/infra-project/src/project.rs:1582:15
     |
 597 | struct Workbench {
     | ---------------- method `record_admitted` not found for this struct
...
1582 |         bench.record_admitted(&change, candidate).expect_err("a rejected candidate cannot record its proposed change");
     |               ^^^^^^^^^^^^^^^ method not found in `project::Workbench`

error[E0599]: no method named `record_admitted` found for struct `project::Workbench` in the current scope
    --> crates/infra/infra-project/src/project.rs:1591:15
     |
 597 | struct Workbench {
     | ---------------- method `record_admitted` not found for this struct
...
1591 |         bench.record_admitted(&change, candidate).expect("the same intended change can be admitted and recorded");
     |               ^^^^^^^^^^^^^^^ method not found in `project::Workbench`

error[E0596]: cannot borrow data in a `&` reference as mutable
   --> crates/infra/infra-project/src/project.rs:658:27
    |
658 |                     apply(&mut self.working.model(), &change);
    |                           ^^^^^^^^^^^^^^^^^^^^^^^^^ cannot borrow as mutable

Some errors have detailed explanations: E0596, E0599.
For more information about an error, try `rustc --explain E0596`.
error: could not compile `infra-project` (lib test) due to 3 previous errors
```

### Deliberate verification mutations

After implementation, two independently targeted mutations were applied, executed and restored before final checks:
1. Ignore read_document's result and return Ok(candidate). The referenced-target deletion test executes one case and fails because a live node reference loses its target.
2. Move candidate? below record(). The transaction test executes one case and fails because a rejected candidate leaves a draft artifact behind.

These are two additional executed failing mutation cases, separate from the nine original privacy reds. The transaction case supplies a real reader-refused detached candidate (deleting referenced nodes), not fabricated diagnostic strings. Positive control then admits and records the same intended replica change.

### mutation-admission

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test read deleting_any_referenced_target_is_refused_without_changing_the_source_owner -- --exact
```

Exit: 101. Raw combined output (`target/review-boundaries-4/mutation-admission.log`):

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Finished `test` profile [unoptimized] target(s) in 1.96s
     Running tests/read.rs (target/debug/deps/read-a2a89aa47a12a0b4)

running 1 test
test deleting_any_referenced_target_is_refused_without_changing_the_source_owner ... FAILED

failures:

---- deleting_any_referenced_target_is_refused_without_changing_the_source_owner stdout ----

thread 'deleting_any_referenced_target_is_refused_without_changing_the_source_owner' (983047) panicked at crates/infra/infra-compiler/tests/read.rs:314:53:
a live reference cannot lose its target: InfraIr { provenance: Provenance { context: "read-back", scanned_at: "2026-08-21T08:00:00Z", scout_version: "0.1.0" }, model: InfraModel { namespaces: {"app": Namespace { identity: Identity { namespace: None, name: "app", uid: "ns-1" }, labels: {} }}, nodes: {}, workloads: {"app/deployment/web": ResolvedWorkload { kind: Deployment, identity: Identity { namespace: Some("app"), name: "web", uid: "d-1" }, labels: {}, replicas: Some(2), selector: {"app": "web"}, governing_service: None, service_account: Resolved { key: ServiceAccountHandle("app/runner") }, template_labels: {"app": "web"}, containers: [ResolvedContainer { name: "main", image: "web:1", env: [ResolvedEnvVar { name: "MODE", source: ConfigMapKey { config_map: Resolved { key: ConfigMapHandle("app/settings") }, key: "mode", optional: false } }, ResolvedEnvVar { name: "TOKEN", source: SecretKey { secret: Resolved { key: SecretHandle("app/creds") }, key: "token", optional: false } }], env_from: [], volume_mounts: [], probes: Probes { liveness: None, readiness: None, startup: None }, resources: Resources { requests: {}, limits: {} } }], volumes: [ResolvedVolume { name: "state", source: Claim { claim: Resolved { key: ClaimHandle("app/data") } } }] }, "app/statefulset/db": ResolvedWorkload { kind: StatefulSet, identity: Identity { namespace: Some("app"), name: "db", uid: "s-1" }, labels: {}, replicas: None, selector: {"app": "db"}, governing_service: Some(Resolved { key: ServiceHandle("app/db-headless") }), service_account: Resolved { key: ServiceAccountHandle("app/default") }, template_labels: {"app": "db"}, containers: [ResolvedContainer { name: "db", image: "db:2", env: [], env_from: [], volume_mounts: [], probes: Probes { liveness: None, readiness: None, startup: None }, resources: Resources { requests: {}, limits: {} } }], volumes: [] }}, services: {"app/db-headless": Service { identity: Identity { namespace: Some("app"), name: "db-headless", uid: "sv-2" }, labels: {}, service_type: "ClusterIP", selector: {"app": "db"}, ports: [ServicePort { name: None, port: 5432, target_port: "5432", protocol: "TCP" }] }, "app/web": Service { identity: Identity { namespace: Some("app"), name: "web", uid: "sv-1" }, labels: {}, service_type: "ClusterIP", selector: {"app": "web"}, ports: [ServicePort { name: None, port: 80, target_port: "80", protocol: "TCP" }] }}, ingresses: {"app/edge": ResolvedIngress { identity: Identity { namespace: Some("app"), name: "edge", uid: "i-1" }, labels: {}, rules: [ResolvedIngressRule { host: Some("web.test"), paths: [ResolvedIngressPath { path: Some("/"), path_type: Some("Prefix"), backend: ResolvedIngressBackend { service: Resolved { key: ServiceHandle("app/web") }, port: Some("80") } }] }], default_backend: None }}, config_maps: {"app/settings": ConfigMap { identity: Identity { namespace: Some("app"), name: "settings", uid: "c-1" }, labels: {}, keys: {"mode": ValueDigest { sha256: "115dc3606fbf8691fb69f2aefec86f2ecd302362a0502b3a9648bf2c4dc8290f", length: 4 }} }}, secrets: {"app/creds": Secret { identity: Identity { namespace: Some("app"), name: "creds", uid: "se-1" }, labels: {}, secret_type: "Opaque", keys: {"token": ValueDigest { sha256: "8a94462377096e0657f57b6e6bc0e29000464398727091d7863726ce50974968", length: 12 }} }}, service_accounts: {"app/default": ServiceAccount { identity: Identity { namespace: Some("app"), name: "default", uid: "sa-2" }, labels: {} }, "app/runner": ServiceAccount { identity: Identity { namespace: Some("app"), name: "runner", uid: "sa-1" }, labels: {} }}, claims: {"app/data": PersistentVolumeClaim { identity: Identity { namespace: Some("app"), name: "data", uid: "pv-1" }, labels: {}, storage_class: None, access_modes: ["ReadWriteOnce"], requested_storage: None, phase: Bound }}, pods: {"app/web-1": ResolvedPod { identity: Identity { namespace: Some("app"), name: "web-1", uid: "p-1" }, labels: {"app": "web"}, phase: Running, ready: true, node: Some(Resolved { key: NodeHandle("node-a") }), owner: None, containers: [ContainerStatus { name: "main", ready: true, restart_count: 0, waiting_reason: None }] }}, replica_sets: None, jobs: None, cron_jobs: None, pod_disruption_budgets: None, horizontal_pod_autoscalers: None, unresolved: [UnresolvedReference { from: "services/app/db-headless", site: "selector", target: PodsMatchingSelector { selector: {"app": "db"} } }] } }
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    deleting_any_referenced_target_is_refused_without_changing_the_source_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 9 filtered out; finished in 0.00s

error: test failed, to rerun pass `-p infra-compiler --test read`
```

### mutation-record-order

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --lib failed_candidate_admission_records_nothing_and_preserves_the_working_owner
```

Exit: 101. Raw combined output (`target/review-boundaries-4/mutation-record-order.log`):

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
   Compiling infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-analyze)
   Compiling infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-spec)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 0.80s
     Running unittests src/lib.rs (target/debug/deps/infra_project-41d61ffb90a92fc1)

running 1 test
test project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner ... FAILED

failures:

---- project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner stdout ----

thread 'project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner' (984215) panicked at crates/infra/infra-project/src/project.rs:1600:9:
assertion failed: bench.drafts.is_empty()
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace


failures:
    project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner

test result: FAILED. 0 passed; 1 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.01s

error: test failed, to rerun pass `-p infra-project --lib`
```

## 4. Baseline, final gates and byte evidence

All package counts below come from the runner's own summary lines for the same five-package command. The total is 244→265: infra-compiler 19→38 (three runtime cases and sixteen doctests), infra-project 40→42, infra-analyze 69→69, infra-spec 64→64, ess-cli 52→52. No cases were added to the latter three packages, whose unchanged lane counts are expected for query/API migration. No existing assertion was weakened, ignored or removed; existing projection callers now assert successful admission before retaining their original assertions.

```text
ess-cli/unit: executed 11 → 11, exit 0
ess-cli/command_surface: executed 5 → 5, exit 0
ess-cli/command_surface_adversary: executed 4 → 4, exit 0
ess-cli/go_conformance: executed 7 → 7, exit 0
ess-cli/output_containment: executed 19 → 19, exit 0
ess-cli/persisted_delivery: executed 6 → 6, exit 0
infra-analyze/unit: executed 19 → 19, exit 0
infra-analyze/analysis: executed 12 → 12, exit 0
infra-analyze/determinism: executed 5 → 5, exit 0
infra-analyze/diagnosis: executed 23 → 23, exit 0
infra-analyze/graph: executed 10 → 10, exit 0
infra-compiler/unit: executed 0 → 0, exit 0
infra-compiler/determinism: executed 7 → 7, exit 0
infra-compiler/read: executed 7 → 10, exit 0
infra-compiler/resolution: executed 5 → 5, exit 0
infra-project/unit: executed 8 → 9, exit 0
infra-project/determinism: executed 6 → 6, exit 0
infra-project/projection: executed 18 → 18, exit 0
infra-project/round_trip: executed 6 → 7, exit 0
infra-project/secrets: executed 2 → 2, exit 0
infra-spec/unit: executed 14 → 14, exit 0
infra-spec/determinism: executed 7 → 7, exit 0
infra-spec/drift: executed 8 → 8, exit 0
infra-spec/simulate: executed 14 → 14, exit 0
infra-spec/spec: executed 21 → 21, exit 0
infra-analyze/doc: executed 0 → 0, exit 0
infra-compiler/doc: executed 0 → 16, exit 0
infra-project/doc: executed 0 → 0, exit 0
infra-spec/doc: executed 0 → 0, exit 0
```

The isolated reader lane executed 10 cases; transaction and four-change round-trip selectors each executed 1, with nonzero selection confirmed by the raw summaries below. Full baseline/final outputs include every suite, including unchanged and zero-documentation lanes.

### baseline

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler -p infra-analyze -p infra-spec -p infra-project -p ess-cli
```

Exit: 0. Raw combined output (`target/review-boundaries-4/baseline.log`):

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling itoa v1.0.18
   Compiling serde_json v1.0.151
   Compiling cfg-if v1.0.4
   Compiling typenum v1.20.1
   Compiling const-oid v0.10.2
   Compiling cpufeatures v0.3.1
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling equivalent v1.0.2
   Compiling thiserror v2.0.20
   Compiling schemars v0.8.22
   Compiling dyn-clone v1.0.20
   Compiling ryu v1.0.23
   Compiling unsafe-libyaml v0.2.11
   Compiling autocfg v1.5.1
   Compiling libc v0.2.189
   Compiling version_check v0.9.5
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.56
   Compiling heck v0.5.0
   Compiling ref-cast v1.0.27
   Compiling parking_lot_core v0.9.12
   Compiling regex-syntax v0.8.11
   Compiling pulldown-cmark v0.13.4
   Compiling bitflags v2.13.1
   Compiling unicase v2.9.0
   Compiling scopeguard v1.2.0
   Compiling utf8parse v0.2.2
   Compiling smallvec v1.16.0
   Compiling pulldown-cmark-escape v0.11.0
   Compiling ahash v0.8.12
   Compiling num-traits v0.2.19
   Compiling once_cell v1.21.4
   Compiling lock_api v0.4.14
   Compiling is_terminal_polyfill v1.70.2
   Compiling anstyle-parse v1.0.0
   Compiling unicode-general-category v1.1.0
   Compiling borrow-or-share v0.2.4
   Compiling anstyle v1.0.14
   Compiling hashbrown v0.17.1
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling bit-vec v0.8.0
   Compiling num-cmp v0.1.0
   Compiling bytecount v0.6.9
   Compiling strsim v0.11.1
   Compiling micromap v0.3.0
   Compiling clap_lex v1.1.0
   Compiling vsimd v0.8.0
   Compiling percent-encoding v2.3.2
   Compiling anstream v1.0.0
   Compiling aho-corasick v1.1.5
   Compiling bit-set v0.8.0
   Compiling outref v0.5.2
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling clap_builder v4.6.6
   Compiling uuid-simd v0.8.0
   Compiling hybrid-array v0.4.14
   Compiling indexmap v2.14.1
   Compiling syn v3.0.4
   Compiling syn v2.0.119
   Compiling num-integer v0.1.47
   Compiling num-complex v0.4.6
   Compiling num-bigint v0.4.8
   Compiling num-iter v0.1.46
   Compiling crypto-common v0.2.2
   Compiling block-buffer v0.12.1
   Compiling parking_lot v0.12.5
   Compiling regex-automata v0.4.18
   Compiling jsonschema-regex v0.52.1
   Compiling semver v1.0.28
   Compiling digest v0.11.3
   Compiling sha2 v0.11.0
   Compiling num-rational v0.4.2
   Compiling serde_derive_internals v0.29.1
   Compiling strum_macros v0.28.0
   Compiling num v0.4.3
   Compiling fraction v0.17.0
   Compiling schemars_derive v0.8.22
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling ref-cast-impl v1.0.27
   Compiling clap_derive v4.6.4
   Compiling strum v0.28.0
   Compiling clap v4.6.6
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling infra-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-domain)
   Compiling serde_yaml v0.9.34+deprecated
   Compiling fluent-uri v0.4.1
   Compiling email_address v0.2.9
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/ess-kubernetes)
   Compiling jsonschema-value v0.52.1
   Compiling referencing v0.52.1
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-primitives)
   Compiling ess-openapi v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-openapi)
   Compiling jsonschema v0.52.1
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
   Compiling ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-domain)
   Compiling infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-analyze)
   Compiling infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-spec)
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-compiler)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/schema-contract)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-gen)
   Compiling ess-realization v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-realization)
   Compiling ess-composition v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-composition)
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-deployment)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/verify/ess-diff)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 19.34s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.26s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.71s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.44s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running unittests src/lib.rs (target/debug/deps/infra_analyze-0ffed922cf4133dc)

running 19 tests
test code::tests::severity_orders_info_below_warning_below_error ... ok
test code::tests::every_code_renders_in_the_diag_namespace_and_the_generated_list_holds_them_all ... ok
test code::tests::wire_strings_are_unique_because_two_rules_sharing_one_code_are_indistinguishable_downstream ... ok
test directions::tests::findings_sharing_a_root_evidence_value_collapse_into_one_direction ... ok
test graph::tests::a_graph_node_reads_its_namespace_off_the_key_and_a_cluster_node_has_none ... ok
test directions::tests::a_clean_candidate_produces_no_direction_and_an_excepted_one_states_its_counts ... ok
test directions::tests::directions_rank_errors_above_warnings_above_info ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test code::tests::the_required_and_optional_reference_codes_disagree_in_severity_by_design ... ok
test graph::tests::a_replicaset_name_derives_its_deployment_only_when_the_hash_confirms_it ... ok
test html::tests::the_severity_classes_cover_all_three_severities_and_none ... ok
test html::tests::html_escaping_defuses_every_metacharacter_it_claims_to ... ok
test invariants::tests::every_code_renders_in_the_prop_namespace_and_wire_strings_are_unique ... ok
test properties::tests::a_bare_image_name_has_neither_registry_nor_tag_nor_digest ... ok
test invariants::tests::a_minority_is_not_a_majority_and_a_bare_half_is_not_either ... ok
test properties::tests::a_digest_pinned_image_reports_the_digest_and_whatever_tag_rides_along ... ok
test properties::tests::a_tagged_image_with_a_registry_port_keeps_both_apart ... ok
test properties::tests::an_image_with_a_registry_port_and_no_tag_is_untagged_not_tagged_5000 ... ok
test properties::tests::a_namespaced_hub_image_has_no_registry_because_team_is_not_a_host ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/analysis.rs (target/debug/deps/analysis-f81ae0c31523e57a)

running 12 tests
test properties_on_an_old_format_bundle_carry_coverage_as_unscanned_not_as_uncovered ... ok
test a_cluster_without_majority_uniformity_yields_no_candidate ... ok
test all_three_candidates_are_mined_from_the_committed_observation_in_code_order ... ok
test a_candidate_with_exceptions_reads_as_uniformity_with_exceptions_not_as_violations ... ok
test properties_name_the_budgets_and_autoscalers_covering_each_workload ... ok
test the_directions_text_states_candidate_exceptions_without_prescribing ... ok
test the_html_page_sections_by_namespace_aggregates_pods_and_badges_by_worst_finding ... ok
test properties_carry_declared_and_observed_replicas_per_workload ... ok
test the_registry_candidate_names_the_dominant_registry_and_lists_every_exception ... ok
test directions_rank_errors_first_and_lead_with_the_autoscaler_aimed_at_nothing ... ok
test the_html_page_writes_out_as_one_self_contained_file ... ok
test the_namespace_filter_scopes_sections_findings_and_directions_alike ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/determinism.rs (target/debug/deps/determinism-22db1f859d880eee)

running 5 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_analysis_uses_no_unordered_map_and_reads_no_clock ... ok
test two_diagnoses_of_one_ir_serialize_byte_identically ... ok
test candidates_directions_and_the_html_page_render_byte_identically_across_two_runs ... ok
test two_graph_constructions_render_byte_identical_documents_and_diagrams ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/diagnosis.rs (target/debug/deps/diagnosis-e32ea7ff2f9732ab)

running 23 tests
test the_new_rules_stay_silent_on_a_bundle_that_did_not_scan_their_kinds ... ok
test a_container_without_probes_fires_and_the_probed_coredns_container_does_not ... ok
test a_multi_replica_workload_without_a_budget_fires_and_a_covered_one_does_not ... ok
test a_container_without_bounds_fires_and_the_bounded_coredns_container_does_not ... ok
test a_pending_claim_fires_and_a_bound_one_does_not ... ok
test an_autoscaler_aimed_at_nothing_is_an_error_and_an_aimed_one_is_not ... ok
test a_suspended_cronjob_is_info_and_a_running_one_is_not ... ok
test a_job_short_of_its_completions_with_failures_fires_and_a_completed_one_does_not ... ok
test a_pod_its_workload_expects_ready_fires_and_a_finished_job_pod_does_not ... ok
test an_autoscaler_pinned_to_one_size_fires_and_a_real_range_does_not ... ok
test a_budget_guarding_nothing_fires_and_the_one_guarding_switchboard_does_not ... ok
test a_required_missing_reference_is_an_error_and_an_optional_one_is_info ... ok
test every_registered_code_fires_at_least_once_on_the_example_observation ... ok
test an_unreferenced_claim_fires_and_the_mounted_one_does_not ... ok
test latest_and_untagged_images_fire_and_a_pinned_tag_does_not ... ok
test findings_arrive_sorted_and_each_carries_its_codes_registered_severity ... ok
test a_crashlooping_container_is_an_error_and_a_creating_one_is_not ... ok
test one_replica_is_info_and_two_replicas_or_a_daemonset_are_not ... ok
test repeated_restarts_fire_and_a_stable_container_does_not ... ok
test the_severity_floor_filters_out_exactly_what_is_below_it ... ok
test a_selector_matching_nothing_is_diagnosed_and_a_matching_one_is_not ... ok
test two_services_selecting_one_workload_set_are_reported_once_together ... ok
test unreferenced_config_fires_and_referenced_or_token_managed_config_does_not ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/graph.rs (target/debug/deps/graph-b921749062e0b541)

running 10 tests
test a_replicaset_whose_deployment_is_gone_and_a_hashless_pod_both_stay_underived ... ok
test a_pod_whose_scanned_replicaset_is_absent_or_deploymentless_is_handled_exactly ... ok
test on_a_bundle_without_replicasets_the_hash_fallback_derives_and_names_itself ... ok
test a_job_pod_chains_to_its_job_and_cronjob_and_a_bare_pod_stays_a_typed_fact ... ok
test every_edge_relation_is_minted_from_the_committed_observation ... ok
test the_selector_edge_carries_the_selector_and_the_env_edge_carries_its_site ... ok
test a_deployment_pod_is_owned_exactly_through_its_observed_replicaset ... ok
test the_mermaid_rendering_groups_by_namespace_and_leaves_the_runtime_layer_to_the_json ... ok
test restricting_to_a_namespace_keeps_its_objects_their_edges_and_the_nodes_they_reach ... ok
test the_json_document_chains_to_the_ir_it_was_built_from ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/infra_compiler-f0c886bf2a1891d1)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-9ec910889020b138)

running 7 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_digest_is_the_full_sha256_all_64_hex_characters ... ok
test editing_scanned_at_changes_provenance_and_not_the_digest ... ok
test a_semantic_change_does_change_the_digest ... ok
test the_compiler_uses_no_unordered_map_and_reads_no_clock ... ok
test compiling_the_same_observation_twice_yields_byte_identical_documents ... ok
test a_bundle_with_reordered_kinds_and_reordered_item_lists_compiles_to_the_identical_ir ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/read.rs (target/debug/deps/read-473bee8fa98a265a)

running 7 tests
test a_foreign_format_is_refused_before_anything_else_is_believed ... ok
test the_fixture_mints_every_handle_kind_or_the_round_trip_proves_too_little ... ok
test a_document_that_does_not_read_as_the_shape_is_refused_as_malformed ... ok
test an_edited_document_is_refused_for_its_digest ... ok
test an_edited_document_with_a_dangling_claim_reports_both_defects_in_one_run ... ok
test a_hand_written_resolved_claim_is_refused_even_when_its_digest_is_freshly_stamped ... ok
test a_persisted_document_reads_back_into_the_identical_ir ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/resolution.rs (target/debug/deps/resolution-1a08a56169ecfe84)

running 5 tests
test a_dangling_reference_is_carried_as_a_fact_and_never_refuses_compilation ... ok
test volume_claims_and_optional_secret_references_resolve_with_their_flags_kept ... ok
test an_absent_service_account_name_resolves_as_default_because_that_is_what_the_kubelet_does ... ok
test a_resolved_reference_is_a_handle_whose_lookup_is_total ... ok
test the_unresolved_site_keeps_the_declared_name_so_the_ir_reads_on_its_own ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/infra_project-ea4cefede791f1bd)

running 8 tests
test patch::tests::a_file_holding_a_container_change_is_strategic_however_it_was_reached ... ok
test patch::tests::a_slug_is_read_back_as_namespace_kind_and_name_even_when_the_name_holds_dots ... ok
test patch::tests::the_join_of_two_types_is_the_one_that_can_carry_both ... ok
test project::tests::a_manifest_port_keeps_the_type_it_was_written_as ... ok
test project::tests::a_generated_budget_names_no_uid_because_nothing_has_assigned_one ... ok
test render::tests::only_an_induced_gap_is_marked_so_a_reader_can_tell_it_from_the_clusters_own ... ok
test project::tests::the_nearest_bound_of_a_range_is_the_bound_the_count_is_outside_of ... ok
test patch::tests::the_container_list_carries_the_merge_key_it_is_matched_by ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-893dd4c6438712f7)

running 6 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_projection_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test every_file_in_the_committed_tree_is_one_the_library_still_produces ... ok
test the_committed_projection_tree_is_what_the_library_produces_right_now ... ok
test two_projections_of_one_specification_and_snapshot_are_byte_identical ... ok
test shuffling_a_bundles_items_changes_no_byte_of_the_tree ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/projection.rs (target/debug/deps/projection-d3013a26367d3b62)

running 18 tests
test a_stated_probe_is_written_into_the_container_that_lacks_it ... ok
test a_remedy_that_states_only_one_missing_half_leaves_the_whole_gap_owed ... ok
test a_false_predicate_is_refused_because_a_condition_names_no_field ... ok
test a_budget_whose_name_is_taken_is_owed_rather_than_written_over ... ok
test a_replica_count_above_the_range_is_lowered_to_the_ceiling ... ok
test two_expectations_that_disagree_leave_one_of_them_refused_rather_than_silently_lost ... ok
test the_same_gap_without_a_stated_value_is_an_obligation_that_names_what_is_missing ... ok
test a_resource_gap_is_patched_only_because_the_specification_states_the_quantities ... ok
test the_obligations_document_names_every_gap_the_tree_does_not_close_and_no_others ... ok
test a_probe_gap_with_nothing_stated_is_owed_and_says_what_to_write_where ... ok
test a_replica_count_below_the_range_is_raised_to_the_floor_and_nothing_more ... ok
test the_tree_holds_a_summary_an_obligations_list_and_nothing_it_did_not_generate ... ok
test every_gap_kind_that_needs_a_decision_gets_one_with_the_class_that_names_it ... ok
test every_gap_the_snapshot_reports_gets_exactly_one_entry_and_no_gap_is_lost ... ok
test a_missing_disruption_budget_becomes_a_manifest_built_from_the_workloads_own_selector ... ok
test a_gap_this_projections_own_changes_open_is_marked_as_such_and_closed_in_the_same_tree ... ok
test one_object_gets_one_patch_file_and_its_type_is_the_one_that_carries_every_change_in_it ... ok
test every_obligation_names_a_decision_rather_than_repeating_the_gap ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/round_trip.rs (target/debug/deps/round_trip-9026362e64fa4a24)

running 6 tests
test apply::tests::a_plain_merge_patch_replaces_a_list_whole_which_is_why_a_container_change_is_not_one ... ok
test apply::tests::a_keyed_list_merges_the_entry_it_names_and_leaves_the_rest_alone ... ok
test apply::tests::a_null_deletes_the_key_it_names_as_rfc_7386_says ... ok
test a_container_patch_emitted_as_a_plain_merge_would_delete_the_containers_it_does_not_name ... ok
test a_corrupted_patch_value_is_caught_and_the_regressed_expectation_is_named ... ok
test applying_the_emitted_tree_closes_every_gap_it_claims_and_moves_nothing_else ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.05s

     Running tests/secrets.rs (target/debug/deps/secrets-da37d14b3f2d0428)

running 2 tests
test a_dangling_secret_reference_is_owed_and_the_obligation_says_why_nothing_can_write_it ... ok
test no_emitted_byte_carries_a_secrets_digest_or_key_name ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running unittests src/lib.rs (target/debug/deps/infra_spec-4bca2e012aa4ac5e)

running 14 tests
test drift::tests::the_change_ordering_puts_membership_before_the_fields_of_a_surviving_member ... ok
test drift::tests::membership_reports_each_side_once_and_nothing_for_a_shared_key ... ok
test facts::tests::every_documented_fact_path_parses_and_the_membership_check_agrees_with_the_list ... ok
test raw::tests::an_id_is_lowercase_dashed_and_starts_with_a_letter ... ok
test raw::tests::the_three_workload_kinds_parse_by_their_ir_spelling_and_nothing_else_does ... ok
test render::tests::a_verdict_marker_is_three_characters_so_the_columns_line_up ... ok
test simulate::tests::a_verdict_is_the_outcome_variant_and_not_a_field_beside_it ... ok
test simulate::tests::an_optional_dangling_reference_is_not_required_and_a_plain_one_is ... ok
test render::tests::the_missing_pair_names_only_what_is_missing ... ok
test simulate::tests::an_undecidable_subject_beside_only_holding_ones_reads_unknown_not_true ... ok
test spec::tests::a_workload_label_selector_cannot_select_a_service_or_the_cluster ... ok
test spec::tests::every_kind_declares_its_wire_name_in_the_generated_vocabulary ... ok
test simulate::tests::an_undecidable_subject_beside_a_gap_still_reads_false ... ok
test spec::tests::only_cluster_scope_selects_an_expectation_that_names_its_own_subject ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-4a1c75fbfcd914d2)

running 7 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test nothing_in_this_crate_can_read_a_wall_clock_because_no_expectation_names_a_duration ... ok
test the_desired_state_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test two_simulations_of_one_specification_and_snapshot_are_byte_identical ... ok
test two_drift_reports_of_one_pair_are_byte_identical ... ok
test the_committed_documents_are_what_the_library_produces_right_now ... ok
test shuffling_a_bundles_items_changes_no_byte_of_either_document ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/drift.rs (target/debug/deps/drift-bdf907f0243c4c76)

running 8 tests
test two_snapshots_of_different_clusters_are_refused_rather_than_compared ... ok
test comparing_a_snapshot_with_itself_reports_no_change_at_all ... ok
test a_reference_change_is_only_reported_for_a_holder_present_in_both_snapshots ... ok
test a_configuration_change_names_the_keys_and_never_a_value ... ok
test a_workloads_replica_count_image_and_labels_each_arrive_as_their_own_typed_change ... ok
test every_change_kind_the_pair_was_built_to_exercise_appears_exactly_where_it_should ... ok
test a_pods_churn_is_not_drift_because_drift_is_over_declared_state ... ok
test reordering_a_templates_containers_is_not_a_change_because_containers_compare_by_name ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/simulate.rs (target/debug/deps/simulate-038789899bcfb715)

running 14 tests
test a_scope_naming_an_observed_but_empty_namespace_is_undecidable_not_vacuously_satisfied ... ok
test a_service_with_no_selector_is_undecidable_rather_than_failing_a_resolution_it_never_claimed ... ok
test one_scope_holding_both_a_contradicted_and_an_undecidable_subject_reads_false ... ok
test a_bundle_that_never_scanned_disruption_budgets_is_undecidable_and_not_uncovered ... ok
test a_digest_pinned_image_satisfies_the_pin_expectation_and_a_tagged_one_does_not ... ok
test every_expectation_kind_holds_somewhere_on_the_fixture_and_fails_somewhere_on_it ... ok
test each_undecidable_expectation_on_the_fixture_carries_the_reason_its_name_promises ... ok
test a_report_names_every_subject_the_scope_selected_including_the_ones_that_held ... ok
test an_optional_dangling_reference_holds_and_a_required_one_does_not ... ok
test a_gap_beside_an_undecidable_subject_still_decides_the_expectation_false ... ok
test the_committed_example_reaches_all_three_verdicts_and_the_counts_are_the_documented_ones ... ok
test an_expectation_the_snapshot_cannot_decide_never_becomes_a_gap ... ok
test a_predicate_reads_the_projections_facts_and_a_false_one_carries_the_values_it_read ... ok
test workload_exists_holds_for_each_of_the_three_kinds_and_fails_when_the_kind_is_the_wrong_one ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/spec.rs (target/debug/deps/spec-d27b9540362abae1)

running 21 tests
test a_format_this_build_does_not_read_is_refused_with_its_own_code ... ok
test a_probes_remedy_for_a_probe_the_expectation_never_asks_for_is_refused ... ok
test a_document_that_does_not_deserialize_is_one_coded_refusal_and_not_a_serde_sentence ... ok
test a_probe_remedy_states_exactly_one_handler_and_neither_is_refused_the_same_way_as_both ... ok
test a_quoted_number_is_refused_as_a_port_name_because_it_is_one ... ok
test a_predicate_reading_a_fact_the_projection_never_states_is_refused_as_a_typo ... ok
test a_remedy_that_states_nothing_is_refused_because_it_leaves_the_gap_where_it_was ... ok
test a_scope_selects_exactly_the_subject_classes_its_shape_can_reach ... ok
test a_remedy_beside_a_kind_that_never_finds_an_empty_field_is_refused_rather_than_carried ... ok
test a_specification_reads_from_json_too_because_json_is_yaml ... ok
test a_remedy_that_validates_is_carried_on_the_expectation_and_a_document_without_one_carries_none ... ok
test a_specification_with_four_defects_reports_four_refusals_in_one_run ... ok
test a_specification_with_no_expectations_is_refused_rather_than_read_as_satisfied ... ok
test a_scope_that_cannot_select_the_expectations_subject_is_refused_in_both_directions ... ok
test an_id_that_is_not_an_identifier_is_refused_and_a_dashed_lowercase_one_is_not ... ok
test every_kind_whose_parameters_can_decide_nothing_is_refused_with_one_code ... ok
test two_expectations_sharing_an_id_are_refused_because_a_report_names_a_verdict_by_it ... ok
test the_validated_type_is_only_reachable_through_validation ... ok
test the_committed_example_specification_validates_and_declares_every_kind ... ok
test a_remedy_changes_no_verdict_because_nothing_evaluates_one ... ok
test the_committed_example_specification_simulates_identically_with_and_without_its_remedies ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests infra_analyze

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_compiler

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_project

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_spec

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### green-reader

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler --test read
```

Exit: 0. Raw combined output (`target/review-boundaries-4/green-reader.log`):

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Finished `test` profile [unoptimized] target(s) in 1.99s
     Running tests/read.rs (target/debug/deps/read-a2a89aa47a12a0b4)

running 10 tests
test a_foreign_format_is_refused_before_anything_else_is_believed ... ok
test the_fixture_mints_every_handle_kind_or_the_round_trip_proves_too_little ... ok
test a_document_that_does_not_read_as_the_shape_is_refused_as_malformed ... ok
test an_edited_document_is_refused_for_its_digest ... ok
test an_edited_document_with_a_dangling_claim_reports_both_defects_in_one_run ... ok
test a_hand_written_resolved_claim_is_refused_even_when_its_digest_is_freshly_stamped ... ok
test a_persisted_document_reads_back_into_the_identical_ir ... ok
test every_handle_lookup_stays_total_after_compile_read_clone_and_checked_transform ... ok
test deleting_any_referenced_target_is_refused_without_changing_the_source_owner ... ok
test privacy_and_noop_transform_preserve_the_frozen_base_writer_document ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

```

### green-project-transaction

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --lib failed_candidate_admission_records_nothing_and_preserves_the_working_owner
```

Exit: 0. Raw combined output (`target/review-boundaries-4/green-project-transaction.log`):

```text
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 0.45s
     Running unittests src/lib.rs (target/debug/deps/infra_project-41d61ffb90a92fc1)

running 1 test
test project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 8 filtered out; finished in 0.02s

```

### green-four-changes

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-project --test round_trip checked_projection_round_trips_all_four_changes_including_probes_and_induced_budget -- --exact
```

Exit: 0. Raw combined output (`target/review-boundaries-4/green-four-changes.log`):

```text
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Finished `test` profile [unoptimized] target(s) in 0.75s
     Running tests/round_trip.rs (target/debug/deps/round_trip-2ed84414750d9f23)

running 1 test
test checked_projection_round_trips_all_four_changes_including_probes_and_induced_budget ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 6 filtered out; finished in 0.01s

```

### final-tests

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo test --locked --offline -p infra-compiler -p infra-analyze -p infra-spec -p infra-project -p ess-cli
```

Exit: 0. Raw combined output (`target/review-boundaries-4/final-tests.log`):

```text
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
   Compiling infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-analyze)
   Compiling infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-spec)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 5.48s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
test tests::no_manifest_or_lockfile_depends_on_aep ... ok
test tests::every_command_and_argument_name_is_unambiguous ... ok

test result: ok. 11 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/command_surface.rs (target/debug/deps/command_surface-f896f6f697ed70aa)

running 5 tests
test the_help_offers_exactly_the_four_areas ... ok
test the_generate_area_help_offers_the_verbs_options_and_the_areas_subcommands ... ok
test a_clap_refusal_differs_only_in_its_usage_line ... ok
test the_generate_area_refuses_its_arguments_beside_a_sibling_verb ... ok
test a_flat_spelling_prints_what_its_area_path_prints ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

     Running tests/command_surface_adversary.rs (target/debug/deps/command_surface_adversary-14ea054bad6c3502)

running 4 tests
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.72s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.45s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 6 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test adversary_noncanonical_topological_order_is_refused_before_execution ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test adversary_duplicate_desired_keys_are_refused_before_any_executor ... ok
test adversary_duplicate_current_keys_block_removal_and_diff ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

     Running unittests src/lib.rs (target/debug/deps/infra_analyze-0ffed922cf4133dc)

running 19 tests
test code::tests::severity_orders_info_below_warning_below_error ... ok
test code::tests::every_code_renders_in_the_diag_namespace_and_the_generated_list_holds_them_all ... ok
test code::tests::the_required_and_optional_reference_codes_disagree_in_severity_by_design ... ok
test code::tests::wire_strings_are_unique_because_two_rules_sharing_one_code_are_indistinguishable_downstream ... ok
test directions::tests::a_clean_candidate_produces_no_direction_and_an_excepted_one_states_its_counts ... ok
test directions::tests::directions_rank_errors_above_warnings_above_info ... ok
test directions::tests::findings_sharing_a_root_evidence_value_collapse_into_one_direction ... ok
test graph::tests::a_graph_node_reads_its_namespace_off_the_key_and_a_cluster_node_has_none ... ok
test graph::tests::a_replicaset_name_derives_its_deployment_only_when_the_hash_confirms_it ... ok
test graph::tests::a_mermaid_label_cannot_close_the_quoted_string_it_sits_in ... ok
test html::tests::the_severity_classes_cover_all_three_severities_and_none ... ok
test html::tests::html_escaping_defuses_every_metacharacter_it_claims_to ... ok
test invariants::tests::every_code_renders_in_the_prop_namespace_and_wire_strings_are_unique ... ok
test invariants::tests::a_minority_is_not_a_majority_and_a_bare_half_is_not_either ... ok
test properties::tests::a_bare_image_name_has_neither_registry_nor_tag_nor_digest ... ok
test properties::tests::a_digest_pinned_image_reports_the_digest_and_whatever_tag_rides_along ... ok
test properties::tests::an_image_with_a_registry_port_and_no_tag_is_untagged_not_tagged_5000 ... ok
test properties::tests::a_namespaced_hub_image_has_no_registry_because_team_is_not_a_host ... ok
test properties::tests::a_tagged_image_with_a_registry_port_keeps_both_apart ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/analysis.rs (target/debug/deps/analysis-f81ae0c31523e57a)

running 12 tests
test a_cluster_without_majority_uniformity_yields_no_candidate ... ok
test properties_on_an_old_format_bundle_carry_coverage_as_unscanned_not_as_uncovered ... ok
test a_candidate_with_exceptions_reads_as_uniformity_with_exceptions_not_as_violations ... ok
test all_three_candidates_are_mined_from_the_committed_observation_in_code_order ... ok
test the_registry_candidate_names_the_dominant_registry_and_lists_every_exception ... ok
test properties_carry_declared_and_observed_replicas_per_workload ... ok
test directions_rank_errors_first_and_lead_with_the_autoscaler_aimed_at_nothing ... ok
test properties_name_the_budgets_and_autoscalers_covering_each_workload ... ok
test the_html_page_sections_by_namespace_aggregates_pods_and_badges_by_worst_finding ... ok
test the_directions_text_states_candidate_exceptions_without_prescribing ... ok
test the_html_page_writes_out_as_one_self_contained_file ... ok
test the_namespace_filter_scopes_sections_findings_and_directions_alike ... ok

test result: ok. 12 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/determinism.rs (target/debug/deps/determinism-22db1f859d880eee)

running 5 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_analysis_uses_no_unordered_map_and_reads_no_clock ... ok
test two_diagnoses_of_one_ir_serialize_byte_identically ... ok
test candidates_directions_and_the_html_page_render_byte_identically_across_two_runs ... ok
test two_graph_constructions_render_byte_identical_documents_and_diagrams ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/diagnosis.rs (target/debug/deps/diagnosis-e32ea7ff2f9732ab)

running 23 tests
test the_new_rules_stay_silent_on_a_bundle_that_did_not_scan_their_kinds ... ok
test a_container_without_probes_fires_and_the_probed_coredns_container_does_not ... ok
test a_suspended_cronjob_is_info_and_a_running_one_is_not ... ok
test a_budget_guarding_nothing_fires_and_the_one_guarding_switchboard_does_not ... ok
test a_multi_replica_workload_without_a_budget_fires_and_a_covered_one_does_not ... ok
test one_replica_is_info_and_two_replicas_or_a_daemonset_are_not ... ok
test a_job_short_of_its_completions_with_failures_fires_and_a_completed_one_does_not ... ok
test a_pod_its_workload_expects_ready_fires_and_a_finished_job_pod_does_not ... ok
test a_crashlooping_container_is_an_error_and_a_creating_one_is_not ... ok
test a_selector_matching_nothing_is_diagnosed_and_a_matching_one_is_not ... ok
test a_required_missing_reference_is_an_error_and_an_optional_one_is_info ... ok
test an_autoscaler_aimed_at_nothing_is_an_error_and_an_aimed_one_is_not ... ok
test an_unreferenced_claim_fires_and_the_mounted_one_does_not ... ok
test every_registered_code_fires_at_least_once_on_the_example_observation ... ok
test a_container_without_bounds_fires_and_the_bounded_coredns_container_does_not ... ok
test an_autoscaler_pinned_to_one_size_fires_and_a_real_range_does_not ... ok
test findings_arrive_sorted_and_each_carries_its_codes_registered_severity ... ok
test latest_and_untagged_images_fire_and_a_pinned_tag_does_not ... ok
test repeated_restarts_fire_and_a_stable_container_does_not ... ok
test the_severity_floor_filters_out_exactly_what_is_below_it ... ok
test two_services_selecting_one_workload_set_are_reported_once_together ... ok
test a_pending_claim_fires_and_a_bound_one_does_not ... ok
test unreferenced_config_fires_and_referenced_or_token_managed_config_does_not ... ok

test result: ok. 23 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/graph.rs (target/debug/deps/graph-b921749062e0b541)

running 10 tests
test a_replicaset_whose_deployment_is_gone_and_a_hashless_pod_both_stay_underived ... ok
test on_a_bundle_without_replicasets_the_hash_fallback_derives_and_names_itself ... ok
test a_pod_whose_scanned_replicaset_is_absent_or_deploymentless_is_handled_exactly ... ok
test a_deployment_pod_is_owned_exactly_through_its_observed_replicaset ... ok
test a_job_pod_chains_to_its_job_and_cronjob_and_a_bare_pod_stays_a_typed_fact ... ok
test every_edge_relation_is_minted_from_the_committed_observation ... ok
test the_mermaid_rendering_groups_by_namespace_and_leaves_the_runtime_layer_to_the_json ... ok
test restricting_to_a_namespace_keeps_its_objects_their_edges_and_the_nodes_they_reach ... ok
test the_selector_edge_carries_the_selector_and_the_env_edge_carries_its_site ... ok
test the_json_document_chains_to_the_ir_it_was_built_from ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running unittests src/lib.rs (target/debug/deps/infra_compiler-f0c886bf2a1891d1)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-9ec910889020b138)

running 7 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_digest_is_the_full_sha256_all_64_hex_characters ... ok
test editing_scanned_at_changes_provenance_and_not_the_digest ... ok
test a_semantic_change_does_change_the_digest ... ok
test the_compiler_uses_no_unordered_map_and_reads_no_clock ... ok
test compiling_the_same_observation_twice_yields_byte_identical_documents ... ok
test a_bundle_with_reordered_kinds_and_reordered_item_lists_compiles_to_the_identical_ir ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/read.rs (target/debug/deps/read-473bee8fa98a265a)

running 10 tests
test the_fixture_mints_every_handle_kind_or_the_round_trip_proves_too_little ... ok
test a_foreign_format_is_refused_before_anything_else_is_believed ... ok
test a_document_that_does_not_read_as_the_shape_is_refused_as_malformed ... ok
test an_edited_document_with_a_dangling_claim_reports_both_defects_in_one_run ... ok
test an_edited_document_is_refused_for_its_digest ... ok
test a_hand_written_resolved_claim_is_refused_even_when_its_digest_is_freshly_stamped ... ok
test a_persisted_document_reads_back_into_the_identical_ir ... ok
test every_handle_lookup_stays_total_after_compile_read_clone_and_checked_transform ... ok
test deleting_any_referenced_target_is_refused_without_changing_the_source_owner ... ok
test privacy_and_noop_transform_preserve_the_frozen_base_writer_document ... ok

test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/resolution.rs (target/debug/deps/resolution-1a08a56169ecfe84)

running 5 tests
test volume_claims_and_optional_secret_references_resolve_with_their_flags_kept ... ok
test an_absent_service_account_name_resolves_as_default_because_that_is_what_the_kubelet_does ... ok
test a_dangling_reference_is_carried_as_a_fact_and_never_refuses_compilation ... ok
test a_resolved_reference_is_a_handle_whose_lookup_is_total ... ok
test the_unresolved_site_keeps_the_declared_name_so_the_ir_reads_on_its_own ... ok

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running unittests src/lib.rs (target/debug/deps/infra_project-ea4cefede791f1bd)

running 9 tests
test patch::tests::a_file_holding_a_container_change_is_strategic_however_it_was_reached ... ok
test patch::tests::the_join_of_two_types_is_the_one_that_can_carry_both ... ok
test project::tests::the_nearest_bound_of_a_range_is_the_bound_the_count_is_outside_of ... ok
test project::tests::a_manifest_port_keeps_the_type_it_was_written_as ... ok
test patch::tests::a_slug_is_read_back_as_namespace_kind_and_name_even_when_the_name_holds_dots ... ok
test patch::tests::the_container_list_carries_the_merge_key_it_is_matched_by ... ok
test render::tests::only_an_induced_gap_is_marked_so_a_reader_can_tell_it_from_the_clusters_own ... ok
test project::tests::a_generated_budget_names_no_uid_because_nothing_has_assigned_one ... ok
test project::tests::failed_candidate_admission_records_nothing_and_preserves_the_working_owner ... ok

test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.02s

     Running tests/determinism.rs (target/debug/deps/determinism-893dd4c6438712f7)

running 6 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test the_projection_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test the_committed_projection_tree_is_what_the_library_produces_right_now ... ok
test every_file_in_the_committed_tree_is_one_the_library_still_produces ... ok
test two_projections_of_one_specification_and_snapshot_are_byte_identical ... ok
test shuffling_a_bundles_items_changes_no_byte_of_the_tree ... ok

test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.15s

     Running tests/projection.rs (target/debug/deps/projection-d3013a26367d3b62)

running 18 tests
test a_false_predicate_is_refused_because_a_condition_names_no_field ... ok
test a_budget_whose_name_is_taken_is_owed_rather_than_written_over ... ok
test a_remedy_that_states_only_one_missing_half_leaves_the_whole_gap_owed ... ok
test a_replica_count_above_the_range_is_lowered_to_the_ceiling ... ok
test a_stated_probe_is_written_into_the_container_that_lacks_it ... ok
test the_same_gap_without_a_stated_value_is_an_obligation_that_names_what_is_missing ... ok
test two_expectations_that_disagree_leave_one_of_them_refused_rather_than_silently_lost ... ok
test every_gap_kind_that_needs_a_decision_gets_one_with_the_class_that_names_it ... ok
test a_probe_gap_with_nothing_stated_is_owed_and_says_what_to_write_where ... ok
test a_missing_disruption_budget_becomes_a_manifest_built_from_the_workloads_own_selector ... ok
test a_resource_gap_is_patched_only_because_the_specification_states_the_quantities ... ok
test the_obligations_document_names_every_gap_the_tree_does_not_close_and_no_others ... ok
test the_tree_holds_a_summary_an_obligations_list_and_nothing_it_did_not_generate ... ok
test every_gap_the_snapshot_reports_gets_exactly_one_entry_and_no_gap_is_lost ... ok
test every_obligation_names_a_decision_rather_than_repeating_the_gap ... ok
test a_gap_this_projections_own_changes_open_is_marked_as_such_and_closed_in_the_same_tree ... ok
test a_replica_count_below_the_range_is_raised_to_the_floor_and_nothing_more ... ok
test one_object_gets_one_patch_file_and_its_type_is_the_one_that_carries_every_change_in_it ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.10s

     Running tests/round_trip.rs (target/debug/deps/round_trip-9026362e64fa4a24)

running 7 tests
test apply::tests::a_null_deletes_the_key_it_names_as_rfc_7386_says ... ok
test apply::tests::a_keyed_list_merges_the_entry_it_names_and_leaves_the_rest_alone ... ok
test apply::tests::a_plain_merge_patch_replaces_a_list_whole_which_is_why_a_container_change_is_not_one ... ok
test checked_projection_round_trips_all_four_changes_including_probes_and_induced_budget ... ok
test a_container_patch_emitted_as_a_plain_merge_would_delete_the_containers_it_does_not_name ... ok
test a_corrupted_patch_value_is_caught_and_the_regressed_expectation_is_named ... ok
test applying_the_emitted_tree_closes_every_gap_it_claims_and_moves_nothing_else ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.13s

     Running tests/secrets.rs (target/debug/deps/secrets-da37d14b3f2d0428)

running 2 tests
test a_dangling_secret_reference_is_owed_and_the_obligation_says_why_nothing_can_write_it ... ok
test no_emitted_byte_carries_a_secrets_digest_or_key_name ... ok

test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

     Running unittests src/lib.rs (target/debug/deps/infra_spec-4bca2e012aa4ac5e)

running 14 tests
test drift::tests::membership_reports_each_side_once_and_nothing_for_a_shared_key ... ok
test drift::tests::the_change_ordering_puts_membership_before_the_fields_of_a_surviving_member ... ok
test raw::tests::an_id_is_lowercase_dashed_and_starts_with_a_letter ... ok
test facts::tests::every_documented_fact_path_parses_and_the_membership_check_agrees_with_the_list ... ok
test render::tests::a_verdict_marker_is_three_characters_so_the_columns_line_up ... ok
test render::tests::the_missing_pair_names_only_what_is_missing ... ok
test simulate::tests::a_verdict_is_the_outcome_variant_and_not_a_field_beside_it ... ok
test raw::tests::the_three_workload_kinds_parse_by_their_ir_spelling_and_nothing_else_does ... ok
test simulate::tests::an_optional_dangling_reference_is_not_required_and_a_plain_one_is ... ok
test simulate::tests::an_undecidable_subject_beside_a_gap_still_reads_false ... ok
test spec::tests::a_workload_label_selector_cannot_select_a_service_or_the_cluster ... ok
test simulate::tests::an_undecidable_subject_beside_only_holding_ones_reads_unknown_not_true ... ok
test spec::tests::every_kind_declares_its_wire_name_in_the_generated_vocabulary ... ok
test spec::tests::only_cluster_scope_selects_an_expectation_that_names_its_own_subject ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/determinism.rs (target/debug/deps/determinism-4a1c75fbfcd914d2)

running 7 tests
test the_scan_sees_a_real_violation_and_ignores_prose_and_substrings ... ok
test nothing_in_this_crate_can_read_a_wall_clock_because_no_expectation_names_a_duration ... ok
test the_desired_state_crate_uses_no_unordered_map_and_reads_no_clock ... ok
test two_simulations_of_one_specification_and_snapshot_are_byte_identical ... ok
test two_drift_reports_of_one_pair_are_byte_identical ... ok
test the_committed_documents_are_what_the_library_produces_right_now ... ok
test shuffling_a_bundles_items_changes_no_byte_of_either_document ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s

     Running tests/drift.rs (target/debug/deps/drift-bdf907f0243c4c76)

running 8 tests
test two_snapshots_of_different_clusters_are_refused_rather_than_compared ... ok
test comparing_a_snapshot_with_itself_reports_no_change_at_all ... ok
test a_workloads_replica_count_image_and_labels_each_arrive_as_their_own_typed_change ... ok
test every_change_kind_the_pair_was_built_to_exercise_appears_exactly_where_it_should ... ok
test a_configuration_change_names_the_keys_and_never_a_value ... ok
test a_reference_change_is_only_reported_for_a_holder_present_in_both_snapshots ... ok
test reordering_a_templates_containers_is_not_a_change_because_containers_compare_by_name ... ok
test a_pods_churn_is_not_drift_because_drift_is_over_declared_state ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/simulate.rs (target/debug/deps/simulate-038789899bcfb715)

running 14 tests
test a_scope_naming_an_observed_but_empty_namespace_is_undecidable_not_vacuously_satisfied ... ok
test a_service_with_no_selector_is_undecidable_rather_than_failing_a_resolution_it_never_claimed ... ok
test a_bundle_that_never_scanned_disruption_budgets_is_undecidable_and_not_uncovered ... ok
test one_scope_holding_both_a_contradicted_and_an_undecidable_subject_reads_false ... ok
test a_digest_pinned_image_satisfies_the_pin_expectation_and_a_tagged_one_does_not ... ok
test a_predicate_reads_the_projections_facts_and_a_false_one_carries_the_values_it_read ... ok
test an_expectation_the_snapshot_cannot_decide_never_becomes_a_gap ... ok
test a_report_names_every_subject_the_scope_selected_including_the_ones_that_held ... ok
test each_undecidable_expectation_on_the_fixture_carries_the_reason_its_name_promises ... ok
test an_optional_dangling_reference_holds_and_a_required_one_does_not ... ok
test every_expectation_kind_holds_somewhere_on_the_fixture_and_fails_somewhere_on_it ... ok
test a_gap_beside_an_undecidable_subject_still_decides_the_expectation_false ... ok
test the_committed_example_reaches_all_three_verdicts_and_the_counts_are_the_documented_ones ... ok
test workload_exists_holds_for_each_of_the_three_kinds_and_fails_when_the_kind_is_the_wrong_one ... ok

test result: ok. 14 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

     Running tests/spec.rs (target/debug/deps/spec-d27b9540362abae1)

running 21 tests
test a_document_that_does_not_deserialize_is_one_coded_refusal_and_not_a_serde_sentence ... ok
test a_probe_remedy_states_exactly_one_handler_and_neither_is_refused_the_same_way_as_both ... ok
test a_format_this_build_does_not_read_is_refused_with_its_own_code ... ok
test a_predicate_reading_a_fact_the_projection_never_states_is_refused_as_a_typo ... ok
test a_quoted_number_is_refused_as_a_port_name_because_it_is_one ... ok
test a_remedy_beside_a_kind_that_never_finds_an_empty_field_is_refused_rather_than_carried ... ok
test a_probes_remedy_for_a_probe_the_expectation_never_asks_for_is_refused ... ok
test a_remedy_that_states_nothing_is_refused_because_it_leaves_the_gap_where_it_was ... ok
test a_remedy_that_validates_is_carried_on_the_expectation_and_a_document_without_one_carries_none ... ok
test a_scope_that_cannot_select_the_expectations_subject_is_refused_in_both_directions ... ok
test a_specification_reads_from_json_too_because_json_is_yaml ... ok
test a_specification_with_four_defects_reports_four_refusals_in_one_run ... ok
test a_specification_with_no_expectations_is_refused_rather_than_read_as_satisfied ... ok
test an_id_that_is_not_an_identifier_is_refused_and_a_dashed_lowercase_one_is_not ... ok
test a_scope_selects_exactly_the_subject_classes_its_shape_can_reach ... ok
test two_expectations_sharing_an_id_are_refused_because_a_report_names_a_verdict_by_it ... ok
test every_kind_whose_parameters_can_decide_nothing_is_refused_with_one_code ... ok
test the_validated_type_is_only_reachable_through_validation ... ok
test the_committed_example_specification_validates_and_declares_every_kind ... ok
test a_remedy_changes_no_verdict_because_nothing_evaluates_one ... ok
test the_committed_example_specification_simulates_identically_with_and_without_its_remedies ... ok

test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests infra_analyze

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_compiler

running 16 tests
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 598) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr::model (line 659) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 564) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 576) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 584) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 546) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 552) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 558) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 570) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 590) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 604) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 623) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 619) - compile fail ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 615) ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr::model (line 651) ... ok
test crates/infra/infra-compiler/src/ir.rs - ir::InfraIr (line 537) ... ok

test result: ok. 16 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.06s

   Doc-tests infra_project

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

   Doc-tests infra_spec

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

### final-fmt

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo fmt --package infra-compiler --package infra-analyze --package infra-spec --package infra-project --package ess-cli --check
```

Exit: 0. Raw combined output (`target/review-boundaries-4/final-fmt.log`):

```text
```

### final-clippy

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
env TMPDIR="$PWD/target" RUSTC_WRAPPER=/usr/bin/sccache SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 CARGO_NET_OFFLINE=true cargo clippy --locked --offline -p infra-compiler -p infra-analyze -p infra-spec -p infra-project -p ess-cli --all-targets -- -D warnings
```

Exit: 0. Raw combined output (`target/review-boundaries-4/final-clippy.log`):

```text
    Checking memchr v2.8.3
    Checking itoa v1.0.18
    Checking cfg-if v1.0.4
    Checking typenum v1.20.1
    Checking const-oid v0.10.2
    Checking cpufeatures v0.3.1
    Checking equivalent v1.0.2
    Checking allocator-api2 v0.2.21
    Checking foldhash v0.2.0
    Checking unsafe-libyaml v0.2.11
    Checking dyn-clone v1.0.20
    Checking ryu v1.0.23
    Checking serde_core v1.0.229
    Checking zmij v1.0.23
    Checking libc v0.2.189
    Checking thiserror v2.0.20
    Checking num-traits v0.2.19
    Checking regex-syntax v0.8.11
    Checking zerocopy v0.8.56
    Checking smallvec v1.16.0
    Checking bitflags v2.13.1
    Checking utf8parse v0.2.2
    Checking pulldown-cmark-escape v0.11.0
    Checking once_cell v1.21.4
    Checking scopeguard v1.2.0
    Checking unicase v2.9.0
    Checking ref-cast v1.0.27
    Checking is_terminal_polyfill v1.70.2
    Checking anstyle-query v1.1.5
    Checking anstyle v1.0.14
    Checking hashbrown v0.17.1
    Checking bit-vec v0.8.0
    Checking anstyle-parse v1.0.0
    Checking lock_api v0.4.14
    Checking colorchoice v1.0.5
    Checking borrow-or-share v0.2.4
    Checking clap_lex v1.1.0
    Checking aho-corasick v1.1.5
    Checking bytecount v0.6.9
    Checking num-cmp v0.1.0
    Checking micromap v0.3.0
    Checking pulldown-cmark v0.13.4
    Checking vsimd v0.8.0
    Checking percent-encoding v2.3.2
    Checking strsim v0.11.1
    Checking outref v0.5.2
    Checking bit-set v0.8.0
    Checking num-integer v0.1.47
    Checking num-complex v0.4.6
    Checking strum v0.28.0
    Checking anstream v1.0.0
    Checking unicode-general-category v1.1.0
    Checking data-encoding v2.11.1
    Checking hybrid-array v0.4.14
    Checking anyhow v1.0.104
    Checking getrandom v0.3.4
    Checking parking_lot_core v0.9.12
    Checking num-bigint v0.4.8
    Checking num-iter v0.1.46
    Checking indexmap v2.14.1
    Checking clap_builder v4.6.6
    Checking uuid-simd v0.8.0
    Checking parking_lot v0.12.5
    Checking regex-automata v0.4.18
    Checking jsonschema-regex v0.52.1
    Checking crypto-common v0.2.2
    Checking block-buffer v0.12.1
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking semver v1.0.28
    Checking num-rational v0.4.2
    Checking digest v0.11.3
    Checking serde_yaml v0.9.34+deprecated
    Checking fluent-uri v0.4.1
    Checking email_address v0.2.9
    Checking sha2 v0.11.0
    Checking num v0.4.3
    Checking fraction v0.17.0
    Checking schemars v0.8.22
    Checking clap v4.6.6
    Checking infra-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-domain)
    Checking ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/ess-kubernetes)
    Checking ess-openapi v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-openapi)
    Checking ahash v0.8.12
    Checking ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-primitives)
    Checking jsonschema-value v0.52.1
    Checking referencing v0.52.1
    Checking regex v1.13.1
    Checking fancy-regex v0.19.0
    Checking ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-domain)
    Checking infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-compiler)
    Checking jsonschema v0.52.1
    Checking infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-analyze)
    Checking infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-spec)
    Checking ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-compiler)
    Checking infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/infra/infra-project)
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-gen)
    Checking ess-realization v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-realization)
    Checking ess-composition v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/specify/ess-composition)
    Checking ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-deployment)
    Checking schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/schema-contract)
    Checking ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/verify/ess-conformance)
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/generate/ess-synth)
    Checking ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/verify/ess-diff)
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 10.25s
```

### diff-check

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
git diff --check
```

Exit: 0. Raw combined output (`target/review-boundaries-4/diff-check.log`):

```text
```

### Canonical and compatibility evidence

The unmodified base CLI generated base-k3d.ir.json before the production repair. cmp against the committed cluster.ir.json was exit 0, and SHA-256 was 3186a06dd21e8f90cd2af371604cf2119a2fe534c383921a31da490465496c09 for both. The final CLI generated final-k3d.ir.json with the same bytes, as recorded below. The new reader/no-op transformation test also compares the frozen committed document, including its newline, rather than only comparing the new writer to itself.

Existing infra-spec tests preserve the committed simulation/drift bytes; existing infra-project determinism tests preserve every generated file and the reverse file inventory. The new round-trip case actually applies emitted replica, resource and probe patches plus the induced PDB to an observation bundle and recompiles it, checking all modeled outcomes. Existing corrupted-patch and strategic/merge controls remain green.

All six handle lookups are called after compile, read, clone and valid transformation. Each of six referenced target-map deletions is separately attempted and refused in the runtime matrix; source document, digest, provenance and previously obtained source handles are checked after each refusal. Successful transformed bytes read back as the same IR.

### base-writer

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
target/debug/ess import kubernetes --path examples/k3d-dev-cluster/observation.json --out target/review-boundaries-4/base-k3d.ir.json --format json
```

Exit: 0. Raw combined output (`target/review-boundaries-4/base-writer.log`):

```text
{
  "adapter": "kubernetes",
  "direction": "import",
  "supported": [
    "cluster",
    "namespace",
    "workload",
    "service",
    "ingress",
    "configuration",
    "secret-shape",
    "runtime"
  ],
  "coverage_gaps": [],
  "obligations": [],
  "refusals": [],
  "unresolved_references": 4,
  "output": "target/review-boundaries-4/base-k3d.ir.json"
}
```

### final-writer

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
target/debug/ess import kubernetes --path examples/k3d-dev-cluster/observation.json --out target/review-boundaries-4/final-k3d.ir.json --format json
```

Exit: 0. Raw combined output (`target/review-boundaries-4/final-writer.log`):

```text
{
  "adapter": "kubernetes",
  "direction": "import",
  "supported": [
    "cluster",
    "namespace",
    "workload",
    "service",
    "ingress",
    "configuration",
    "secret-shape",
    "runtime"
  ],
  "coverage_gaps": [],
  "obligations": [],
  "refusals": [],
  "unresolved_references": 4,
  "output": "target/review-boundaries-4/final-k3d.ir.json"
}
```

### canonical-comparison

Command (cwd `/home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants`):

```console
cmp target/review-boundaries-4/base-k3d.ir.json target/review-boundaries-4/final-k3d.ir.json
sha256sum target/review-boundaries-4/base-k3d.ir.json target/review-boundaries-4/final-k3d.ir.json examples/k3d-dev-cluster/cluster.ir.json
```

Exit: 0. Raw combined output (`target/review-boundaries-4/canonical-comparison.log`):

```text
3186a06dd21e8f90cd2af371604cf2119a2fe534c383921a31da490465496c09  target/review-boundaries-4/base-k3d.ir.json
3186a06dd21e8f90cd2af371604cf2119a2fe534c383921a31da490465496c09  target/review-boundaries-4/final-k3d.ir.json
3186a06dd21e8f90cd2af371604cf2119a2fe534c383921a31da490465496c09  examples/k3d-dev-cluster/cluster.ir.json
```

## 5. Deliberate limits and omissions

- This is a Rust API migration: external callers using the old public model field or total project return type must migrate. No inventory or execution of external Rust consumers is claimed.
- The persisted infra-ir/1 shape, reference encoding and digest algorithm are unchanged. The executed compatibility check is the exact base-writer fixture comparison and current reader admission; this is not an execution matrix of separately installed historical readers or a deployment claim.
- read_document still does not revalidate every original domain-value invariant or rederive unresolved facts. Checked transformations guarantee its existing relational/shape admission only. Observation completeness and selector semantics remain the later story's work.
- Handles remain owner-specific under the existing contract. Cross-IR branding/generativity was not introduced. Source handles are not promised usable against a different transformed owner if that owner's sites/targets changed.
- The generic checked transformation pays cloning and serialization/admission per generated candidate. No performance claim or benchmark was added.
- No new serialized refusal variant or format version was invented. Project admission failures use the existing typed reader errors and abort before CLI output writes.
- No dependency, root lockfile, generated/example product, public guide, scanner/Secret code or planning/Git/lifecycle file was modified. Existing secret projection tests ran in the assigned package suite; the scanner's own suite/mutation gates are coordinator integration work.
- Full workspace task check and site-build were not run; the brief assigns those integration gates to the coordinator.
- Runner timings are retained in the raw logs (baseline build 5-package run and final build/test output, plus strict Clippy 10.25s). Overall token/duration accounting was not available and is not invented.

## 6. Paths, resources and handoff

Wrote outside the assigned worktree: none. All logs, exit files, base/final writer captures, this report and temporary probe/compiler artifacts are beneath /home/timo/.local/state/worktree/trees/b10x/ess/review-infra-ir-invariants/target. Authored source/tests/design are only the assigned paths listed above. The retired ad hoc probe's temporary sources/artifacts remain under target/review-boundaries-4/public-api; no managed tree or build directory was removed.

All Cargo commands used the brief's exact environment, including the coordinator-owned idle-disabled w4 socket, worktree-local TMPDIR, offline mode and no CARGO_TARGET_DIR. No cache lifecycle/override was performed.

Disk available before the baseline build: 141112229888 bytes, observed by df -B1, above the 8589934592-byte reserve. Handoff observation:

```text
Filesystem        1B-blocks         Used    Available Use% Mounted on
/dev/nvme0n1p2 910126964736 726303838208 137515773952  85% /
```

Scratch sizes measured with separate `du -sb` calls (before adding this short size record): 295950	target/review-boundaries-4; 1064197518	target. The total target measurement includes the review scratch.

All assigned source edits and reports are ready for the coordinator to inspect, commit and route to a different adversary. No coordinator patch or extra scope is required. This handoff relinquishes writes.
