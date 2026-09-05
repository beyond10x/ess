# Service SDK compatibility experiment: frozen ESS 0.18 candidate

The unchanged Service SDK at `48833c6d14ec37cb3b614fca05cf7dd78f63b743` compiled against frozen ESS `acb7859e3202ffdc1ca840dde67f7ca4da33c746`. All three focused tests, all 26 selected package tests, package formatting and strict Clippy passed. A separate copy of its actual generated synthetic service passed `cargo check --all-targets --locked --offline` with the candidate ESS graph. All 63 generated files, including the exact four-file generated Rust set and manifest, remained unchanged during compilation.

The cross-generator comparison produced 63 files on both sides: 14 changed, 49 identical, no added or removed paths. Each checker accepted its own bytes and refused the other generator's bytes as whole-file drift; regenerating an untouched copy of the old output with the candidate produced exactly the fresh candidate's 63 files and passed its checker. Source identity, canonical ESS IR, neutral plans, source-bound SDK outputs and measured whole-model outputs stayed byte-identical for this fixture.

This is a complete SDK ESS **0.13.1-to-0.18.0** source/generation compatibility experiment. It is not an isolated F01 counterfactual, a published SDK pin upgrade, a release or deployment. SDK source, manifests, Git history and planning were not changed. NEW's root Cargo.lock was changed only by Cargo for the authorized scratch source-substitution experiment and remains for coordinator archival/restoration.

## Exact subjects and setup boundaries

- OLD SDK tree: `/home/timo/.local/state/worktree/trees/b10x/service-sdk/ess-sdk-compat-old`.
- NEW SDK tree: `/home/timo/.local/state/worktree/trees/b10x/service-sdk/ess-sdk-compat-new`.
- Both SDK trees retain published HEAD `48833c6d14ec37cb3b614fca05cf7dd78f63b743`, SDK version 0.5.11.
- OLD ESS source: seven Git packages 0.13.1 at `d1a66772a91b5411d942d7a45bbf08dfc5de4651`.
- Candidate ESS source: seven local packages 0.18.0 at frozen HEAD `acb7859e3202ffdc1ca840dde67f7ca4da33c746`, checkout `/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba`.
- The same retained synthetic `service/1` package and four input files were used throughout. Its declared SDK Git revision is the real published SDK revision above. All inputs and the original 63-file output are checked against the old retained maps before each command. Original OLD binary SHA256 is `56e9c3b761ab65fb6bf7a43ee2d11808b7e65bbf331b562153e01f20fadb8e14`; retained candidate binary SHA256 is `0e543ed67f6971b75a62e74dc5f42f91cd43946bc71cac899517084d35d91196` (40,620,464 bytes).
- Exact HEAD, clean ESS tracked/untracked status, all seven crate identities and both SDK HEADs were checked before every candidate command and after successful stages. Final ESS status is clean at the frozen subject. OLD status is clean; NEW status is only `M Cargo.lock`. The coordinator maintained the source freeze throughout; no further ESS reads or builds are needed after the final evidence snapshot.

All candidate subprocesses used the byte-identical retained runner with this environment:

```text
TMPDIR=/home/timo/.local/state/worktree/trees/b10x/service-sdk/ess-sdk-compat-new/target/ess-review-compat
RUSTC_WRAPPER=/usr/bin/sccache
SCCACHE_SERVER_UDS=/home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba/target/w4-cache.sock
CARGO_INCREMENTAL=0
CARGO_PROFILE_DEV_DEBUG=0
CARGO_PROFILE_TEST_DEBUG=0
CARGO_CACHE_RUSTC_INFO=0
CARGO_NET_OFFLINE=true
```

No candidate fetch occurred. The earlier OLD generated-service setup used one separately authorized normal Cargo fetch after its exact missing source was reported; that older history is preserved in `old-generated-compile-checkpoint.md` and is not a candidate fetch.

## Source substitution and complete resolved graphs

