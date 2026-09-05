unit: story:review-persisted-delivery-validation — Restore delivery IR invariants at persisted read boundaries
verdict: green
cases: executed 53→67 overall; ess-deployment 7→18; ess-cli 46→49; initial meaningful red: 9 failed across 11 executed cases
origin: n/a — implementation answers brief and review F02; origin classification belongs to adversary
wrote-outside-worktree: none authored
needs-coordinator: no implementation prerequisite; ready for independent review, coordinator-owned integration and cleanup

1. Unit and acceptance

Malformed persisted delivery documents are refused before analysis or external execution while valid compiler-produced documents retain canonical bytes.

Read the complete story, exact implementor charter, tree AGENTS, brief, resource supplement and F02. Confirmed inferred shared validation surfaces, generic CLI JSON/YAML readers, missing executor abstraction, available compiler-produced fixtures, mutable models, original bundle map-key loss, and ORAS-before-Helm execution order. No inferred file/scope location was wrong. No planning mutation or Git lifecycle operation was performed. The dependency graph had no depends_on edge for this active story.

2. Observed diff

`git diff --stat` followed by `git diff --no-index --stat /dev/null <new-file>` for each untracked source file (no staging):

```text
 crates/edge/ess-cli/src/main.rs                    |  11 +
 crates/generate/ess-deployment/src/build.rs        |  70 ++-
 crates/generate/ess-deployment/src/component.rs    | 108 +++-
 crates/generate/ess-deployment/src/environment.rs  | 126 ++++-
 crates/generate/ess-deployment/src/lib.rs          |   1 +
 crates/generate/ess-deployment/src/release.rs      |  84 ++-
 crates/generate/ess-deployment/src/runtime.rs      | 180 ++++++-
 crates/generate/ess-deployment/src/stack.rs        | 179 ++++++-
 crates/generate/ess-deployment/tests/deployment.rs | 562 +++++++++++++++++++++
 9 files changed, 1278 insertions(+), 43 deletions(-)
 .../edge/ess-cli/tests/persisted_delivery.rs       | 176 +++++++++++++++++++++
 1 file changed, 176 insertions(+)
 .../edge/ess-cli/tests/support/fake_delivery.rs    | 27 ++++++++++++++++++++++
 1 file changed, 27 insertions(+)
 .../generate/ess-deployment/src/validation.rs      | 75 ++++++++++++++++++++++
 1 file changed, 75 insertions(+)
```

The complete source/test patch is saved beside this report as `implementation.patch`; source and tests remain uncommitted.

Implementation inventory and decisions:

| Envelope | Validation restored |
| --- | --- |
| BuildIr | Exact format, platform nonemptiness/coordinates, output map identity, dependency resolution/cycles, canonical lexical Kahn order, node paths/argv/mounts/secrets, output node kinds/repository requirement; reuses compile_build through a private authored DTO reconstruction. |
| RuntimeIr | Exact format, process/container/workload/endpoint map identities and references, replicas > 0, workload container membership, component uniqueness across workloads, existing slot/environment, mount, volume and probe checks. Included BuildIr additionally proves build digest and process image/output kind relationships. |
| ComponentIr | Exact format and compile_component rules for semantic major spelling, relative input paths and independent release units; no filesystem reads. |
| ReleaseManifest | Exact format, source commit spelling, nonempty artifacts, artifact identities and OCI platform attachments, four required evidence attachments. Public mutation is rechecked by verify_release. |
| ReleaseBundle | Exact format, original release-map keys before any values-only rebuilding, expected runtime/chart units, identities/kinds, nested checked decoding, release cross-digests, runtime-to-build digest and process-image relationships. A self-consistent hash cannot bypass graph validation. |
| ReleaseCatalog | Exact format, every nested candidate document and its available runtime/build/semantic digest relationships before selection, including unselected entries; resolve_stack rechecks public mutations. |
| StackLock | Exact format, external-system identity, Helm chart selection, runtime artifact identities/platform attachments and OCI presence, requirement shape, dependency references and cycles. Composition-local service keys deliberately need not equal LockedSystem.system. compile_deployment rechecks public mutations. |
| DeploymentIr | Exact format, cluster/release/namespace bindings, map identity, chart/image kinds and image attachments, audience/service-account implication, dependency references/cycles and complete canonical lexical Kahn order. |

A private checked-deserialization macro creates typed Wire DTOs and validates before returning each envelope. JSON, YAML, direct Serde, serde_json::Value and nested envelopes use that boundary. A streaming map visitor rejects duplicate keys before BTreeMap discards them; recursive tests exercise every populated nested object/map in the compiled fixtures, with explicit map fixtures for build environments and deployment config. Authored-only environment DTO fields retain their parsing contract; nested types shared with persisted IR require strict map decoding.

Public mutation policy: BuildIr, RuntimeIr and ComponentIr already have private invariant-bearing fields. Mutable manifest/catalog/bundle/lock/deployment models expose validate() and are rechecked by consuming verification/compiler entrypoints. CLI diff and reconcile explicitly validate desired and current plans before affected/removed analysis or any process launch. Serialization/digest calculation remains an encoding operation and is not an assertion that an arbitrarily mutated public model is valid.

3. Red evidence

Production stayed unchanged until the following reader and CLI cases failed. The first reader compile attempt had a JSON macro syntax error and ran no cases; it is retained separately. The next run executed 8 reader cases (1 passed, 7 failed). The expanded matrix run retained every assertion and measured all table mutations. The late-chart CLI run recorded ORAS then Helm before rejecting the second release. Its valid fake-process control passed. Additional post-correction coverage checks convenience routes, catalog/mutable entrypoints and recursive duplicate-map enumeration.

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment persisted_`

Exit: 101

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
error: no rules expected `.`
   --> crates/generate/ess-deployment/tests/deployment.rs:542:42
    |
542 |         "systems": ["frontend", "worker"].map(|service| serde_json::json!({
    |                                          ^ no rules expected this token in macro call
    |
    = note: while trying to match end of macro

error: no rules expected `.`
   --> crates/generate/ess-deployment/tests/deployment.rs:552:43
    |
552 |         "releases": ["frontend", "worker"].map(|service| serde_json::json!({
    |                                           ^ no rules expected this token in macro call
    |
    = note: while trying to match end of macro

error: could not compile `ess-deployment` (test "deployment") due to 2 previous errors
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment persisted_`