`candidate.config.toml` patches all six direct SDK ESS dependencies plus transitive `ess-primitives`; the exact direct inventory comes from SDK Cargo.toml:51–56. Both metadata assertions require exactly the seven ESS packages below, source `null` and the exact corresponding manifest path below the frozen checkout, with no duplicate or remaining old ESS Git package. All SDK packages must likewise resolve from the exact NEW checkout's corresponding crate path.

| ESS package | Path below frozen checkout |
| --- | --- |
| ess-compiler | `crates/specify/ess-compiler/Cargo.toml` |
| ess-deployment | `crates/generate/ess-deployment/Cargo.toml` |
| ess-domain | `crates/specify/ess-domain/Cargo.toml` |
| ess-gen | `crates/generate/ess-gen/Cargo.toml` |
| ess-realization | `crates/specify/ess-realization/Cargo.toml` |
| ess-synth | `crates/generate/ess-synth/Cargo.toml` |
| ess-primitives, transitive | `crates/specify/ess-primitives/Cargo.toml` |

The SDK candidate graph contains **260 packages**: 12 local SDK crates, seven local ESS crates, 229 registry packages, nine Connectors packages at `235558c11f5fc2e4b8f8440474fb975df49d5329`, two Eventlog packages at `b7e8f0d87b01c403415546d311952cb155caf16f`, and one Identity package at `e3231bc34eb65695ba0486e1bbe3a0f1ec5b2a23`. Full raw metadata with edges/features is `candidate-graph-updated.log`; every package ID, version, source and absolute manifest path is in `candidate-graph-updated-packages.json`. The earlier `candidate-graph.log` is explicitly the refused old graph, not the accepted candidate graph.

The generated-service candidate graph contains **256 packages**: the same 12 local SDK and seven local ESS packages, the generated root, 224 registry packages, and those same 12 other Git packages. Its full graph and per-package inventory are `candidate-generated-graph.log` and `candidate-generated-graph-packages.json`. Its separate config additionally patches the six declared generated SDK dependencies (engine, connectors, catalog, HTTP, host and conformance) to the exact NEW checkout; their SDK path dependencies account for the other six SDK crates. The generated manifest retains its original published SDK Git declaration. This is a local source-substitution experiment, not proof that an unmodified published dependency pin selects the candidate ESS.

The targeted SDK root lock update changed exactly seven ESS package identities from old Git 0.13.1 to local 0.18.0. No other package identity, registry resolution or same-identity package record changed. Comparing the separately resolved OLD and candidate generated-service locks gives the same result: exactly those seven ESS identity changes; no registry changes or other package-record changes. Thus no newly changed registry package needs reconciliation with the frozen candidate lock. The root SDK graph and generated graph remain different scoped graphs (for example root indexmap 2.14.1 versus generated indexmap 2.14.2); neither is represented as the frozen ESS workspace's own graph.

## Preserved setup attempts

1. Initial `init` passed. Initial `resolve` ran Cargo metadata successfully (exit 0), but Cargo kept the seven old locked ESS 0.13.1 Git packages and warned all seven candidate 0.18.0 patches were unused. The exact-source assertion correctly stopped the outer stage (exit 1) before any candidate build. The only first lock change was seven `patch.unused` records. `candidate-resolution-refusal-1.json`, original graph/log and `candidate-root-lock.diff` preserve this result.
2. After reporting it, the coordinator authorized a bounded offline `cargo update` selecting exactly those seven ESS packages. `resume-resolution.py` records the lock before and after, full diff and package comparison, and runs fresh locked offline metadata. Both commands passed and the unchanged exact-source assertions accepted all seven candidate paths before compilation. This was lock-selection setup; it was not an API or cache failure.
3. Initial strict Clippy used `cargo --config <scratch-config> clippy ... --locked --offline -- -D warnings` and exited 101 before any checks with `cannot update the lock file ... because --locked was passed`. The accepted lock remained byte-identical. After reporting the refusal, the coordinator authorized moving the same `--config` after `clippy`, with all package/target/offline/locked/lint flags retained. `candidate-clippy-forwarded` passed (exit 0, 8.383 seconds); the root lock remained byte-identical before/after. This confirms the configuration-forwarding adjustment works for cargo-clippy in this environment. No child-process argv trace was captured, so the report does not claim a deeper internal trace of the initial failure. No source, pin, assertion or lint relaxation occurred.