Exit: 101

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 0.70s
     Running unittests src/lib.rs (target/debug/deps/ess_deployment-e5782514a987b31e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 8 tests
test persisted_component_and_release_readers_refuse_invalid_local_documents ... FAILED
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... FAILED
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... FAILED
test persisted_readers_reject_duplicate_map_keys_before_collection ... FAILED
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... FAILED
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... FAILED
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... FAILED
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok

failures:

---- persisted_component_and_release_readers_refuse_invalid_local_documents stdout ----

thread 'persisted_component_and_release_readers_refuse_invalid_local_documents' (623737) panicked at crates/generate/ess-deployment/tests/deployment.rs:565:5:
JSON admitted {"component":"oracle","format":"future/99","inputs":{"build":"ess/build.yaml","realization":"ess/realization.yaml","runtime":"ess/runtime.yaml","specification":"spec/oracle"},"release_units":{"chart":"oracle-chart","runtime":"oracle-runtime"},"semantic_version":"v1","system":"oracle"}
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints stdout ----

thread 'persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints' (623735) panicked at crates/generate/ess-deployment/tests/deployment.rs:565:5:
JSON admitted {"build":"oracle-runtime","format":"future/99","nodes":{"base":{"digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000","kind":"oci_base","reference":"docker.io/library/alpine"},"chart-file":{"from":"compile","kind":"artifact","path":"/src/chart.tgz"},"compile":{"argv":["cp","/src/oracle","/usr/local/bin/oracle"],"base":"base","kind":"run","mounts":[{"from":"source","kind":"input","target":"/src"}],"network":"none"},"runtime-image":{"config":{"entrypoint":["/usr/local/bin/oracle"],"user":"10001"},"kind":"image","rootfs":"compile"},"source":{"destination":"/src","kind":"source","path":"."}},"order":["base","source","compile","chart-file","runtime-image"],"outputs":{"app":{"kind":"oci_image","name":"app","node":"runtime-image","release_unit":"oracle-runtime","repository":"registry.example/oracle"},"chart":{"kind":"helm_chart","name":"chart","node":"chart-file","release_unit":"oracle-chart"}},"platforms":[{"architecture":"amd64","os":"linux"}],"secrets":["registry-token"]}

---- persisted_runtime_readers_refuse_local_relationship_and_slot_defects stdout ----

thread 'persisted_runtime_readers_refuse_local_relationship_and_slot_defects' (623742) panicked at crates/generate/ess-deployment/tests/deployment.rs:565:5:
JSON admitted {"build_digest":"sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8","containers":{"server":{"audiences":["urn:example:oracle"],"config":[{"environment":"LOG_LEVEL","kind":"optional","name":"log-level"}],"endpoints":[{"endpoint":"api","environment":"CARRIER_URL","name":"carrier-api","system":"carrier"}],"http_port":8080,"liveness_path":"/live","name":"server","process":"server","readiness_path":"/ready","secrets":[{"environment":"DATABASE_PASSWORD","key":"password","name":"database-password"}],"volume_mounts":[{"mount_path":"/var/lib/oracle","volume":"data"}]}},"format":"future/99","processes":{"server":{"image":"app","name":"server"}},"provided_endpoints":{"api":{"container":"server","name":"api","scheme":"http","workload":"oracle"}},"realization_digest":"sha256:44dfdc5dd06d4dba4d19b1d530892d2deef694af03d1ee85dfa16315c0639714","runtime":"oracle-runtime","semantic_digest":"sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee","workloads":{"oracle":{"components":["dispatch-service","order-service"],"containers":["server"],"name":"oracle","replicas":1,"volumes":[{"name":"data","size":"1Gi"}]}}}

---- persisted_readers_reject_duplicate_map_keys_before_collection stdout ----

thread 'persisted_readers_reject_duplicate_map_keys_before_collection' (623741) panicked at crates/generate/ess-deployment/tests/deployment.rs:753:9:
duplicate key admitted {"format":"ess-build-ir/1","build":"oracle-runtime","platforms":[{"os":"linux","architecture":"amd64"}],"secrets":["registry-token"],"nodes":{"base":{"kind":"oci_base","reference":"docker.io/library/alpine","digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000"},"chart-file":{"kind":"artifact","from":"compile","path":"/src/chart.tgz"},"compile":{"kind":"run","base":"base","argv":["cp","/src/oracle","/usr/local/bin/oracle"],"mounts":[{"kind":"input","from":"source","target":"/src"}],"network":"none"},"runtime-image":{"kind":"image","rootfs":"compile","config":{"entrypoint":["/usr/local/bin/oracle"],"user":"10001"}},"source":{"kind":"source","path":".","destination":"/src"}},"order":["base","source","compile","chart-file","runtime-image"],"outputs":{"app":{"kind":"oci_image","name":"app","node":"runtime-image","release_unit":"oracle-runtime","repository":"registry.example/oracle"},"app":{"name":"app","release_unit":"oracle-runtime","node":"runtime-image","kind":"oci_image","repository":"registry.example/oracle"},"chart":{"name":"chart","release_unit":"oracle-chart","node":"chart-file","kind":"helm_chart"}}}

---- persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs stdout ----

thread 'persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs' (623736) panicked at crates/generate/ess-deployment/tests/deployment.rs:565:5:
JSON admitted {"build":{"build":"oracle-runtime","format":"ess-build-ir/1","nodes":{"base":{"digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000","kind":"oci_base","reference":"docker.io/library/alpine"},"chart-file":{"from":"compile","kind":"artifact","path":"/src/chart.tgz"},"compile":{"argv":["cp","/src/oracle","/usr/local/bin/oracle"],"base":"base","kind":"run","mounts":[{"from":"source","kind":"input","target":"/src"}],"network":"none"},"runtime-image":{"config":{"entrypoint":["/usr/local/bin/oracle"],"user":"10001"},"kind":"image","rootfs":"compile"},"source":{"destination":"/src","kind":"source","path":"."}},"order":["base","source","compile","chart-file","runtime-image"],"outputs":{"app":{"kind":"oci_image","name":"app","node":"runtime-image","release_unit":"oracle-runtime","repository":"registry.example/oracle"},"chart":{"kind":"helm_chart","name":"chart","node":"chart-file","release_unit":"oracle-chart"}},"platforms":[{"architecture":"amd64","os":"linux"}],"secrets":["registry-token"]},"component":{"component":"oracle","format":"ess-component-ir/1","inputs":{"build":"ess/build.yaml","realization":"ess/realization.yaml","runtime":"ess/runtime.yaml","specification":"spec/oracle"},"release_units":{"chart":"oracle-chart","runtime":"oracle-runtime"},"semantic_version":"v1","system":"oracle"},"format":"future/99","releases":{"oracle-chart":{"artifacts":{"chart":{"build_output":"chart","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"helm_chart","reference":"oci://registry.example/charts/oracle"}},"build_digest":"sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8","evidence":{"conformance":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/conformance"},"provenance":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/provenance"},"sbom":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/sbom"},"signature":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/signature"}},"format":"ess-release/1","release_unit":"oracle-chart","runtime_digest":"sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910","semantic_digest":"sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee","source_commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","system":"oracle","version":"4.5.6"},"oracle-runtime":{"artifacts":{"app":{"build_output":"app","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"oci_image","platforms":{"linux/amd64":"sha256:1111111111111111111111111111111111111111111111111111111111111111"},"reference":"registry.example/oracle"}},"build_digest":"sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8","evidence":{"conformance":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/conformance"},"provenance":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/provenance"},"sbom":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/sbom"},"signature":{"digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","reference":"registry.example/evidence/signature"}},"format":"ess-release/1","release_unit":"oracle-runtime","runtime_digest":"sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910","semantic_digest":"sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee","source_commit":"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","system":"oracle","version":"1.2.3"}},"runtime":{"build_digest":"sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8","containers":{"server":{"audiences":["urn:example:oracle"],"config":[{"environment":"LOG_LEVEL","kind":"optional","name":"log-level"}],"endpoints":[{"endpoint":"api","environment":"CARRIER_URL","name":"carrier-api","system":"carrier"}],"http_port":8080,"liveness_path":"/live","name":"server","process":"server","readiness_path":"/ready","secrets":[{"environment":"DATABASE_PASSWORD","key":"password","name":"database-password"}],"volume_mounts":[{"mount_path":"/var/lib/oracle","volume":"data"}]}},"format":"ess-runtime-ir/1","processes":{"server":{"image":"app","name":"server"}},"provided_endpoints":{"api":{"container":"server","name":"api","scheme":"http","workload":"oracle"}},"realization_digest":"sha256:44dfdc5dd06d4dba4d19b1d530892d2deef694af03d1ee85dfa16315c0639714","runtime":"oracle-runtime","semantic_digest":"sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee","workloads":{"oracle":{"components":["dispatch-service","order-service"],"containers":["server"],"name":"oracle","replicas":1,"volumes":[{"name":"data","size":"1Gi"}]}}}}

---- persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order stdout ----

thread 'persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order' (623738) panicked at crates/generate/ess-deployment/tests/deployment.rs:565:5:
JSON admitted {"cluster":"test-cluster","environment":"test","format":"future/99","releases":{"frontend":{"audiences":["urn:example:oracle"],"chart":{"build_output":"chart","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"helm_chart","reference":"oci://registry.example/charts/oracle"},"endpoints":{"carrier-api":"https://example.invalid"},"images":{"app":{"build_output":"app","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"oci_image","platforms":{"linux/amd64":"sha256:1111111111111111111111111111111111111111111111111111111111111111"},"reference":"registry.example/oracle"}},"namespace":"test","release_name":"frontend","secrets":{"database-password":{"key":"password","name":"database"}},"service":"frontend","service_account":"frontend"},"worker":{"audiences":["urn:example:oracle"],"chart":{"build_output":"chart","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"helm_chart","reference":"oci://registry.example/charts/oracle"},"endpoints":{"carrier-api":"https://example.invalid"},"images":{"app":{"build_output":"app","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"oci_image","platforms":{"linux/amd64":"sha256:1111111111111111111111111111111111111111111111111111111111111111"},"reference":"registry.example/oracle"}},"namespace":"test","release_name":"worker","secrets":{"database-password":{"key":"password","name":"database"}},"service":"worker","service_account":"worker"}},"rollout_order":["frontend","worker"],"stack_digest":"sha256:398b76d0e732b4ead1473aa45b4860256fba775896f1505f2c60e13b6966f5e1"}

---- persisted_lock_readers_preserve_local_service_identity_and_reject_invariants stdout ----

thread 'persisted_lock_readers_preserve_local_service_identity_and_reject_invariants' (623740) panicked at crates/generate/ess-deployment/tests/deployment.rs:565:5:
JSON admitted {"composition_digest":"sha256:067a90a6beaeb001d58c26f9d5bbb223d4e3622aedc50eaa9294800ecb683943","external_systems":{"carrier":{"contract":"carrier-http/v1","managed":false,"system":"carrier"}},"format":"future/99","stack":"example","stack_digest":"sha256:6e61fba20a00458d379f66e5bef39f7db3191ccf41a73af00bd5d4bb577f1f19","systems":{"frontend":{"build_digest":"sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8","chart":{"build_output":"chart","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"helm_chart","reference":"oci://registry.example/charts/oracle"},"chart_release_digest":"sha256:b562b1879795b15691371fa113eefe9c6217400f72400af205f45730c85c7194","chart_release_unit":"oracle-chart","chart_version":"4.5.6","depends_on":[],"release_digest":"sha256:a532bd07c01ee96f8f73b2338315846beb5eb28309a4e1bf111f51b44f8fffd8","release_unit":"oracle-runtime","runtime":{"audiences":["urn:example:oracle"],"config":{"log-level":"optional"},"endpoint_names":{"carrier-api":"api"},"endpoints":{"carrier-api":"carrier"},"provided_endpoints":{"api":{"port":8080,"scheme":"http"}},"secrets":["database-password"]},"runtime_artifacts":{"app":{"build_output":"app","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"oci_image","platforms":{"linux/amd64":"sha256:1111111111111111111111111111111111111111111111111111111111111111"},"reference":"registry.example/oracle"}},"runtime_digest":"sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910","semantic_digest":"sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee","semantic_version":"v1","system":"oracle","version":"1.2.3"},"worker":{"build_digest":"sha256:02eb745407234752589456dd6c076ef0e785830612abcdab6f152f7acdfc5fc8","chart":{"build_output":"chart","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"helm_chart","reference":"oci://registry.example/charts/oracle"},"chart_release_digest":"sha256:b562b1879795b15691371fa113eefe9c6217400f72400af205f45730c85c7194","chart_release_unit":"oracle-chart","chart_version":"4.5.6","depends_on":[],"release_digest":"sha256:a532bd07c01ee96f8f73b2338315846beb5eb28309a4e1bf111f51b44f8fffd8","release_unit":"oracle-runtime","runtime":{"audiences":["urn:example:oracle"],"config":{"log-level":"optional"},"endpoint_names":{"carrier-api":"api"},"endpoints":{"carrier-api":"carrier"},"provided_endpoints":{"api":{"port":8080,"scheme":"http"}},"secrets":["database-password"]},"runtime_artifacts":{"app":{"build_output":"app","digest":"sha256:1111111111111111111111111111111111111111111111111111111111111111","kind":"oci_image","platforms":{"linux/amd64":"sha256:1111111111111111111111111111111111111111111111111111111111111111"},"reference":"registry.example/oracle"}},"runtime_digest":"sha256:5538a120fc1a22dde331fb39808d1e4e72bf0c69a79068049875886ce18b3910","semantic_digest":"sha256:4288d50a003fa7d5b39743327880aa7e2f97ff6d9408f8a5ddb908c8b6af79ee","semantic_version":"v1","system":"oracle","version":"1.2.3"}}}


failures:
    persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints
    persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs
    persisted_component_and_release_readers_refuse_invalid_local_documents
    persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order
    persisted_lock_readers_preserve_local_service_identity_and_reject_invariants
    persisted_readers_reject_duplicate_map_keys_before_collection
    persisted_runtime_readers_refuse_local_relationship_and_slot_defects

test result: FAILED. 1 passed; 7 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p ess-deployment --test deployment`
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment persisted_`

Exit: 101

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 0.65s
     Running unittests src/lib.rs (target/debug/deps/ess_deployment-e5782514a987b31e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 8 tests
test persisted_component_and_release_readers_refuse_invalid_local_documents ... FAILED
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... FAILED
test persisted_readers_reject_duplicate_map_keys_before_collection ... FAILED
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... FAILED
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... FAILED
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... FAILED
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... FAILED
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok

failures:

---- persisted_component_and_release_readers_refuse_invalid_local_documents stdout ----

thread 'persisted_component_and_release_readers_refuse_invalid_local_documents' (629552) panicked at crates/generate/ess-deployment/tests/deployment.rs:586:5:
admitted mutations: ["/format: \"future/99\"", "/semantic_version: \"1.0\"", "/inputs/runtime: \"../escape\"", "/inputs/build: \"\"", "/release_units/chart: \"oracle-runtime\""]
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints stdout ----

thread 'persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints' (629550) panicked at crates/generate/ess-deployment/tests/deployment.rs:586:5:
admitted mutations: ["/format: \"future/99\"", "/platforms: []", "/platforms/0/os: \"\"", "/platforms/0/architecture: \"\"", "/order: []", "/order: [\"base\",\"base\"]", "/order: [\"source\",\"base\",\"compile\",\"chart-file\",\"runtime-image\"]", "/nodes/compile/base: \"missing\"", "/nodes/compile/base: \"runtime-image\"", "/nodes/source/path: \"../escape\"", "/nodes/source/destination: \"relative\"", "/nodes/base/reference: \"alpine:latest\"", "/nodes/compile/argv: []", "/nodes/compile/argv: [\"\"]", "/nodes/compile/mounts: [{\"kind\":\"secret\",\"secret\":\"missing\",\"target\":\"/secret\"}]", "/nodes/compile/mounts/0/target: \"relative\"", "/nodes/chart-file/path: \"relative\"", "/outputs/app/name: \"renamed\"", "/outputs/app/node: \"missing\"", "/outputs/app/node: \"chart-file\"", "/outputs/app/repository: null", "/outputs: {}"]

---- persisted_readers_reject_duplicate_map_keys_before_collection stdout ----

thread 'persisted_readers_reject_duplicate_map_keys_before_collection' (629556) panicked at crates/generate/ess-deployment/tests/deployment.rs:759:9:
duplicate key admitted {"format":"ess-build-ir/1","build":"oracle-runtime","platforms":[{"os":"linux","architecture":"amd64"}],"secrets":["registry-token"],"nodes":{"base":{"kind":"oci_base","reference":"docker.io/library/alpine","digest":"sha256:0000000000000000000000000000000000000000000000000000000000000000"},"chart-file":{"kind":"artifact","from":"compile","path":"/src/chart.tgz"},"compile":{"kind":"run","base":"base","argv":["cp","/src/oracle","/usr/local/bin/oracle"],"mounts":[{"kind":"input","from":"source","target":"/src"}],"network":"none"},"runtime-image":{"kind":"image","rootfs":"compile","config":{"entrypoint":["/usr/local/bin/oracle"],"user":"10001"}},"source":{"kind":"source","path":".","destination":"/src"}},"order":["base","source","compile","chart-file","runtime-image"],"outputs":{"app":{"kind":"oci_image","name":"app","node":"runtime-image","release_unit":"oracle-runtime","repository":"registry.example/oracle"},"app":{"name":"app","release_unit":"oracle-runtime","node":"runtime-image","kind":"oci_image","repository":"registry.example/oracle"},"chart":{"name":"chart","release_unit":"oracle-chart","node":"chart-file","kind":"helm_chart"}}}

---- persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs stdout ----

thread 'persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs' (629551) panicked at crates/generate/ess-deployment/tests/deployment.rs:586:5:
admitted mutations: ["/format: \"future/99\""]

---- persisted_runtime_readers_refuse_local_relationship_and_slot_defects stdout ----

thread 'persisted_runtime_readers_refuse_local_relationship_and_slot_defects' (629557) panicked at crates/generate/ess-deployment/tests/deployment.rs:586:5:
admitted mutations: ["/format: \"future/99\"", "/processes/server/name: \"other\"", "/containers/server/name: \"other\"", "/containers/server/process: \"missing\"", "/workloads/oracle/name: \"other\"", "/workloads/oracle/replicas: 0", "/workloads/oracle/containers: []", "/workloads/oracle/containers: [\"missing\"]", "/containers/server/http_port: null", "/containers/server/config/0/kind: \"literal\"", "/containers/server/secrets/0/environment: \"LOG_LEVEL\"", "/containers/server/secrets/0/name: \"log-level\"", "/containers/server/volume_mounts/0/mount_path: \"/var/../escape\"", "/containers/server/volume_mounts/0/volume: \"missing\"", "/workloads/oracle/volumes/0/size: \"\"", "/provided_endpoints/api/name: \"other\"", "/provided_endpoints/api/workload: \"missing\"", "/provided_endpoints/api/container: \"missing\""]

---- persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order stdout ----

thread 'persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order' (629553) panicked at crates/generate/ess-deployment/tests/deployment.rs:586:5:
admitted mutations: ["/format: \"future/99\"", "/cluster: \"\"", "/rollout_order: []", "/rollout_order: [\"frontend\"]", "/rollout_order: [\"frontend\",\"frontend\"]", "/rollout_order: [\"frontend\",\"missing\"]", "/rollout_order: [\"worker\",\"frontend\"]", "/releases/frontend/service: \"other\"", "/releases/frontend/release_name: \"\"", "/releases/frontend/namespace: \"\"", "/releases/frontend/service_account: \"\"", "/releases/frontend/chart/kind: \"binary\"", "/releases/frontend/images/app/build_output: \"other\"", "/releases/frontend/images/app/kind: \"binary\"", "/releases/frontend/images/app/platforms: {}"]

---- persisted_lock_readers_preserve_local_service_identity_and_reject_invariants stdout ----

thread 'persisted_lock_readers_preserve_local_service_identity_and_reject_invariants' (629555) panicked at crates/generate/ess-deployment/tests/deployment.rs:586:5:
admitted mutations: ["/format: \"future/99\"", "/external_systems/carrier/system: \"other\"", "/systems/frontend/depends_on: [\"missing\"]", "/systems/frontend/depends_on: [\"frontend\"]", "/systems/frontend/chart/kind: \"binary\"", "/systems/frontend/runtime_artifacts/app/build_output: \"other\"", "/systems/frontend/runtime_artifacts/app/platforms: {}", "/systems/frontend/runtime_artifacts: {}", "/systems/frontend/runtime/config/log-level: \"literal\"", "/systems/frontend/runtime/endpoint_names: {\"absent\":\"api\"}"]


failures:
    persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints
    persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs
    persisted_component_and_release_readers_refuse_invalid_local_documents
    persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order
    persisted_lock_readers_preserve_local_service_identity_and_reject_invariants
    persisted_readers_reject_duplicate_map_keys_before_collection
    persisted_runtime_readers_refuse_local_relationship_and_slot_defects

test result: FAILED. 1 passed; 7 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.03s

error: test failed, to rerun pass `-p ess-deployment --test deployment`
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli --test persisted_delivery`

Exit: 101

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.21s
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 3 tests
test invalid_current_removal_is_refused_before_analysis_and_execution ... FAILED
test entire_desired_plan_is_refused_before_oras_or_helm ... FAILED
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok

failures:

---- invalid_current_removal_is_refused_before_analysis_and_execution stdout ----

thread 'invalid_current_removal_is_refused_before_analysis_and_execution' (627723) panicked at crates/edge/ess-cli/tests/persisted_delivery.rs:96:9:
invalid current state was admitted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- entire_desired_plan_is_refused_before_oras_or_helm stdout ----

thread 'entire_desired_plan_is_refused_before_oras_or_helm' (627722) panicked at crates/edge/ess-cli/tests/persisted_delivery.rs:81:13:
admitted /format: test — 2 release(s) reconciled, 0 removed



failures:
    entire_desired_plan_is_refused_before_oras_or_helm
    invalid_current_removal_is_refused_before_analysis_and_execution

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p ess-cli --test persisted_delivery`
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli --test persisted_delivery`

Exit: 101

```text
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 0.21s
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 3 tests
test invalid_current_removal_is_refused_before_analysis_and_execution ... FAILED
test entire_desired_plan_is_refused_before_oras_or_helm ... FAILED
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok

failures:

---- invalid_current_removal_is_refused_before_analysis_and_execution stdout ----

thread 'invalid_current_removal_is_refused_before_analysis_and_execution' (629616) panicked at crates/edge/ess-cli/tests/persisted_delivery.rs:96:9:
invalid current state was admitted
note: run with `RUST_BACKTRACE=1` environment variable to display a backtrace

---- entire_desired_plan_is_refused_before_oras_or_helm stdout ----

thread 'entire_desired_plan_is_refused_before_oras_or_helm' (629615) panicked at crates/edge/ess-cli/tests/persisted_delivery.rs:31:33:
executors ran: oras
helm



failures:
    entire_desired_plan_is_refused_before_oras_or_helm
    invalid_current_removal_is_refused_before_analysis_and_execution

test result: FAILED. 1 passed; 2 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.08s

error: test failed, to rerun pass `-p ess-cli --test persisted_delivery`
```

4. Baseline, green execution, formatting and lint gates

The first baseline failed before compilation because the required long TMPDIR made sccache's startup notification socket exceed SUN_LEN. The coordinator supplied the exact server socket in resource-supplement.md. Cargo kept the prescribed TMPDIR, own target, wrapper and cache settings. To obtain an actual unmodified runner baseline after test authoring, the authored test file was preserved in scratch, the base test bytes temporarily restored from HEAD, the baseline run, and the authored tests restored before the red run. No assertion was removed from the final source.

The initial baseline command had the same environment except SCCACHE_SERVER_UDS was not yet set:

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment`

Exit: 101

```text
error: process didn't exit successfully: `/usr/bin/sccache /home/timo/.rustup/toolchains/stable-x86_64-unknown-linux-gnu/bin/rustc -vV` (exit status: 2)
--- stderr
sccache: error: path must be shorter than SUN_LEN

```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment`

Exit: 0

```text
   Compiling proc-macro2 v1.0.107
   Compiling quote v1.0.47
   Compiling unicode-ident v1.0.24
   Compiling serde_core v1.0.229
   Compiling serde v1.0.229
   Compiling zmij v1.0.23
   Compiling itoa v1.0.18
   Compiling serde_json v1.0.151
   Compiling typenum v1.20.1
   Compiling memchr v2.8.3
   Compiling thiserror v2.0.20
   Compiling schemars v0.8.22
   Compiling equivalent v1.0.2
   Compiling dyn-clone v1.0.20
   Compiling hashbrown v0.17.1
   Compiling unsafe-libyaml v0.2.11
   Compiling ryu v1.0.23
   Compiling const-oid v0.10.2
   Compiling cfg-if v1.0.4
   Compiling cpufeatures v0.3.1
   Compiling syn v3.0.4
   Compiling syn v2.0.119
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling indexmap v2.14.1
   Compiling hybrid-array v0.4.14
   Compiling block-buffer v0.12.1
   Compiling crypto-common v0.2.2
   Compiling digest v0.11.3
   Compiling semver v1.0.28
   Compiling sha2 v0.11.0
   Compiling serde_derive_internals v0.29.1
   Compiling serde_yaml v0.9.34+deprecated
   Compiling schemars_derive v0.8.22
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-primitives)
   Compiling ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-domain)
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-compiler)
   Compiling ess-realization v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-realization)
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 9.87s
     Running unittests src/lib.rs (target/debug/deps/ess_deployment-e5782514a987b31e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 7 tests
test input_order_does_not_change_locked_bytes ... ok
test canonical_build_ir_restores_an_omitted_empty_secret_set ... ok
test undeclared_secrets_and_cycles_are_stage_strict_refusals ... ok
test build_graph_is_canonical_and_projects_executable_buildkit_inputs ... ok
test helm_defaults_materialize_typed_secret_slots_without_secret_bytes ... ok
test component_release_bundle_is_canonical_and_revalidates_after_transport ... ok
test realization_runtime_release_stack_and_environment_form_one_exact_chain ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s

   Doc-tests ess_deployment

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli`

Exit: 0

```text
   Compiling serde_core v1.0.229
   Compiling memchr v2.8.3
   Compiling serde v1.0.229
   Compiling serde_json v1.0.151
   Compiling foldhash v0.2.0
   Compiling allocator-api2 v0.2.21
   Compiling autocfg v1.5.1
   Compiling libc v0.2.189
   Compiling version_check v0.9.5
   Compiling heck v0.5.0
   Compiling getrandom v0.3.4
   Compiling zerocopy v0.8.56
   Compiling ref-cast v1.0.27
   Compiling parking_lot_core v0.9.12
   Compiling pulldown-cmark v0.13.4
   Compiling regex-syntax v0.8.11
   Compiling syn v3.0.4
   Compiling pulldown-cmark-escape v0.11.0
   Compiling bitflags v2.13.1
   Compiling scopeguard v1.2.0
   Compiling once_cell v1.21.4
   Compiling smallvec v1.16.0
   Compiling utf8parse v0.2.2
   Compiling unicase v2.9.0
   Compiling num-traits v0.2.19
   Compiling hashbrown v0.17.1
   Compiling lock_api v0.4.14
   Compiling ahash v0.8.12
   Compiling is_terminal_polyfill v1.70.2
   Compiling borrow-or-share v0.2.4
   Compiling anstyle-query v1.1.5
   Compiling colorchoice v1.0.5
   Compiling anstyle v1.0.14
   Compiling bit-vec v0.8.0
   Compiling unicode-general-category v1.1.0
   Compiling strum_macros v0.28.0
   Compiling anstyle-parse v1.0.0
   Compiling num-cmp v0.1.0
   Compiling vsimd v0.8.0
   Compiling indexmap v2.14.1
   Compiling clap_lex v1.1.0
   Compiling strsim v0.11.1
   Compiling micromap v0.3.0
   Compiling bit-set v0.8.0
   Compiling percent-encoding v2.3.2
   Compiling bytecount v0.6.9
   Compiling outref v0.5.2
   Compiling jsonschema-regex v0.52.1
   Compiling data-encoding v2.11.1
   Compiling anyhow v1.0.104
   Compiling uuid-simd v0.8.0
   Compiling num-integer v0.1.47
   Compiling num-complex v0.4.6
   Compiling anstream v1.0.0
   Compiling num-bigint v0.4.8
   Compiling num-iter v0.1.46
   Compiling parking_lot v0.12.5
   Compiling aho-corasick v1.1.5
   Compiling num-rational v0.4.2
   Compiling clap_builder v4.6.6
   Compiling num v0.4.3
   Compiling fraction v0.17.0
   Compiling strum v0.28.0
   Compiling regex-automata v0.4.18
   Compiling semver v1.0.28
   Compiling serde_derive v1.0.229
   Compiling thiserror-impl v2.0.20
   Compiling ref-cast-impl v1.0.27
   Compiling clap_derive v4.6.4
   Compiling thiserror v2.0.20
   Compiling clap v4.6.6
   Compiling schemars v0.8.22
   Compiling serde_yaml v0.9.34+deprecated
   Compiling infra-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-domain)
   Compiling fluent-uri v0.4.1
   Compiling email_address v0.2.9
   Compiling ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/ess-kubernetes)
   Compiling jsonschema-value v0.52.1
   Compiling fancy-regex v0.19.0
   Compiling regex v1.13.1
   Compiling referencing v0.52.1
   Compiling ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-primitives)
   Compiling ess-openapi v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-openapi)
   Compiling jsonschema v0.52.1
   Compiling ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-domain)
   Compiling infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-compiler)
   Compiling infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-analyze)
   Compiling infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-spec)
   Compiling ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-compiler)
   Compiling infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-project)
   Compiling schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/schema-contract)
   Compiling ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-gen)
   Compiling ess-realization v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-realization)
   Compiling ess-composition v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-composition)
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
   Compiling ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/verify/ess-conformance)
   Compiling ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-synth)
   Compiling ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/verify/ess-diff)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 16.06s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
test tests::the_first_level_is_exactly_the_four_areas ... ok
test tests::the_generate_area_answers_to_the_flat_spelling_and_to_its_own ... ok
test tests::every_leaf_is_reachable_by_its_area_path_and_by_its_flat_spelling ... ok
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

test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.25s

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
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.70s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.42s

```

Runner-count lanes (counts from the summary lines below and above):

- ess-deployment unit lane: executed 0→0, exit 0; no unit cases added.
- ess-deployment tests/deployment.rs: executed 7→18, exit 0.
- ess-deployment doctest lane: executed 0→0, exit 0; no doctests added.
- ess-cli unit lane: executed 11→11, exit 0; no cases added to this lane.
- ess-cli command_surface: executed 5→5, exit 0.
- ess-cli command_surface_adversary: executed 4→4, exit 0.
- ess-cli go_conformance: executed 7→7, exit 0.
- ess-cli output_containment: executed 19→19, exit 0; existing assertions preserved.
- ess-cli persisted_delivery: new lane absent from the base package run; initial red executed 3 (1 passed, 2 failed) → final executed 3 (all passed), exit 0. No fabricated base runner count is asserted for an absent executable.
- Full package totals: ess-deployment executed 7→18; ess-cli executed 46→49. Existing unchanged lanes are not presented as evidence that they executed the new cases.

The isolated mechanism runs precede the final unfiltered package gates:

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment persisted_`