Original preparation and OLD setup reports remain unchanged; all failed attempts remain visible and are not counted as completed analysis or tests.

## Executed test and comparison results

The three existing named integration cases each passed in isolation: one test, zero failures, five filtered out per lane:

- `one_runtime_ir_derives_identical_client_and_connector_surfaces`
- `cli_generate_then_check_detects_byte_drift`
- `unified_package_emits_compilable_service_and_connector_factory_sources`

The third case still invokes rustfmt rather than compiling the generated service. Its name is not used as compilation evidence. The distinct actual generated-service check below supplies that evidence.

The selected package suite passed **26 tests**: service-builder 14 (eight unit and six integration), service-conformance seven, and service-runtime-ir five. These include the three focused cases again; the isolated runs are not added to claim 29 distinct tests. Package-scoped formatting passed. Final strict package Clippy with all targets and `-D warnings` passed.

| Generator/checker operation | Exit | Observed result |
| --- | ---: | --- |
| OLD check OLD, retained earlier control | 0 | Current; original control reused without rerunning |
| Candidate check OLD | 1 | Whole-file drift, 14 changed paths |
| Candidate generate fresh | 0 | 63 files |
| Candidate check fresh candidate | 0 | Current |
| OLD check candidate | 1 | Same 14 changed paths |
| Candidate regenerate separate copied OLD output | 0 | 63 files, exactly fresh candidate bytes |
| Candidate check regenerated copy | 0 | Current |

The SDK checker recompiles its own expected artifacts and compares complete UTF-8 file bytes (`crates/service-builder/src/main.rs:48–67`, `tree.rs:87–106`). Its ownership manifest records full-byte hashes (`tree.rs:180–193`). Both nonzero cross-checks reached the explicit generated-output-drift diagnostic. They are not evidence that an independent old stamp parser rejected `slice-sha256/2`. This experiment invokes neither an `ess-diff/2` nor an `ess-impact/3` report reader.

### Observed changed bytes and attribution limits

The complete unified diff is `candidate-observed-output.diff`; old, candidate and regenerated complete path/size/SHA maps are preserved. Fourteen changed paths are:

- `deployment/chart/values.schema.json`
- `ess/projections/asyncapi/demo-service.yaml`
- `ess/projections/docs/domains/demo-todo.md`
- `ess/projections/openapi/demo-service.yaml`
- `ess/projections/schema/commands/demo.todo.AddItem.schema.json`
- `ess/projections/schema/entities/demo.todo.Item.schema.json`
- `ess/projections/schema/events/demo.todo.ItemAdded.schema.json`
- `ess/projections/schema/types/demo.todo.ContentRef.schema.json`
- `ess/projections/schema/types/demo.todo.Item.State.schema.json`
- `ess/projections/schema/types/demo.todo.ItemId.schema.json`
- `ess/projections/schema/types/demo.todo.ItemRow.schema.json`
- `ess/projections/schema/types/demo.todo.OwnerRef.schema.json`
- `ess/projections/site/domains/demo-todo.html`
- `service-builder.manifest.json`

Twelve are ESS sliced projection artifacts. Eight JSON Schemas and the two domain documentation/site files gained the `slice-sha256/2:` profile while retaining their digest suffixes. The two service API projections gained that profile and changed their suffix from `f559630adb726683deaa689269836d96570222f55f5fad25463f13d667106fa1` to `4c1ae20632c7684955bb4971d48df78c172c3d3b90e37cd145e7ebd5e5ebc875`. The Helm values schema independently gained an explicit empty, closed secrets-object schema; the builder ownership manifest changed to reflect all 13 changed owned files. These are observations of the full version upgrade. No alternate build isolating F01 was run, so the 14-file count and service suffix change are not asserted as exclusively caused by F01.