Exit: 0

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 2.63s
     Running unittests src/lib.rs (target/debug/deps/ess_deployment-e5782514a987b31e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 8 tests
test persisted_component_and_release_readers_refuse_invalid_local_documents ... ok
test persisted_readers_reject_duplicate_map_keys_before_collection ... ok
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... ok
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... ok
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... ok
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... ok
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... ok

test result: ok. 8 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.05s

```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli --test persisted_delivery`

Exit: 0

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 3.59s
     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 3 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-deployment`

Exit: 0

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `test` profile [unoptimized] target(s) in 2.63s
     Running unittests src/lib.rs (target/debug/deps/ess_deployment-e5782514a987b31e)

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

     Running tests/deployment.rs (target/debug/deps/deployment-f7d4304711ce6995)

running 18 tests
test input_order_does_not_change_locked_bytes ... ok
test canonical_build_ir_restores_an_omitted_empty_secret_set ... ok
test build_graph_is_canonical_and_projects_executable_buildkit_inputs ... ok
test undeclared_secrets_and_cycles_are_stage_strict_refusals ... ok
test helm_defaults_materialize_typed_secret_slots_without_secret_bytes ... ok
test component_release_bundle_is_canonical_and_revalidates_after_transport ... ok
test realization_runtime_release_stack_and_environment_form_one_exact_chain ... ok
test persisted_component_and_release_readers_refuse_invalid_local_documents ... ok
test persisted_readers_reject_duplicate_map_keys_before_collection ... ok
test persisted_build_readers_refuse_invalid_graphs_and_compiler_constraints ... ok
test persisted_documents_preserve_compiler_bytes_across_all_public_reader_routes ... ok
test persisted_deployment_readers_reject_invalid_release_sets_and_canonical_order ... ok
test mutable_public_documents_are_rechecked_at_consuming_entrypoints ... ok
test persisted_runtime_readers_refuse_local_relationship_and_slot_defects ... ok
test persisted_convenience_readers_and_catalogs_use_the_checked_boundary ... ok
test persisted_lock_readers_preserve_local_service_identity_and_reject_invariants ... ok
test persisted_bundle_checks_original_keys_and_consistently_rehashed_nested_graphs ... ok
test persisted_duplicate_keys_are_rejected_at_every_populated_nested_map ... ok

test result: ok. 18 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.12s

   Doc-tests ess_deployment

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s

```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo test --locked -p ess-cli`

Exit: 0

```text
   Compiling ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
   Compiling ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `test` profile [unoptimized] target(s) in 3.73s
     Running unittests src/main.rs (target/debug/deps/ess-2a340c79ed142cb3)

running 11 tests
test tests::a_symlink_above_the_requested_root_is_refused ... ok
test tests::caller_selected_parent_roots_resolve_without_creating_discarded_directories ... ok
test tests::normalizing_a_requested_root_does_not_hide_a_symlink_traversal ... ok
test tests::generated_file_conflicts_are_refused_before_new_directories_are_created ... ok
test tests::projection_files_and_existing_aliases_are_checked_as_one_set ... ok
test tests::every_artifact_destination_is_checked_before_the_first_write ... ok
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
test the_generate_area_honours_the_arguments_it_accepts_or_refuses_them ... ok
test the_generate_area_does_not_run_a_sibling_verb_against_a_path_it_was_not_given ... ok
test a_flat_spelling_prints_what_its_area_path_prints_when_clap_refuses ... ok
test the_generate_usage_line_admits_the_arguments_the_command_takes ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.04s

     Running tests/go_conformance.rs (target/debug/deps/go_conformance-ba60d23811c1c6c2)

running 7 tests
test the_emitted_package_holds_a_correct_go_implementation_to_the_whole_suite ... ok
test a_view_that_drops_rows_fails_the_scenarios_that_say_how_many_it_holds ... ok
test one_deliberate_defect_fails_the_scenarios_responsible_for_it_and_no_others ... ok
test a_view_returned_in_the_wrong_order_fails_exactly_the_scenarios_that_assert_its_order ... ok
test the_emitted_runner_reads_a_positional_assertion_and_refuses_one_in_an_unordered_view ... ok
test the_emitted_runner_stops_a_scan_and_fails_a_target_that_builds_the_whole_listing ... ok
test the_emitted_runner_holds_a_window_and_fails_a_target_whose_clock_never_moves ... ok

test result: ok. 7 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.69s

     Running tests/output_containment.rs (target/debug/deps/output_containment-6c42862c3bb02e7c)

running 19 tests
test composition_keeps_native_non_utf8_and_backslash_filenames_distinct ... ok
test a_hardlinked_destination_is_refused_before_other_files_change ... ok
test an_escaping_include_is_refused_before_any_output_changes ... ok
test composition_keeps_disjoint_caller_selected_filenames_and_parent_roots ... ok
test composition_refuses_cancelled_parent_links_before_disjoint_companions_change ... ok
test composition_preserves_disjoint_files_inside_generated_directories ... ok
test composition_refuses_companion_links_before_any_other_output_changes ... ok
test a_valid_nested_include_keeps_the_existing_site_layout_and_bytes ... ok
test composition_companion_outputs_cannot_collide_with_the_generated_client_tree ... ok
test late_site_asset_aliases_refuse_before_even_creating_output_directories ... ok
test symlink_roots_parents_and_destinations_are_refused_before_writing ... ok
test composition_does_not_reinterpret_directory_spelling_as_a_named_output_file ... ok
test requested_root_normalization_preserves_parent_roots_and_rejects_hidden_files ... ok
test include_aliases_and_duplicate_generated_pages_are_refused_before_writing ... ok
test composition_companions_form_one_output_set_even_without_a_generated_tree ... ok
test local_projection_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok
test composition_preflight_includes_companion_generated_aliases_and_both_companions ... ok
test noncanonical_and_platform_paths_are_refused_before_writing ... ok
test local_generation_sinks_refuse_late_conflicts_before_any_generated_file_changes ... ok

test result: ok. 19 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.41s

     Running tests/persisted_delivery.rs (target/debug/deps/persisted_delivery-15fb048ddae516a0)

running 3 tests
test valid_plan_reaches_both_local_fake_executors_in_rollout_order ... ok
test invalid_current_removal_is_refused_before_analysis_and_execution ... ok
test entire_desired_plan_is_refused_before_oras_or_helm ... ok

test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.11s

```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo fmt -p ess-deployment --check`

Exit: 0

```text
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo fmt -p ess-cli --check`

Exit: 0

```text
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p ess-deployment --all-targets -- -D warnings`

Exit: 0

```text
    Checking ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Finished `dev` profile [unoptimized] target(s) in 1.20s
```

Command: `env TMPDIR="/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/target" SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock RUSTC_WRAPPER=/usr/bin/sccache CARGO_INCREMENTAL=0 CARGO_PROFILE_DEV_DEBUG=0 CARGO_PROFILE_TEST_DEBUG=0 CARGO_CACHE_RUSTC_INFO=0 cargo clippy --locked -p ess-cli --all-targets -- -D warnings`

Exit: 0

```text
    Checking memchr v2.8.3
    Checking allocator-api2 v0.2.21
    Checking foldhash v0.2.0
    Checking regex-syntax v0.8.11
    Checking serde_core v1.0.229
    Checking unicase v2.9.0
    Checking pulldown-cmark-escape v0.11.0
    Checking bitflags v2.13.1
    Checking num-traits v0.2.19
    Checking libc v0.2.189
    Checking zerocopy v0.8.56
    Checking smallvec v1.16.0
    Checking utf8parse v0.2.2
    Checking scopeguard v1.2.0
    Checking once_cell v1.21.4
    Checking thiserror v2.0.20
    Checking ref-cast v1.0.27
    Checking anstyle v1.0.14
    Checking bit-vec v0.8.0
    Checking anstyle-query v1.1.5
    Checking borrow-or-share v0.2.4
    Checking is_terminal_polyfill v1.70.2
    Checking colorchoice v1.0.5
    Checking lock_api v0.4.14
    Checking bit-set v0.8.0
    Checking clap_lex v1.1.0
    Checking hashbrown v0.17.1
    Checking anstyle-parse v1.0.0
    Checking num-integer v0.1.47
    Checking num-complex v0.4.6
    Checking num-cmp v0.1.0
    Checking bytecount v0.6.9
    Checking micromap v0.3.0
    Checking vsimd v0.8.0
    Checking getrandom v0.3.4
    Checking parking_lot_core v0.9.12
    Checking outref v0.5.2
    Checking percent-encoding v2.3.2
    Checking strsim v0.11.1
    Checking num-bigint v0.4.8
    Checking num-iter v0.1.46
    Checking jsonschema-regex v0.52.1
    Checking strum v0.28.0
    Checking unicode-general-category v1.1.0
    Checking indexmap v2.14.1
    Checking data-encoding v2.11.1
    Checking anyhow v1.0.104
    Checking parking_lot v0.12.5
    Checking uuid-simd v0.8.0
    Checking anstream v1.0.0
    Checking aho-corasick v1.1.5
    Checking pulldown-cmark v0.13.4
    Checking num-rational v0.4.2
    Checking num v0.4.3
    Checking fraction v0.17.0
    Checking clap_builder v4.6.6
    Checking regex-automata v0.4.18
    Checking serde v1.0.229
    Checking serde_json v1.0.151
    Checking semver v1.0.28
    Checking clap v4.6.6
    Checking serde_yaml v0.9.34+deprecated
    Checking ahash v0.8.12
    Checking fluent-uri v0.4.1
    Checking email_address v0.2.9
    Checking schemars v0.8.22
    Checking infra-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-domain)
    Checking jsonschema-value v0.52.1
    Checking ess-kubernetes v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/ess-kubernetes)
    Checking referencing v0.52.1
    Checking ess-openapi v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-openapi)
    Checking ess-primitives v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-primitives)
    Checking fancy-regex v0.19.0
    Checking regex v1.13.1
    Checking infra-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-compiler)
    Checking ess-domain v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-domain)
    Checking jsonschema v0.52.1
    Checking infra-analyze v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-analyze)
    Checking infra-spec v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-spec)
    Checking ess-compiler v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-compiler)
    Checking infra-project v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/infra/infra-project)
    Checking ess-gen v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-gen)
    Checking ess-realization v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-realization)
    Checking ess-composition v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/specify/ess-composition)
    Checking schema-contract v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/schema-contract)
    Checking ess-deployment v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-deployment)
    Checking ess-conformance v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/verify/ess-conformance)
    Checking ess-synth v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/generate/ess-synth)
    Checking ess-diff v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/verify/ess-diff)
    Checking ess-cli v0.18.0 (/home/timo/.local/state/worktree/trees/b10x/ess/review-persisted-delivery-validation/crates/edge/ess-cli)
    Finished `dev` profile [unoptimized] target(s) in 9.17s
```

The standalone Rust fake-process fixture was also formatted with `rustfmt --edition 2021 crates/edge/ess-cli/tests/support/fake_delivery.rs` (exit 0). Final fixture changes after package execution were whitespace only. A final module-declaration placement cleanup was whitespace/order only. Intermediate compiler/lint/format failures and corrections remain in the scratch logs, including the initial long-function lint findings (resolved by extracting bounded validation helpers) and one trailing-blank-line fmt failure (subsequent check above is green). No cargo fmt --all was used.

5. Compatibility, scope and limits

Existing compiler fixture canonical JSON survives JSON and YAML round trips byte-for-byte, including omitted empty build secrets. Existing format versions, field serialization order/defaults and writers remain unchanged; checked decoding intentionally refuses documents that earlier readers admitted despite compiler-invariant defects. Build and deployment order vectors are compared against their actual deterministic Kahn compiler algorithm; invalid order is not repaired. Collection field ordering retains existing BTreeMap/BTreeSet semantics.

No semantic/realization source was reconstructed from a digest. Standalone runtime validation does not prove semantic component existence/completeness, replica bounds or statefulness. Lock validation cannot prove omitted catalog/stack selection; deployment validation cannot infer omitted required bindings. Evidence attachments are required structurally, but authenticity, registry/cache origin, conformance truth, recovery and later I/O rollback remain outside this unit. No full Kubernetes, URL, quantity or repository-name validation was invented.

The CLI fake processes are Rust, compile locally, and run with a PATH containing only the fake ORAS/Helm executables. A valid first/invalid later release and an invalid current removal plan both refuse without a fake-call log; the valid control records ORAS, Helm, Helm. No real Docker, Helm, registry, cluster, credentials or external messaging was used. Full workspace/site gates belong to the coordinator; this report claims only the assigned package gates.

Only crates/generate/ess-deployment/**, crates/edge/ess-cli/** and assigned target scratch/fixtures were authored. The private Wire DTO/visitor helper adds no new persisted construct or format, so no design expansion prerequisite was found. Original output-containment and credential-boundary assertions remain unchanged.

Free-space reserve: required 8,589,934,592 bytes; observed before the first build 141,551,620,096 bytes. Final df output:

```text
       Avail
140306542592
```

Token usage and full agent duration are unavailable and are not estimated.

6. Outside paths and handoff

Authored outside-worktree paths: none. The coordinator-owned shared compiler-cache socket is `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w3-cache.sock`; it was used only as authorized by resource-supplement.md. Build outputs, Rust fake executors, temporary fixtures, logs, proposed patch and this report remain within the assigned managed tree.

No Git staging/commit/publication, AEP mutation, cache purge, worktree finish/gc or deletion was performed. Source writes are relinquished for independent review. Coordinator owns review, integration, publication and managed cleanup.