The retained model/source digest is unchanged at `b244efe82a03ac7aa62b53360b49e4b6691452581b94bcfe9ec2120015a271c8`. The measured whole-model topology provenance retains the bare digest `4c1ae20632c7684955bb4971d48df78c172c3d3b90e37cd145e7ebd5e5ebc875`; both whole-model topology files are byte-identical. For the entity-schema example, the old sliced digest `2eeea2a8d1b0ade5be0505edd75799a06d1525ffc1b0101c115e39f7fc01ef4f` becomes `slice-sha256/2:2eeea2a8d1b0ade5be0505edd75799a06d1525ffc1b0101c115e39f7fc01ef4f`, with the same source digest. The complete extracted digest-valued JSON fields and matching provenance lines are in `candidate-matrix.json`.

These explicitly measured controls all stayed byte-identical:

| Control | Bytes | SHA256, identical OLD/candidate |
| --- | ---: | --- |
| `ess/ir.json` | 7701 | `c79255f47083d1764d83ea66d61aa00a676865a6bc385fa5523c47d113efa0ab` |
| `ess/synthesis/PLAN.md` | 1745 | `cc0a35c74081d50430d6ffa8e5f54a1831e0aa4d5865c7ebfe0a920ec1f82b41` |
| `ess/synthesis/plan.json` | 2704 | `774b4b188591524ae7b1564762d46e13798b29d7c88370a5f17854eeb5468085` |
| `runtime/ir.json` | 8311 | `e370ef143250313078df352f86f8988e3678306d7240b1e43616890f8554af5e` |
| `runtime/realization-plan.json` | 5369 | `69b6364bc44d9776958b3eaa190be61369c8b62660fc54accd79eec59bc0f52d` |
| `client/plan.json` | 2108 | `0ca506c0909481329e8007e5bddb325f6123110224ef5a96a0541d596b0365b7` |
| `connectors/contribution.json` | 1090 | `f443b46c1890e417f43b933e1d29fbc5629d0871d7bfce67287a1af97693c42c` |
| `conformance/scenario.yaml` | 257 | `4a275d6bbd5e2e2896da00f6f065156eaf869dec6ec53ef21664119195f9017a` |
| `http/openapi.json` | 8856 | `7c45cee11fae9742b950b252728e7f7212a9c30efa316f52fce5d17db52d92f9` |
| `catalog/service-catalog.json` | 12176 | `7646911db833c5d834a95f7209bda42b4f16bab4e88177bcd27515eda63ad93c` |
| `ess/projections/docs/topology.md` | 1067 | `322d414613ba029f84ef09e239019b1758a8244392f5b8961e4c66799b694546` |
| `ess/projections/site/topology.html` | 2799 | `b18ed439fe193447a1d36320350ba0335f80ddc449b0ed6d6413932d645149cc` |


All four generated Rust files, all other deployment files, and the remaining unchanged paths are listed in the complete maps. Source-bound runtime IR retains the same source, synthesis and obligation-catalog digests. The single unchanged fixture does not establish arbitrary-source semantic compatibility or cover every F01 semantic distinction; those belong to ESS's separate review/gate.

## Actual generated-service compilation

The candidate 63-file output was copied byte-for-byte to `output-candidate-compile`. Its four Rust files include `rust/Cargo.toml`, `rust/src/lib.rs`, `rust/src/main.rs` and `rust/tests/generated_scenarios.rs`. A new parent `Cargo.toml` containing only `[workspace]`, `resolver = "2"`, `members = ["rust"]` isolates this scratch fixture from the enclosing SDK workspace. It does not edit a generated file. `candidate-generated.config.toml` provides the explicit ESS and SDK patches, and Cargo produces a separate local fixture lock.

Offline metadata and exact source assertions passed before `cargo check --all-targets --locked --offline`. The actual check passed in 10.606 seconds using the wrapper's own `output-candidate-compile/target`; logs show `Checking demo-generated-service v0.1.0` and successful completion. This checks the default `standalone-host` feature's library, binary and test targets on the host platform. It does not run the standalone service or its generated test binary, test other feature/platform combinations, publish a pin, or demonstrate deployment. The selected SDK conformance suite is separately recorded above.

After checking, every copied generated hash still matched the pre-copy map, the exact generated Rust file set was unchanged, and the pristine candidate output still matched its original 63-file map. Final verification repeated those checks and confirmed the original OLD 63-file map and inputs remain unchanged.

## Exact execution and raw evidence

Every lane ran from NEW with the recorded offline environment. Each `<lane>.json` stores exact argv/cwd/environment/exit/duration and points to its complete `<lane>.log`. `candidate-final-facts.json` also collects every exact command and hashes both its command record and raw log; `candidate-evidence-hashes.json` covers all retained evidence files. The compact table below reports every attempted Cargo/binary lane without duplicating the full commands or logs.

Outer orchestration used the prepared `candidate.py` stages `init`, initial `resolve`, `tests`, `matrix`, `generated-check`, and initial `gates`, each with explicit `--subject-commit acb7859e3202ffdc1ca840dde67f7ca4da33c746 --source-checkout /home/timo/.local/state/worktree/trees/b10x/ess/wt-752828a285ba`. The two authorized adjustments used `resume-resolution.py` and `finish-clippy.py` with the same arguments. The unmodified prepared script, both adjustment scripts, configs and all refusal records are retained. `collect-final-evidence.py` performed final read/hash checks without invoking Cargo or changing source.

| Lane / matching command record and raw log prefix | Exit | Seconds | Raw log SHA256 |
| --- | ---: | ---: | --- |
| `candidate-build` | 0 | 0.129 | `1094a36867300c1838cdc4ee40efe961d83bac574665f8ee57a40d5b1e88698c` |
| `candidate-case-1` | 0 | 13.961 | `39d8a361c783dd4c3e240de1b89f67136785d9b3deaf857dd274efa56c312cbe` |
| `candidate-case-2` | 0 | 1.006 | `cf91bd92e25a84b7c45756a506c52ed165740a410ca3823aff4888204921c910` |
| `candidate-case-3` | 0 | 0.287 | `f38e221012ec93ecd3e4ef524eb32798b5e6acd8cd30f32898c97c72a23488f2` |
| `candidate-check-candidate` | 0 | 0.134 | `5c87639384bdba9e9c39eb117541b62fbead9daf58d3af1ba4b3619f606b3a78` |
| `candidate-check-old` | 1 | 0.135 | `6bf9a3ab5c769c545c192c8857a4c694a08bd8413570e56cccb6584bc30b423b` |
| `candidate-check-regenerated` | 0 | 0.136 | `104c2ba0b4d30390b0d402811008bdf5324edc9a1691d71562328a821b49301d` |
| `candidate-clippy-forwarded` | 0 | 8.383 | `2128ca7066f9a01e576b5d012c8de6d53f12c417eb4f2fa563ec1f9d1f5a9a3a` |
| `candidate-clippy` | 101 | 0.107 | `9d32427e4a789399e6aec152c5ebbd05580f8f580a97edb634fba8a6c182ec56` |
| `candidate-fmt` | 0 | 0.088 | `e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855` |
| `candidate-generate` | 0 | 0.136 | `de6ddc193a9a9f338c4de69b5a12c4c5e00884017579ae82a764ed2709296028` |
| `candidate-generated-check` | 0 | 10.606 | `2ab7b0526e1eb71c69623d7f2e0ef9daa65f9f9774d29932ded1143a389cfc0d` |
| `candidate-generated-graph` | 0 | 0.472 | `52e9a54091a53541403bb6ccdf23e8ee402c9cb1c317e939b04dd749f6e7d119` |
| `candidate-graph-updated` | 0 | 0.156 | `cc9cfab5f20e04eae011ca5f6a62ab1c25b867f72c301476b0f429982e8dd400` |
| `candidate-graph` | 0 | 0.220 | `5f43ab03391e214738ba20f12faa5217ee50ae11effdaa25140c1b1a11186e1d` |
| `candidate-package-tests` | 0 | 3.718 | `d37cc12e3a85a3e585809c3643bf7c61b867061f9382a9dca984747375594335` |
| `candidate-regenerate-old` | 0 | 0.143 | `4cc913333cb871f5d336040afed32d193645dda5af4732dd896fe63561ccffad` |
| `candidate-targeted-update` | 0 | 0.362 | `d10308c7960b57f0fd09374420e40e946a2105d77145c0188972b6769cf09278` |
| `old-check-candidate` | 1 | 0.130 | `b8f83e5f6e7367150649fe12ee283c7e55b901bceebc017825f5dc32eecaec33` |

## Lock and artifact hashes

| Evidence | SHA256 |
| --- | --- |
| Original OLD and NEW SDK root lock | `c1ea6417abe7af9175e61b650729bdbf88f5d62153fd6a9f20caf076bec9c746` |
| Initial unused-patch lock, before targeted update | `ec2e3acf218dfaefb1a447a411702ed90e942ff019a790b32fac5d7114a9502b` |
| Accepted NEW root lock, unchanged through tests/checks/Clippy | `e828189fd54838468fe55b6ebae401fe6370403d5b333efac069fda8d5220ef6` |
| OLD generated-service local lock | `f5f69e7fd44b4dafd7298e4a9597e3233b1a18fb89257e78e7d5188a885e5e03` |
| Candidate generated-service local lock | `1906545e311d6545248f3471b0e086bac073d4aa26cf3201417a171019ec1acf` |
| Frozen ESS own workspace lock, read-only reference | `47a1167a65ef3555ded0b7d20024aceaf148fdb5ec220628eaac615f736e5ba7` |
| Original OLD output map | `79fcb85b49bf233a33024113fc0b8a7e5d5b56af9eab4b8bb944a8958da43e6d` |
| `candidate-graph-updated.log` | `cc9cfab5f20e04eae011ca5f6a62ab1c25b867f72c301476b0f429982e8dd400` |
| `candidate-graph-updated-packages.json` | `e76888b8bbf510a581c4da7a4961520a3c85c5ff777fd3815290b195b7eac235` |
| `candidate-generated-graph.log` | `52e9a54091a53541403bb6ccdf23e8ee402c9cb1c317e939b04dd749f6e7d119` |
| `candidate-generated-graph-packages.json` | `f3e39c3826a03f8f07e11245f4d0fd878c919b62e68c6e700f2d1cdc53f4b1ab` |
| `output-candidate-map.json` | `c0cf08483ed14384addc438f2fbf1ea6e3ab9985241512b1e20b1f4e5c5f17bd` |
| `output-regenerated-map.json` | `4416f3ad669ffd538c8c8958dab5fae146dd7b10ede6c8693d00c5074fcc4262` |
| `candidate-matrix.json` | `587ddc5a16c9f2c2d496b9d3eccfc550300a4c49858ad025a548ae3c2f37f522` |
| `candidate-observed-output.diff` | `f07b584c7841c3064133f3ac6402a2fff89ecc6f04382751a07dcdd42c6372f1` |
| `candidate-final-facts.json` | `4929a4cf0f98ac5c287653b29845bbb188bb83ea78b655d96d57567de32812b4` |
| `candidate-evidence-hashes.json` | `d7cff13a64dcd1875fd1d4fe23f99968401a5cf37050fa67f1b459c6c22674c2` |

Hashes of all earlier setup reports, preparation files, configs, logs and command records are included in that inventory. This report and its renderer are finalized and hashed separately after the evidence inventory; the report does not claim to hash itself.

## Resource accounting and handoff

At the final source/preservation snapshot, OLD total target is 863,724 KiB, including scratch 299,344 KiB and nested generated compile target 249,452 KiB. NEW total target is 1,085,224 KiB, including scratch 306,064 KiB and nested generated compile target 249,880 KiB. These totals include their nested directories; do not add them twice. Available space is 92,183,445,504 bytes, above the 8 GiB reserve. The final facts/report add only small evidence files afterward; no additional build ran.

No SDK source or published pin repair was needed for this bounded experiment. A persistent SDK migration still requires its own governed Cargo.toml/Cargo.lock change, review/gates/publication and adopter regeneration; this scratch result neither performs nor substitutes for that work. No release or deployment occurred. All SDK writes and frozen ESS reads/builds are relinquished to the coordinator, who owns archival, root-lock restoration and managed cleanup.
